use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use clipper_client::{ClipperClient, SearchFilters};
use std::path::PathBuf;
use tokio::sync::mpsc;

mod config;

#[derive(Parser)]
#[command(name = "clipper-cli")]
#[command(about = "Command-line interface for Clipper", long_about = None)]
struct Cli {
    /// Path to config file (same format as Clipper desktop app settings.json)
    #[arg(short, long, env = "CLIPPER_CONFIG")]
    config: Option<PathBuf>,

    /// Server URL (defaults to config file or Clipper desktop app config if available, otherwise http://localhost:3000)
    #[arg(short, long, env = "CLIPPER_URL")]
    url: Option<String>,

    /// Bearer token for authentication (defaults to config file or Clipper desktop app config if available)
    #[arg(short, long, env = "CLIPPER_TOKEN")]
    token: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new clip
    #[clap(alias = "c")]
    Create {
        /// Clip content
        content: String,

        /// Tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// Additional notes
        #[arg(short, long)]
        notes: Option<String>,
    },

    /// Get a clip by ID
    #[clap(alias = "g")]
    Get {
        /// Clip ID
        id: String,

        /// Output format: json or text (content only)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Update a clip's tags and/or notes
    #[clap(alias = "u")]
    Update {
        /// Clip ID
        id: String,

        /// New tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// New additional notes
        #[arg(short, long)]
        notes: Option<String>,
    },

    /// Search clips
    #[clap(alias = "s")]
    Search {
        /// Search query
        query: String,

        /// Filter by tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// Filter by start date (ISO 8601 format)
        #[arg(long)]
        start_date: Option<String>,

        /// Filter by end date (ISO 8601 format)
        #[arg(long)]
        end_date: Option<String>,

        /// Page number (starting from 1)
        #[arg(short, long, default_value = "1")]
        page: usize,

        /// Number of items per page
        #[arg(long, default_value = "20")]
        page_size: usize,

        /// Output format: json or text (content only with IDs)
        #[arg(short = 'f', long, default_value = "json")]
        format: String,
    },

    /// Delete a clip by ID
    #[clap(alias = "d")]
    Delete {
        /// Clip ID
        id: String,
    },

    /// Watch for real-time notifications via WebSocket (outputs NDJSON)
    #[clap(alias = "w")]
    Watch,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cli = Cli::parse();

    // Load config from specified file, or fall back to Clipper desktop app config
    // Priority: CLI arg --config > CLIPPER_CONFIG env > desktop app config
    let file_config = if let Some(config_path) = &cli.config {
        config::load_config_from_path(config_path)
    } else {
        config::load_desktop_config()
    };

    // Resolve URL: CLI arg > env var > config file > default
    let url = cli.url.unwrap_or_else(|| {
        file_config
            .as_ref()
            .map(|c| c.server_url.clone())
            .unwrap_or_else(|| "http://localhost:3000".to_string())
    });

    // Resolve token: CLI arg > env var > config file > None
    let token = cli
        .token
        .or_else(|| file_config.and_then(|c| c.token));

    let client = match token {
        Some(token) => ClipperClient::new_with_token(url, token),
        None => ClipperClient::new(url),
    };

    match cli.command {
        Commands::Create {
            content,
            tags,
            notes,
        } => {
            let tags_vec = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let clip = client
                .create_clip(content, tags_vec, notes)
                .await
                .context("Failed to create clip")?;

            println!("{}", serde_json::to_string_pretty(&clip)?);
        }

        Commands::Get { id, format } => {
            let clip = client.get_clip(&id).await.context("Failed to get clip")?;

            match format.as_str() {
                "text" => {
                    println!("{}", clip.content);
                }
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&clip)?);
                }
                _ => {
                    anyhow::bail!("Invalid format. Use 'json' or 'text'");
                }
            }
        }

        Commands::Update { id, tags, notes } => {
            let tags_vec = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

            let clip = client
                .update_clip(&id, tags_vec, notes)
                .await
                .context("Failed to update clip")?;

            println!("{}", serde_json::to_string_pretty(&clip)?);
        }

        Commands::Search {
            query,
            tags,
            start_date,
            end_date,
            page,
            page_size,
            format,
        } => {
            let tags_vec = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

            let start_date_parsed = start_date
                .map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .context("Invalid start_date format, use ISO 8601")
                        .map(|dt| dt.with_timezone(&Utc))
                })
                .transpose()?;

            let end_date_parsed = end_date
                .map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .context("Invalid end_date format, use ISO 8601")
                        .map(|dt| dt.with_timezone(&Utc))
                })
                .transpose()?;

            let filters = SearchFilters {
                start_date: start_date_parsed,
                end_date: end_date_parsed,
                tags: tags_vec,
            };

            let result = client
                .search_clips(&query, filters, page, page_size)
                .await
                .context("Failed to search clips")?;

            match format.as_str() {
                "text" => {
                    for clip in result.items {
                        println!("{}\n{}\n", clip.id, clip.content);
                    }
                    eprintln!(
                        "Page {} of {} (Total: {} clips)",
                        result.page, result.total_pages, result.total
                    );
                }
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                _ => {
                    anyhow::bail!("Invalid format. Use 'json' or 'text'");
                }
            }
        }

        Commands::Delete { id } => {
            client
                .delete_clip(&id)
                .await
                .context("Failed to delete clip")?;

            println!("Clip {} deleted successfully", id);
        }

        Commands::Watch => {
            let (tx, mut rx) = mpsc::unbounded_channel();

            let _handle = client
                .subscribe_notifications(tx)
                .await
                .context("Failed to connect to WebSocket")?;

            // Receive notifications and output as NDJSON (one JSON object per line)
            while let Some(notification) = rx.recv().await {
                let json = serde_json::to_string(&notification)?;
                println!("{}", json);
            }
        }
    }

    Ok(())
}

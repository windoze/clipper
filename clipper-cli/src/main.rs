use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use clipper_client::{fetch_server_certificate, ClipperClient, SearchFilters};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use tokio::sync::mpsc;
use url::Url;

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

    /// List clips
    #[clap(alias = "l")]
    List {
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
        #[arg(long, default_value = "100")]
        page_size: usize,

        /// Output format: json or text (content only with IDs)
        #[arg(short = 'f', long, default_value = "json")]
        format: String,
    },

    /// Upload a file to create a clip
    Upload {
        /// Path to the file to upload
        file: PathBuf,

        /// Tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// Additional notes
        #[arg(short, long)]
        notes: Option<String>,

        /// Content override (defaults to file path)
        #[arg(short, long)]
        content: Option<String>,
    },

    /// Create a short URL for a clip
    Share {
        /// Clip ID
        id: String,

        /// Expiration time in hours (0 = never expires, omit for server default)
        #[arg(short, long)]
        expires: Option<u32>,

        /// Output format: json (full metadata) or url (just the URL)
        #[arg(short, long, default_value = "url")]
        format: String,
    },

    /// Export all clips to a tar.gz archive
    #[clap(alias = "e")]
    Export {
        /// Output file path (default: clipper_export_<timestamp>.tar.gz)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Import clips from a tar.gz archive
    #[clap(alias = "i")]
    Import {
        /// Path to the tar.gz archive to import
        file: PathBuf,

        /// Output format: json (full result) or text (summary only)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set restrictive permissions for newly created files and directories.
    // On Unix: Sets umask to 0o077 (files 0600, directories 0700)
    // On Windows: This is a no-op; directories are secured after creation with ACLs
    clipper_security::set_restrictive_umask();

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
        .clone()
        .or_else(|| file_config.as_ref().and_then(|c| c.token.clone()));

    // Get trusted certificates from config
    let mut trusted_certificates = file_config
        .as_ref()
        .map(|c| c.trusted_certificates.clone())
        .unwrap_or_default();

    // Get config path for saving trusted certificates
    let config_path = cli
        .config
        .clone()
        .or_else(|| file_config.as_ref().and_then(|c| c.config_path.clone()))
        .or_else(config::get_default_config_path);

    // Check certificate for HTTPS URLs
    if url.starts_with("https://") {
        trusted_certificates = check_and_trust_certificate(&url, trusted_certificates, config_path.as_deref()).await?;
    }

    let client = match &token {
        Some(token) => ClipperClient::new_with_trusted_certs(&url, Some(token.clone()), trusted_certificates),
        None => ClipperClient::new_with_trusted_certs(&url, None, trusted_certificates),
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

        Commands::List {
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
                .list_clips(filters, page, page_size)
                .await
                .context("Failed to list clips")?;

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

        Commands::Upload {
            file,
            tags,
            notes,
            content,
        } => {
            let tags_vec = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            // Use absolute path as content if not specified
            let file_path = std::fs::canonicalize(&file)
                .with_context(|| format!("Failed to resolve path: {}", file.display()))?;
            let content = Some(content.unwrap_or_else(|| file_path.to_string_lossy().to_string()));

            let filename = file
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".to_string());

            let bytes = tokio::fs::read(&file)
                .await
                .with_context(|| format!("Failed to read file: {}", file.display()))?;

            let clip = client
                .upload_file_bytes_with_content(bytes, filename, tags_vec, notes, content)
                .await
                .context("Failed to upload file")?;

            println!("{}", serde_json::to_string_pretty(&clip)?);
        }

        Commands::Share {
            id,
            expires,
            format,
        } => {
            let short_url = client
                .create_short_url(&id, expires)
                .await
                .context("Failed to create short URL")?;

            match format.as_str() {
                "url" => {
                    println!("{}", short_url.full_url);
                }
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&short_url)?);
                }
                _ => {
                    anyhow::bail!("Invalid format. Use 'json' or 'url'");
                }
            }
        }

        Commands::Export { output } => {
            // Check if server supports export/import
            let server_info = client
                .get_server_info()
                .await
                .context("Failed to get server info")?;
            if !server_info.config.export_import_enabled {
                anyhow::bail!("Server does not support export/import functionality");
            }

            // Generate default filename with timestamp if not specified
            let output_path = output.unwrap_or_else(|| {
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                PathBuf::from(format!("clipper_export_{}.tar.gz", timestamp))
            });

            eprintln!("Exporting clips to {}...", output_path.display());

            let bytes_written = client
                .export_to_file(&output_path)
                .await
                .context("Failed to export clips")?;

            let size_mb = bytes_written as f64 / (1024.0 * 1024.0);
            if size_mb >= 1.0 {
                eprintln!("Export complete: {:.2} MB written to {}", size_mb, output_path.display());
            } else {
                let size_kb = bytes_written as f64 / 1024.0;
                eprintln!("Export complete: {:.2} KB written to {}", size_kb, output_path.display());
            }
        }

        Commands::Import { file, format } => {
            // Check if server supports export/import
            let server_info = client
                .get_server_info()
                .await
                .context("Failed to get server info")?;
            if !server_info.config.export_import_enabled {
                anyhow::bail!("Server does not support export/import functionality");
            }

            // Verify file exists
            if !file.exists() {
                anyhow::bail!("File not found: {}", file.display());
            }

            eprintln!("Importing clips from {}...", file.display());

            let result = client
                .import_from_file(&file)
                .await
                .context("Failed to import clips")?;

            match format.as_str() {
                "text" => {
                    eprintln!("Import complete:");
                    eprintln!("  Imported: {} clips ({} with attachments)",
                        result.imported_count, result.attachments_imported);
                    eprintln!("  Skipped:  {} clips (duplicates)", result.skipped_count);
                }
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                _ => {
                    anyhow::bail!("Invalid format. Use 'json' or 'text'");
                }
            }
        }
    }

    Ok(())
}

/// Check if the server's certificate is trusted, and prompt user to trust if not.
/// Returns the updated trusted certificates map.
async fn check_and_trust_certificate(
    server_url: &str,
    mut trusted_certificates: HashMap<String, String>,
    config_path: Option<&std::path::Path>,
) -> Result<HashMap<String, String>> {
    // Parse URL to get host and port
    let parsed_url = Url::parse(server_url).context("Invalid server URL")?;
    let host = parsed_url
        .host_str()
        .context("URL has no host")?
        .to_string();
    let port = parsed_url.port().unwrap_or(443);

    // Fetch the certificate
    let cert_info = match fetch_server_certificate(&host, port).await {
        Ok(info) => info,
        Err(e) => {
            // Connection failed, but might be a different error
            anyhow::bail!("Failed to connect to {}: {}", server_url, e);
        }
    };

    // Check if certificate is system-trusted (valid CA chain)
    if cert_info.is_system_trusted {
        return Ok(trusted_certificates);
    }

    // Check if we already trust this certificate
    if let Some(trusted_fp) = trusted_certificates.get(&host) {
        if trusted_fp == &cert_info.fingerprint {
            return Ok(trusted_certificates);
        }
        // Fingerprint changed! Warn the user
        eprintln!();
        eprintln!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@");
        eprintln!("@    WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!    @");
        eprintln!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@");
        eprintln!("IT IS POSSIBLE THAT SOMEONE IS DOING SOMETHING NASTY!");
        eprintln!("Someone could be eavesdropping on you right now (man-in-the-middle attack)!");
        eprintln!("It is also possible that the host certificate has just been changed.");
        eprintln!();
        eprintln!("Host: {}", host);
        eprintln!("Expected fingerprint: {}", trusted_fp);
        eprintln!("Received fingerprint: {}", cert_info.fingerprint);
        eprintln!();
        anyhow::bail!("Host certificate verification failed. If you trust this change, remove the old entry from your config file and try again.");
    }

    // New untrusted certificate - prompt user like SSH does
    eprintln!();
    eprintln!("The authenticity of host '{}' can't be established.", host);
    eprintln!("The server's certificate is not signed by a trusted Certificate Authority (CA).");
    eprintln!("This could mean:");
    eprintln!("  - The server is using a self-signed certificate");
    eprintln!("  - The server's CA is not in your system's trust store");
    eprintln!("  - Someone may be intercepting your connection (man-in-the-middle attack)");
    eprintln!();
    eprintln!("Certificate SHA256 fingerprint:");
    eprintln!("  {}", format_fingerprint_short(&cert_info.fingerprint));
    eprintln!();

    // Show full fingerprint in a more readable format
    eprintln!("Full fingerprint (verify with server administrator):");
    for chunk in cert_info.fingerprint.split(':').collect::<Vec<_>>().chunks(8) {
        eprintln!("  {}", chunk.join(":"));
    }
    eprintln!();

    // Ask for confirmation
    eprint!("Are you sure you want to trust this certificate and continue connecting (yes/no)? ");
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input != "yes" && input != "y" {
        anyhow::bail!("Host certificate not trusted. Connection aborted.");
    }

    // User confirmed - save the certificate
    trusted_certificates.insert(host.clone(), cert_info.fingerprint.clone());

    // Try to save to config file
    if let Some(path) = config_path {
        match config::save_trusted_certificate(path, &host, &cert_info.fingerprint) {
            Ok(()) => {
                eprintln!();
                eprintln!("Warning: Permanently added '{}' to the list of trusted hosts.", host);
            }
            Err(e) => {
                eprintln!();
                eprintln!("Warning: Could not save trusted certificate to config: {}", e);
                eprintln!("The certificate will be trusted for this session only.");
            }
        }
    } else {
        eprintln!();
        eprintln!("Warning: No config file available. Certificate trusted for this session only.");
    }

    Ok(trusted_certificates)
}

/// Format fingerprint in a shorter display format (first 16 bytes as base64-like)
fn format_fingerprint_short(fingerprint: &str) -> String {
    // Just show the fingerprint in a condensed format
    fingerprint.replace(":", "").to_lowercase()
}

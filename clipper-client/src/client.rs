use crate::error::{ClientError, Result};
use crate::models::{
    Clip, ClipNotification, CreateClipRequest, PagedResult, SearchFilters, ServerInfo,
    UpdateClipRequest, WsAuthRequest, WsAuthResponse,
};
use futures_util::{SinkExt, StreamExt};
use reqwest::StatusCode;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::io::ReaderStream;
use url::Url;

/// Connection timeout - if no message received within this time, consider connection dead
/// Server sends ping every 30s, so we wait 60s (2x interval) before timing out
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);

/// Client for interacting with the Clipper server
#[derive(Clone)]
pub struct ClipperClient {
    base_url: String,
    client: reqwest::Client,
    /// Optional Bearer token for authentication
    token: Option<String>,
}

impl ClipperClient {
    /// Create a new Clipper client
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the Clipper server (e.g., "http://localhost:3000")
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
            token: None,
        }
    }

    /// Create a new Clipper client with Bearer token authentication
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the Clipper server (e.g., "http://localhost:3000")
    /// * `token` - Bearer token for authentication
    pub fn new_with_token(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
            token: Some(token.into()),
        }
    }

    /// Set the Bearer token for authentication
    ///
    /// # Arguments
    /// * `token` - Bearer token for authentication, or None to disable authentication
    pub fn set_token(&mut self, token: Option<String>) {
        self.token = token;
    }

    /// Get the current Bearer token
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Apply authentication header to a request builder if a token is set
    fn apply_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.token {
            Some(token) => builder.header("Authorization", format!("Bearer {}", token)),
            None => builder,
        }
    }

    /// Get the base URL of the server
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get server version and configuration information
    ///
    /// # Returns
    /// Server info including version, uptime, and configuration (including max upload size)
    pub async fn get_server_info(&self) -> Result<ServerInfo> {
        let url = format!("{}/version", self.base_url);
        let response = self.apply_auth(self.client.get(&url)).send().await?;

        self.handle_response(response).await
    }

    /// Create a new clip
    ///
    /// # Arguments
    /// * `content` - Text content of the clip
    /// * `tags` - List of tags for the clip
    /// * `additional_notes` - Optional additional notes
    pub async fn create_clip(
        &self,
        content: String,
        tags: Vec<String>,
        additional_notes: Option<String>,
    ) -> Result<Clip> {
        let url = format!("{}/clips", self.base_url);
        let request = CreateClipRequest {
            content,
            tags,
            additional_notes,
        };

        let response = self
            .apply_auth(self.client.post(&url).json(&request))
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Upload a file to create a clip using a stream
    ///
    /// # Arguments
    /// * `reader` - An async reader (stream) for the file content
    /// * `original_filename` - The original filename
    /// * `tags` - List of tags for the clip
    /// * `additional_notes` - Optional additional notes
    ///
    /// # Example
    /// ```no_run
    /// use clipper_client::ClipperClient;
    /// use tokio::fs::File;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClipperClient::new("http://localhost:3000");
    /// let file = File::open("document.txt").await?;
    ///
    /// let clip = client.upload_file(
    ///     file,
    ///     "document.txt".to_string(),
    ///     vec!["docs".to_string()],
    ///     None,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_file<R>(
        &self,
        reader: R,
        original_filename: String,
        tags: Vec<String>,
        additional_notes: Option<String>,
    ) -> Result<Clip>
    where
        R: AsyncRead + Send + Sync + 'static,
    {
        let url = format!("{}/clips/upload", self.base_url);

        // Convert AsyncRead to a stream of bytes
        let stream = ReaderStream::new(reader);
        let body = reqwest::Body::wrap_stream(stream);

        let file_part = reqwest::multipart::Part::stream(body).file_name(original_filename);

        let mut form = reqwest::multipart::Form::new().part("file", file_part);

        if !tags.is_empty() {
            form = form.text("tags", tags.join(","));
        }

        if let Some(notes) = additional_notes {
            form = form.text("additional_notes", notes);
        }

        let response = self
            .apply_auth(self.client.post(&url).multipart(form))
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Upload file bytes to create a clip
    ///
    /// # Arguments
    /// * `bytes` - The file content as bytes
    /// * `filename` - The filename to use
    /// * `tags` - List of tags for the clip
    /// * `additional_notes` - Optional additional notes
    ///
    /// # Example
    /// ```no_run
    /// use clipper_client::ClipperClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClipperClient::new("http://localhost:3000");
    /// let png_bytes = vec![0u8; 100]; // PNG file bytes
    ///
    /// let clip = client.upload_file_bytes(
    ///     png_bytes,
    ///     "image.png".to_string(),
    ///     vec!["image".to_string()],
    ///     None,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_file_bytes(
        &self,
        bytes: Vec<u8>,
        filename: String,
        tags: Vec<String>,
        additional_notes: Option<String>,
    ) -> Result<Clip> {
        self.upload_file_bytes_with_content(bytes, filename, tags, additional_notes, None)
            .await
    }

    /// Upload file bytes to create a clip with optional content override
    ///
    /// # Arguments
    /// * `bytes` - The file content as bytes
    /// * `filename` - The filename to use
    /// * `tags` - List of tags for the clip
    /// * `additional_notes` - Optional additional notes
    /// * `content` - Optional content override (e.g., full file path instead of filename)
    ///
    /// # Example
    /// ```no_run
    /// use clipper_client::ClipperClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClipperClient::new("http://localhost:3000");
    /// let png_bytes = vec![0u8; 100]; // PNG file bytes
    ///
    /// let clip = client.upload_file_bytes_with_content(
    ///     png_bytes,
    ///     "image.png".to_string(),
    ///     vec!["image".to_string()],
    ///     None,
    ///     Some("/path/to/image.png".to_string()),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_file_bytes_with_content(
        &self,
        bytes: Vec<u8>,
        filename: String,
        tags: Vec<String>,
        additional_notes: Option<String>,
        content: Option<String>,
    ) -> Result<Clip> {
        let url = format!("{}/clips/upload", self.base_url);

        let file_part = reqwest::multipart::Part::bytes(bytes).file_name(filename);

        let mut form = reqwest::multipart::Form::new().part("file", file_part);

        if !tags.is_empty() {
            form = form.text("tags", tags.join(","));
        }

        if let Some(notes) = additional_notes {
            form = form.text("additional_notes", notes);
        }

        if let Some(content_value) = content {
            form = form.text("content", content_value);
        }

        let response = self
            .apply_auth(self.client.post(&url).multipart(form))
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get a clip by ID
    ///
    /// # Arguments
    /// * `id` - The clip ID
    pub async fn get_clip(&self, id: &str) -> Result<Clip> {
        let url = format!("{}/clips/{}", self.base_url, id);
        let response = self.apply_auth(self.client.get(&url)).send().await?;

        self.handle_response(response).await
    }

    /// Update a clip's tags and/or additional notes
    ///
    /// # Arguments
    /// * `id` - The clip ID
    /// * `tags` - Optional new tags
    /// * `additional_notes` - Optional new additional notes
    pub async fn update_clip(
        &self,
        id: &str,
        tags: Option<Vec<String>>,
        additional_notes: Option<String>,
    ) -> Result<Clip> {
        let url = format!("{}/clips/{}", self.base_url, id);
        let request = UpdateClipRequest {
            tags,
            additional_notes,
        };

        let response = self
            .apply_auth(self.client.put(&url).json(&request))
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Search clips with optional filters and paging
    ///
    /// # Arguments
    /// * `query` - Search query string
    /// * `filters` - Optional filters (date range, tags)
    /// * `page` - Page number (starting from 1)
    /// * `page_size` - Number of items per page
    pub async fn search_clips(
        &self,
        query: &str,
        filters: SearchFilters,
        page: usize,
        page_size: usize,
    ) -> Result<PagedResult> {
        let mut url = Url::parse(&format!("{}/clips/search", self.base_url))?;

        url.query_pairs_mut().append_pair("q", query);
        url.query_pairs_mut().append_pair("page", &page.to_string());
        url.query_pairs_mut()
            .append_pair("page_size", &page_size.to_string());

        if let Some(start_date) = filters.start_date {
            url.query_pairs_mut()
                .append_pair("start_date", &start_date.to_rfc3339());
        }

        if let Some(end_date) = filters.end_date {
            url.query_pairs_mut()
                .append_pair("end_date", &end_date.to_rfc3339());
        }

        if let Some(tags) = filters.tags {
            url.query_pairs_mut().append_pair("tags", &tags.join(","));
        }

        let response = self.apply_auth(self.client.get(url)).send().await?;

        self.handle_response(response).await
    }

    /// List all clips with optional filters and paging
    ///
    /// # Arguments
    /// * `filters` - Optional filters (date range, tags)
    /// * `page` - Page number (starting from 1)
    /// * `page_size` - Number of items per page
    pub async fn list_clips(
        &self,
        filters: SearchFilters,
        page: usize,
        page_size: usize,
    ) -> Result<PagedResult> {
        let mut url = Url::parse(&format!("{}/clips", self.base_url))?;

        url.query_pairs_mut().append_pair("page", &page.to_string());
        url.query_pairs_mut()
            .append_pair("page_size", &page_size.to_string());

        if let Some(start_date) = filters.start_date {
            url.query_pairs_mut()
                .append_pair("start_date", &start_date.to_rfc3339());
        }

        if let Some(end_date) = filters.end_date {
            url.query_pairs_mut()
                .append_pair("end_date", &end_date.to_rfc3339());
        }

        if let Some(tags) = filters.tags {
            url.query_pairs_mut().append_pair("tags", &tags.join(","));
        }

        let response = self.apply_auth(self.client.get(url)).send().await?;

        self.handle_response(response).await
    }

    /// Download a clip's file attachment as bytes
    ///
    /// # Arguments
    /// * `id` - The clip ID
    pub async fn download_file(&self, id: &str) -> Result<Vec<u8>> {
        let url = format!("{}/clips/{}/file", self.base_url, id);
        let response = self.apply_auth(self.client.get(&url)).send().await?;

        match response.status() {
            StatusCode::OK => {
                let bytes = response.bytes().await?;
                Ok(bytes.to_vec())
            }
            StatusCode::NOT_FOUND => Err(ClientError::NotFound(format!(
                "File not found for clip {}",
                id
            ))),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(ClientError::ServerError {
                    status: status.as_u16(),
                    message: error_text,
                })
            }
        }
    }

    /// Delete a clip by ID
    ///
    /// # Arguments
    /// * `id` - The clip ID
    pub async fn delete_clip(&self, id: &str) -> Result<()> {
        let url = format!("{}/clips/{}", self.base_url, id);
        let response = self.apply_auth(self.client.delete(&url)).send().await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::NOT_FOUND => Err(ClientError::NotFound(format!("Clip {} not found", id))),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(ClientError::ServerError {
                    status: status.as_u16(),
                    message: error_text,
                })
            }
        }
    }

    /// Connect to the server's WebSocket endpoint and receive real-time notifications
    ///
    /// # Arguments
    /// * `channel` - A tokio mpsc sender to push notifications to
    ///
    /// # Returns
    /// A task handle that runs the WebSocket connection
    pub async fn subscribe_notifications(
        &self,
        channel: mpsc::UnboundedSender<ClipNotification>,
    ) -> Result<tokio::task::JoinHandle<Result<()>>> {
        let ws_url = self
            .base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let ws_url = format!("{}/ws", ws_url);

        let (ws_stream, _) = self.connect_websocket(&ws_url).await?;

        let (mut write, mut read) = ws_stream.split();

        // If we have a token, send auth message and wait for response
        if let Some(token) = &self.token {
            let auth_msg = WsAuthRequest::Auth {
                token: token.clone(),
            };
            let auth_json = serde_json::to_string(&auth_msg)
                .map_err(|e| ClientError::WebSocket(format!("Failed to serialize auth: {}", e)))?;

            write
                .send(Message::Text(auth_json.into()))
                .await
                .map_err(|e| ClientError::WebSocket(format!("Failed to send auth: {}", e)))?;

            // Wait for auth response with timeout
            let auth_timeout = Duration::from_secs(10);
            let auth_result = tokio::time::timeout(auth_timeout, async {
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            // Try to parse as auth response
                            match serde_json::from_str::<WsAuthResponse>(&text) {
                                Ok(WsAuthResponse::AuthSuccess) => {
                                    return Ok(());
                                }
                                Ok(WsAuthResponse::AuthError { message }) => {
                                    return Err(ClientError::WebSocket(format!(
                                        "WebSocket auth failed: {}",
                                        message
                                    )));
                                }
                                Err(_) => {
                                    // Not an auth response, this shouldn't happen during auth phase
                                    return Err(ClientError::WebSocket(
                                        "Unexpected message during auth".to_string(),
                                    ));
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            return Err(ClientError::WebSocket(
                                "Connection closed during auth".to_string(),
                            ));
                        }
                        Ok(Message::Ping(_) | Message::Pong(_)) => {
                            // Ignore ping/pong during auth
                            continue;
                        }
                        Err(e) => {
                            return Err(ClientError::WebSocket(format!(
                                "WebSocket error during auth: {}",
                                e
                            )));
                        }
                        _ => continue,
                    }
                }
                Err(ClientError::WebSocket(
                    "Connection closed before auth response".to_string(),
                ))
            })
            .await;

            match auth_result {
                Ok(Ok(())) => {
                    // Auth successful, continue
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    return Err(ClientError::WebSocket("Auth timeout".to_string()));
                }
            }
        }

        let handle = tokio::spawn(async move {
            loop {
                // Use timeout to detect stale connections
                // Server sends ping every 30s, so we should receive something within 60s
                let msg = tokio::time::timeout(CONNECTION_TIMEOUT, read.next()).await;

                match msg {
                    Ok(Some(Ok(Message::Text(text)))) => {
                        match serde_json::from_str::<ClipNotification>(&text) {
                            Ok(notification) => {
                                if channel.send(notification).is_err() {
                                    // Channel closed, exit loop
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse notification: {}", e);
                            }
                        }
                    }
                    Ok(Some(Ok(Message::Ping(data)))) => {
                        // Respond to ping with pong to keep connection alive
                        if write.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Ok(Some(Ok(Message::Pong(_)))) => {
                        // Server responded to our ping (if we sent one), connection is alive
                    }
                    Ok(Some(Ok(Message::Close(_)))) => {
                        break;
                    }
                    Ok(Some(Err(e))) => {
                        return Err(ClientError::WebSocket(e.to_string()));
                    }
                    Ok(None) => {
                        // Stream ended
                        break;
                    }
                    Err(_) => {
                        // Timeout - no message received within CONNECTION_TIMEOUT
                        eprintln!("WebSocket connection timeout - no messages received");
                        return Err(ClientError::WebSocket(
                            "Connection timeout - no heartbeat received".to_string(),
                        ));
                    }
                    _ => {}
                }
            }
            Ok(())
        });

        Ok(handle)
    }

    /// Connect to a WebSocket URL with proper TLS handling
    ///
    /// Note: Authentication is handled via message-based auth after connection,
    /// not via headers, since WebSocket doesn't reliably support Authorization headers.
    async fn connect_websocket(
        &self,
        url: &str,
    ) -> Result<(
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>,
    )> {
        use tokio_tungstenite::tungstenite::client::IntoClientRequest;

        let parsed_url = url
            .parse::<Url>()
            .map_err(|e| ClientError::WebSocket(format!("Invalid URL: {}", e)))?;

        let is_secure = parsed_url.scheme() == "wss";

        // Create a WebSocket request from the URL (this handles all required WS headers)
        let request = url
            .into_client_request()
            .map_err(|e| ClientError::WebSocket(format!("Failed to build request: {}", e)))?;

        // Note: We don't add Authorization header here because WebSocket
        // doesn't reliably support it. Auth is done via message after connection.

        if is_secure {
            // For WSS connections, use a custom TLS connector
            use tokio_tungstenite::Connector;

            // Build rustls config
            let mut root_store = rustls::RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            #[cfg(feature = "danger-accept-invalid-certs")]
            let config = {
                // Accept any certificate (for development with self-signed certs)
                let config = rustls::ClientConfig::builder()
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(NoVerifier))
                    .with_no_client_auth();
                Arc::new(config)
            };

            #[cfg(not(feature = "danger-accept-invalid-certs"))]
            let config = {
                // Use proper certificate validation with system roots
                let config = rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();
                Arc::new(config)
            };

            let connector = Connector::Rustls(config);

            tokio_tungstenite::connect_async_tls_with_config(request, None, false, Some(connector))
                .await
                .map_err(|e| ClientError::WebSocket(e.to_string()))
        } else {
            // For WS connections, use the simple connect_async
            tokio_tungstenite::connect_async(request)
                .await
                .map_err(|e| ClientError::WebSocket(e.to_string()))
        }
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                let data = response.json().await?;
                Ok(data)
            }
            StatusCode::NOT_FOUND => {
                let error_text = response.text().await.unwrap_or_default();
                Err(ClientError::NotFound(error_text))
            }
            StatusCode::BAD_REQUEST => {
                let error_text = response.text().await.unwrap_or_default();
                Err(ClientError::BadRequest(error_text))
            }
            StatusCode::UNAUTHORIZED => {
                let error_text = response.text().await.unwrap_or_default();
                Err(ClientError::Unauthorized(error_text))
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(ClientError::ServerError {
                    status: status.as_u16(),
                    message: error_text,
                })
            }
        }
    }
}

/// Certificate verifier that accepts any certificate (for development only)
#[cfg(feature = "danger-accept-invalid-certs")]
#[derive(Debug)]
struct NoVerifier;

#[cfg(feature = "danger-accept-invalid-certs")]
impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

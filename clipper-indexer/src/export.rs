//! Export and import functionality for clipper data.
//!
//! This module provides functions to export all clipboard entries and their attachments
//! to a tar.gz archive, and to import from such an archive with deduplication.

use crate::error::{IndexerError, Result};
use crate::models::ClipboardEntry;
use chrono::{DateTime, Utc};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};

const MIB: u64 = 1024 * 1024;

/// Metadata for an exported clip, stored in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedClip {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_notes: Option<String>,
    /// The original filename of the attachment (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,
    /// Optional language identifier for the clip content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// The path within the archive where the file attachment is stored (if any)
    /// Format: "files/{id}_{original_filename}" or "files/{id}" if no original filename
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_path: Option<String>,
}

impl From<ClipboardEntry> for ExportedClip {
    fn from(entry: ClipboardEntry) -> Self {
        let attachment_path =
            entry
                .file_attachment
                .as_ref()
                .map(|_| match &entry.original_filename {
                    Some(filename) => format!("files/{}_{}", entry.id, filename),
                    None => format!("files/{}", entry.id),
                });

        Self {
            id: entry.id,
            content: entry.content,
            created_at: entry.created_at,
            tags: entry.tags,
            additional_notes: entry.additional_notes,
            original_filename: entry.original_filename,
            language: entry.language,
            attachment_path,
        }
    }
}

/// Manifest file that lists all clips in the archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportManifest {
    /// Version of the export format
    pub version: u32,
    /// When the export was created
    pub exported_at: DateTime<Utc>,
    /// Total number of clips in the export
    pub clip_count: usize,
    /// Total number of file attachments
    pub attachment_count: usize,
    /// List of all exported clips
    pub clips: Vec<ExportedClip>,
}

impl ExportManifest {
    pub const CURRENT_VERSION: u32 = 1;
    pub const MANIFEST_FILENAME: &'static str = "manifest.json";

    pub fn new(clips: Vec<ExportedClip>) -> Self {
        let attachment_count = clips.iter().filter(|c| c.attachment_path.is_some()).count();
        Self {
            version: Self::CURRENT_VERSION,
            exported_at: Utc::now(),
            clip_count: clips.len(),
            attachment_count,
            clips,
        }
    }
}

/// Result of an import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Number of clips imported
    pub imported_count: usize,
    /// Number of clips skipped (already existed)
    pub skipped_count: usize,
    /// Number of file attachments imported
    pub attachments_imported: usize,
    /// IDs of newly imported clips
    pub imported_ids: Vec<String>,
    /// IDs of skipped clips (duplicates)
    pub skipped_ids: Vec<String>,
}

/// Builder for creating export archives
pub struct ExportBuilder {
    clips: Vec<(ExportedClip, Option<bytes::Bytes>)>,
}

impl ExportBuilder {
    pub fn new() -> Self {
        Self { clips: Vec::new() }
    }

    /// Add a clip to the export, with optional file attachment content
    pub fn add_clip(&mut self, clip: ExportedClip, attachment_content: Option<bytes::Bytes>) {
        self.clips.push((clip, attachment_content));
    }

    /// Build the tar.gz archive and write it to a file
    ///
    /// This is more memory-efficient for large archives as it writes directly
    /// to disk instead of building the entire archive in memory.
    pub fn build_to_file<P: AsRef<Path>>(self, path: P) -> Result<()> {
        let file = File::create(path.as_ref())?;
        let writer = BufWriter::new(file);
        self.build_to_writer(writer)?;
        Ok(())
    }

    /// Build the tar.gz archive and write it to any writer
    fn build_to_writer<W: Write>(self, writer: W) -> Result<()> {
        let encoder = GzEncoder::new(writer, Compression::default());
        let mut builder = Builder::new(encoder);

        // Create manifest
        let exported_clips: Vec<ExportedClip> = self.clips.iter().map(|(c, _)| c.clone()).collect();
        let manifest = ExportManifest::new(exported_clips);
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| IndexerError::Serialization(e.to_string()))?;

        // Add manifest to archive
        let manifest_bytes = manifest_json.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_size(manifest_bytes.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(Utc::now().timestamp() as u64);
        // Use append_data to handle the path - it automatically handles long paths
        // via GNU long-name extension if needed
        builder.append_data(
            &mut header,
            ExportManifest::MANIFEST_FILENAME,
            manifest_bytes,
        )?;

        // Add file attachments
        for (clip, attachment_content) in &self.clips {
            if let (Some(attachment_path), Some(content)) =
                (&clip.attachment_path, attachment_content)
            {
                let mut header = tar::Header::new_gnu();
                header.set_size(content.len() as u64);
                header.set_mode(0o644);
                header.set_mtime(clip.created_at.timestamp() as u64);
                // Use append_data to handle paths that may exceed 100 bytes
                // (e.g., files/{id}_{original_filename} with long filenames)
                builder.append_data(&mut header, attachment_path, content.as_ref())?;
            }
        }

        // Finish the archive
        let encoder = builder.into_inner()?;
        encoder.finish()?;

        Ok(())
    }
}

impl Default for ExportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Limits applied while parsing import archives.
#[derive(Debug, Clone, Copy)]
pub struct ImportLimits {
    /// Maximum number of tar entries, including manifest, attachments, and unknown entries.
    pub max_entries: usize,
    /// Maximum number of `files/*` attachment entries.
    pub max_file_entries: usize,
    /// Maximum allowed size for `manifest.json`.
    pub max_manifest_size_bytes: u64,
    /// Maximum allowed size for one attachment entry.
    pub max_attachment_size_bytes: u64,
    /// Maximum cumulative uncompressed size across all entries.
    pub max_uncompressed_size_bytes: u64,
}

impl ImportLimits {
    pub fn from_max_upload_size(max_upload_size_bytes: u64) -> Self {
        let max_upload_size_bytes = max_upload_size_bytes.max(1);
        Self {
            max_attachment_size_bytes: max_upload_size_bytes,
            max_uncompressed_size_bytes: max_upload_size_bytes
                .saturating_mul(4)
                .max(max_upload_size_bytes),
            ..Self::default()
        }
    }
}

impl Default for ImportLimits {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_file_entries: 10_000,
            max_manifest_size_bytes: 16 * MIB,
            max_attachment_size_bytes: 100 * MIB,
            max_uncompressed_size_bytes: 512 * MIB,
        }
    }
}

/// A parsed attachment stored on disk while the import parser is alive.
#[derive(Debug, Clone)]
pub struct ImportAttachment {
    path: PathBuf,
    size_bytes: u64,
}

impl ImportAttachment {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }
}

/// Parser for reading import archives.
pub struct ImportParser {
    manifest: ExportManifest,
    files: HashMap<String, ImportAttachment>,
    _temp_dir: tempfile::TempDir,
}

impl ImportParser {
    /// Parse a tar.gz archive from a reader.
    fn parse_archive<R: Read>(reader: R, limits: ImportLimits) -> Result<Self> {
        let decoder = GzDecoder::new(reader);
        let mut archive = Archive::new(decoder);

        let temp_dir = tempfile::TempDir::new()?;
        let mut manifest: Option<ExportManifest> = None;
        let mut files = HashMap::new();
        let mut entry_count = 0_usize;
        let mut file_count = 0_usize;
        let mut total_uncompressed_size = 0_u64;

        for entry_result in archive.entries()? {
            entry_count += 1;
            if entry_count > limits.max_entries {
                return Err(IndexerError::PayloadTooLarge(format!(
                    "Import archive contains too many entries (limit: {})",
                    limits.max_entries
                )));
            }

            let mut entry = entry_result?;
            let entry_size = entry.size();
            total_uncompressed_size =
                total_uncompressed_size
                    .checked_add(entry_size)
                    .ok_or_else(|| {
                        IndexerError::PayloadTooLarge(
                            "Import archive uncompressed size overflowed".to_string(),
                        )
                    })?;
            if total_uncompressed_size > limits.max_uncompressed_size_bytes {
                return Err(IndexerError::PayloadTooLarge(format!(
                    "Import archive uncompressed size exceeds limit of {} bytes",
                    limits.max_uncompressed_size_bytes
                )));
            }

            let path = entry.path()?.to_string_lossy().to_string();

            if path == ExportManifest::MANIFEST_FILENAME {
                if entry_size > limits.max_manifest_size_bytes {
                    return Err(IndexerError::PayloadTooLarge(format!(
                        "Import manifest exceeds limit of {} bytes",
                        limits.max_manifest_size_bytes
                    )));
                }

                let mut content = String::with_capacity(entry_size as usize);
                entry.read_to_string(&mut content)?;
                manifest = Some(
                    serde_json::from_str(&content)
                        .map_err(|e| IndexerError::Serialization(e.to_string()))?,
                );
            } else if path.starts_with("files/") {
                file_count += 1;
                if file_count > limits.max_file_entries {
                    return Err(IndexerError::PayloadTooLarge(format!(
                        "Import archive contains too many attachments (limit: {})",
                        limits.max_file_entries
                    )));
                }

                if entry_size > limits.max_attachment_size_bytes {
                    return Err(IndexerError::PayloadTooLarge(format!(
                        "Import attachment '{}' exceeds limit of {} bytes",
                        path, limits.max_attachment_size_bytes
                    )));
                }

                let attachment_path = temp_dir.path().join(format!("attachment-{}", file_count));
                let mut attachment_file = File::create(&attachment_path)?;
                let bytes_written = std::io::copy(&mut entry, &mut attachment_file)?;
                if bytes_written != entry_size {
                    return Err(IndexerError::InvalidInput(format!(
                        "Attachment '{}' ended early while reading import archive",
                        path
                    )));
                }

                files.insert(
                    path,
                    ImportAttachment {
                        path: attachment_path,
                        size_bytes: entry_size,
                    },
                );
            }
        }

        let manifest = manifest.ok_or_else(|| {
            IndexerError::InvalidInput("Archive missing manifest.json".to_string())
        })?;

        // Validate manifest version
        if manifest.version > ExportManifest::CURRENT_VERSION {
            return Err(IndexerError::InvalidInput(format!(
                "Unsupported export format version: {}. Maximum supported: {}",
                manifest.version,
                ExportManifest::CURRENT_VERSION
            )));
        }

        Ok(Self {
            manifest,
            files,
            _temp_dir: temp_dir,
        })
    }

    /// Parse a tar.gz archive from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::from_bytes_with_limits(data, ImportLimits::default())
    }

    /// Parse a tar.gz archive from bytes using explicit limits.
    pub fn from_bytes_with_limits(data: &[u8], limits: ImportLimits) -> Result<Self> {
        Self::parse_archive(data, limits)
    }

    /// Parse a tar.gz archive from a file path.
    ///
    /// This is more memory-efficient for large archives as it streams from disk
    /// instead of requiring the entire archive to be loaded into memory first.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_file_with_limits(path, ImportLimits::default())
    }

    /// Parse a tar.gz archive from a file path using explicit limits.
    pub fn from_file_with_limits<P: AsRef<Path>>(path: P, limits: ImportLimits) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        Self::parse_archive(reader, limits)
    }

    /// Get the manifest.
    pub fn manifest(&self) -> &ExportManifest {
        &self.manifest
    }

    /// Get the list of clips from the manifest.
    pub fn clips(&self) -> &[ExportedClip] {
        &self.manifest.clips
    }

    /// Get the file attachment content for a clip by its attachment path.
    ///
    /// This reads a single attachment from its temporary file. Import code should prefer
    /// `get_attachment_file` to avoid loading attachment bytes into memory.
    pub fn get_attachment(&self, attachment_path: &str) -> Option<bytes::Bytes> {
        self.files
            .get(attachment_path)
            .and_then(|attachment| std::fs::read(&attachment.path).ok().map(bytes::Bytes::from))
    }

    /// Get the temporary file for an attachment.
    pub fn get_attachment_file(&self, attachment_path: &str) -> Option<&ImportAttachment> {
        self.files.get(attachment_path)
    }

    /// Get all parsed file attachments.
    pub fn attachments(&self) -> &HashMap<String, ImportAttachment> {
        &self.files
    }
}

/// Deduplication helper - checks if a clip should be imported based on content hash
pub fn should_import_clip(
    clip: &ExportedClip,
    existing_ids: &HashSet<String>,
    existing_content_hashes: &HashSet<u64>,
) -> bool {
    // Skip if ID already exists
    if existing_ids.contains(&clip.id) {
        return false;
    }

    // Skip if content hash already exists (dedup by content)
    let content_hash = calculate_content_hash(clip);
    if existing_content_hashes.contains(&content_hash) {
        return false;
    }

    true
}

/// Calculate a hash for deduplication purposes
/// Uses content + created_at + tags as the basis for deduplication
pub fn calculate_content_hash(clip: &ExportedClip) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    clip.content.hash(&mut hasher);
    clip.created_at.timestamp().hash(&mut hasher);
    for tag in &clip.tags {
        tag.hash(&mut hasher);
    }
    if let Some(notes) = &clip.additional_notes {
        notes.hash(&mut hasher);
    }
    if let Some(filename) = &clip.original_filename {
        filename.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn build_archive_bytes(builder: ExportBuilder) -> Vec<u8> {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        builder
            .build_to_file(temp_file.path())
            .expect("Failed to build archive");
        std::fs::read(temp_file.path()).expect("Failed to read archive")
    }

    #[test]
    fn test_export_builder_creates_valid_archive() {
        let clip = ExportedClip {
            id: "test123".to_string(),
            content: "Hello, World!".to_string(),
            created_at: Utc::now(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            additional_notes: Some("Some notes".to_string()),
            original_filename: None,
            language: None,
            attachment_path: None,
        };

        let mut builder = ExportBuilder::new();
        builder.add_clip(clip, None);

        // Write to temp file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();
        builder
            .build_to_file(temp_path)
            .expect("Failed to build archive");

        // Verify file is not empty
        let metadata = std::fs::metadata(temp_path).expect("Failed to get metadata");
        assert!(metadata.len() > 0);

        // Parse it back from file
        let parser = ImportParser::from_file(temp_path).expect("Failed to parse archive");
        assert_eq!(parser.manifest().clip_count, 1);
        assert_eq!(parser.clips()[0].id, "test123");
    }

    #[test]
    fn test_export_with_attachment() {
        let clip = ExportedClip {
            id: "test456".to_string(),
            content: "File content".to_string(),
            created_at: Utc::now(),
            tags: vec![],
            additional_notes: None,
            original_filename: Some("test.txt".to_string()),
            language: None,
            attachment_path: Some("files/test456_test.txt".to_string()),
        };

        let attachment = bytes::Bytes::from("This is the file content");

        let mut builder = ExportBuilder::new();
        builder.add_clip(clip, Some(attachment.clone()));

        // Write to temp file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();
        builder
            .build_to_file(temp_path)
            .expect("Failed to build archive");

        // Parse it back from file
        let parser = ImportParser::from_file(temp_path).expect("Failed to parse archive");
        assert_eq!(parser.manifest().attachment_count, 1);

        let retrieved = parser
            .get_attachment("files/test456_test.txt")
            .expect("Attachment not found");
        assert_eq!(retrieved, attachment);

        let attachment_file = parser
            .get_attachment_file("files/test456_test.txt")
            .expect("Attachment file not found");
        assert_eq!(attachment_file.size_bytes(), attachment.len() as u64);
        assert!(attachment_file.path().exists());
    }

    #[test]
    fn test_content_hash_deduplication() {
        let clip1 = ExportedClip {
            id: "id1".to_string(),
            content: "Same content".to_string(),
            created_at: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            tags: vec!["tag".to_string()],
            additional_notes: None,
            original_filename: None,
            language: None,
            attachment_path: None,
        };

        let clip2 = ExportedClip {
            id: "id2".to_string(), // Different ID
            content: "Same content".to_string(),
            created_at: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            tags: vec!["tag".to_string()],
            additional_notes: None,
            original_filename: None,
            language: None,
            attachment_path: None,
        };

        let hash1 = calculate_content_hash(&clip1);
        let hash2 = calculate_content_hash(&clip2);

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_export_with_long_filename() {
        // Create a filename that exceeds the 100-byte tar path limit
        // The path format is "files/{id}_{original_filename}"
        // With id = 36 chars (UUID) + "files/" (6) + "_" (1) = 43 chars prefix
        // So we need a filename > 57 chars to exceed 100 bytes
        let long_filename = "a".repeat(100) + ".txt"; // 104 chars

        let clip = ExportedClip {
            id: "12345678-1234-1234-1234-123456789012".to_string(),
            content: "File with long name".to_string(),
            created_at: Utc::now(),
            tags: vec![],
            additional_notes: None,
            original_filename: Some(long_filename.clone()),
            language: None,
            attachment_path: Some(format!(
                "files/12345678-1234-1234-1234-123456789012_{}",
                long_filename
            )),
        };

        let attachment = bytes::Bytes::from("Long filename content");

        let mut builder = ExportBuilder::new();
        builder.add_clip(clip.clone(), Some(attachment.clone()));

        // Write to temp file - this should not fail with "path too long" error
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();
        builder
            .build_to_file(temp_path)
            .expect("Failed to build archive with long filename");

        // Verify we can parse it back from file
        let parser = ImportParser::from_file(temp_path).expect("Failed to parse archive");
        assert_eq!(parser.manifest().clip_count, 1);
        assert_eq!(parser.manifest().attachment_count, 1);

        // Verify the attachment can be retrieved
        let retrieved = parser
            .get_attachment(&clip.attachment_path.unwrap())
            .expect("Attachment not found");
        assert_eq!(retrieved, attachment);
    }

    #[test]
    fn test_import_rejects_too_many_entries() {
        let clip = ExportedClip {
            id: "entry-limit".to_string(),
            content: "Entry limit".to_string(),
            created_at: Utc::now(),
            tags: vec![],
            additional_notes: None,
            original_filename: None,
            language: None,
            attachment_path: None,
        };

        let mut builder = ExportBuilder::new();
        builder.add_clip(clip, None);
        let archive = build_archive_bytes(builder);

        let limits = ImportLimits {
            max_entries: 0,
            ..ImportLimits::default()
        };
        let err = match ImportParser::from_bytes_with_limits(&archive, limits) {
            Ok(_) => panic!("Expected import to fail"),
            Err(err) => err,
        };

        assert!(matches!(err, IndexerError::PayloadTooLarge(_)));
    }

    #[test]
    fn test_import_rejects_oversized_attachment() {
        let clip = ExportedClip {
            id: "attachment-limit".to_string(),
            content: "Attachment limit".to_string(),
            created_at: Utc::now(),
            tags: vec![],
            additional_notes: None,
            original_filename: Some("limit.txt".to_string()),
            language: None,
            attachment_path: Some("files/attachment-limit_limit.txt".to_string()),
        };

        let mut builder = ExportBuilder::new();
        builder.add_clip(clip, Some(bytes::Bytes::from_static(b"large")));
        let archive = build_archive_bytes(builder);

        let limits = ImportLimits {
            max_attachment_size_bytes: 4,
            ..ImportLimits::default()
        };
        let err = match ImportParser::from_bytes_with_limits(&archive, limits) {
            Ok(_) => panic!("Expected import to fail"),
            Err(err) => err,
        };

        assert!(matches!(err, IndexerError::PayloadTooLarge(_)));
    }

    #[test]
    fn test_import_rejects_excessive_uncompressed_size() {
        let clip = ExportedClip {
            id: "total-limit".to_string(),
            content: "Total limit".to_string(),
            created_at: Utc::now(),
            tags: vec![],
            additional_notes: None,
            original_filename: None,
            language: None,
            attachment_path: None,
        };

        let mut builder = ExportBuilder::new();
        builder.add_clip(clip, None);
        let archive = build_archive_bytes(builder);

        let limits = ImportLimits {
            max_uncompressed_size_bytes: 8,
            ..ImportLimits::default()
        };
        let err = match ImportParser::from_bytes_with_limits(&archive, limits) {
            Ok(_) => panic!("Expected import to fail"),
            Err(err) => err,
        };

        assert!(matches!(err, IndexerError::PayloadTooLarge(_)));
    }
}

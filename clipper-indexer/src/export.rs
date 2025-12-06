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
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use tar::{Archive, Builder};

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

/// Parser for reading import archives
pub struct ImportParser {
    manifest: ExportManifest,
    files: std::collections::HashMap<String, bytes::Bytes>,
}

impl ImportParser {
    /// Parse a tar.gz archive from a reader
    fn parse_archive<R: Read>(reader: R) -> Result<Self> {
        let decoder = GzDecoder::new(reader);
        let mut archive = Archive::new(decoder);

        let mut manifest: Option<ExportManifest> = None;
        let mut files = std::collections::HashMap::new();

        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?.to_string_lossy().to_string();

            if path == ExportManifest::MANIFEST_FILENAME {
                let mut content = String::new();
                entry.read_to_string(&mut content)?;
                manifest = Some(
                    serde_json::from_str(&content)
                        .map_err(|e| IndexerError::Serialization(e.to_string()))?,
                );
            } else if path.starts_with("files/") {
                let mut content = Vec::new();
                entry.read_to_end(&mut content)?;
                files.insert(path, bytes::Bytes::from(content));
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

        Ok(Self { manifest, files })
    }

    /// Parse a tar.gz archive from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::parse_archive(data)
    }

    /// Parse a tar.gz archive from a file path
    ///
    /// This is more memory-efficient for large archives as it streams from disk
    /// instead of requiring the entire archive to be loaded into memory first.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        Self::parse_archive(reader)
    }

    /// Get the manifest
    pub fn manifest(&self) -> &ExportManifest {
        &self.manifest
    }

    /// Get the list of clips from the manifest
    pub fn clips(&self) -> &[ExportedClip] {
        &self.manifest.clips
    }

    /// Get the file attachment content for a clip by its attachment path
    pub fn get_attachment(&self, attachment_path: &str) -> Option<bytes::Bytes> {
        self.files.get(attachment_path).cloned()
    }

    /// Get all file attachments
    pub fn attachments(&self) -> &std::collections::HashMap<String, bytes::Bytes> {
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

    #[test]
    fn test_export_builder_creates_valid_archive() {
        let clip = ExportedClip {
            id: "test123".to_string(),
            content: "Hello, World!".to_string(),
            created_at: Utc::now(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            additional_notes: Some("Some notes".to_string()),
            original_filename: None,
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
}

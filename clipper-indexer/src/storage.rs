use crate::error::{IndexerError, Result};
use bytes::Bytes;
use object_store::{local::LocalFileSystem, path::Path as ObjectPath, ObjectStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct FileStorage {
    store: Arc<LocalFileSystem>,
    base_path: PathBuf,
}

impl FileStorage {
    #[allow(clippy::result_large_err)]
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        // Create the directory if it doesn't exist
        std::fs::create_dir_all(&base_path)?;

        let store =
            LocalFileSystem::new_with_prefix(&base_path).map_err(IndexerError::ObjectStore)?;

        Ok(Self {
            store: Arc::new(store),
            base_path,
        })
    }

    pub async fn put_file(&self, source_path: impl AsRef<Path>) -> Result<String> {
        let source_path = source_path.as_ref();

        // Read the file content
        let content = tokio::fs::read(source_path).await?;

        // Generate a unique filename using UUID
        let file_name = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| IndexerError::InvalidInput("Invalid file name".to_string()))?;

        let unique_id = uuid::Uuid::new_v4();
        let extension = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let stored_file_name = if extension.is_empty() {
            format!("{}_{}", unique_id, file_name)
        } else {
            format!(
                "{}_{}.{}",
                unique_id,
                file_name.trim_end_matches(&format!(".{}", extension)),
                extension
            )
        };

        let object_path = ObjectPath::from(stored_file_name.as_str());

        // Store the file
        self.store
            .put(&object_path, Bytes::from(content).into())
            .await
            .map_err(IndexerError::ObjectStore)?;

        Ok(stored_file_name)
    }

    pub async fn put_file_bytes(&self, content: Bytes, original_filename: &str) -> Result<String> {
        // Generate a unique filename using UUID
        let unique_id = uuid::Uuid::new_v4();

        // Extract extension from original filename
        let extension = Path::new(original_filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let base_name = Path::new(original_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        let stored_file_name = if extension.is_empty() {
            format!("{}_{}", unique_id, base_name)
        } else {
            format!("{}_{}.{}", unique_id, base_name, extension)
        };

        let object_path = ObjectPath::from(stored_file_name.as_str());

        // Store the file
        self.store
            .put(&object_path, content.into())
            .await
            .map_err(IndexerError::ObjectStore)?;

        Ok(stored_file_name)
    }

    pub async fn get_file(&self, file_key: &str) -> Result<Bytes> {
        let object_path = ObjectPath::from(file_key);

        let result = self
            .store
            .get(&object_path)
            .await
            .map_err(IndexerError::ObjectStore)?;

        let bytes = result.bytes().await.map_err(IndexerError::ObjectStore)?;

        Ok(bytes)
    }

    pub async fn delete_file(&self, file_key: &str) -> Result<()> {
        let object_path = ObjectPath::from(file_key);

        self.store
            .delete(&object_path)
            .await
            .map_err(IndexerError::ObjectStore)?;

        Ok(())
    }

    pub fn get_base_path(&self) -> &Path {
        &self.base_path
    }
}

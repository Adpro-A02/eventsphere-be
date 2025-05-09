use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::io;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use tracing::{debug, error};

use crate::error::AppError;
use crate::config::Config;

/// Interface for image storage operations
#[async_trait]
pub trait ImageStorage: Send + Sync {
    /// Save an image to storage and return its public URL
    async fn save_image(&self, path: &str, data: &[u8], extension: &str) -> Result<String, AppError>;
    
    /// Delete an image from storage
    async fn delete_image(&self, url: &str) -> Result<(), AppError>;
}

/// File system implementation of image storage
pub struct FileSystemImageStorage {
    uploads_dir: PathBuf,
    base_url: String,
}

impl FileSystemImageStorage {
    /// Create a new file system storage instance
    pub fn new(config: &Config) -> Self {
        Self {
            uploads_dir: PathBuf::from(&config.uploads_dir),
            base_url: config.media_base_url.clone(),
        }
    }
    
    /// Ensure the target directory exists
    async fn ensure_directory_exists(&self, path: &Path) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::Storage(format!("Failed to create directory: {}", e)))?;
        }
        Ok(())
    }
}

#[async_trait]
impl ImageStorage for FileSystemImageStorage {
    async fn save_image(&self, path: &str, data: &[u8], extension: &str) -> Result<String, AppError> {
        // Generate a unique filename
        let filename = format!("{}.{}", Uuid::new_v4(), extension);
        let file_path = self.uploads_dir.join(path).join(&filename);
        
        debug!("Saving image to: {:?}", file_path);
        
        // Ensure the directory exists
        self.ensure_directory_exists(&file_path).await?;
        
        // Write the file
        let mut file = File::create(&file_path)
            .await
            .map_err(|e| AppError::Storage(format!("Failed to create file: {}", e)))?;
            
        file.write_all(data)
            .await
            .map_err(|e| AppError::Storage(format!("Failed to write file: {}", e)))?;
            
        // Return the URL
        let url_path = format!("{}/{}/{}", self.base_url, path, filename);
        debug!("Image saved, URL: {}", url_path);
        
        Ok(url_path)
    }
    
    async fn delete_image(&self, url: &str) -> Result<(), AppError> {
        // Extract the path from the URL
        let base_url = &self.base_url;
        if !url.starts_with(base_url) {
            return Err(AppError::Validation(format!("Invalid image URL: {}", url)));
        }
        
        let path = url.trim_start_matches(base_url);
        let file_path = self.uploads_dir.join(path.trim_start_matches('/'));
        
        debug!("Deleting image at: {:?}", file_path);
        
        // Delete the file
        match fs::remove_file(&file_path).await {
            Ok(_) => {
                debug!("Image deleted successfully");
                Ok(())
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                debug!("Image file not found, considering delete successful");
                Ok(())
            },
            Err(e) => {
                error!("Failed to delete image: {}", e);
                Err(AppError::Storage(format!("Failed to delete file: {}", e)))
            }
        }
    }
}
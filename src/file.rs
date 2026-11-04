//! File system operations with safety guarantees
//! Follows: Defensive Programming, Fail-Fast, Error Handling

use std::path::{Path, PathBuf};

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_directory: bool,
    pub is_hidden: bool,
    pub modified: u64,
}

/// File operation result
pub type FileResult<T> = Result<T, FileError>;

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// File manager
pub struct FileManager {
    current_path: PathBuf,
}

impl FileManager {
    pub fn new(path: PathBuf) -> Self {
        FileManager {
            current_path: path,
        }
    }

    pub fn current_path(&self) -> &Path {
        &self.current_path
    }

    pub fn change_directory(&mut self, path: &Path) -> FileResult<()> {
        if !path.exists() {
            return Err(FileError::NotFound(path.display().to_string()));
        }
        if !path.is_dir() {
            return Err(FileError::NotFound("Not a directory".to_string()));
        }
        self.current_path = path.to_path_buf();
        Ok(())
    }

    pub fn list_directory(&self) -> FileResult<Vec<FileMetadata>> {
        let entries = std::fs::read_dir(&self.current_path)?;
        let mut files = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let is_hidden = name.starts_with('.');
            
            let file_meta = FileMetadata {
                name: name.clone(),
                path: entry.path(),
                size: metadata.len(),
                is_directory: metadata.is_dir(),
                is_hidden,
                modified: metadata.modified()
                    .unwrap()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            
            files.push(file_meta);
        }
        
        Ok(files)
    }

    pub fn create_directory(&self, name: &str) -> FileResult<()> {
        let path = self.current_path.join(name);
        std::fs::create_dir(&path)?;
        Ok(())
    }

    pub fn delete_file(&self, name: &str) -> FileResult<()> {
        let path = self.current_path.join(name);
        if !path.exists() {
            return Err(FileError::NotFound(name.to_string()));
        }
        std::fs::remove_file(&path)?;
        Ok(())
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_manager() {
        let fm = FileManager::default();
        let path = fm.current_path();
        assert!(path.exists());
    }
}

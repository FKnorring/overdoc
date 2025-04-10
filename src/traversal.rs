use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use crate::config::Config;

/// Represents a file found during repository traversal
#[derive(Debug, Clone)]
pub struct RepoFile {
    /// Full path to the file
    pub path: PathBuf,
    
    /// File extension (if any)
    pub extension: Option<String>,
    
    /// File size in bytes
    pub size: u64,
    
    /// Whether the file is in a directory that starts with a dot
    pub in_dot_directory: bool,
}

impl RepoFile {
    /// Creates a new RepoFile from a DirEntry
    fn from_entry(entry: &DirEntry) -> Result<Self> {
        let metadata = entry.metadata()
            .context("Failed to read file metadata")?;
        
        // Check if the file is in a dot directory
        let in_dot_directory = entry.path().components().any(|c| {
            if let std::path::Component::Normal(name) = c {
                if let Some(name_str) = name.to_str() {
                    return name_str.starts_with('.');
                }
            }
            false
        });
        
        // Get the file extension
        let extension = entry.path()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());
        
        Ok(RepoFile {
            path: entry.path().to_path_buf(),
            extension: extension.map(String::from),
            size: metadata.len(),
            in_dot_directory,
        })
    }
}

/// Traverse a repository and collect all files
pub fn traverse_repository(repo_path: &str, config: &Config) -> Result<Vec<RepoFile>> {
    let path = Path::new(repo_path);
    
    if !path.exists() {
        return Err(anyhow::anyhow!("Repository path does not exist: {}", repo_path));
    }
    
    if !path.is_dir() {
        return Err(anyhow::anyhow!("Repository path is not a directory: {}", repo_path));
    }
    
    info!("Starting repository traversal at: {}", repo_path);
    
    let walker = WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_ignored_by_default(e, config));
    
    let mut files = Vec::new();
    
    for entry in walker {
        let entry = entry.context("Error accessing directory entry")?;
        
        // Skip directories
        if entry.file_type().is_dir() {
            continue;
        }
        
        // Process files
        match RepoFile::from_entry(&entry) {
            Ok(file) => {
                debug!("Found file: {:?}", file.path);
                files.push(file);
            },
            Err(err) => {
                warn!("Error processing file {}: {}", entry.path().display(), err);
            }
        }
    }
    
    info!("Repository traversal complete. Found {} files", files.len());
    
    Ok(files)
}

/// Check if a directory entry should be ignored by default rules
fn is_ignored_by_default(entry: &DirEntry, config: &Config) -> bool {
    let path = entry.path();
    let file_name = entry.file_name().to_string_lossy();
    
    // Check if it's a directory to ignore
    if entry.file_type().is_dir() {
        for ignore_dir in &config.ignore_directories {
            // Exact match
            if file_name == ignore_dir.as_str() {
                debug!("Ignoring directory: {}", path.display());
                return true;
            }
        }
    }
    
    // Don't ignore by default
    false
} 
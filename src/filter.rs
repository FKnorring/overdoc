use log::{debug, info};
use std::path::Path;

use crate::config::Config;
use crate::traversal::RepoFile;

/// Apply configured filters to the list of files
pub fn apply_filters(files: Vec<RepoFile>, config: &Config) -> Vec<RepoFile> {
    info!("Applying filters to {} files", files.len());

    let filtered_files: Vec<RepoFile> = files
        .into_iter()
        .filter(|file| !should_ignore_file(file, config))
        .collect();

    info!("After filtering, {} files remain", filtered_files.len());

    filtered_files
}

/// Check if a file should be ignored based on configuration rules
fn should_ignore_file(file: &RepoFile, config: &Config) -> bool {
    let path = &file.path;
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let path_str = path.to_string_lossy().to_string();

    // Allow our own source files for development purposes
    if path_str.contains("src") && file.extension.as_deref() == Some("rs") {
        debug!(
            "Including Rust source file for analysis: {}",
            path.display()
        );
        return false;
    }

    // Special handling for external Python and TypeScript/JavaScript files
    if let Some(ext) = &file.extension {
        let ext_str = ext.as_str();
        if (ext_str == "py" || ext_str == "ts" || ext_str == "tsx" || ext_str == "js")
            && !path_str.contains("node_modules")
            && !path_str.contains("venv")
            && !path_str.contains(".venv")
        {
            debug!(
                "Including {} file for analysis: {}",
                ext_str.to_uppercase(),
                path.display()
            );
            return false;
        }
    }

    // Ignore files in dot directories (like .git)
    if file.in_dot_directory {
        debug!("Ignoring file in dot directory: {}", path.display());
        return true;
    }

    // Check file size limit
    if config.default_settings.max_file_size_kb > 0 {
        let size_kb = file.size / 1024;
        if size_kb > config.default_settings.max_file_size_kb as u64 {
            debug!("Ignoring large file ({}KB): {}", size_kb, path.display());
            return true;
        }
    }

    // Check global ignore patterns
    if matches_any_pattern(path, &config.ignore_patterns) {
        debug!("Ignoring file by global pattern: {}", path.display());
        return true;
    }

    // Check language-specific rules
    if let Some(ext) = &file.extension {
        // Find matching language config
        for (lang, lang_config) in &config.languages {
            if lang_config.extensions.iter().any(|e| e == ext) {
                debug!("File {} matches language: {}", path.display(), lang);

                // Check language-specific ignore files
                if lang_config.ignore_files.iter().any(|f| file_name == f) {
                    debug!("Ignoring language-specific file: {}", path.display());
                    return true;
                }

                // Check if file is in a language-specific ignored directory
                for ignore_dir in &lang_config.ignore_directories {
                    if path.components().any(|c| {
                        if let std::path::Component::Normal(name) = c {
                            if let Some(name_str) = name.to_str() {
                                return name_str == ignore_dir;
                            }
                        }
                        false
                    }) {
                        debug!(
                            "Ignoring file in language-specific directory: {}",
                            path.display()
                        );
                        return true;
                    }
                }
            }
        }
    } else if !config.default_settings.include_no_extension {
        // Ignore files with no extension if configured to do so
        debug!("Ignoring file with no extension: {}", path.display());
        return true;
    }

    // Don't ignore this file
    false
}

/// Check if a path matches any of the given patterns
fn matches_any_pattern(path: &Path, patterns: &[String]) -> bool {
    // Simplified pattern matching
    let path_str = path.to_string_lossy().to_string();

    for pattern in patterns {
        // Simple wildcard matching for now
        if pattern_matches(&path_str, pattern) {
            return true;
        }
    }

    false
}

/// Simple pattern matching implementation
fn pattern_matches(path: &str, pattern: &str) -> bool {
    // Very basic wildcard matching
    if pattern == "*" {
        return true;
    }

    if pattern.starts_with("*") && pattern.ends_with("*") {
        let inner = &pattern[1..pattern.len() - 1];
        return path.contains(inner);
    }

    if pattern.starts_with("*") {
        let suffix = &pattern[1..];
        return path.ends_with(suffix);
    }

    if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len() - 1];
        return path.starts_with(prefix);
    }

    // Exact match
    path == pattern
}

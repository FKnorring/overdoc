use anyhow::Result;
use log::{debug, info};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::traversal::RepoFile;

/// Represents an exported entity from a file
#[derive(Debug, Clone)]
pub struct ExportedEntity {
    /// Name of the exported entity
    pub name: String,

    /// Path to the file containing the export
    pub file_path: PathBuf,

    /// Line number where the export is defined
    pub line_number: usize,

    /// Type of export (e.g., function, class, variable)
    pub export_type: String,

    /// Usage count - how many times this export is referenced
    pub usage_count: usize,
}

/// Represents an import reference to an exported entity
#[derive(Debug, Clone)]
pub struct ImportReference {
    /// Name of the imported entity
    pub name: String,

    /// Path to the file that imports the entity
    pub file_path: PathBuf,

    /// Line number where the import occurs
    pub line_number: usize,

    /// Original import statement
    pub import_statement: String,
}

/// Map of file paths to sets of exported entities
pub type ExportsMap = HashMap<String, Vec<ExportedEntity>>;

/// Map of entity names to import references
pub type ImportsMap = HashMap<String, Vec<ImportReference>>;

/// Scan a repository for exports and imports
pub fn scan_repository(files: &[RepoFile], config: &Config) -> Result<(ExportsMap, ImportsMap)> {
    info!("Scanning repository for exports and imports");

    let mut exports_map: ExportsMap = HashMap::new();
    let mut imports_map: ImportsMap = HashMap::new();

    for file in files {
        if let Some(extension) = &file.extension {
            // Find the language config for this file
            for (lang_name, lang_config) in &config.languages {
                if lang_config.extensions.iter().any(|ext| ext == extension) {
                    debug!("Processing {} file: {}", lang_name, file.path.display());

                    // Read file content
                    let file_content = match fs::read_to_string(&file.path) {
                        Ok(content) => content,
                        Err(err) => {
                            debug!("Error reading file {}: {}", file.path.display(), err);
                            continue;
                        }
                    };

                    // Extract exports
                    let file_exports =
                        extract_exports(&file.path, &file_content, &lang_config.export_patterns);

                    // Store exports
                    if !file_exports.is_empty() {
                        let path_str = file.path.to_string_lossy().to_string();
                        exports_map.insert(path_str.clone(), file_exports);
                        debug!("Found exports in file: {}", path_str);
                    }

                    // Extract imports
                    let file_imports =
                        extract_imports(&file.path, &file_content, &lang_config.import_patterns);

                    // Store imports
                    for import in file_imports {
                        imports_map
                            .entry(import.name.clone())
                            .or_default()
                            .push(import);
                    }

                    // We found the language for this file, no need to check others
                    break;
                }
            }
        }
    }

    info!("Found exports in {} files", exports_map.len());
    info!("Found imports for {} unique entities", imports_map.len());

    Ok((exports_map, imports_map))
}

/// Extract exports from file content using regex patterns
fn extract_exports(file_path: &Path, content: &str, patterns: &[String]) -> Vec<ExportedEntity> {
    let mut exports = Vec::new();

    // Compile all export patterns
    let compiled_patterns: Vec<_> = patterns
        .iter()
        .filter_map(|pattern| match Regex::new(pattern) {
            Ok(regex) => Some(regex),
            Err(err) => {
                debug!("Invalid export pattern '{}': {}", pattern, err);
                None
            }
        })
        .collect();

    // Apply each pattern to the content
    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed line numbers

        for regex in &compiled_patterns {
            for captures in regex.captures_iter(line) {
                // The first capture group should be the entity name
                if captures.len() > 1 {
                    if let Some(name_match) = captures.get(captures.len() - 1) {
                        let name = name_match.as_str().trim().to_string();

                        // Determine export type based on the regex pattern or content
                        let export_type = determine_export_type(line);

                        exports.push(ExportedEntity {
                            name,
                            file_path: file_path.to_path_buf(),
                            line_number: line_num,
                            export_type,
                            usage_count: 0, // Will be updated later
                        });
                    }
                }
            }
        }
    }

    exports
}

/// Extract imports from file content using regex patterns
fn extract_imports(file_path: &Path, content: &str, patterns: &[String]) -> Vec<ImportReference> {
    let mut imports = Vec::new();

    // Rust-specific import handling
    if file_path.extension().and_then(|e| e.to_str()) == Some("rs") {
        let rust_imports = extract_rust_imports(file_path, content);
        imports.extend(rust_imports);

        // If we found Rust imports, return them directly
        if !imports.is_empty() {
            return imports;
        }
    }

    // Fallback to generic pattern-based import extraction
    // Compile all import patterns
    let compiled_patterns: Vec<_> = patterns
        .iter()
        .filter_map(|pattern| match Regex::new(pattern) {
            Ok(regex) => Some(regex),
            Err(err) => {
                debug!("Invalid import pattern '{}': {}", pattern, err);
                None
            }
        })
        .collect();

    // Apply each pattern to the content
    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed line numbers

        for regex in &compiled_patterns {
            for captures in regex.captures_iter(line) {
                // The first capture group should be the entity name
                if captures.len() > 1 {
                    if let Some(name_match) = captures.get(1) {
                        let name = name_match.as_str().trim().to_string();

                        // Handle comma-separated imports
                        for part in name.split(',') {
                            let import_name = part.trim().to_string();
                            if !import_name.is_empty() {
                                imports.push(ImportReference {
                                    name: import_name,
                                    file_path: file_path.to_path_buf(),
                                    line_number: line_num,
                                    import_statement: line.trim().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    imports
}

/// Extract imports from Rust file content
fn extract_rust_imports(file_path: &Path, content: &str) -> Vec<ImportReference> {
    let mut imports = Vec::new();

    // Simple regex for Rust imports: use .*?;
    // This capture is simpler than relying on regex patterns from config
    let use_regex = Regex::new(r"use\s+([^;]+);").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;
        let line = line.trim();

        // Skip comments
        if line.starts_with("//") {
            continue;
        }

        // Simple use statements
        if let Some(caps) = use_regex.captures(line) {
            if let Some(import_path) = caps.get(1) {
                let import_str = import_path.as_str().trim();

                // Parse the path to extract the actual names
                parse_rust_import_path(import_str, line_num, line, file_path, &mut imports);
            }
        }
    }

    imports
}

/// Parse a Rust import path to extract individual imports
fn parse_rust_import_path(
    import_path: &str,
    line_num: usize,
    line: &str,
    file_path: &Path,
    imports: &mut Vec<ImportReference>,
) {
    // Handle crate-level imports: use crate::
    if import_path.starts_with("crate::") {
        let path_parts: Vec<&str> = import_path[6..].split("::").collect();

        if !path_parts.is_empty() {
            // Last part is the actual import
            let last_part = path_parts.last().unwrap();

            // Check if it's a brace import: {a, b, c}
            if last_part.contains('{') && last_part.contains('}') {
                let brace_regex = Regex::new(r"\{([^}]+)\}").unwrap();
                if let Some(caps) = brace_regex.captures(last_part) {
                    if let Some(items) = caps.get(1) {
                        for item in items.as_str().split(',') {
                            let item = item.trim();
                            if !item.is_empty() {
                                imports.push(ImportReference {
                                    name: item.to_string(),
                                    file_path: file_path.to_path_buf(),
                                    line_number: line_num,
                                    import_statement: line.to_string(),
                                });
                            }
                        }
                    }
                }
            } else {
                // Simple import
                imports.push(ImportReference {
                    name: last_part.to_string(),
                    file_path: file_path.to_path_buf(),
                    line_number: line_num,
                    import_statement: line.to_string(),
                });
            }
        }
    }
    // Handle standard library and external crate imports
    else {
        // Check for brace imports
        if import_path.contains('{') && import_path.contains('}') {
            let brace_regex = Regex::new(r"\{([^}]+)\}").unwrap();
            if let Some(caps) = brace_regex.captures(import_path) {
                if let Some(items) = caps.get(1) {
                    for item in items.as_str().split(',') {
                        let item = item.trim();
                        if !item.is_empty() {
                            imports.push(ImportReference {
                                name: item.to_string(),
                                file_path: file_path.to_path_buf(),
                                line_number: line_num,
                                import_statement: line.to_string(),
                            });
                        }
                    }
                }
            }
        } else {
            // Simple import
            let parts: Vec<&str> = import_path.split("::").collect();
            if !parts.is_empty() {
                let name = parts.last().unwrap().to_string();
                imports.push(ImportReference {
                    name,
                    file_path: file_path.to_path_buf(),
                    line_number: line_num,
                    import_statement: line.to_string(),
                });
            }
        }
    }
}

/// Determine the type of export based on the line content
fn determine_export_type(line: &str) -> String {
    let line = line.trim();

    if line.contains("function ") || line.contains("fn ") {
        return "function".to_string();
    } else if line.contains("class ") {
        return "class".to_string();
    } else if line.contains("interface ") {
        return "interface".to_string();
    } else if line.contains("struct ") {
        return "struct".to_string();
    } else if line.contains("enum ") {
        return "enum".to_string();
    } else if line.contains("trait ") {
        return "trait".to_string();
    } else if line.contains("const ") {
        return "constant".to_string();
    } else if line.contains("let ") || line.contains("var ") {
        return "variable".to_string();
    } else if line.contains("type ") {
        return "type".to_string();
    } else if line.contains("mod ") {
        return "module".to_string();
    }

    "unknown".to_string()
}

/// Check if an entity with the given name exists in the exports map
pub fn find_export_by_name<'a>(
    exports_map: &'a ExportsMap,
    name: &str,
) -> Option<(&'a String, &'a ExportedEntity)> {
    for (file_path, exports) in exports_map {
        if let Some(export) = exports.iter().find(|e| e.name == name) {
            return Some((file_path, export));
        }
    }
    None
}

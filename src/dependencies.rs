use anyhow::Result;
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::exports::{ExportsMap, ImportsMap};

/// Represents a dependency graph of the repository
#[derive(Debug)]
pub struct DependencyGraph {
    /// Map of files to their dependencies
    file_dependencies: HashMap<String, HashSet<String>>,

    /// Map of files to files that depend on them
    reverse_dependencies: HashMap<String, HashSet<String>>,

    /// Map of files to their importance score
    importance_scores: HashMap<String, usize>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        DependencyGraph {
            file_dependencies: HashMap::new(),
            reverse_dependencies: HashMap::new(),
            importance_scores: HashMap::new(),
        }
    }

    /// Get files sorted by importance score (descending)
    pub fn get_files_by_importance(&self) -> Vec<(String, usize)> {
        let mut files: Vec<(String, usize)> = self
            .importance_scores
            .iter()
            .map(|(file, score)| (file.clone(), *score))
            .collect();

        // Sort by score in descending order
        files.sort_by(|a, b| b.1.cmp(&a.1));

        files
    }

    /// Get the importance score for a file
    pub fn get_file_importance(&self, file_path: &str) -> usize {
        *self.importance_scores.get(file_path).unwrap_or(&0)
    }

    /// Get files that depend on the given file
    pub fn get_dependent_files(&self, file_path: &str) -> Vec<String> {
        match self.reverse_dependencies.get(file_path) {
            Some(deps) => deps.iter().cloned().collect(),
            None => Vec::new(),
        }
    }

    /// Get files that the given file depends on
    pub fn get_dependencies(&self, file_path: &str) -> Vec<String> {
        match self.file_dependencies.get(file_path) {
            Some(deps) => deps.iter().cloned().collect(),
            None => Vec::new(),
        }
    }
}

/// Build a dependency graph from exports and imports
pub fn build_dependency_graph(
    exports_map: &mut ExportsMap,
    imports_map: &ImportsMap,
) -> Result<DependencyGraph> {
    info!("Building dependency graph");

    let mut graph = DependencyGraph::new();

    // Helper to add a dependency relationship
    let mut add_dependency = |from: &str, to: &str| {
        // Add to file dependencies
        graph
            .file_dependencies
            .entry(from.to_string())
            .or_default()
            .insert(to.to_string());

        // Add to reverse dependencies
        graph
            .reverse_dependencies
            .entry(to.to_string())
            .or_default()
            .insert(from.to_string());
    };

    // Process all imports and connect them to exports
    for (import_name, import_refs) in imports_map {
        // Try to find an export with this name
        for (export_file_path, exports) in exports_map.iter_mut() {
            for export in exports.iter_mut() {
                if export.name == *import_name {
                    // Update the usage count
                    export.usage_count += import_refs.len();

                    // Add dependency relationships
                    for import_ref in import_refs {
                        let import_file_path = import_ref.file_path.to_string_lossy().to_string();

                        // Don't add self-dependencies
                        if import_file_path != *export_file_path {
                            add_dependency(&import_file_path, export_file_path);
                            debug!("Dependency: {} -> {}", import_file_path, export_file_path);
                        }
                    }
                }
            }
        }
    }

    // Calculate importance scores based on usage counts and dependencies
    calculate_importance_scores(&mut graph, exports_map);

    info!(
        "Dependency graph built with {} files",
        graph.file_dependencies.len()
    );

    Ok(graph)
}

/// Calculate importance scores for files based on export usage and dependencies
fn calculate_importance_scores(graph: &mut DependencyGraph, exports_map: &ExportsMap) {
    // For each file, calculate its importance score
    for (file_path, exports) in exports_map {
        // Base score is the sum of usage counts for all exports
        let usage_score: usize = exports.iter().map(|e| e.usage_count).sum();

        // Additional score based on number of files that depend on this file
        let dependent_files = graph
            .reverse_dependencies
            .get(file_path)
            .map(|deps| deps.len())
            .unwrap_or(0);

        // Calculate total score
        let importance_score = usage_score + (dependent_files * 2);

        // Store the score
        graph
            .importance_scores
            .insert(file_path.clone(), importance_score);

        debug!(
            "File {} has importance score {}",
            file_path, importance_score
        );
    }
}

/// Calculate directory importance based on file importance
pub fn calculate_directory_importance(
    graph: &DependencyGraph,
    exports_map: &ExportsMap,
) -> HashMap<String, usize> {
    let mut dir_scores: HashMap<String, usize> = HashMap::new();

    // Sum up scores for all files in each directory
    for (file_path, _) in exports_map {
        let path = Path::new(file_path);

        // Get all parent directories
        let mut current = path;
        while let Some(parent) = current.parent() {
            if parent.to_string_lossy().is_empty() {
                break;
            }

            let dir_path = parent.to_string_lossy().to_string();
            let file_score = graph.get_file_importance(file_path);

            // Add the file's score to the directory score
            *dir_scores.entry(dir_path.clone()).or_default() += file_score;

            current = parent;
        }
    }

    // Sort by importance
    let mut dirs: Vec<(String, usize)> = dir_scores
        .iter()
        .map(|(dir, score)| (dir.clone(), *score))
        .collect();

    dirs.sort_by(|a, b| b.1.cmp(&a.1));

    info!("Calculated importance for {} directories", dir_scores.len());
    for (dir, score) in dirs.iter().take(5) {
        info!("Directory importance: {} = {}", dir, score);
    }

    dir_scores
}

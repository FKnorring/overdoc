use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Builder;
use log::{info, LevelFilter};
use std::fs;
use std::path::Path;

mod config;
mod dependencies;
mod exports;
mod filter;
mod metrics;
mod traversal;

/// OverDoc: Automatic documentation generation tool
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Repository directory to analyze (absolute or relative path)
    #[clap(short, long, default_value = ".", value_name = "DIRECTORY")]
    repo_path: String,

    /// Path to configuration file
    #[clap(short, long, value_name = "FILE")]
    config_path: Option<String>,

    /// Verbose output
    #[clap(short, long)]
    verbose: bool,

    /// Show top N important files
    #[clap(short = 'n', long, default_value = "10")]
    top_files: usize,

    /// Output directory for analysis results
    #[clap(short = 'o', long, default_value = "out", value_name = "DIRECTORY")]
    output_dir: String,

    /// Skip metrics analysis (for faster processing)
    #[clap(long)]
    skip_metrics: bool,
}

fn main() -> Result<()> {
    // Initialize logger with appropriate level
    let mut builder = Builder::new();

    // Set log level based on verbose flag
    let args = Args::parse();

    if args.verbose {
        builder.filter_level(LevelFilter::Debug);
    } else {
        builder.filter_level(LevelFilter::Info);
    }

    builder.init();

    if args.verbose {
        info!("Verbose mode enabled");
    }

    // Create output directory if it doesn't exist
    let output_dir = Path::new(&args.output_dir);
    if !output_dir.exists() {
        info!("Creating output directory: {}", output_dir.display());
        fs::create_dir_all(output_dir).context("Failed to create output directory")?;
    }

    // Load configuration
    let config_path = args
        .config_path
        .unwrap_or_else(|| "overdoc.yaml".to_string());
    let config = config::load_config(&config_path)
        .context(format!("Failed to load configuration from {}", config_path))?;

    info!("Starting repository analysis at: {}", args.repo_path);

    // Phase 1: Traverse repository and filter files
    let files = traversal::traverse_repository(&args.repo_path, &config)
        .context("Failed to traverse repository")?;

    info!("Found {} files for analysis", files.len());

    let filtered_files = filter::apply_filters(files, &config);

    info!(
        "After filtering, {} files remain for documentation",
        filtered_files.len()
    );

    // Phase 2: Scan for exports and imports
    let (mut exports_map, imports_map) = exports::scan_repository(&filtered_files, &config)
        .context("Failed to scan repository for exports and imports")?;

    // Count exports
    let total_exports = exports_map.values().map(|v| v.len()).sum::<usize>();
    info!(
        "Found {} exported entities across {} files",
        total_exports,
        exports_map.len()
    );

    // Build dependency graph
    let dependency_graph = dependencies::build_dependency_graph(&mut exports_map, &imports_map)
        .context("Failed to build dependency graph")?;

    // Calculate directory importance
    let dir_importance =
        dependencies::calculate_directory_importance(&dependency_graph, &exports_map);

    // Display top important files
    let top_files = dependency_graph.get_files_by_importance();

    info!("Top {} important files:", args.top_files);

    // Phase 3: Detailed metrics analysis (new)
    let repository_metrics = if !args.skip_metrics {
        info!("Starting detailed metrics analysis...");
        // Convert filtered_files to a vector of strings
        let file_paths: Vec<String> = filtered_files
            .iter()
            .map(|file| file.path.to_string_lossy().to_string())
            .collect();

        // Calculate initial metrics
        let mut metrics = metrics::analyze_repository(&file_paths)
            .context("Failed to analyze repository metrics")?;

        // Calculate export importance for each file using data from exports_map
        let max_importance = dependency_graph
            .get_files_by_importance()
            .iter()
            .map(|(_, score)| *score)
            .max()
            .unwrap_or(1);

        // Normalize export importance and add to metrics
        for (file_path, importance) in dependency_graph.get_files_by_importance().iter() {
            if let Some(file_metrics) = metrics.file_metrics.get_mut(file_path) {
                // Normalize to 0-1 scale
                let normalized_importance = *importance as f64 / max_importance as f64;
                file_metrics.with_export_importance(normalized_importance);

                // Recalculate knowledge score if complexity metrics exist
                if let Some(complexity) = &file_metrics.complexity_metrics {
                    // Clone complexity before we use it
                    let complexity_clone = complexity.clone();
                    file_metrics.knowledge_score = Some(metrics::calculate_knowledge_score(
                        file_metrics,
                        &complexity_clone,
                    ));
                }
            }
        }

        // Rebuild knowledge hotspots with updated scores
        let mut knowledge_hotspots: Vec<(String, f64)> = metrics
            .file_metrics
            .iter()
            .map(|(path, metrics)| (path.clone(), metrics.knowledge_score()))
            .collect();

        knowledge_hotspots
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        metrics.knowledge_hotspots = knowledge_hotspots;

        info!(
            "Metrics analysis complete: {} files, {} total lines, {} code lines",
            metrics.total_files, metrics.total_lines, metrics.total_code_lines
        );

        Some(metrics)
    } else {
        info!("Skipping detailed metrics analysis (--skip-metrics flag used)");
        None
    };

    // Create a markdown file with the analysis results
    let mut analysis_content = format!("# OverDoc Analysis Results\n\n");
    analysis_content.push_str("## Repository: ");
    analysis_content.push_str(&args.repo_path);
    analysis_content.push_str("\n\n");

    // Add summary statistics
    analysis_content.push_str("## Summary\n\n");
    analysis_content.push_str(&format!(
        "- Total files analyzed: {}\n",
        filtered_files.len()
    ));
    analysis_content.push_str(&format!("- Total exported entities: {}\n", total_exports));
    analysis_content.push_str(&format!("- Files with exports: {}\n", exports_map.len()));

    // Add metrics summary if available
    if let Some(metrics) = &repository_metrics {
        analysis_content.push_str(&format!("- Total lines of code: {}\n", metrics.total_lines));
        analysis_content.push_str(&format!("- Code lines: {}\n", metrics.total_code_lines));
        analysis_content.push_str(&format!(
            "- Comment lines: {}\n",
            metrics.total_comment_lines
        ));
        analysis_content.push_str(&format!("- Blank lines: {}\n", metrics.total_blank_lines));
        analysis_content.push_str(&format!(
            "- Comment ratio: {:.2}%\n",
            metrics.avg_comment_ratio * 100.0
        ));
        analysis_content.push_str(&format!(
            "- Average lines per file: {}\n",
            metrics.avg_lines_per_file
        ));

        // Add complexity metrics summary
        analysis_content.push_str(&format!(
            "- Average cyclomatic complexity: {:.2}\n",
            metrics.avg_cyclomatic_complexity
        ));
        analysis_content.push_str(&format!(
            "- Average cognitive complexity: {:.2}\n",
            metrics.avg_cognitive_complexity
        ));
        analysis_content.push_str(&format!(
            "- Average maintainability index: {:.2}\n",
            metrics.avg_maintainability_index
        ));

        // Add language distribution
        analysis_content.push_str("\n### Language Distribution\n\n");
        let mut lang_dist: Vec<(String, usize)> = metrics
            .language_distribution
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        lang_dist.sort_by(|a, b| b.1.cmp(&a.1));

        for (lang, count) in lang_dist {
            let percentage = (count as f64 / metrics.total_files as f64) * 100.0;
            analysis_content.push_str(&format!(
                "- {}: {} files ({:.1}%)\n",
                lang, count, percentage
            ));
        }

        // Add knowledge hotspots section
        if !metrics.knowledge_hotspots.is_empty() {
            analysis_content.push_str("\n### Knowledge Hotspots\n\n");
            analysis_content.push_str("Files with highest knowledge scores (combining complexity, size, and importance):\n\n");

            for (idx, (file, score)) in metrics.knowledge_hotspots.iter().take(5).enumerate() {
                analysis_content.push_str(&format!(
                    "{}. **{}** (Knowledge Score: {:.1})\n",
                    idx + 1,
                    file,
                    score
                ));
            }
        }
    }

    analysis_content.push_str("\n");

    // Add top important files
    analysis_content.push_str("## Top Important Files\n\n");
    for (idx, (file_path, score)) in top_files.iter().take(args.top_files).enumerate() {
        info!("  {}. {} (Score: {})", idx + 1, file_path, score);
        analysis_content.push_str(&format!(
            "{}. **{}** (Score: {})\n",
            idx + 1,
            file_path,
            score
        ));

        // If verbose, show the exports and their usage counts
        if args.verbose && idx < 5 {
            if let Some(exports) = exports_map.get(file_path) {
                for export in exports {
                    info!(
                        "     - {} {} (used {} times)",
                        export.export_type, export.name, export.usage_count
                    );
                    analysis_content.push_str(&format!(
                        "   - {} `{}` (used {} times)\n",
                        export.export_type, export.name, export.usage_count
                    ));
                }
            }
        }

        // Add metrics for this file if available
        if let Some(metrics) = &repository_metrics {
            if let Some(file_metrics) = metrics.file_metrics.get(file_path) {
                analysis_content.push_str(&format!(
                    "   - Lines: {} (Code: {}, Comments: {}, Blank: {})\n",
                    file_metrics.line_count,
                    file_metrics.code_lines,
                    file_metrics.comment_lines,
                    file_metrics.blank_lines
                ));

                analysis_content.push_str(&format!(
                    "   - Functions: {}, Comment ratio: {:.1}%\n",
                    file_metrics.function_count,
                    file_metrics.comment_ratio() * 100.0
                ));

                if !file_metrics.declaration_count.is_empty() {
                    let decl_str = file_metrics
                        .declaration_count
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<String>>()
                        .join(", ");

                    analysis_content.push_str(&format!("   - Declarations: {}\n", decl_str));
                }

                // Add complexity metrics if available
                if let Some(complexity) = &file_metrics.complexity_metrics {
                    analysis_content.push_str(&format!(
                        "   - Complexity: {} (Cyclomatic: {:.1}, Cognitive: {:.1})\n",
                        complexity.description(),
                        complexity.cyclomatic_complexity,
                        complexity.cognitive_complexity
                    ));

                    analysis_content.push_str(&format!(
                        "   - Maintainability Index: {:.1} (Higher is better)\n",
                        complexity.maintainability_index
                    ));

                    analysis_content.push_str(&format!(
                        "   - Knowledge Score: {:.1}\n",
                        file_metrics.knowledge_score()
                    ));
                }
            }
        }

        analysis_content.push_str("\n");
    }

    // Display top important directories
    let mut dir_scores: Vec<(String, usize)> = dir_importance.into_iter().collect();
    dir_scores.sort_by(|a, b| b.1.cmp(&a.1));

    info!("Top {} important directories:", args.top_files);
    analysis_content.push_str("## Top Important Directories\n\n");

    for (idx, (dir_path, score)) in dir_scores.iter().take(args.top_files).enumerate() {
        info!("  {}. {} (Score: {})", idx + 1, dir_path, score);
        analysis_content.push_str(&format!(
            "{}. **{}** (Score: {})\n",
            idx + 1,
            dir_path,
            score
        ));

        // If we have metrics, add directory metrics summary
        if let Some(metrics) = &repository_metrics {
            // Get all files in this directory
            let dir_files: Vec<String> = filtered_files
                .iter()
                .map(|file| file.path.to_string_lossy().to_string())
                .filter(|path| path.starts_with(dir_path))
                .collect();

            let dir_file_count = dir_files.len();
            let mut dir_line_count = 0;
            let mut dir_function_count = 0;

            for file in &dir_files {
                if let Some(file_metrics) = metrics.file_metrics.get(file) {
                    dir_line_count += file_metrics.line_count;
                    dir_function_count += file_metrics.function_count;
                }
            }

            analysis_content.push_str(&format!(
                "   - Files: {}, Total lines: {}, Functions: {}\n",
                dir_file_count, dir_line_count, dir_function_count
            ));
        }

        analysis_content.push_str("\n");
    }

    // Save the analysis to a file
    let output_file = output_dir.join("analysis_results.md");
    fs::write(&output_file, analysis_content).context(format!(
        "Failed to write analysis to {}",
        output_file.display()
    ))?;

    info!("Analysis saved to {}", output_file.display());

    Ok(())
}

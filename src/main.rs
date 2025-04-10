use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Builder;
use log::{info, LevelFilter};

mod config;
mod dependencies;
mod exports;
mod filter;
mod traversal;

/// OverDoc: Automatic documentation generation tool
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the repository to analyze
    #[clap(short, long, default_value = ".")]
    repo_path: String,

    /// Path to configuration file
    #[clap(short, long)]
    config_path: Option<String>,

    /// Verbose output
    #[clap(short, long)]
    verbose: bool,

    /// Show top N important files
    #[clap(short = 'n', long, default_value = "10")]
    top_files: usize,
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
    for (idx, (file_path, score)) in top_files.iter().take(args.top_files).enumerate() {
        info!("  {}. {} (Score: {})", idx + 1, file_path, score);

        // If verbose, show the exports and their usage counts
        if args.verbose && idx < 5 {
            if let Some(exports) = exports_map.get(file_path) {
                for export in exports {
                    info!(
                        "     - {} {} (used {} times)",
                        export.export_type, export.name, export.usage_count
                    );
                }
            }
        }
    }

    // Display top important directories
    let mut dir_scores: Vec<(String, usize)> = dir_importance.into_iter().collect();
    dir_scores.sort_by(|a, b| b.1.cmp(&a.1));

    info!("Top {} important directories:", args.top_files);
    for (idx, (dir_path, score)) in dir_scores.iter().take(args.top_files).enumerate() {
        info!("  {}. {} (Score: {})", idx + 1, dir_path, score);
    }

    Ok(())
}

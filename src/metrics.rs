use anyhow::{Context, Result};
use log::{debug, warn};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Stores basic metrics for a single file
#[derive(Debug, Clone)]
pub struct FileMetrics {
    pub path: String,
    pub line_count: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub file_size_bytes: u64,
    pub function_count: usize,
    pub declaration_count: HashMap<String, usize>, // Types like struct, enum, trait, etc.
    pub complexity_metrics: Option<ComplexityMetrics>,
    pub knowledge_score: Option<f64>,
    pub export_importance: Option<f64>, // New field to track importance based on exports
}

/// Enhanced metrics for code complexity
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: f64,
    pub max_nesting_depth: f64,
    pub cognitive_complexity: f64,
    pub halstead_volume: f64,
    pub halstead_difficulty: f64,
    pub halstead_effort: f64,
    pub halstead_time: f64,
    pub maintainability_index: f64,
}

impl ComplexityMetrics {
    /// Create a new empty ComplexityMetrics
    pub fn new() -> Self {
        ComplexityMetrics {
            cyclomatic_complexity: 0.0,
            max_nesting_depth: 0.0,
            cognitive_complexity: 0.0,
            halstead_volume: 0.0,
            halstead_difficulty: 0.0,
            halstead_effort: 0.0,
            halstead_time: 0.0,
            maintainability_index: 0.0,
        }
    }

    /// Returns a formatted description of the metrics
    pub fn description(&self) -> String {
        format!(
            "Cyclomatic: {:.1}, Cognitive: {:.1}, Maintainability: {:.1}",
            self.cyclomatic_complexity, self.cognitive_complexity, self.maintainability_index
        )
    }
}

impl FileMetrics {
    /// Calculate the comment ratio (comments / (code + comments))
    pub fn comment_ratio(&self) -> f64 {
        if self.code_lines + self.comment_lines == 0 {
            return 0.0;
        }
        self.comment_lines as f64 / (self.code_lines + self.comment_lines) as f64
    }

    /// Add complexity metrics to this file metrics
    pub fn with_complexity(&mut self, complexity: ComplexityMetrics) -> &mut Self {
        // Clone complexity before moving it into the Option
        let complexity_clone = complexity.clone();
        self.complexity_metrics = Some(complexity);
        self.knowledge_score = Some(calculate_knowledge_score(self, &complexity_clone));
        self
    }

    /// Get the knowledge score or a default
    pub fn knowledge_score(&self) -> f64 {
        self.knowledge_score.unwrap_or(0.0)
    }

    /// Add export importance data to this file
    pub fn with_export_importance(&mut self, importance: f64) -> &mut Self {
        self.export_importance = Some(importance);
        self
    }

    /// Get the export importance or a default
    pub fn export_importance(&self) -> f64 {
        self.export_importance.unwrap_or(0.0)
    }
}

/// File metrics for the entire repository
#[derive(Debug)]
pub struct RepositoryMetrics {
    pub file_metrics: HashMap<String, FileMetrics>,
    pub total_files: usize,
    pub total_lines: usize,
    pub total_code_lines: usize,
    pub total_comment_lines: usize,
    pub total_blank_lines: usize,
    pub total_size_bytes: u64,
    pub language_distribution: HashMap<String, usize>, // Extension -> file count
    pub avg_file_size: u64,
    pub avg_lines_per_file: usize,
    pub avg_comment_ratio: f64,
    pub avg_cyclomatic_complexity: f64,
    pub avg_cognitive_complexity: f64,
    pub avg_maintainability_index: f64,
    pub knowledge_hotspots: Vec<(String, f64)>, // Files sorted by knowledge score
}

/// Analyzes a file to extract metrics
fn analyze_file(file_path: &Path) -> Result<FileMetrics> {
    debug!("Analyzing metrics for file: {}", file_path.display());

    // Get file size
    let metadata = fs::metadata(file_path).context("Failed to get file metadata")?;
    let file_size = metadata.len();

    // Read file contents
    let content = fs::read_to_string(file_path).context("Failed to read file")?;
    let lines: Vec<&str> = content.lines().collect();

    let mut code_lines = 0;
    let mut comment_lines = 0;
    let mut blank_lines = 0;
    let mut in_block_comment = false;
    let mut function_count = 0;
    let mut declarations = HashMap::new();

    // Determine file language from extension
    let extension = file_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    // Process lines based on file type
    for line in &lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            blank_lines += 1;
            continue;
        }

        match extension.as_str() {
            "rs" => {
                // Rust language
                if in_block_comment {
                    comment_lines += 1;
                    if trimmed.contains("*/") {
                        in_block_comment = false;
                    }
                } else if trimmed.starts_with("//") {
                    comment_lines += 1;
                } else if trimmed.starts_with("/*") {
                    comment_lines += 1;
                    if !trimmed.contains("*/") {
                        in_block_comment = true;
                    }
                } else {
                    code_lines += 1;

                    // Count functions (simple heuristic)
                    if trimmed.contains("fn ") && !trimmed.contains(";") {
                        function_count += 1;
                    }

                    // Count declarations (simple heuristic)
                    for decl_type in &["struct ", "enum ", "trait ", "impl ", "type "] {
                        if trimmed.contains(decl_type) && !trimmed.contains(";") {
                            let key = decl_type.trim().to_string();
                            *declarations.entry(key).or_insert(0) += 1;
                        }
                    }
                }
            }
            "js" | "ts" | "tsx" | "jsx" => {
                // JavaScript/TypeScript
                if in_block_comment {
                    comment_lines += 1;
                    if trimmed.contains("*/") {
                        in_block_comment = false;
                    }
                } else if trimmed.starts_with("//") {
                    comment_lines += 1;
                } else if trimmed.starts_with("/*") {
                    comment_lines += 1;
                    if !trimmed.contains("*/") {
                        in_block_comment = true;
                    }
                } else {
                    code_lines += 1;

                    // Count functions (simple heuristic)
                    if (trimmed.contains("function ") || trimmed.contains("=>"))
                        && !trimmed.contains(";")
                    {
                        function_count += 1;
                    }

                    // Count declarations
                    for decl_type in &["class ", "interface ", "type ", "enum "] {
                        if trimmed.contains(decl_type) && !trimmed.contains(";") {
                            let key = decl_type.trim().to_string();
                            *declarations.entry(key).or_insert(0) += 1;
                        }
                    }
                }
            }
            // Add more languages as needed
            _ => {
                // Generic fallback
                if trimmed.starts_with("#") || trimmed.starts_with("//") {
                    comment_lines += 1;
                } else {
                    code_lines += 1;
                }
            }
        }
    }

    let file_path_str = file_path.to_string_lossy().to_string();

    // Create basic file metrics
    let mut file_metrics = FileMetrics {
        path: file_path_str.clone(),
        line_count: lines.len(),
        code_lines,
        comment_lines,
        blank_lines,
        file_size_bytes: file_size,
        function_count,
        declaration_count: declarations,
        complexity_metrics: None,
        knowledge_score: None,
        export_importance: None,
    };

    // Calculate complexity metrics if the file isn't too large
    if file_size < 1024 * 1024 {
        // Skip files larger than 1MB for performance
        match analyze_file_complexity(&file_path_str, &content) {
            Ok(complexity) => {
                file_metrics.with_complexity(complexity);
            }
            Err(err) => {
                warn!(
                    "Failed to analyze complexity for {}: {}",
                    file_path.display(),
                    err
                );
            }
        }
    }

    Ok(file_metrics)
}

/// Analyze all files in a repository to gather metrics
pub fn analyze_repository(file_paths: &[String]) -> Result<RepositoryMetrics> {
    let mut file_metrics = HashMap::new();
    let mut total_lines = 0;
    let mut total_code_lines = 0;
    let mut total_comment_lines = 0;
    let mut total_blank_lines = 0;
    let mut total_size_bytes = 0;
    let mut language_distribution = HashMap::new();
    let mut total_cyclomatic_complexity = 0.0;
    let mut total_cognitive_complexity = 0.0;
    let mut total_maintainability_index = 0.0;
    let mut files_with_complexity = 0;

    for file_path in file_paths {
        let path = Path::new(file_path);

        match analyze_file(path) {
            Ok(metrics) => {
                // Update totals
                total_lines += metrics.line_count;
                total_code_lines += metrics.code_lines;
                total_comment_lines += metrics.comment_lines;
                total_blank_lines += metrics.blank_lines;
                total_size_bytes += metrics.file_size_bytes;

                // Update language distribution
                if let Some(ext) = path.extension() {
                    let extension = ext.to_string_lossy().to_lowercase();
                    *language_distribution.entry(extension).or_insert(0) += 1;
                } else {
                    *language_distribution
                        .entry("unknown".to_string())
                        .or_insert(0) += 1;
                }

                // Update complexity metrics if available
                if let Some(complexity) = &metrics.complexity_metrics {
                    total_cyclomatic_complexity += complexity.cyclomatic_complexity;
                    total_cognitive_complexity += complexity.cognitive_complexity;
                    total_maintainability_index += complexity.maintainability_index;
                    files_with_complexity += 1;
                }

                file_metrics.insert(file_path.clone(), metrics);
            }
            Err(err) => {
                warn!("Failed to analyze file {}: {}", file_path, err);
            }
        }
    }

    let total_files = file_metrics.len();

    // Calculate averages
    let avg_file_size = if total_files > 0 {
        total_size_bytes / total_files as u64
    } else {
        0
    };

    let avg_lines_per_file = if total_files > 0 {
        total_lines / total_files
    } else {
        0
    };

    let avg_comment_ratio = if total_code_lines + total_comment_lines > 0 {
        total_comment_lines as f64 / (total_code_lines + total_comment_lines) as f64
    } else {
        0.0
    };

    // Calculate average complexity metrics
    let avg_cyclomatic_complexity = if files_with_complexity > 0 {
        total_cyclomatic_complexity / files_with_complexity as f64
    } else {
        0.0
    };

    let avg_cognitive_complexity = if files_with_complexity > 0 {
        total_cognitive_complexity / files_with_complexity as f64
    } else {
        0.0
    };

    let avg_maintainability_index = if files_with_complexity > 0 {
        total_maintainability_index / files_with_complexity as f64
    } else {
        0.0
    };

    // Identify knowledge hotspots (files with highest knowledge scores)
    let mut knowledge_hotspots: Vec<(String, f64)> = file_metrics
        .iter()
        .map(|(path, metrics)| (path.clone(), metrics.knowledge_score()))
        .collect();

    // Sort by knowledge score in descending order
    knowledge_hotspots.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    Ok(RepositoryMetrics {
        file_metrics,
        total_files,
        total_lines,
        total_code_lines,
        total_comment_lines,
        total_blank_lines,
        total_size_bytes,
        language_distribution,
        avg_file_size,
        avg_lines_per_file,
        avg_comment_ratio,
        avg_cyclomatic_complexity,
        avg_cognitive_complexity,
        avg_maintainability_index,
        knowledge_hotspots,
    })
}

/// Calculate complexity metrics for a file
pub fn calculate_complexity_metrics(
    file_path: &str,
    content: &str,
) -> Result<HashMap<String, f64>> {
    let mut metrics = HashMap::new();

    // Get file extension to determine language
    let path = Path::new(file_path);
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    // Simple implementation - will need to be extended with a proper parser for more accurate results
    let lines: Vec<&str> = content.lines().collect();

    // Nesting depth
    let mut max_depth = 0;
    let mut current_depth = 0;

    for line in &lines {
        let trimmed = line.trim();

        // Count opening braces/brackets
        let open_count = trimmed.matches('{').count()
            + trimmed.matches('(').count()
            + trimmed.matches('[').count();

        // Count closing braces/brackets
        let close_count = trimmed.matches('}').count()
            + trimmed.matches(')').count()
            + trimmed.matches(']').count();

        current_depth += open_count as isize - close_count as isize;
        if current_depth > max_depth {
            max_depth = current_depth;
        }
    }

    metrics.insert("max_nesting_depth".to_string(), max_depth as f64);

    // Simple approximation of cyclomatic complexity
    // Count branching statements
    let mut complexity = 1; // Base complexity

    match extension.as_str() {
        "rs" => {
            // Rust language
            for line in &lines {
                let trimmed = line.trim();
                if trimmed.contains("if ")
                    || trimmed.contains("else ")
                    || trimmed.contains("match ")
                    || trimmed.contains("for ")
                    || trimmed.contains("while ")
                {
                    complexity += 1;
                }
            }
        }
        "js" | "ts" | "tsx" | "jsx" => {
            // JavaScript/TypeScript
            for line in &lines {
                let trimmed = line.trim();
                if trimmed.contains("if ")
                    || trimmed.contains("else ")
                    || trimmed.contains("switch ")
                    || trimmed.contains("case ")
                    || trimmed.contains("for ")
                    || trimmed.contains("while ")
                    || trimmed.contains("? ")
                {
                    complexity += 1;
                }
            }
        }
        // Add more languages as needed
        _ => {
            // Generic fallback - simple approximation
            for line in &lines {
                let trimmed = line.trim();
                if trimmed.contains("if ")
                    || trimmed.contains("else ")
                    || trimmed.contains("for ")
                    || trimmed.contains("while ")
                {
                    complexity += 1;
                }
            }
        }
    }

    metrics.insert("cyclomatic_complexity".to_string(), complexity as f64);

    Ok(metrics)
}

/// Store Halstead metrics operators and operands
struct HalsteadData {
    unique_operators: usize, // n1
    total_operators: usize,  // N1
    unique_operands: usize,  // n2
    total_operands: usize,   // N2
}

impl HalsteadData {
    /// Calculate Halstead Volume: N * log2(n)
    /// where N = N1 + N2 (total operators + operands)
    /// and n = n1 + n2 (unique operators + operands)
    fn volume(&self) -> f64 {
        let n = (self.unique_operators + self.unique_operands) as f64;
        let n_total = (self.total_operators + self.total_operands) as f64;

        if n <= 0.0 {
            return 0.0;
        }

        n_total * (n.log2())
    }

    /// Calculate Halstead Difficulty: (n1/2) * (N2/n2)
    fn difficulty(&self) -> f64 {
        if self.unique_operands == 0 {
            return 0.0;
        }

        (self.unique_operators as f64 / 2.0)
            * (self.total_operands as f64 / self.unique_operands as f64)
    }

    /// Calculate Halstead Effort: D * V
    fn effort(&self, volume: f64, difficulty: f64) -> f64 {
        difficulty * volume
    }

    /// Calculate Halstead Time (in seconds): E / 18
    fn time(&self, effort: f64) -> f64 {
        effort / 18.0
    }
}

/// Analyze file to calculate enhanced complexity metrics
pub fn analyze_file_complexity(file_path: &str, content: &str) -> Result<ComplexityMetrics> {
    let mut metrics = ComplexityMetrics::new();

    // Get file extension to determine language
    let path = Path::new(file_path);
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let lines: Vec<&str> = content.lines().collect();

    // Calculate basic complexity metrics first
    let basic_metrics = calculate_complexity_metrics(file_path, content)?;
    metrics.cyclomatic_complexity = *basic_metrics.get("cyclomatic_complexity").unwrap_or(&1.0);
    metrics.max_nesting_depth = *basic_metrics.get("max_nesting_depth").unwrap_or(&0.0);

    // Calculate cognitive complexity
    metrics.cognitive_complexity = calculate_cognitive_complexity(&lines, &extension);

    // Calculate Halstead metrics
    let halstead_data = calculate_halstead_data(&lines, &extension);
    metrics.halstead_volume = halstead_data.volume();
    metrics.halstead_difficulty = halstead_data.difficulty();
    metrics.halstead_effort =
        halstead_data.effort(metrics.halstead_volume, metrics.halstead_difficulty);
    metrics.halstead_time = halstead_data.time(metrics.halstead_effort);

    // Calculate maintainability index
    // MI = 171 - 5.2 * ln(V) - 0.23 * CC - 16.2 * ln(LOC)
    let loc = lines.len() as f64;
    if loc > 0.0 && metrics.halstead_volume > 0.0 {
        metrics.maintainability_index = 171.0
            - 5.2 * metrics.halstead_volume.ln()
            - 0.23 * metrics.cyclomatic_complexity
            - 16.2 * loc.ln();

        // Normalize to 0-100 scale if outside range
        if metrics.maintainability_index > 100.0 {
            metrics.maintainability_index = 100.0;
        } else if metrics.maintainability_index < 0.0 {
            metrics.maintainability_index = 0.0;
        }
    } else {
        metrics.maintainability_index = 100.0; // Default for small files
    }

    Ok(metrics)
}

/// Calculate cognitive complexity (a more advanced measure that accounts for the mental effort)
fn calculate_cognitive_complexity(lines: &[&str], language: &str) -> f64 {
    let mut complexity = 0.0;
    let mut nesting_level: usize = 0;

    match language {
        "rs" => {
            // Rust-specific cognitive complexity calculation
            for line in lines {
                let trimmed = line.trim();

                // Increment for nesting structures
                if trimmed.contains("if ")
                    || trimmed.contains("match ")
                    || trimmed.contains("for ")
                    || trimmed.contains("while ")
                {
                    // Base complexity for the structure
                    complexity += 1.0;

                    // Additional complexity for nesting level
                    complexity += nesting_level as f64;

                    // Increase nesting level if this line has a block start
                    if trimmed.contains("{") {
                        nesting_level += 1;
                    }
                }
                // Handle else statements (add complexity without incrementing nesting)
                else if trimmed.contains("else ") {
                    complexity += 1.0;
                    if trimmed.contains("{") && !trimmed.contains("if ") {
                        nesting_level += 1;
                    }
                }
                // Handle closures separately
                else if trimmed.contains("|") && trimmed.contains("| {") {
                    complexity += 1.0;
                    nesting_level += 1;
                }
                // Handle block ends
                else if trimmed.contains("}") {
                    let close_count = trimmed.matches('}').count();
                    nesting_level = nesting_level.saturating_sub(close_count as usize);
                }

                // Additional complexity for logical operators
                if trimmed.contains("&&") || trimmed.contains("||") {
                    complexity += 0.5
                        * (trimmed.matches("&&").count() + trimmed.matches("||").count()) as f64;
                }
            }
        }
        "js" | "ts" | "tsx" | "jsx" => {
            // JavaScript/TypeScript cognitive complexity calculation
            for line in lines {
                let trimmed = line.trim();

                // Increment for control structures
                if trimmed.contains("if ")
                    || trimmed.contains("for ")
                    || trimmed.contains("while ")
                    || trimmed.contains("switch ")
                {
                    complexity += 1.0;
                    complexity += nesting_level as f64;

                    if trimmed.contains("{") {
                        nesting_level += 1;
                    }
                }
                // Handle else and case statements
                else if trimmed.contains("else ") || trimmed.contains("case ") {
                    complexity += 1.0;
                    if trimmed.contains("{") && !trimmed.contains("if ") {
                        nesting_level += 1;
                    }
                }
                // Handle arrow functions and other blocks
                else if (trimmed.contains("=>") && trimmed.contains("{"))
                    || (trimmed.contains("function") && trimmed.contains("{"))
                {
                    complexity += 1.0;
                    nesting_level += 1;
                }
                // Handle ternary operators (? :)
                else if trimmed.contains(" ? ") {
                    complexity += 1.0;
                }
                // Handle block ends
                else if trimmed.contains("}") {
                    let close_count = trimmed.matches('}').count();
                    nesting_level = nesting_level.saturating_sub(close_count as usize);
                }

                // Additional complexity for logical operators
                if trimmed.contains("&&") || trimmed.contains("||") {
                    complexity += 0.5
                        * (trimmed.matches("&&").count() + trimmed.matches("||").count()) as f64;
                }
            }
        }
        _ => {
            // Generic calculation for other languages
            for line in lines {
                let trimmed = line.trim();

                // Simple heuristic for control structures
                if trimmed.contains("if ") || trimmed.contains("for ") || trimmed.contains("while ")
                {
                    complexity += 1.0;
                    complexity += nesting_level as f64;

                    if trimmed.contains("{") {
                        nesting_level += 1;
                    }
                }
                // Handle else statements
                else if trimmed.contains("else ") {
                    complexity += 1.0;
                }
                // Handle block ends
                else if trimmed.contains("}") {
                    let close_count = trimmed.matches('}').count();
                    nesting_level = nesting_level.saturating_sub(close_count as usize);
                }
            }
        }
    }

    complexity
}

/// Calculate Halstead metrics data using language-specific tokens
fn calculate_halstead_data(lines: &[&str], language: &str) -> HalsteadData {
    let mut operators = HashMap::new();
    let mut operands = HashMap::new();

    match language {
        "rs" => {
            // Rust operators
            let operator_patterns = [
                "+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "!", "&",
                "|", "^", "<<", ">>", "=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=",
                ">>=", ".", "->", "=>", "::", ";", ",", "if", "else", "match", "for", "while",
                "loop", "break", "continue", "return", "fn", "struct", "enum", "impl", "trait",
            ];

            for line in lines {
                let trimmed = line.trim();

                // Skip comments
                if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                    continue;
                }

                // Find operators
                for op in &operator_patterns {
                    let count = count_occurrences(trimmed, op);
                    if count > 0 {
                        *operators.entry(op.to_string()).or_insert(0) += count;
                    }
                }

                // Extract identifiers/operands (simplified approach)
                // This would be better with a proper parser, but using a simple heuristic
                for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '_') {
                    if !word.is_empty() && !operator_patterns.contains(&word) {
                        let word = word.trim();
                        if !word.is_empty() && !word.parse::<f64>().is_ok() {
                            // Skip numeric literals
                            *operands.entry(word.to_string()).or_insert(0) += 1;
                        }
                    }
                }

                // Count numeric literals
                for part in trimmed.split(|c: char| !c.is_digit(10) && c != '.') {
                    if !part.is_empty() && part.parse::<f64>().is_ok() {
                        *operands.entry(part.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
        "js" | "ts" | "tsx" | "jsx" => {
            // JavaScript/TypeScript operators
            let operator_patterns = [
                "+", "-", "*", "/", "%", "==", "===", "!=", "!==", "<", ">", "<=", ">=", "&&",
                "||", "!", "&", "|", "^", "<<", ">>", ">>>", "=", "+=", "-=", "*=", "/=", "%=",
                "&=", "|=", "^=", "<<=", ">>=", ">>>=", ".", "=>", "++", "--", "?", ":", ";", ",",
                "if", "else", "switch", "case", "for", "while", "do", "break", "continue",
                "return", "function", "class", "new", "this", "super",
            ];

            // Similar approach as with Rust
            for line in lines {
                let trimmed = line.trim();

                // Skip comments
                if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                    continue;
                }

                // Find operators
                for op in &operator_patterns {
                    let count = count_occurrences(trimmed, op);
                    if count > 0 {
                        *operators.entry(op.to_string()).or_insert(0) += count;
                    }
                }

                // Extract identifiers/operands
                for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '_') {
                    if !word.is_empty() && !operator_patterns.contains(&word) {
                        let word = word.trim();
                        if !word.is_empty() && !word.parse::<f64>().is_ok() {
                            *operands.entry(word.to_string()).or_insert(0) += 1;
                        }
                    }
                }

                // Count numeric literals
                for part in trimmed.split(|c: char| !c.is_digit(10) && c != '.') {
                    if !part.is_empty() && part.parse::<f64>().is_ok() {
                        *operands.entry(part.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
        _ => {
            // Generic approach for other languages
            let operator_patterns = [
                "+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "!", "&",
                "|", "=", ".", ";", ",", "if", "else", "for", "while", "return",
            ];

            for line in lines {
                let trimmed = line.trim();

                // Skip comments (generic approach)
                if trimmed.starts_with("//")
                    || trimmed.starts_with("#")
                    || trimmed.starts_with("/*")
                {
                    continue;
                }

                // Find operators
                for op in &operator_patterns {
                    let count = count_occurrences(trimmed, op);
                    if count > 0 {
                        *operators.entry(op.to_string()).or_insert(0) += count;
                    }
                }

                // Extract identifiers/operands
                for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '_') {
                    if !word.is_empty() && !operator_patterns.contains(&word) {
                        let word = word.trim();
                        if !word.is_empty() && !word.parse::<f64>().is_ok() {
                            *operands.entry(word.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    // Calculate Halstead metrics
    let unique_operators = operators.len();
    let total_operators = operators.values().sum();

    let unique_operands = operands.len();
    let total_operands = operands.values().sum();

    HalsteadData {
        unique_operators,
        total_operators,
        unique_operands,
        total_operands,
    }
}

/// Count occurrences of a pattern in a string
fn count_occurrences(text: &str, pattern: &str) -> usize {
    // This is a simplified approach; a more accurate implementation would use regex
    // or a proper tokenizer to handle cases like comments and string literals
    let mut count = 0;
    let mut pos = 0;

    while let Some(idx) = text[pos..].find(pattern) {
        count += 1;
        pos += idx + pattern.len();

        if pos >= text.len() {
            break;
        }
    }

    count
}

/// Calculate "knowledge score" for a file based on various metrics
pub fn calculate_knowledge_score(
    file_metrics: &FileMetrics,
    complexity: &ComplexityMetrics,
) -> f64 {
    // File size factor - using log scale to avoid overweighting large files
    // but still giving some importance to file size
    let size_factor = (file_metrics.line_count as f64).ln().max(1.0) * 2.0;

    // Complexity factors - core of the knowledge score
    // Higher values indicate more complex code requiring more knowledge
    let cc_norm = complexity.cyclomatic_complexity.min(50.0) / 50.0; // Normalize to 0-1
    let cog_norm = complexity.cognitive_complexity.min(200.0) / 200.0; // Normalize to 0-1

    // Combined complexity - cognitive complexity is weighted higher
    // as it better represents mental effort to understand
    let complexity_factor = (cc_norm * 15.0) + (cog_norm * 25.0);

    // Maintainability - lower maintainability means higher knowledge required
    // Inverse relationship with maintainability index
    let maintainability_norm = ((100.0 - complexity.maintainability_index) / 100.0).min(1.0);
    let maintainability_factor = maintainability_norm * 20.0;

    // Code structure complexity - more functions and declarations means more knowledge
    let functions_norm = (file_metrics.function_count as f64).min(20.0) / 20.0;
    let function_factor = functions_norm * 15.0;

    // Declarations indicate entities that need to be understood
    let decl_count = file_metrics.declaration_count.values().sum::<usize>() as f64;
    let decl_norm = decl_count.min(10.0) / 10.0;
    let declaration_factor = decl_norm * 10.0;

    // Export importance - files with more exports are more important
    let export_factor = file_metrics.export_importance() * 15.0;

    // Combined knowledge score with all factors
    let knowledge_score = size_factor
        + complexity_factor
        + maintainability_factor
        + function_factor
        + declaration_factor
        + export_factor;

    // Normalize to a 0-100 scale with a more balanced distribution
    // This ensures we get a range of values rather than most files at 100
    let normalized_score = (knowledge_score * 0.85).min(100.0);

    normalized_score
}

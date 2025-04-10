use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Main configuration structure for OverDoc
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Global ignore patterns for all languages
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
    
    /// Directory patterns to ignore (e.g., node_modules, target)
    #[serde(default)]
    pub ignore_directories: Vec<String>,
    
    /// Language-specific configuration
    #[serde(default)]
    pub languages: HashMap<String, LanguageConfig>,
    
    /// Default settings to apply when language-specific ones aren't provided
    #[serde(default)]
    pub default_settings: DefaultSettings,
}

/// Configuration for a specific programming language
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LanguageConfig {
    /// File extensions for this language
    pub extensions: Vec<String>,
    
    /// Language-specific files to ignore
    #[serde(default)]
    pub ignore_files: Vec<String>,
    
    /// Language-specific directories to ignore
    #[serde(default)]
    pub ignore_directories: Vec<String>,
    
    /// Import structures to recognize
    #[serde(default)]
    pub import_patterns: Vec<String>,
    
    /// Export structures to recognize
    #[serde(default)]
    pub export_patterns: Vec<String>,
}

/// Default settings to use when language-specific ones aren't provided
#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultSettings {
    /// Whether to include files with no extension
    #[serde(default = "default_as_false")]
    pub include_no_extension: bool,
    
    /// Default file size limit in KB (0 means no limit)
    #[serde(default)]
    pub max_file_size_kb: usize,
}

impl Default for DefaultSettings {
    fn default() -> Self {
        DefaultSettings {
            include_no_extension: false,
            max_file_size_kb: 1024, // 1MB default limit
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ignore_patterns: vec![
                "*.min.*".to_string(),
                "*.map".to_string(),
                "*.lock".to_string(),
                ".gitignore".to_string(),
                ".git/*".to_string(),
            ],
            ignore_directories: vec![
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".git".to_string(),
            ],
            languages: HashMap::new(),
            default_settings: DefaultSettings::default(),
        }
    }
}

/// Helper function for default boolean values in serde
fn default_as_false() -> bool {
    false
}

/// Load configuration from a YAML file
pub fn load_config(config_path: &str) -> Result<Config> {
    // Check if config file exists
    let path = Path::new(config_path);
    
    if !path.exists() {
        return Ok(Config::default());
    }
    
    // Read and parse the config file
    let config_str = fs::read_to_string(path)
        .context(format!("Failed to read config file at {}", config_path))?;
    
    let config: Config = serde_yaml::from_str(&config_str)
        .context("Failed to parse YAML configuration")?;
    
    Ok(config)
}

/// Create a default configuration file if one doesn't exist
pub fn create_default_config(config_path: &str) -> Result<()> {
    let path = Path::new(config_path);
    
    if path.exists() {
        return Ok(());
    }
    
    let default_config = Config::default();
    let yaml = serde_yaml::to_string(&default_config)
        .context("Failed to serialize default configuration")?;
    
    fs::write(path, yaml).context("Failed to write default configuration file")?;
    
    Ok(())
} 
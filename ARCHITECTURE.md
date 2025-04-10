# OverDoc: Architecture Overview

## Introduction

OverDoc is an intelligent, language-agnostic documentation generation tool that identifies high-traffic areas of a project and automatically generates and maintains documentation for these key entities. This document outlines the architecture, key modules, and data flows of the OverDoc system.

## Core Modules

### 1. Configuration (`config.rs`)

The configuration module is responsible for loading and managing user settings from the `overdoc.yaml` file. It defines:

- **Config**: Main configuration structure containing global settings and language-specific configurations
- **LanguageConfig**: Language-specific settings including file extensions, patterns for exports/imports
- **DefaultSettings**: Fallback settings when language-specific ones aren't provided

### 2. Repository Traversal (`traversal.rs`)

This module handles scanning the filesystem to identify all files for analysis:

- **RepoFile**: Represents a file found during traversal with properties like path, extension, and size
- **traverse_repository()**: Main function that walks the directory tree and collects files
- **is_ignored_by_default()**: Filters out ignored directories/files based on configuration

### 3. File Filtering (`filter.rs`)

Applies filtering rules to determine which files should be analyzed:

- **apply_filters()**: Main function that filters the list of files based on configuration
- **should_ignore_file()**: Checks if a file should be ignored based on extensions, patterns, and other rules
- **matches_any_pattern()**: Pattern matching utility for file filtering

### 4. Export/Import Analysis (`exports.rs`)

Analyzes source code to identify exported entities and import references:

- **ExportedEntity**: Represents an entity exported from a file (functions, classes, etc.)
- **ImportReference**: Represents a reference/import of an entity from another file
- **scan_repository()**: Main function that processes files to find exports and imports
- **extract_exports()**: Extracts exported entities from file content using regex patterns
- **extract_imports()**: Extracts import references from file content

### 5. Dependency Analysis (`dependencies.rs`)

Builds a dependency graph based on the exports and imports:

- **DependencyGraph**: Data structure representing code dependencies between files
- **build_dependency_graph()**: Builds the graph connecting imports to exports
- **calculate_importance_scores()**: Assigns importance scores to files based on usage
- **calculate_directory_importance()**: Calculates importance metrics for directories

### 6. Main Application (`main.rs`)

Orchestrates the overall process:

- Parses command-line arguments
- Loads configuration
- Initiates repository traversal and analysis
- Displays results about important files and directories

## Data Flow

1. **Configuration Loading**:
   - Load `overdoc.yaml` configuration file
   - Parse global and language-specific settings

2. **Repository Traversal**:
   - Scan the specified directory recursively
   - Create `RepoFile` objects for each file found
   - Apply initial filtering based on directory names

3. **File Filtering**:
   - Apply configuration-based filters to the file list
   - Filter out files based on size, extensions, and patterns
   - Return a filtered list of files for analysis

4. **Export/Import Analysis**:
   - For each filtered file, identify its language based on extension
   - Apply language-specific regex patterns to find exports
   - Apply language-specific regex patterns to find imports
   - Build maps of exports and imports

5. **Dependency Graph Construction**:
   - Connect imports to their corresponding exports
   - Build a graph of file dependencies
   - Calculate importance scores for each file based on:
     - Number of times its exports are used
     - Number of files that depend on it

6. **Result Generation**:
   - Identify the most important files and directories
   - Display results to the user

## Key Data Structures

### ExportsMap

Maps file paths to lists of exported entities:
```
HashMap<String, Vec<ExportedEntity>>
```

### ImportsMap

Maps entity names to lists of import references:
```
HashMap<String, Vec<ImportReference>>
```

### DependencyGraph

Consists of:
- **file_dependencies**: Maps files to their dependencies
- **reverse_dependencies**: Maps files to files that depend on them
- **importance_scores**: Maps files to their importance score

## Configuration

OverDoc uses a YAML configuration file (`overdoc.yaml`) with:

- Global ignore patterns for all languages
- Directory patterns to ignore
- Language-specific configurations:
  - File extensions
  - Import/export regex patterns
  - Language-specific files and directories to ignore
- Default settings for files with no specific language configuration

## Importance Calculation

A file's importance is determined by:
1. Sum of usage counts for all exports from the file
2. Number of other files that depend on this file (weighted)

This allows OverDoc to prioritize documenting the most heavily-used and critical components of a codebase. 
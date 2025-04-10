## Development Plan

### Phase 1: Foundation & Analysis
1. Project setup with core dependencies in rust
2. Setup structure for traversing a repository
3. Add config for language specific file import / export structures
4. Filtering of unneccesary files and directories (build directories, files and dirs starting with ., etc..)
5. Language specific filtering, (certain config files)

### Phase 2: Export & Usage Analysis
1. Scan files to identify exported entities using language-specific patterns
2. Track references to exports by scanning imports across the repository
3. Build a graph of dependencies between files
4. Calculate usage metrics for each exported entity
5. Rank files and directories by importance based on usage count
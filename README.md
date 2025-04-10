# OverDoc: Automatic Documentation Tool

## Project Overview
OverDoc is an intelligent, language agnostic documentation generation tool that identifies high-traffic areas of a project and automatically generates and maintains documentation for these key entities.

## Metrics & Analysis

OverDoc analyzes repositories to identify important code components using a combination of dependency analysis, complexity metrics, and usage patterns.

### Knowledge Score

The **Knowledge Score** is a key metric that attempts to quantify how much "developer knowledge" is required to understand and maintain a file. Files with higher knowledge scores are typically:

- More complex (high cyclomatic and cognitive complexity)
- Frequently referenced by other files
- Contain many functions and declarations
- Difficult to maintain (low maintainability index)
- Larger in size (but normalized to avoid over-penalizing long files)

A higher knowledge score indicates files that:
1. Are critical to the project
2. Require more mental effort to understand
3. Would benefit most from thorough documentation
4. Present higher risk when modified
5. May be candidates for refactoring

Knowledge scores range from 0-100, with higher scores indicating files that contain more critical knowledge.

## Usage

```bash
cargo run -- -r /path/to/repository
```

For more options:
```bash
cargo run -- --help
```
# OverDoc: Automatic Documentation Tool

## Project Overview
OverDoc is an intelligent, language agnostic documentation generation tool that identifies high-traffic areas of a project and automatically generates and maintains documentation for these key entities.
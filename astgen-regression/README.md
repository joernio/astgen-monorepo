# astgen-regression

Config-driven regression testing framework for language-specific astgen variants.

## Overview

`astgen-regression` provides automated regression testing for AST generation tools (astgen) across different programming languages. It:

- Clones real-world codebases (corpora) at specific versions
- Builds your astgen from base and PR branches using git worktrees
- Generates AST outputs from both versions (**median wall-clock** timing with optional warmup and repeated runs—configurable under `execute`)
- Compares outputs and produces detailed Markdown reports
- Integrates with GitHub Actions for automated CI testing

## Installation

```bash
# Clone the repository
git clone https://github.com/joernio/astgen-regression.git
cd astgen-regression

# Create and activate virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install dependencies and package in editable mode
pip install -r requirements.txt
pip install -e .
```

**Dependencies:**
- Python 3.12+
- PyYAML - Configuration parsing
- Jinja2 - Template rendering
- deepdiff - Structural JSON comparison

## Quick Start

1. **Initialize configuration**
   ```bash
   astgen-regression init --language javascript
   ```
   This creates `regression.yaml` and `.github/workflows/javascript-astgen-regression.yml`.

2. **Configure corpora**
   Edit `regression.yaml` to define test codebases:
   ```yaml
   corpora:
     - name: "lodash"
       label: "lodash@4.17.21"
       clone_url: "https://github.com/lodash/lodash.git"
       tag: "4.17.21"
   ```

3. **Run locally**
   ```bash
   astgen-regression local
   ```
   This creates git worktrees, builds base and PR versions, runs regression tests, and generates a Markdown report.

4. **Review results**
   Open `regression-report.md` to see detailed comparisons.

## Commands

### `init`

Initialize regression testing for a new project.

```bash
astgen-regression init [--language LANG] [--config PATH] [--force]
```

Options:
- `--language`: Pre-fill config template for a specific language (javascript, rust, swift, dotnet, ruby, go, abap)
- `--config`: Config file path (default: `regression.yaml`)
- `--force`: Overwrite existing files

Creates:
- `regression.yaml`: Configuration file
- `.github/workflows/{language}-astgen-regression.yml`: GitHub Actions workflow (e.g., `javascript-astgen-regression.yml`)

### `compare`

Compare two pre-built distributions (typically used in CI).

```bash
astgen-regression compare \
  --base-dist ./dist-base \
  --pr-dist ./dist-pr \
  [--config PATH] \
  [--pr-number NUM] \
  [--base-ref REF] \
  [--pr-ref REF] \
  [--output-diffs DIR]
```

Options:
- `--base-dist`: Path to base distribution directory (required)
- `--pr-dist`: Path to PR distribution directory (required)
- `--config`: Config file path (default: `regression.yaml`)
- `--pr-number`: PR number for report metadata
- `--base-ref`: Human-readable base reference (e.g., `main @ abc1234`)
- `--pr-ref`: Human-readable PR reference (e.g., `feature @ def5678`)
- `--output-diffs`: Directory to write full diff files

Workflow:
1. Clones and prepares corpora
2. Runs astgen from both distributions (**repeated** runs per `execute.iterations` / `execute.warmup`; see **execute** section above)
3. Compares outputs (JSON normalization, text diffs) using the last timed run’s artifacts. For any single file whose combined normalized line count (base + PR) exceeds **20 000 lines**, the full unified diff is replaced with an `@@ diff omitted: … @@` placeholder; the structural summary (`N added, N removed, N changed`) is still reported.
4. Generates Markdown report and optional diff files
5. Exits with code 1 if regressions detected (useful for local validation; in CI the workflow catches this to allow PR comments)
### `local`

Run regression testing locally using git worktrees.

```bash
astgen-regression local [--base-branch BRANCH] [--config PATH]
```

Options:
- `--base-branch`: Base branch to compare against (default: from config or `main`)
- `--config`: Config file path (default: `regression.yaml`)

Workflow:
1. Builds the current branch (PR) in your repo, then creates a git worktree for the base branch and builds there
2. Runs `astgen-regression compare` with the two build output directories
3. Cleans up the worktree
4. Displays Markdown report

## Configuration File

The `regression.yaml` file defines your testing configuration:

```yaml
project:
  name: "my-astgen"           # Project name
  language: "javascript"       # Primary language

build:
  install_command: "yarn install"    # Optional: install dependencies
  build_command: "yarn build"        # Build the astgen
  dist_dir: "dist"                   # Output directory with built artifacts

execute:
  command: "node {dist_dir}/astgen.js -i {input_dir} -o {output_dir}"
  timeout: 600                       # Timeout in seconds
  # Wall-clock timing: median of `iterations` timed runs after `warmup` untimed runs.
  # Output is wiped before every run. Defaults: iterations=5, warmup=1.
  # iterations: 5
  # warmup: 1

artifacts:
  - name: "AST"                      # Artifact type name
    pattern: "*.json"                # File pattern to compare
    description: "Abstract Syntax Tree files"

corpora:
  - name: "lodash"                   # Unique corpus identifier
    label: "lodash@4.17.21"          # Human-readable label
    clone_url: "https://github.com/lodash/lodash.git"
    tag: "4.17.21"                   # Git tag to checkout
    input_subdir: "src"              # Optional: subdirectory to process

github:
  base_branch: "main"                # Base branch for comparisons
  python_version: "3.12"             # Python version for CI
```

**Security Note**: The `execute.command` and `build` commands are executed using a shell. The configuration file should be version-controlled and reviewed as part of your normal code review process. Do not use configuration files from untrusted sources.

### Configuration Sections

#### `project`
- `name`: Project identifier (used in reports)
- `language`: Primary language (for documentation)

#### `build`
- `install_command`: Optional command to install dependencies
- `build_command`: Command to build the astgen
- `dist_dir`: Directory containing built artifacts (relative to repo root)

#### `execute`
- `command`: Command template to run astgen
  - `{dist_dir}`: Replaced with distribution directory path
  - `{input_dir}`: Replaced with corpus source directory
  - `{output_dir}`: Replaced with output directory for artifacts
- `timeout`: Maximum execution time in seconds (each subprocess invocation)
- `iterations` (optional): How many **timed** runs to use for wall-clock stats (default **5**). Set to `1` for a single timed run (lower CI cost).
- `warmup` (optional): How many **untimed** runs to execute before timing (default **1**). Omitted from the median; use `0` to skip warmup. Warmup and timed runs each start from a **clean** output directory so every iteration measures the same work.

**Timing behavior (compare / CI):** For each corpus, astgen runs for the base and PR distributions using `warmup + iterations` invocations per side. The report shows the **median** of the timed runs and labels the table row `Wall-clock time (median of N)` when `iterations > 1`. Per-run seconds are logged to **stderr** (for example `[corpus]/base run 2/5: …`) so you can see runner variance in CI logs. **Diffs** compare artifacts from the **last successful timed run** on each side.

For noisy hosts (for example macOS GitHub runners) or debug builds, prefer **release** builds in `build_command` / `dist_dir` so timing deltas are meaningful; Swift projects often use `swift build -c release` and `.build/release`.

#### `artifacts`
List of artifact types to compare:
- `name`: Artifact type identifier
- `pattern`: Glob pattern matching output files
- `description`: Human-readable description

#### `corpora`
List of test codebases:
- `name`: Unique identifier (used in directories and reports)
- `label`: Display name with version info
- `clone_url`: Git repository URL
- `tag`: Git tag or commit to checkout
- `input_subdir`: Optional subdirectory to use as input (relative to clone root)

#### `github`
- `base_branch`: Default base branch for comparisons
- `python_version`: Python version for GitHub Actions
- `runs_on` (optional): GitHub-hosted runner label for the generated workflow (for example `macos-latest` for Swift; default in the template is `ubuntu-latest` when omitted)
## GitHub Actions Integration

The generated workflow (`.github/workflows/{language}-astgen-regression.yml`) runs automatically on:
- Pull requests to the base branch
- Manual workflow dispatch

The workflow:
1. Checks out the PR branch (your astgen project)
2. Checks out the astgen-regression repository
3. Installs astgen-regression from source
4. Builds distributions from both base and PR versions
5. Runs `astgen-regression compare`
6. Uploads Markdown report and diffs as artifacts
7. Posts report to PR as a comment (requires GitHub token with appropriate permissions)

**Note:** The workflow always succeeds (even when regressions are detected) to allow the PR comment to be posted. The report clearly indicates whether regressions were found. If you want to block PRs with regressions, add a branch protection rule that requires manual approval after reviewing the regression report.

## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/joernio/astgen-regression.git
cd astgen-regression

# Create virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install development dependencies (includes runtime dependencies + pytest)
pip install -r requirements-dev.txt
pip install -e .

# Run tests
pytest tests/ -v
```

### Project Structure

```
astgen-regression/
├── src/
│   └── astgen_regression/
│       ├── __init__.py
│       ├── __main__.py
│       ├── cli.py            # Entry point and argument parsing
│       ├── commands/         # Command implementations
│       │   ├── __init__.py
│       │   ├── compare.py    # Compare command
│       │   ├── init.py       # Init command
│       │   └── local.py      # Local command
│       ├── config.py         # Configuration loading
│       ├── corpus.py         # Corpus cloning and management
│       ├── executor.py       # Astgen execution
│       ├── metrics.py        # Performance metrics collection
│       ├── compare.py        # Output comparison and diffing
│       ├── report.py         # Markdown report generation
│       ├── worktree.py       # Git worktree management
│       └── templates/        # Jinja2 templates
│           ├── regression.yaml.j2
│           └── workflow.yml.j2
├── tests/                    # Test suite
└── docs/                     # Documentation and examples
    └── examples/
```

### Architecture

**Modular Design**: Each module has a single responsibility:
- `config.py`: Configuration loading and validation
- `corpus.py`: Clone and prepare test corpora
- `executor.py`: Run astgen (single run or warmup + median-of-N timed runs; wipes output between iterations)
- `compare.py`: Compare outputs (JSON-aware structural comparison using deepdiff, normalized text diffs)
- `metrics.py`: Collect and format performance statistics
- `report.py`: Generate Markdown reports with collapsible details
- `worktree.py`: Manage git worktrees for local testing
- `commands/`: Command implementations using the above modules

**Three Usage Modes**:
1. **Local testing** (`local`): Developer workflow with worktrees
2. **CI comparison** (`compare`): GitHub Actions with pre-built distributions
3. **Initialization** (`init`): Project setup with templates

**Template System**: Uses Jinja2 for generating:
- Config files with language-specific defaults
- GitHub Actions workflows

### Running Tests

```bash
# Run all tests
pytest tests/ -v

# Run specific test file
pytest tests/test_config.py -v

# Run with coverage
pytest tests/ --cov=astgen_regression --cov-report=html
```

## Examples

See `docs/examples/` for complete configuration examples:
- `javascript-example.yaml`: JavaScript/TypeScript project
- `swift-example.yaml`: Swift project

## License

Apache License 2.0

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## Support

For issues, questions, or contributions, visit:
https://github.com/joernio/astgen-monorepo

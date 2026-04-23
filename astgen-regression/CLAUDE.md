# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`astgen-regression` is a Python 3.12+ framework for regression testing language-specific AST generation tools (astgen). It clones real-world codebases at specific versions, builds astgen from base and PR branches using git worktrees, compares AST outputs, and generates detailed Markdown reports with diff analysis.

**Key Design Principle:** The framework is designed to run in two contexts:
1. **Local development**: Runs in the astgen project repository where the developer is working
2. **CI (GitHub Actions)**: Generated workflow runs in the astgen project, but checks out astgen-regression separately to `.astgen-regression/`

## Code Intelligence with LSP

**Always use LSP (Language Server Protocol) when exploring or modifying this codebase.** The LSP provides rich code intelligence for Python including:

- **Type information and documentation** - Use `hover` to get function signatures, type annotations, and docstrings
- **Symbol navigation** - Use `goToDefinition` to jump to function/class definitions, `findReferences` to find all usages
- **Code structure** - Use `documentSymbol` to list all symbols (functions, classes, variables) in a file
- **Call hierarchies** - Use `prepareCallHierarchy`, `incomingCalls`, `outgoingCalls` to understand function call relationships

**LSP startup:** The LSP server (Pyright) may take a few seconds to initialize. If you get "server is starting" errors, wait briefly and retry the operation.

**When to use LSP:**
- Before modifying a function, use `findReferences` to understand all call sites
- When debugging, use call hierarchy to trace execution flow
- When adding features, use `documentSymbol` to understand module structure
- When refactoring, use `goToDefinition` to navigate between related code

**Example workflow:**
1. Use `documentSymbol` to list functions in a module
2. Use `hover` to understand function signatures and types
3. Use `findReferences` to see where functions are called
4. Use `goToDefinition` to navigate to implementations
5. Use call hierarchy to understand execution flow

## Development Commands

### Setup
```bash
# Initial setup
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -r requirements.txt
pip install -e .

# Development setup (includes pytest)
pip install -r requirements-dev.txt
```

### Testing
```bash
# Run all tests
pytest tests/ -v

# Run specific test file
pytest tests/test_config.py -v

# Run specific test function
pytest tests/test_commands.py::test_cmd_compare_exits_with_code_1_on_regressions -v

# Run with coverage
pytest tests/ --cov=astgen_regression --cov-report=html
```

### Running the Tool
```bash
# Initialize configuration (creates regression.yaml and workflow)
astgen-regression init --language javascript

# Run local regression test (requires git worktree support)
astgen-regression local

# Compare two pre-built distributions
astgen-regression compare \
  --base-dist ./dist-base \
  --pr-dist ./dist-pr \
  --base-ref "main @ abc1234" \
  --pr-ref "feature @ def5678"
```

## Architecture

### Module Responsibilities

**Core modules** (single purpose, reusable):
- `config.py` - Load and validate `regression.yaml` configuration
- `corpus.py` - Clone and prepare test codebases at specific git tags
- `executor.py` - Execute astgen commands with timeout and error handling
- `compare.py` - Compare outputs (JSON-aware normalization, unified text diffs)
- `metrics.py` - Collect file counts and size statistics from output directories
- `report.py` - Generate Markdown reports with collapsible diff sections
- `worktree.py` - Manage git worktrees for isolated builds

**Command orchestrators** (no business logic, delegates to core modules):
- `commands/init.py` - Generate config and workflow files from templates
- `commands/compare.py` - Compare two distributions, exit with code 1 on regressions
- `commands/local.py` - Create worktrees, build both versions, run compare

**Infrastructure**:
- `cli.py` - Argument parsing and subcommand dispatch
- `__main__.py` - Entry point for `python -m astgen_regression`
- `templates/*.j2` - Jinja2 templates for config and workflow generation

### Data Flow

```
Config (regression.yaml)
  ↓
Command Orchestrator (commands/*.py)
  ↓
Core Modules (config, corpus, executor, compare, metrics)
  ↓
Report Generation (report.py)
  ↓
Output (Markdown to stdout, diffs to files)
```

### Exit Code Behavior

**Critical design decision**: `compare` command exits with code 1 when regressions detected:
- **Local use**: Failure signals problems to developer
- **CI use**: GitHub workflow catches exit code with `|| true` to prevent workflow failure, allowing PR comment to be posted

This dual behavior is intentional - same tool, different contexts.

## Configuration Structure

The `regression.yaml` file has five sections:

1. **project**: Metadata (name, language)
2. **build**: How to build the astgen (`install_command`, `build_command`, `dist_dir`)
3. **execute**: How to run astgen (`command` template with `{dist_dir}`, `{input_dir}`, `{output_dir}` placeholders)
4. **artifacts**: What to compare (`name`, `pattern`, `description`)
5. **corpora**: Test codebases (`name`, `label`, `clone_url`, `tag`, optional `input_subdir`)

**Security note**: `execute.command` and `build.*_command` run with `shell=True`. Configuration files must be version-controlled and reviewed. This is documented in README.md.

## Template System

Templates live in `src/astgen_regression/templates/`:
- `regression.yaml.j2` - Config file template with language-specific defaults
- `workflow.yml.j2` - GitHub Actions workflow template

**Language-specific defaults** are in `commands/init.py:LANGUAGE_DEFAULTS` (setup actions, build commands).

**Important**: Generated workflows run in the user's astgen project, not in astgen-regression. The workflow must checkout astgen-regression separately before installing.

## Testing Strategy

Tests use `pytest` with mocking for external dependencies:
- `unittest.mock` for subprocess calls, file I/O
- `tempfile.TemporaryDirectory` for isolated test environments
- Mock external dependencies: git, filesystem, subprocess
- Test both success and failure paths
- Validate specific behaviors (exit codes, file contents, error messages)

**Current coverage**: ~85-90% of critical paths. Missing tests are for init command (simple template rendering) and local command (integration test).

## Key Design Patterns

### JSON Comparison
`compare.py` normalizes JSON before comparison:
1. Parse JSON
2. Sort all keys recursively
3. Re-serialize with consistent formatting
4. Compare as text with unified diff

This handles semantically identical JSON with different key orders.

### Collapsible Diffs
`report.py` uses `<details>` HTML tags for diff sections:
- Default collapsed for readability
- Full diffs still available in report
- Optional `--output-diffs` writes full diffs to separate files

### Worktree Isolation
`local` command uses git worktrees to build base and PR versions in parallel directories:
1. Build PR version first (in current directory)
2. Create worktree for base branch at `.worktrees/regression-base`
3. Build base version in worktree
4. Compare distributions
5. Always cleanup worktree (in `finally` block)

This avoids switching branches and losing uncommitted changes.

## Common Gotchas

1. **Virtual environment**: Always activate `.venv` before development. The project uses modern Python 3.12+ features (dict[str, Any], Path | None).

2. **Exit codes**: Don't change `compare` exit code behavior without understanding dual context (local vs CI). CI workflow relies on `|| true` pattern.

3. **Template paths**: Templates are installed as package data via `pyproject.toml`. They're accessed via `importlib.resources`, not direct file paths.

4. **Worktree cleanup**: Always use `try/finally` for worktree operations. Failed cleanup leaves `.worktrees/` directory that breaks future runs.

5. **Mock objects in tests**: When mocking args with Path attributes (like `args.base_dist`), create actual Path objects or directories. Tests fail if code calls `.exists()` on string paths.

6. **Report marker**: PR comment updates use `<!-- astgen-regression -->` HTML comment as marker. This string appears in both `report.py` and `workflow.yml.j2` - keep them synchronized.

7. **Timeout handling**: Both corpus cloning (5 min) and astgen execution (configurable) have timeouts. Tests must mock `subprocess.run` to avoid actual timeouts.

## Error Handling Conventions

- All ERROR messages prefix with `ERROR:` and write to stderr
- All WARNING messages prefix with `WARNING:` and write to stderr
- Reports write to stdout (allows redirection: `> report.md`)
- Exit code 1 for all errors (config, git, build, execution failures)
- Error messages include actionable suggestions when possible

## Language Support

Currently supported languages (in `commands/init.py:LANGUAGE_DEFAULTS`):
- javascript (Node.js/yarn setup)
- rust (Rust toolchain)
- swift (Swift Package Manager)
- dotnet (.NET SDK)
- ruby (Ruby/Bundle setup)
- go (Go toolchain)
- abap (build configuration TBD)

To add new language: Update `LANGUAGE_DEFAULTS` dict in `commands/init.py` with:
- `setup_action`: GitHub Actions setup step (uses, with params)
- `build_command`: Default build command
- `install_command`: Optional dependency install command
- `dist_dir`: Default distribution directory

## GitHub Actions Integration

Generated workflow (`.github/workflows/regression.yml`) runs on pull requests:

**Critical workflow steps**:
1. Checkout PR branch with `fetch-depth: 0` (required for worktree support)
2. **Checkout astgen-regression to `.astgen-regression/`** (the workflow runs in user's project)
3. Install astgen-regression from source (`cd .astgen-regression && pip install -e .`)
4. Setup language environment (optional, language-specific)
5. Build PR version
6. Build base version in worktree
7. Run regression with `|| true` (catches exit code 1)
8. Always cleanup worktree
9. Upload diff artifacts
10. Post/update PR comment; never add co-authors

**Never fails workflow**: Report always posts, even with regressions. This is intentional design.

# AST Generator Monorepo Consolidation Design

**Date:** 2026-04-16  
**Status:** Approved

## Overview

This design consolidates seven language-specific AST generation repositories plus a new regression testing framework into a single monorepo (`joernio/astgen-monorepo`) while maintaining complete independence for each tool's build, test, versioning, and release processes.

## Goals

1. **Co-locate all AST generators** in a single repository for easier discovery and management
2. **Maintain complete independence** for each language tool (separate versions, releases, dependencies)
3. **Preserve git history** from all source repositories
4. **Use path-based CI triggers** so changes only affect the relevant language
5. **Support independent releases** via prefixed git tags
6. **Include regression testing framework** as a peer tool in the monorepo

## Source Repositories

The following repositories will be consolidated:

- `joernio/astgen` → `javascript-astgen/` (default branch: `main`)
- `joernio/rust_ast_gen` → `rust-astgen/` (default branch: `main`)
- `joernio/SwiftAstGen` → `swift-astgen/` (default branch: **`master`**)
- `joernio/DotNetAstGen` → `dotnet-astgen/` (default branch: `main`)
- `joernio/goastgen` → `go-astgen/` (default branch: `main`)
- `joernio/ruby_ast_gen` → `ruby-astgen/` (default branch: `main`)
- `joernio/abap_ast_gen` → `abap-astgen/` (empty repo, defer until code exists)

The `astgen-regression/` directory is **not migrated from a separate repository**. It is created fresh in the monorepo as a new top-level directory for the regression testing framework (currently developed locally, not published to GitHub).

## Repository Structure

```
joernio/astgen-monorepo/
├── .github/
│   └── workflows/
│       ├── dotnet-astgen-ci.yml
│       ├── dotnet-astgen-release.yml
│       ├── go-astgen-ci.yml
│       ├── go-astgen-release.yml
│       ├── javascript-astgen-ci.yml
│       ├── javascript-astgen-release.yml
│       ├── ruby-astgen-ci.yml
│       ├── ruby-astgen-release.yml
│       ├── rust-astgen-ci.yml
│       ├── rust-astgen-release.yml
│       ├── swift-astgen-ci.yml
│       ├── swift-astgen-release.yml
│       ├── astgen-regression-ci.yml
│       └── astgen-regression-release.yml
│
├── abap-astgen/
│   ├── (deferred — joernio/abap_ast_gen is currently empty)
│   └── (migrate once the repo has code)
│
├── dotnet-astgen/
│   ├── (all files from joernio/DotNetAstGen with preserved history)
│   ├── DotNetAstGen.csproj (version via NEXT_VERSION env var)
│   └── ...
│
├── go-astgen/
│   ├── (all files from joernio/goastgen with preserved history)
│   ├── main.go (version via ldflags)
│   └── ...
│
├── javascript-astgen/
│   ├── (all files from joernio/astgen with preserved history)
│   ├── package.json (with "version")
│   └── ...
│
├── ruby-astgen/
│   ├── (all files from joernio/ruby_ast_gen with preserved history)
│   ├── lib/ruby_ast_gen/version.rb (with VERSION constant)
│   └── ...
│
├── rust-astgen/
│   ├── (all files from joernio/rust_ast_gen with preserved history)
│   ├── Cargo.toml (with version)
│   └── ...
│
├── swift-astgen/
│   ├── (all files from joernio/SwiftAstGen with preserved history)
│   ├── Package.swift (tag-driven versioning)
│   └── ...
│
├── astgen-regression/
│   ├── (created fresh — not migrated from a separate repo)
│   ├── pyproject.toml (with version)
│   └── ...
│
└── README.md  (minimal - explains structure and points to each tool)
```

### Key Principles

- **Each language directory is completely self-contained** - No shared code or dependencies
- **No cross-language coupling** - Each tool builds and runs independently
- **Root README is minimal** - Just a directory pointing to each tool
- **Git history preserved** - Full commit history migrated for each tool
- **Regression testing as peer** - Testing framework lives alongside the tools it tests

## CI/CD Design

### Approach: Simple Path-Based Workflows

Each language tool has two dedicated workflows:
1. **CI workflow** - Build, test, lint on pushes and PRs
2. **Release workflow** - Create GitHub releases with binaries on tag push

This approach maximizes:
- **Independence** - Language teams own their workflows completely
- **Clarity** - Each workflow file maps directly to one language
- **Simplicity** - Uses GitHub's native path filtering, no custom scripts

### Important: Use Existing Workflows as Starting Point

The CI and release workflow patterns below are **generic templates**. Before implementation, clone all source repositories and review their actual `.github/workflows/` directories. Each language may have specific steps, matrix strategies, multi-platform builds, or toolchain configurations that must be preserved. The monorepo workflows should be adapted from the real workflows, not written from scratch based on these templates.

### Existing Workflow Inventory

| Repository | Workflows | Notes |
|-----------|-----------|-------|
| joernio/astgen | `pr.yml`, `regression.yml`, `release.yml` | Has regression testing |
| joernio/rust_ast_gen | `pr.yml`, `release.yml` | |
| joernio/SwiftAstGen | `pr.yml`, `regression.yml`, `release.yml` | Has regression testing; legacy `scripts/regression.py` also present |
| joernio/DotNetAstGen | `pr.yml`, `release.yml` | |
| joernio/goastgen | `pr.yml`, `release.yml` | |
| joernio/ruby_ast_gen | `pr.yaml`, `release.yml` | Note: `.yaml` extension (not `.yml`) |
| joernio/abap_ast_gen | None | Empty repo |

**Monorepo mapping:** Each existing `pr.yml` becomes `<language>-astgen-ci.yml` with added path filters and `working-directory`. Each `release.yml` becomes `<language>-astgen-release.yml` with tag pattern changed to `<language>-astgen/v*`. Existing `regression.yml` workflows are adapted to work with the co-located `astgen-regression/` directory.

### CI Workflow Pattern

File: `.github/workflows/<language>-astgen-ci.yml`

```yaml
name: <Language> AST Generator CI

on:
  push:
    branches: [main]
    paths:
      - '<language>-astgen/**'
      - '.github/workflows/<language>-astgen-ci.yml'
  pull_request:
    paths:
      - '<language>-astgen/**'
      - '.github/workflows/<language>-astgen-ci.yml'

jobs:
  build-and-test:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: <language>-astgen
    
    steps:
      - uses: actions/checkout@v6
      
      - name: Setup <Language> Environment
        # Language-specific setup (Node.js, Rust, .NET SDK, etc.)
      
      - name: Build
        run: <language-specific build command>
      
      - name: Test
        run: <language-specific test command>
      
      - name: Lint
        run: <language-specific lint command>
```

**Behavior:**
- Only runs when files in `<language>-astgen/` change
- Works in the language directory context
- Completely independent from other languages

### Release Workflow Pattern

File: `.github/workflows/<language>-astgen-release.yml`

```yaml
name: <Language> AST Generator Release

on:
  push:
    tags:
      - '<language>-astgen/v*'

jobs:
  release:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: <language>-astgen
    
    steps:
      - uses: actions/checkout@v6
      
      - name: Extract version from tag
        id: version
        run: echo "VERSION=${GITHUB_REF#refs/tags/<language>-astgen/v}" >> $GITHUB_OUTPUT
      
      - name: Setup <Language> Environment
        # Language-specific setup
      
      - name: Build release artifacts
        run: <language-specific release build>
      
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v3
        with:
          tag_name: ${{ github.ref_name }}
          name: <Language> AST Generator v${{ steps.version.outputs.VERSION }}
          files: |
            <language>-astgen/<binary-path>
```

**Behavior:**
- Triggered only by tags matching `<language>-astgen/v*`
- Builds optimized/release binaries
- Creates GitHub Release with downloadable artifacts

### Multi-Platform Builds

For languages requiring platform-specific binaries, use matrix strategies:

```yaml
jobs:
  release:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            platform: linux-amd64
          - os: macos-latest
            platform: macos-arm64
          - os: windows-latest
            platform: windows-amd64
```

Upload all platform binaries to the same release.

## Versioning and Release Management

### Version Storage

Each language maintains its version in the most appropriate format:

| Language | Version Location | Format | Notes |
|----------|-----------------|--------|-------|
| JavaScript | `package.json` | `"version": "3.42.0"` | Standard npm version field |
| Rust | `Cargo.toml` | `version = "0.1.0"` | Standard Cargo version field |
| .NET | `DotNetAstGen.csproj` | `<Version>$(NEXT_VERSION)</Version>` | Injected via env var at build time; defaults to `0.0.1-local` |
| Ruby | `lib/ruby_ast_gen/version.rb` | `VERSION = "0.58.0"` | Constant in version module, referenced by gemspec |
| Swift | Tag-driven | No version file | Releases driven by git tags only; no embedded version in `Package.swift` |
| Go | `main.go` | `var Version = "dev"` | Injected via ldflags at build time; module path is `privado.ai/goastgen` |
| ABAP | TBD | TBD | Repo is currently empty |
| Python (regression) | `pyproject.toml` | `version = "1.0.0"` | Created fresh in monorepo |

**Principle:** Use language-native version files where standard practice exists. For languages using build-time injection (.NET, Go) or tag-driven releases (Swift), the monorepo release workflow extracts the version from the git tag.

### Tagging Convention

Tags follow the pattern: `<language>-astgen/v<semver>`

**Examples** (based on current versions):
```
dotnet-astgen/v0.43.0
go-astgen/v0.1.0
javascript-astgen/v3.42.0
ruby-astgen/v0.58.0
rust-astgen/v0.1.0
swift-astgen/v0.3.9
astgen-regression/v1.0.0
```

**Benefits:**
- Clear namespace separation per language
- GitHub can filter tags and releases
- Git commands can filter by prefix: `git tag -l "rust-astgen/*"`
- No tag conflicts between languages

### Release Process

**Manual steps** (performed by maintainers):

1. **Update version** in the language directory
   ```bash
   cd rust-astgen
   # Edit Cargo.toml: version = "2.1.0"
   git commit -am "rust-astgen: bump version to 2.1.0"
   git push origin main
   ```

2. **Create and push tag**
   ```bash
   git tag rust-astgen/v2.1.0
   git push origin rust-astgen/v2.1.0
   ```

3. **Automated workflow runs**
   - Builds release binaries
   - Creates GitHub Release
   - Attaches binaries as assets

**Result:** GitHub Release at `https://github.com/joernio/astgen-monorepo/releases/tag/rust-astgen/v2.1.0`

## Migration Strategy

### Git History Preservation

Use `git filter-repo` to rewrite each repository's history so all files appear under the target subdirectory, then merge into the monorepo. This preserves:
- All commits with original messages, authors, and timestamps
- Git blame functionality
- Commit graphs

### Migration Process

**Prerequisites:**
- Install `git-filter-repo`: `pip install git-filter-repo`
- Clone all source repositories locally

**For each source repository:**

1. **Clone with full history**
   ```bash
   git clone https://github.com/joernio/rust_ast_gen.git rust-temp
   cd rust-temp
   ```

2. **Rewrite history to subdirectory**
   ```bash
   git filter-repo --to-subdirectory-filter rust-astgen/
   ```
   
   Note: After `git filter-repo`, the repository is rewritten in place. This is destructive to the local clone, which is why we work on a temporary clone.

3. **Merge into monorepo**
   ```bash
   cd ../astgen-monorepo
   git remote add rust-temp ../rust-temp
   git fetch rust-temp --tags
   git merge --allow-unrelated-histories rust-temp/main -m "Migrate rust-astgen from joernio/rust_ast_gen"
   git remote remove rust-temp
   ```
   
   **Important:** Use the actual default branch name from the source repo. Most repos use `main`, but SwiftAstGen uses `master`.

4. **Clean up**
   ```bash
   rm -rf ../rust-temp
   ```

### Migration Order

```
1. Create new repository: joernio/astgen-monorepo
2. Initialize with minimal README.md
3. Commit initial structure

4. Migrate repositories in order:
   a. javascript-astgen    (from joernio/astgen, branch: main)
   b. rust-astgen          (from joernio/rust_ast_gen, branch: main)
   c. swift-astgen         (from joernio/SwiftAstGen, branch: **master**)
   d. dotnet-astgen        (from joernio/DotNetAstGen, branch: main)
   e. go-astgen            (from joernio/goastgen, branch: main)
   f. ruby-astgen          (from joernio/ruby_ast_gen, branch: main)
   — abap-astgen: DEFERRED (joernio/abap_ast_gen is empty)

5. Create astgen-regression/ directory (fresh, not migrated from a repo)
   - Copy source from local development
   - No git history to preserve

6. Add GitHub Actions workflows to .github/workflows/

7. Test workflow triggers:
   - Make a test commit to one language directory
   - Verify only that language's CI runs
   - Test tag-based release trigger

8. Create initial tags matching current versions:
   git tag javascript-astgen/v3.42.0
   git tag rust-astgen/v0.1.0
   git tag swift-astgen/v0.3.9
   git tag dotnet-astgen/v0.43.0
   git tag go-astgen/v0.1.0
   git tag ruby-astgen/v0.58.0
   git tag astgen-regression/v1.0.0
   git push origin --tags

9. Post-migration actions (manual, see "Post-Migration Actions (Manual)" section)
```

### Post-Migration Actions (Manual)

> **These steps require human judgment and must be performed manually.** They are NOT part of any automated migration script.

1. **Archive old repositories**
   - Mark as read-only in GitHub settings
   - Add prominent notice in README redirecting to monorepo
   - Update repository description: "ARCHIVED - Moved to joernio/astgen-monorepo"
   - Do NOT delete (preserves existing links and references)

2. **Update documentation**
   - Update installation instructions to reference monorepo
   - Update build badges in READMEs
   - Update contribution guidelines

### Rollback Plan

If critical issues arise during migration:

1. **Old repositories remain available** (archived but not deleted)
2. **Can revert to using old repos** by un-archiving them
3. **Individual tool history** can be extracted from monorepo using `git filter-repo` if needed
4. **No data loss** - all history preserved in both locations during transition period

## Regression Testing Integration

### Directory Placement

The `astgen-regression` tool becomes `/astgen-regression/` in the monorepo, making it a peer to the language tools rather than a separate repository.

**Benefits:**
- Single repository to test all AST generators
- Easier to coordinate changes affecting multiple tools
- Shared version control for tools and tests
- Relative path references possible

### Configuration Changes

Regression test configs need updated paths when testing from the monorepo:

**Before (separate repos):**
```yaml
build:
  build_command: "cargo build --release"
  dist_dir: "target/release"

execute:
  command: "{dist_dir}/rust-astgen -i {input_dir} -o {output_dir}"
```

**After (monorepo):**
```yaml
build:
  build_command: "cd rust-astgen && cargo build --release"
  dist_dir: "rust-astgen/target/release"

execute:
  command: "{dist_dir}/rust-astgen -i {input_dir} -o {output_dir}"
```

The key changes:
1. `build_command` changes directory to the language subdirectory
2. `dist_dir` includes the language directory prefix
3. `execute.command` template works the same (uses `dist_dir` placeholder)

### Optional CI Integration

Each language's CI workflow can optionally run regression tests:

```yaml
# In rust-astgen-ci.yml (optional)
- name: Run regression tests
  run: |
    cd ../astgen-regression
    pip install -e .
    astgen-regression compare \
      --config ../rust-astgen/regression.yaml \
      --base-dist ../rust-astgen/target/release \
      --pr-dist ../rust-astgen/target/release
```

**This is optional** - each language team decides whether to include regression testing in their CI.

### Regression Framework CI

The `astgen-regression` tool itself has CI workflows:

- `astgen-regression-ci.yml` - Tests the Python framework
- `astgen-regression-release.yml` - Publishes to PyPI and/or GitHub releases
- Path trigger: `astgen-regression/**`
- Tag pattern: `astgen-regression/v*`

## Root README Structure

The root `README.md` serves as a minimal directory to the tools:

```markdown
# Joern AST Generators

A monorepo containing language-specific AST generation tools for the Joern code analysis platform.

## Available Tools

Each tool is independently versioned and released:

- **[JavaScript AST Generator](./javascript-astgen/)** - Generate ASTs for JavaScript/TypeScript code
- **[Rust AST Generator](./rust-astgen/)** - Generate ASTs for Rust code  
- **[Swift AST Generator](./swift-astgen/)** - Generate ASTs for Swift code
- **[.NET AST Generator](./dotnet-astgen/)** - Generate ASTs for C#/F#/.NET code
- **[Go AST Generator](./go-astgen/)** - Generate ASTs for Go code
- **[Ruby AST Generator](./ruby-astgen/)** - Generate ASTs for Ruby code
- **[ABAP AST Generator](./abap-astgen/)** - Generate ASTs for ABAP code

## Regression Testing

- **[AST Generator Regression Framework](./astgen-regression/)** - Testing framework for all AST generators

## Repository Structure

Each language tool is self-contained with its own:
- Build system and dependencies
- Tests and documentation
- CI/CD workflows
- Independent versioning and releases

## Releases

Each tool uses prefixed tags for releases:
- `javascript-astgen/v1.0.0`
- `rust-astgen/v2.0.0`
- etc.

See the [Releases page](../../releases) for downloadable binaries.

## Contributing

Each tool has its own contribution guidelines. See the README in each directory.

## License

See individual tool directories for license information.
```

**Design Principles:**
- **Minimal** - No duplication of information that lives in subdirectories
- **Navigation-focused** - Clear links to each tool
- **No technical details** - Details belong in each tool's README
- **Simple maintenance** - Adding a new language just requires one line

## Trade-offs and Decisions

### Why Simple Path-Based Workflows (Not Matrix)?

**Chosen:** Separate workflow files per language  
**Rejected:** Centralized matrix workflow with path detection

**Rationale:**
- Maximizes independence - language teams control their workflows
- Simpler to understand - direct mapping between file and language
- Easier to customize - no need to work around matrix constraints
- Reliable - uses GitHub's native path filtering
- Better failure isolation - one language's CI issues don't affect others

**Cost:** More workflow files (~14 total once all languages active), some structural duplication

**Mitigation:** Duplication is acceptable given the independence requirement; could add workflow templates later if needed

### Why Preserve Git History?

**Chosen:** Use `git filter-repo` to preserve all commits  
**Rejected:** Fresh start with current state

**Rationale:**
- Preserves blame/authorship for debugging and credit
- Maintains audit trail for compliance/security
- Allows bisecting historical bugs
- Respects contributors' work

**Cost:** More complex migration process, potential for history conflicts

**Mitigation:** Migration is one-time cost; history stays clean with proper filtering

### Why Co-locate Regression Testing?

**Chosen:** Include `astgen-regression` in monorepo  
**Rejected:** Keep as separate repository

**Rationale:**
- Tests and tools naturally belong together
- Easier to coordinate changes across multiple tools
- Single clone for all development work
- Can reference tools via relative paths

**Cost:** Regression framework is conceptually different from language tools

**Mitigation:** Clear directory structure makes distinction obvious; independent versioning preserves separation

### Why Minimal Root README?

**Chosen:** Directory-style root README  
**Rejected:** Comprehensive documentation at root

**Rationale:**
- Avoids duplication - details live in subdirectories
- Easier maintenance - changes only affect one place
- Clear ownership - each tool documents itself
- Scales well - adding languages doesn't bloat root

**Cost:** Users need to navigate to subdirectories for details

**Mitigation:** This is expected behavior for monorepos; clear links make navigation easy

## Success Criteria

The migration is successful when:

1. **All tools build and test independently** in the monorepo
2. **CI triggers work correctly** - changes to one tool don't trigger others
3. **Releases work independently** - can tag and release any tool without affecting others
4. **Git history is preserved** - `git blame` and `git log` show original commits
5. **Old repositories are archived** with clear migration notices
6. **Documentation is updated** across all tools and external references
7. **First successful release** created from monorepo for at least one language

## Future Considerations

### Potential Shared Infrastructure (Not in Initial Scope)

If patterns emerge across languages, consider extracting:
- Common CI workflow templates (via composite actions)
- Shared test corpora for regression testing
- Common build scripts or utilities

**Important:** Only add shared infrastructure if clear duplication emerges and language teams agree it's beneficial.

### Alternative Tagging Schemes

If the prefixed tag approach proves cumbersome:
- Consider using GitHub Releases with custom metadata instead of tags
- Or use separate release branches per language
- Current design is flexible enough to switch later if needed

### Multi-Language Releases

If demand emerges for "all-in-one" releases:
- Could add a workflow that packages all languages into a single release
- Tags like `all-tools/v1.0.0` that bundle current versions of each
- Not in initial scope - requires more coordination

## Implementation Plan

The detailed implementation plan will be created using the `writing-plans` skill after this design is approved.

**High-level phases:**

1. **Phase 1: Repository Setup** (1 week)
   - Create joernio/astgen-monorepo
   - Migrate git history for all tools
   - Verify history preservation

2. **Phase 2: Workflow Implementation** (1 week)
   - Create CI workflows for all languages
   - Create release workflows for all languages
   - Test path-based triggers

3. **Phase 3: Testing and Validation** (1 week)
   - Run CI for each language
   - Test release process with beta tags
   - Verify regression testing integration

4. **Phase 4: Documentation and Migration** (1 week)
   - Update all READMEs
   - Archive old repositories
   - Notify users and update external links

5. **Phase 5: First Production Releases** (ongoing)
   - Create initial tags matching current versions
   - Monitor for issues
   - Address feedback

**Total estimated time:** 4-5 weeks with proper testing and validation.

## Conclusion

This design provides a clear path to consolidating seven AST generation tools into a single monorepo while maintaining complete independence for each tool. The approach prioritizes simplicity, clarity, and independence over optimization, resulting in a maintainable structure that scales well as new languages are added.

Key benefits:
- ✅ Single repository for discovery and management
- ✅ Complete independence per language
- ✅ Preserved git history
- ✅ Simple, explicit CI/CD
- ✅ Independent versioning and releases
- ✅ Integrated regression testing
- ✅ Clear migration path with rollback options

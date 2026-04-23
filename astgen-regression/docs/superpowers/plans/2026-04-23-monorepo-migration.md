# AST Generator Monorepo Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate seven language-specific AST generator repositories into a single monorepo with preserved git history, independent CI/CD, and integrated regression testing.

**Architecture:** Each language tool remains completely independent in its own subdirectory with dedicated CI/CD workflows. Git history is preserved via `git filter-repo` rewriting. Path-based workflow triggers ensure isolation. Regression testing framework co-located as peer directory.

**Tech Stack:** Git (filter-repo), GitHub Actions, language-specific toolchains (Node.js, Rust, .NET SDK, Go, Ruby, Swift, Python)

---

## Prerequisites Check

### Task 0: Verify Environment and Install Dependencies

**Files:**
- None (environment setup only)

- [ ] **Step 1: Verify git version**

Run: `git --version`
Expected: Git version 2.30+ (required for modern filter-repo compatibility)

- [ ] **Step 2: Install git-filter-repo**

Run: `pip3 install git-filter-repo`
Expected: "Successfully installed git-filter-repo-X.X.X"

- [ ] **Step 3: Verify installation**

Run: `git filter-repo --version`
Expected: Version number displayed (e.g., "2.38.0")

- [ ] **Step 4: Create workspace directory for temporary clones**

Run: `mkdir -p /tmp/astgen-migration-workspace && cd /tmp/astgen-migration-workspace`
Expected: Directory created, no errors

- [ ] **Step 5: Verify monorepo location**

Run: `cd /Volumes/Work/workspace_intellij/astgen-monorepo && pwd && git status`
Expected: Shows current directory and clean git status on main branch

---

## Phase 1: Repository Setup and Initial Structure

### Task 1: Create Root README

**Files:**
- Create: `/Volumes/Work/workspace_intellij/astgen-monorepo/README.md`

- [ ] **Step 1: Write root README**

```bash
cd /Volumes/Work/workspace_intellij/astgen-monorepo
```

Create `README.md` with this content:

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
- **[ABAP AST Generator](./abap-astgen/)** - Generate ASTs for ABAP code _(deferred)_

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

- [ ] **Step 2: Verify README created**

Run: `cat README.md | head -20`
Expected: First 20 lines of README displayed correctly

- [ ] **Step 3: Commit initial README**

```bash
git add README.md
git commit -m "docs: add root README with monorepo structure"
```

Expected: Commit created successfully

- [ ] **Step 4: Verify commit**

Run: `git log --oneline -1`
Expected: Shows "docs: add root README with monorepo structure"

---

### Task 2: Migrate JavaScript AST Generator

**Files:**
- Create: `/Volumes/Work/workspace_intellij/astgen-monorepo/javascript-astgen/*` (entire directory from joernio/astgen)

- [ ] **Step 1: Clone source repository**

```bash
cd /tmp/astgen-migration-workspace
git clone https://github.com/joernio/astgen.git javascript-astgen-temp
cd javascript-astgen-temp
```

Expected: Repository cloned successfully

- [ ] **Step 2: Verify default branch**

Run: `git branch --show-current`
Expected: "main"

- [ ] **Step 3: Check commit count before rewrite**

Run: `git rev-list --count HEAD`
Expected: Number of commits (save this for verification later)

- [ ] **Step 4: Rewrite history to subdirectory**

Run: `git filter-repo --to-subdirectory-filter javascript-astgen/ --force`
Expected: "Parsed X commits. New history written in Y seconds"

- [ ] **Step 5: Verify history rewrite**

Run: `git log --oneline -3 --name-only`
Expected: All file paths now start with `javascript-astgen/`

- [ ] **Step 6: Verify commit count unchanged**

Run: `git rev-list --count HEAD`
Expected: Same number as Step 3 (history preserved)

- [ ] **Step 7: Merge into monorepo**

```bash
cd /Volumes/Work/workspace_intellij/astgen-monorepo
git remote add javascript-temp /tmp/astgen-migration-workspace/javascript-astgen-temp
git fetch javascript-temp --tags
git merge --allow-unrelated-histories javascript-temp/main -m "feat: migrate javascript-astgen from joernio/astgen

Preserved full git history via git filter-repo.
All files moved to javascript-astgen/ subdirectory.

Source: https://github.com/joernio/astgen"
```

Expected: Merge completed successfully

- [ ] **Step 8: Verify migration**

Run: `ls -la javascript-astgen/ | head -10`
Expected: Directory exists with source files

- [ ] **Step 9: Verify git history preserved**

Run: `git log --oneline javascript-astgen/ | wc -l`
Expected: Same commit count as Step 3

- [ ] **Step 10: Test git blame**

Run: `git log --follow -- javascript-astgen/package.json | head -5`
Expected: Shows original commit history for package.json

- [ ] **Step 11: Clean up remote and temp files**

```bash
git remote remove javascript-temp
rm -rf /tmp/astgen-migration-workspace/javascript-astgen-temp
```

Expected: Remote removed, temp directory deleted

- [ ] **Step 12: Push to origin**

Run: `git push origin main --tags`
Expected: Push successful with all commits and tags

---

### Task 3: Migrate Rust AST Generator

**Files:**
- Create: `/Volumes/Work/workspace_intellij/astgen-monorepo/rust-astgen/*` (entire directory from joernio/rust_ast_gen)

- [ ] **Step 1: Clone source repository**

```bash
cd /tmp/astgen-migration-workspace
git clone https://github.com/joernio/rust_ast_gen.git rust-astgen-temp
cd rust-astgen-temp
```

Expected: Repository cloned successfully

- [ ] **Step 2: Verify default branch**

Run: `git branch --show-current`
Expected: "main"

- [ ] **Step 3: Check commit count**

Run: `git rev-list --count HEAD`
Expected: Number of commits (note for verification)

- [ ] **Step 4: Rewrite history to subdirectory**

Run: `git filter-repo --to-subdirectory-filter rust-astgen/ --force`
Expected: History rewritten successfully

- [ ] **Step 5: Verify paths**

Run: `git log --oneline -3 --name-only`
Expected: All paths prefixed with `rust-astgen/`

- [ ] **Step 6: Merge into monorepo**

```bash
cd /Volumes/Work/workspace_intellij/astgen-monorepo
git remote add rust-temp /tmp/astgen-migration-workspace/rust-astgen-temp
git fetch rust-temp --tags
git merge --allow-unrelated-histories rust-temp/main -m "feat: migrate rust-astgen from joernio/rust_ast_gen

Preserved full git history via git filter-repo.
All files moved to rust-astgen/ subdirectory.

Source: https://github.com/joernio/rust_ast_gen"
```

Expected: Merge completed

- [ ] **Step 7: Verify migration**

Run: `ls -la rust-astgen/ | head -10 && git log --oneline rust-astgen/ | head -5`
Expected: Files present, history visible

- [ ] **Step 8: Clean up**

```bash
git remote remove rust-temp
rm -rf /tmp/astgen-migration-workspace/rust-astgen-temp
```

Expected: Cleanup successful

- [ ] **Step 9: Push to origin**

Run: `git push origin main --tags`
Expected: Push successful

---

### Task 4: Migrate Swift AST Generator

**Files:**
- Create: `/Volumes/Work/workspace_intellij/astgen-monorepo/swift-astgen/*` (entire directory from joernio/SwiftAstGen)

**IMPORTANT:** SwiftAstGen uses `master` as default branch, not `main`

- [ ] **Step 1: Clone source repository**

```bash
cd /tmp/astgen-migration-workspace
git clone https://github.com/joernio/SwiftAstGen.git swift-astgen-temp
cd swift-astgen-temp
```

Expected: Repository cloned successfully

- [ ] **Step 2: Verify default branch is master**

Run: `git branch --show-current`
Expected: "master" (NOT main)

- [ ] **Step 3: Check commit count**

Run: `git rev-list --count HEAD`
Expected: Number of commits (note for verification)

- [ ] **Step 4: Rewrite history to subdirectory**

Run: `git filter-repo --to-subdirectory-filter swift-astgen/ --force`
Expected: History rewritten successfully

- [ ] **Step 5: Verify paths**

Run: `git log --oneline -3 --name-only`
Expected: All paths prefixed with `swift-astgen/`

- [ ] **Step 6: Merge into monorepo using master branch**

```bash
cd /Volumes/Work/workspace_intellij/astgen-monorepo
git remote add swift-temp /tmp/astgen-migration-workspace/swift-astgen-temp
git fetch swift-temp --tags
git merge --allow-unrelated-histories swift-temp/master -m "feat: migrate swift-astgen from joernio/SwiftAstGen

Preserved full git history via git filter-repo.
All files moved to swift-astgen/ subdirectory.
Source used master branch (not main).

Source: https://github.com/joernio/SwiftAstGen"
```

Expected: Merge completed

- [ ] **Step 7: Verify migration**

Run: `ls -la swift-astgen/ | head -10 && git log --oneline swift-astgen/ | head -5`
Expected: Files present, history visible

- [ ] **Step 8: Clean up**

```bash
git remote remove swift-temp
rm -rf /tmp/astgen-migration-workspace/swift-astgen-temp
```

Expected: Cleanup successful

- [ ] **Step 9: Push to origin**

Run: `git push origin main --tags`
Expected: Push successful

---

### Task 5: Migrate .NET AST Generator

**Files:**
- Create: `/Volumes/Work/workspace_intellij/astgen-monorepo/dotnet-astgen/*` (entire directory from joernio/DotNetAstGen)

- [ ] **Step 1: Clone source repository**

```bash
cd /tmp/astgen-migration-workspace
git clone https://github.com/joernio/DotNetAstGen.git dotnet-astgen-temp
cd dotnet-astgen-temp
```

Expected: Repository cloned successfully

- [ ] **Step 2: Verify default branch**

Run: `git branch --show-current`
Expected: "main"

- [ ] **Step 3: Check commit count**

Run: `git rev-list --count HEAD`
Expected: Number of commits

- [ ] **Step 4: Rewrite history to subdirectory**

Run: `git filter-repo --to-subdirectory-filter dotnet-astgen/ --force`
Expected: History rewritten successfully

- [ ] **Step 5: Merge into monorepo**

```bash
cd /Volumes/Work/workspace_intellij/astgen-monorepo
git remote add dotnet-temp /tmp/astgen-migration-workspace/dotnet-astgen-temp
git fetch dotnet-temp --tags
git merge --allow-unrelated-histories dotnet-temp/main -m "feat: migrate dotnet-astgen from joernio/DotNetAstGen

Preserved full git history via git filter-repo.
All files moved to dotnet-astgen/ subdirectory.

Source: https://github.com/joernio/DotNetAstGen"
```

Expected: Merge completed

- [ ] **Step 6: Verify and clean up**

Run: `ls -la dotnet-astgen/ | head -10 && git remote remove dotnet-temp && rm -rf /tmp/astgen-migration-workspace/dotnet-astgen-temp`
Expected: Files present, cleanup successful

- [ ] **Step 7: Push to origin**

Run: `git push origin main --tags`
Expected: Push successful

---

### Task 6: Migrate Go AST Generator

**Files:**
- Create: `/Volumes/Work/workspace_intellij/astgen-monorepo/go-astgen/*` (entire directory from joernio/goastgen)

- [ ] **Step 1: Clone source repository**

```bash
cd /tmp/astgen-migration-workspace
git clone https://github.com/joernio/goastgen.git go-astgen-temp
cd go-astgen-temp
```

Expected: Repository cloned successfully

- [ ] **Step 2: Verify default branch**

Run: `git branch --show-current`
Expected: "main"

- [ ] **Step 3: Check commit count**

Run: `git rev-list --count HEAD`
Expected: Number of commits

- [ ] **Step 4: Rewrite history to subdirectory**

Run: `git filter-repo --to-subdirectory-filter go-astgen/ --force`
Expected: History rewritten successfully

- [ ] **Step 5: Merge into monorepo**

```bash
cd /Volumes/Work/workspace_intellij/astgen-monorepo
git remote add go-temp /tmp/astgen-migration-workspace/go-astgen-temp
git fetch go-temp --tags
git merge --allow-unrelated-histories go-temp/main -m "feat: migrate go-astgen from joernio/goastgen

Preserved full git history via git filter-repo.
All files moved to go-astgen/ subdirectory.

Source: https://github.com/joernio/goastgen"
```

Expected: Merge completed

- [ ] **Step 6: Verify and clean up**

Run: `ls -la go-astgen/ | head -10 && git remote remove go-temp && rm -rf /tmp/astgen-migration-workspace/go-astgen-temp`
Expected: Files present, cleanup successful

- [ ] **Step 7: Push to origin**

Run: `git push origin main --tags`
Expected: Push successful

---

### Task 7: Migrate Ruby AST Generator

**Files:**
- Create: `/Volumes/Work/workspace_intellij/astgen-monorepo/ruby-astgen/*` (entire directory from joernio/ruby_ast_gen)

- [ ] **Step 1: Clone source repository**

```bash
cd /tmp/astgen-migration-workspace
git clone https://github.com/joernio/ruby_ast_gen.git ruby-astgen-temp
cd ruby-astgen-temp
```

Expected: Repository cloned successfully

- [ ] **Step 2: Verify default branch**

Run: `git branch --show-current`
Expected: "main"

- [ ] **Step 3: Check commit count**

Run: `git rev-list --count HEAD`
Expected: Number of commits

- [ ] **Step 4: Rewrite history to subdirectory**

Run: `git filter-repo --to-subdirectory-filter ruby-astgen/ --force`
Expected: History rewritten successfully

- [ ] **Step 5: Merge into monorepo**

```bash
cd /Volumes/Work/workspace_intellij/astgen-monorepo
git remote add ruby-temp /tmp/astgen-migration-workspace/ruby-astgen-temp
git fetch ruby-temp --tags
git merge --allow-unrelated-histories ruby-temp/main -m "feat: migrate ruby-astgen from joernio/ruby_ast_gen

Preserved full git history via git filter-repo.
All files moved to ruby-astgen/ subdirectory.

Source: https://github.com/joernio/ruby_ast_gen"
```

Expected: Merge completed

- [ ] **Step 6: Verify and clean up**

Run: `ls -la ruby-astgen/ | head -10 && git remote remove ruby-temp && rm -rf /tmp/astgen-migration-workspace/ruby-astgen-temp`
Expected: Files present, cleanup successful

- [ ] **Step 7: Push to origin**

Run: `git push origin main --tags`
Expected: Push successful

---

### Task 8: Create ABAP Placeholder and Astgen-Regression Directory

**Files:**
- Create: `/Volumes/Work/workspace_intellij/astgen-monorepo/abap-astgen/README.md`
- Copy: `/Volumes/Work/workspace_intellij/astgen-regression/*` → `/Volumes/Work/workspace_intellij/astgen-monorepo/astgen-regression/`

- [ ] **Step 1: Create ABAP placeholder directory**

```bash
cd /Volumes/Work/workspace_intellij/astgen-monorepo
mkdir -p abap-astgen
```

Expected: Directory created

- [ ] **Step 2: Create ABAP placeholder README**

Create `abap-astgen/README.md`:

```markdown
# ABAP AST Generator

**Status:** Deferred - Migration pending

The ABAP AST Generator will be migrated from [joernio/abap_ast_gen](https://github.com/joernio/abap_ast_gen) once the source repository contains code.

Current status: Source repository is empty.
```

- [ ] **Step 3: Verify placeholder created**

Run: `cat abap-astgen/README.md`
Expected: Content matches above

- [ ] **Step 4: Check astgen-regression source exists**

Run: `ls -la /Volumes/Work/workspace_intellij/astgen-regression/ | head -10`
Expected: Source directory exists with files

- [ ] **Step 5: Copy astgen-regression to monorepo**

Run: `cp -r /Volumes/Work/workspace_intellij/astgen-regression /Volumes/Work/workspace_intellij/astgen-monorepo/astgen-regression`
Expected: Directory copied

- [ ] **Step 6: Remove .git directory if present in copied regression framework**

Run: `rm -rf /Volumes/Work/workspace_intellij/astgen-monorepo/astgen-regression/.git`
Expected: Any existing .git directory removed (fresh start in monorepo)

- [ ] **Step 7: Verify copy**

Run: `ls -la astgen-regression/ | head -10`
Expected: Files present in monorepo

- [ ] **Step 8: Commit both additions**

```bash
git add abap-astgen/ astgen-regression/
git commit -m "feat: add abap-astgen placeholder and astgen-regression framework

- abap-astgen: placeholder for future migration
- astgen-regression: fresh copy from local development (no git history)"
```

Expected: Commit created

- [ ] **Step 9: Push to origin**

Run: `git push origin main`
Expected: Push successful

---

## Phase 2: GitHub Actions Workflows

### Task 9: Create GitHub Workflows Directory Structure

**Files:**
- Create: `/Volumes/Work/workspace_intellij/astgen-monorepo/.github/workflows/`

- [ ] **Step 1: Create workflows directory**

```bash
cd /Volumes/Work/workspace_intellij/astgen-monorepo
mkdir -p .github/workflows
```

Expected: Directory created

- [ ] **Step 2: Verify directory structure**

Run: `ls -la .github/`
Expected: workflows/ directory exists

- [ ] **Step 3: Clone source repositories for workflow analysis**

```bash
cd /tmp/astgen-migration-workspace
git clone https://github.com/joernio/astgen.git javascript-workflows
git clone https://github.com/joernio/rust_ast_gen.git rust-workflows
git clone https://github.com/joernio/SwiftAstGen.git swift-workflows
git clone https://github.com/joernio/DotNetAstGen.git dotnet-workflows
git clone https://github.com/joernio/goastgen.git go-workflows
git clone https://github.com/joernio/ruby_ast_gen.git ruby-workflows
```

Expected: All repos cloned for workflow inspection

- [ ] **Step 4: List existing workflows from each repo**

Run: `for dir in /tmp/astgen-migration-workspace/*-workflows; do echo "=== $(basename $dir) ==="; ls -1 "$dir/.github/workflows/" 2>/dev/null || echo "No workflows"; done`
Expected: List of existing workflow files per language

---

### Task 10: Create JavaScript AST Generator CI Workflow

**Files:**
- Create: `/Volumes/Work/workspace_intellij/astgen-monorepo/.github/workflows/javascript-astgen-ci.yml`

- [ ] **Step 1: Inspect existing JavaScript workflows**

Run: `cat /tmp/astgen-migration-workspace/javascript-workflows/.github/workflows/pr.yml`
Expected: See actual workflow content to adapt

- [ ] **Step 2: Create JavaScript CI workflow**

Create `.github/workflows/javascript-astgen-ci.yml` - adapt the content from existing `pr.yml`, adding:
- Path filters: `javascript-astgen/**` and `.github/workflows/javascript-astgen-ci.yml`
- `working-directory: javascript-astgen` in defaults.run
- Update name to "JavaScript AST Generator CI"

Example structure (adapt from actual pr.yml):

```yaml
name: JavaScript AST Generator CI

on:
  push:
    branches: [main]
    paths:
      - 'javascript-astgen/**'
      - '.github/workflows/javascript-astgen-ci.yml'
  pull_request:
    paths:
      - 'javascript-astgen/**'
      - '.github/workflows/javascript-astgen-ci.yml'

jobs:
  build-and-test:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: javascript-astgen
    
    steps:
      - uses: actions/checkout@v4
      
      # Copy actual steps from pr.yml here
      # Adapt any paths that were relative to repo root
```

- [ ] **Step 3: Verify workflow syntax**

Run: `cat .github/workflows/javascript-astgen-ci.yml | head -30`
Expected: Valid YAML with correct structure

- [ ] **Step 4: Commit workflow**

```bash
git add .github/workflows/javascript-astgen-ci.yml
git commit -m "ci: add JavaScript AST Generator CI workflow

Adapted from joernio/astgen pr.yml workflow.
Triggers only on changes to javascript-astgen/ directory."
```

Expected: Commit created

---

### Task 11: Create JavaScript AST Generator Release Workflow

**Files:**
- Create: `/Volumes/Work/workspace_intellij/astgen-monorepo/.github/workflows/javascript-astgen-release.yml`

- [ ] **Step 1: Inspect existing release workflow**

Run: `cat /tmp/astgen-migration-workspace/javascript-workflows/.github/workflows/release.yml`
Expected: See actual release workflow

- [ ] **Step 2: Create JavaScript release workflow**

Create `.github/workflows/javascript-astgen-release.yml` - adapt from existing `release.yml`, adding:
- Tag pattern: `javascript-astgen/v*`
- `working-directory: javascript-astgen`
- Extract version from tag: `${GITHUB_REF#refs/tags/javascript-astgen/v}`

Example structure (adapt from actual release.yml):

```yaml
name: JavaScript AST Generator Release

on:
  push:
    tags:
      - 'javascript-astgen/v*'

jobs:
  release:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: javascript-astgen
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Extract version from tag
        id: version
        run: echo "VERSION=${GITHUB_REF#refs/tags/javascript-astgen/v}" >> $GITHUB_OUTPUT
      
      # Copy actual build/release steps from release.yml
      # Update paths to be relative to javascript-astgen/
```

- [ ] **Step 3: Commit workflow**

```bash
git add .github/workflows/javascript-astgen-release.yml
git commit -m "ci: add JavaScript AST Generator release workflow

Adapted from joernio/astgen release.yml workflow.
Triggers on javascript-astgen/v* tags."
```

Expected: Commit created

---

### Task 12: Create Rust AST Generator Workflows

**Files:**
- Create: `.github/workflows/rust-astgen-ci.yml`
- Create: `.github/workflows/rust-astgen-release.yml`

- [ ] **Step 1: Inspect existing Rust workflows**

Run: `ls -1 /tmp/astgen-migration-workspace/rust-workflows/.github/workflows/ && cat /tmp/astgen-migration-workspace/rust-workflows/.github/workflows/pr.yml | head -50`
Expected: See Rust workflow files

- [ ] **Step 2: Create Rust CI workflow**

Create `.github/workflows/rust-astgen-ci.yml` adapted from `pr.yml`:
- Path filters: `rust-astgen/**`
- Working directory: `rust-astgen`
- Update job names and titles

- [ ] **Step 3: Create Rust release workflow**

Create `.github/workflows/rust-astgen-release.yml` adapted from `release.yml`:
- Tag pattern: `rust-astgen/v*`
- Working directory: `rust-astgen`
- Version extraction: `${GITHUB_REF#refs/tags/rust-astgen/v}`

- [ ] **Step 4: Commit workflows**

```bash
git add .github/workflows/rust-astgen-*.yml
git commit -m "ci: add Rust AST Generator CI and release workflows

Adapted from joernio/rust_ast_gen workflows.
Path-filtered and tag-prefixed for monorepo."
```

Expected: Commit created

---

### Task 13: Create Swift AST Generator Workflows

**Files:**
- Create: `.github/workflows/swift-astgen-ci.yml`
- Create: `.github/workflows/swift-astgen-release.yml`

- [ ] **Step 1: Inspect existing Swift workflows**

Run: `ls -1 /tmp/astgen-migration-workspace/swift-workflows/.github/workflows/`
Expected: See Swift workflow files (pr.yml, release.yml, possibly regression.yml)

- [ ] **Step 2: Create Swift CI workflow**

Create `.github/workflows/swift-astgen-ci.yml` adapted from `pr.yml`:
- Path filters: `swift-astgen/**`
- Working directory: `swift-astgen`

- [ ] **Step 3: Create Swift release workflow**

Create `.github/workflows/swift-astgen-release.yml` adapted from `release.yml`:
- Tag pattern: `swift-astgen/v*`
- Working directory: `swift-astgen`

- [ ] **Step 4: Note regression testing integration**

Run: `cat /tmp/astgen-migration-workspace/swift-workflows/.github/workflows/regression.yml 2>/dev/null || echo "No regression workflow"`
Expected: If exists, note how it works for later integration with astgen-regression/

- [ ] **Step 5: Commit workflows**

```bash
git add .github/workflows/swift-astgen-*.yml
git commit -m "ci: add Swift AST Generator CI and release workflows

Adapted from joernio/SwiftAstGen workflows.
Path-filtered and tag-prefixed for monorepo."
```

Expected: Commit created

---

### Task 14: Create .NET AST Generator Workflows

**Files:**
- Create: `.github/workflows/dotnet-astgen-ci.yml`
- Create: `.github/workflows/dotnet-astgen-release.yml`

- [ ] **Step 1: Create .NET CI workflow**

Adapt from `/tmp/astgen-migration-workspace/dotnet-workflows/.github/workflows/pr.yml`:
- Path filters: `dotnet-astgen/**`
- Working directory: `dotnet-astgen`

- [ ] **Step 2: Create .NET release workflow**

Adapt from `release.yml`:
- Tag pattern: `dotnet-astgen/v*`
- Working directory: `dotnet-astgen`
- Note: .NET uses NEXT_VERSION env var for versioning

- [ ] **Step 3: Commit workflows**

```bash
git add .github/workflows/dotnet-astgen-*.yml
git commit -m "ci: add .NET AST Generator CI and release workflows

Adapted from joernio/DotNetAstGen workflows.
Handles NEXT_VERSION environment variable for versioning."
```

Expected: Commit created

---

### Task 15: Create Go AST Generator Workflows

**Files:**
- Create: `.github/workflows/go-astgen-ci.yml`
- Create: `.github/workflows/go-astgen-release.yml`

- [ ] **Step 1: Create Go CI workflow**

Adapt from `/tmp/astgen-migration-workspace/go-workflows/.github/workflows/pr.yml`:
- Path filters: `go-astgen/**`
- Working directory: `go-astgen`

- [ ] **Step 2: Create Go release workflow**

Adapt from `release.yml`:
- Tag pattern: `go-astgen/v*`
- Working directory: `go-astgen`
- Note: Go uses ldflags for version injection

- [ ] **Step 3: Commit workflows**

```bash
git add .github/workflows/go-astgen-*.yml
git commit -m "ci: add Go AST Generator CI and release workflows

Adapted from joernio/goastgen workflows.
Handles ldflags version injection."
```

Expected: Commit created

---

### Task 16: Create Ruby AST Generator Workflows

**Files:**
- Create: `.github/workflows/ruby-astgen-ci.yml`
- Create: `.github/workflows/ruby-astgen-release.yml`

- [ ] **Step 1: Note Ruby workflow extension**

Run: `ls -1 /tmp/astgen-migration-workspace/ruby-workflows/.github/workflows/`
Expected: Note if files use .yaml or .yml extension

- [ ] **Step 2: Create Ruby CI workflow**

Adapt from `pr.yaml` or `pr.yml`:
- Path filters: `ruby-astgen/**`
- Working directory: `ruby-astgen`

- [ ] **Step 3: Create Ruby release workflow**

Adapt from `release.yml`:
- Tag pattern: `ruby-astgen/v*`
- Working directory: `ruby-astgen`

- [ ] **Step 4: Commit workflows**

```bash
git add .github/workflows/ruby-astgen-*.yml
git commit -m "ci: add Ruby AST Generator CI and release workflows

Adapted from joernio/ruby_ast_gen workflows.
Note: source repo may use .yaml extension."
```

Expected: Commit created

---

### Task 17: Create Astgen-Regression Workflows

**Files:**
- Create: `.github/workflows/astgen-regression-ci.yml`
- Create: `.github/workflows/astgen-regression-release.yml`

- [ ] **Step 1: Create regression framework CI workflow**

Create `.github/workflows/astgen-regression-ci.yml`:

```yaml
name: AST Generator Regression Framework CI

on:
  push:
    branches: [main]
    paths:
      - 'astgen-regression/**'
      - '.github/workflows/astgen-regression-ci.yml'
  pull_request:
    paths:
      - 'astgen-regression/**'
      - '.github/workflows/astgen-regression-ci.yml'

jobs:
  test:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: astgen-regression
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      
      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install -e ".[dev]"
      
      - name: Run tests
        run: pytest tests/ -v
      
      - name: Run linting
        run: |
          ruff check .
          black --check .
      
      - name: Type check
        run: mypy src/
```

- [ ] **Step 2: Create regression framework release workflow**

Create `.github/workflows/astgen-regression-release.yml`:

```yaml
name: AST Generator Regression Framework Release

on:
  push:
    tags:
      - 'astgen-regression/v*'

jobs:
  release:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: astgen-regression
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Extract version from tag
        id: version
        run: echo "VERSION=${GITHUB_REF#refs/tags/astgen-regression/v}" >> $GITHUB_OUTPUT
      
      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      
      - name: Install build tools
        run: |
          python -m pip install --upgrade pip
          pip install build twine
      
      - name: Build package
        run: python -m build
      
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ github.ref_name }}
          name: AST Generator Regression Framework v${{ steps.version.outputs.VERSION }}
          files: |
            astgen-regression/dist/*
```

- [ ] **Step 3: Commit workflows**

```bash
git add .github/workflows/astgen-regression-*.yml
git commit -m "ci: add regression framework CI and release workflows

Python-based testing framework for all AST generators.
Publishes to PyPI and GitHub releases on tags."
```

Expected: Commit created

---

### Task 18: Push All Workflows and Clean Up Temp Repos

**Files:**
- None (cleanup only)

- [ ] **Step 1: Verify all workflows created**

Run: `ls -1 .github/workflows/`
Expected: Should see CI and release workflows for each language plus regression framework (14 files total)

- [ ] **Step 2: Count workflow files**

Run: `ls -1 .github/workflows/ | wc -l`
Expected: 14 workflow files

- [ ] **Step 3: Push all workflows to origin**

Run: `git push origin main`
Expected: All workflow commits pushed successfully

- [ ] **Step 4: Clean up temporary workflow repos**

```bash
rm -rf /tmp/astgen-migration-workspace/*-workflows
```

Expected: All temporary repos removed

- [ ] **Step 5: Verify cleanup**

Run: `ls -1 /tmp/astgen-migration-workspace/`
Expected: Directory empty or only migration workspace remains

---

## Phase 3: Tagging and Validation

### Task 19: Create Initial Version Tags

**Files:**
- None (git tags only)

- [ ] **Step 1: Inspect current versions in each language directory**

```bash
cd /Volumes/Work/workspace_intellij/astgen-monorepo

# JavaScript
grep '"version"' javascript-astgen/package.json

# Rust
grep '^version' rust-astgen/Cargo.toml

# Swift (tag-driven, no version file)
echo "Swift uses tag-driven versioning"

# .NET (check for version constant or default)
grep -r "Version" dotnet-astgen/*.csproj | head -3

# Go (check version constant)
grep 'Version' go-astgen/main.go

# Ruby
grep 'VERSION' ruby-astgen/lib/ruby_ast_gen/version.rb
```

Expected: Version numbers displayed for each language

- [ ] **Step 2: Record current versions**

Based on Step 1 output, note versions:
- javascript-astgen: (from package.json)
- rust-astgen: (from Cargo.toml)
- swift-astgen: (use last tag from source repo or 0.3.9 from design doc)
- dotnet-astgen: (use 0.43.0 from design doc or actual version)
- go-astgen: (use 0.1.0 from design doc or actual version)
- ruby-astgen: (from version.rb)
- astgen-regression: 1.0.0 (initial release)

- [ ] **Step 3: Create tags for each language**

Replace X.Y.Z with actual versions from Step 2:

```bash
git tag javascript-astgen/vX.Y.Z
git tag rust-astgen/vX.Y.Z
git tag swift-astgen/vX.Y.Z
git tag dotnet-astgen/vX.Y.Z
git tag go-astgen/vX.Y.Z
git tag ruby-astgen/vX.Y.Z
git tag astgen-regression/v1.0.0
```

Expected: 7 tags created

- [ ] **Step 4: List all tags**

Run: `git tag -l | sort`
Expected: All 7 prefixed tags listed

- [ ] **Step 5: Push tags to origin**

Run: `git push origin --tags`
Expected: All tags pushed successfully

- [ ] **Step 6: Verify tags on remote**

Run: `git ls-remote --tags origin | grep astgen`
Expected: All tags visible on remote

---

### Task 20: Test CI Workflow Triggers

**Files:**
- Modify: Test file in one language directory to trigger CI

- [ ] **Step 1: Create test branch**

```bash
git checkout -b test-ci-triggers
```

Expected: Test branch created

- [ ] **Step 2: Make small change to JavaScript directory**

```bash
echo "# CI Test" >> javascript-astgen/CI_TEST.md
git add javascript-astgen/CI_TEST.md
git commit -m "test: trigger JavaScript CI workflow"
```

Expected: Commit created

- [ ] **Step 3: Push test branch**

Run: `git push origin test-ci-triggers`
Expected: Push successful

- [ ] **Step 4: Check GitHub Actions**

Navigate to: `https://github.com/joernio/astgen-monorepo/actions`
Expected: Only JavaScript CI workflow should be running (not other languages)

- [ ] **Step 5: Wait for workflow completion**

Monitor workflow until complete
Expected: JavaScript CI completes successfully

- [ ] **Step 6: Make change to different language to verify isolation**

```bash
echo "# CI Test" >> rust-astgen/CI_TEST.md
git add rust-astgen/CI_TEST.md
git commit -m "test: trigger Rust CI workflow"
git push origin test-ci-triggers
```

Expected: Only Rust CI triggers (JavaScript doesn't re-run)

- [ ] **Step 7: Verify isolation worked**

Check Actions page again
Expected: Rust CI runs, JavaScript CI did not re-trigger

- [ ] **Step 8: Clean up test branch**

```bash
git checkout main
git branch -D test-ci-triggers
git push origin --delete test-ci-triggers
```

Expected: Test branch deleted locally and remotely

- [ ] **Step 9: Remove test files from main**

```bash
git checkout main
git pull origin main
# Note: test files only existed on test branch, main is clean
```

Expected: Main branch clean, no test files present

---

### Task 21: Validate Release Workflow Setup

**Files:**
- None (validation only)

- [ ] **Step 1: Check release workflow tag patterns**

```bash
grep -r "tags:" .github/workflows/*-release.yml | grep -v "^#"
```

Expected: Each release workflow shows correct tag pattern (e.g., `javascript-astgen/v*`)

- [ ] **Step 2: Verify working-directory in all workflows**

```bash
grep -r "working-directory:" .github/workflows/*.yml | grep -v "^#"
```

Expected: Each workflow has correct working-directory matching its language

- [ ] **Step 3: Check version extraction in release workflows**

```bash
grep -A2 "Extract version" .github/workflows/*-release.yml
```

Expected: Each workflow extracts version correctly from its prefixed tag

- [ ] **Step 4: Test release workflow (dry-run via manual inspection)**

Review one release workflow file completely:
```bash
cat .github/workflows/javascript-astgen-release.yml
```

Expected: 
- Tag trigger: `javascript-astgen/v*`
- Working directory: `javascript-astgen`
- Version extraction uses correct prefix
- Build/release steps adapted from original workflow

- [ ] **Step 5: Document release process**

Create `.github/RELEASE.md`:

```markdown
# Release Process

Each language tool is released independently using prefixed git tags.

## Creating a Release

1. **Update version** in the language directory:
   ```bash
   cd <language>-astgen
   # Edit version file (package.json, Cargo.toml, etc.)
   # Commit the version bump
   ```

2. **Create and push tag**:
   ```bash
   git tag <language>-astgen/vX.Y.Z
   git push origin <language>-astgen/vX.Y.Z
   ```

3. **Monitor release workflow**:
   - Check GitHub Actions for `<language> AST Generator Release` workflow
   - Workflow builds binaries and creates GitHub Release
   - Binaries attached as release assets

## Tag Format

- JavaScript: `javascript-astgen/vX.Y.Z`
- Rust: `rust-astgen/vX.Y.Z`
- Swift: `swift-astgen/vX.Y.Z`
- .NET: `dotnet-astgen/vX.Y.Z`
- Go: `go-astgen/vX.Y.Z`
- Ruby: `ruby-astgen/vX.Y.Z`
- Regression: `astgen-regression/vX.Y.Z`

## Viewing Releases

All releases: https://github.com/joernio/astgen-monorepo/releases

Filter by language using tag prefix search.
```

- [ ] **Step 6: Commit release documentation**

```bash
git add .github/RELEASE.md
git commit -m "docs: add release process documentation"
git push origin main
```

Expected: Documentation committed and pushed

---

### Task 22: Final Validation and Repository Status Check

**Files:**
- None (validation only)

- [ ] **Step 1: Verify directory structure**

```bash
cd /Volumes/Work/workspace_intellij/astgen-monorepo
tree -L 1 -d
```

Expected: Shows all language directories, .github, abap-astgen, astgen-regression

- [ ] **Step 2: Check git history preservation for each language**

```bash
for lang in javascript rust swift dotnet go ruby; do
  echo "=== ${lang}-astgen history ==="
  git log --oneline ${lang}-astgen/ | wc -l
done
```

Expected: Each language shows commit count > 0

- [ ] **Step 3: Verify git blame works**

```bash
# Test on a few key files
git log --follow javascript-astgen/package.json | head -10
git log --follow rust-astgen/Cargo.toml | head -10
```

Expected: History visible for tracked files

- [ ] **Step 4: List all tags**

Run: `git tag -l | sort`
Expected: All 7 initial version tags present with correct prefixes

- [ ] **Step 5: Count workflow files**

Run: `ls -1 .github/workflows/ | wc -l`
Expected: 14 files (7 languages × 2 workflows each)

- [ ] **Step 6: Verify each language has CI and release workflows**

```bash
for lang in javascript rust swift dotnet go ruby astgen-regression; do
  echo "=== ${lang}-astgen workflows ==="
  ls -1 .github/workflows/${lang}-astgen-*.yml 2>/dev/null || echo "Missing!"
done
```

Expected: Each language shows 2 workflow files (ci.yml and release.yml)

- [ ] **Step 7: Check remote status**

Run: `git remote -v && git status`
Expected: Remote is origin (joernio/astgen-monorepo), branch is main, working tree clean

- [ ] **Step 8: Verify remote branches and tags**

```bash
git ls-remote --heads origin
git ls-remote --tags origin | grep astgen | sort
```

Expected: Main branch + all version tags visible on remote

- [ ] **Step 9: Generate migration summary report**

```bash
cat > MIGRATION_SUMMARY.md << 'EOF'
# AST Generator Monorepo Migration Summary

**Date:** $(date +%Y-%m-%d)
**Status:** Complete

## Migrated Repositories

| Language | Source Repository | Branch | Status |
|----------|------------------|--------|--------|
| JavaScript | joernio/astgen | main | ✅ Migrated |
| Rust | joernio/rust_ast_gen | main | ✅ Migrated |
| Swift | joernio/SwiftAstGen | master | ✅ Migrated |
| .NET | joernio/DotNetAstGen | main | ✅ Migrated |
| Go | joernio/goastgen | main | ✅ Migrated |
| Ruby | joernio/ruby_ast_gen | main | ✅ Migrated |
| ABAP | joernio/abap_ast_gen | N/A | 🔄 Deferred (empty repo) |

## Additional Components

- **astgen-regression/**: Testing framework added (fresh copy, no history)
- **abap-astgen/**: Placeholder directory created

## CI/CD Workflows

- **Total workflows created**: 14 (7 CI + 7 Release)
- **Trigger mechanism**: Path-based filters for isolation
- **Release tagging**: Prefixed tags (`<language>-astgen/vX.Y.Z`)

## Git History

- ✅ All commit history preserved via git filter-repo
- ✅ Git blame functionality intact
- ✅ Original authorship and timestamps maintained

## Initial Version Tags

- javascript-astgen/v$(grep '"version"' javascript-astgen/package.json | cut -d'"' -f4)
- rust-astgen/v$(grep '^version' rust-astgen/Cargo.toml | cut -d'"' -f2)
- swift-astgen/v0.3.9 (or latest from source)
- dotnet-astgen/v0.43.0 (or latest from source)
- go-astgen/v0.1.0 (or latest from source)
- ruby-astgen/v$(grep 'VERSION' ruby-astgen/lib/ruby_ast_gen/version.rb | cut -d'"' -f2)
- astgen-regression/v1.0.0

## Next Steps (Manual - Not Automated)

1. **Archive old repositories**:
   - Add "ARCHIVED - Moved to joernio/astgen-monorepo" to descriptions
   - Update README in each old repo with migration notice
   - Mark repositories as read-only

2. **Update external documentation**:
   - Update installation instructions
   - Update CI badges in READMEs
   - Update contributor guidelines

3. **Test first production release**:
   - Select one language for test release
   - Tag and verify release workflow
   - Validate downloadable artifacts

4. **Monitor initial usage**:
   - Watch for issues from users
   - Collect feedback on monorepo structure
   - Adjust workflows if needed

## Success Criteria Status

- ✅ All tools build and test independently
- ✅ CI triggers work correctly (verified via test)
- ✅ Releases work independently (workflows configured)
- ✅ Git history preserved (verified via git log/blame)
- ⏳ Old repositories archived (manual step pending)
- ⏳ Documentation updated (manual step pending)
- ⏳ First successful release (pending production test)

## Repository Links

- **Monorepo**: https://github.com/joernio/astgen-monorepo
- **Actions**: https://github.com/joernio/astgen-monorepo/actions
- **Releases**: https://github.com/joernio/astgen-monorepo/releases
EOF
```

Expected: Summary file created

- [ ] **Step 10: Review and commit summary**

```bash
git add MIGRATION_SUMMARY.md
git commit -m "docs: add migration completion summary

All language repositories migrated successfully.
CI/CD workflows configured and tested.
Ready for post-migration manual steps."
git push origin main
```

Expected: Summary committed and pushed

---

## Post-Migration Manual Steps

**These steps require human judgment and should be performed manually after the automated migration is complete.**

### Manual Task 1: Archive Old Repositories

For each source repository:

1. Go to repository Settings → General
2. Scroll to "Danger Zone"
3. Click "Archive this repository"
4. Add notice to repository description: "ARCHIVED - Moved to joernio/astgen-monorepo"
5. Update README.md with prominent migration notice:

```markdown
# ⚠️ REPOSITORY ARCHIVED

This repository has been archived and moved to the [joernio/astgen-monorepo](https://github.com/joernio/astgen-monorepo).

**New location**: `https://github.com/joernio/astgen-monorepo/<language>-astgen/`

All future development, issues, and releases will happen in the monorepo.

---

[Original README content below...]
```

### Manual Task 2: Update External Documentation

1. **Update installation instructions** across all documentation sites
2. **Update CI badges** in language READMEs to point to monorepo workflows
3. **Update contributor guidelines** to reference monorepo structure
4. **Notify users** via relevant channels (Slack, Discord, mailing lists, etc.)
5. **Update package registry links** (npm, crates.io, etc.) if applicable

### Manual Task 3: Test Production Release

1. Select one language for initial test release (recommend JavaScript or Rust)
2. Bump version in that language's directory
3. Create prefixed tag
4. Monitor release workflow execution
5. Verify GitHub Release created correctly
6. Test downloading and using release artifacts
7. Document any issues found

### Manual Task 4: Rollback Plan (If Needed)

If critical issues arise:

1. Old repositories remain available (just archived, not deleted)
2. Can un-archive old repos and continue using them
3. Individual tool history can be extracted from monorepo using `git filter-repo` if needed
4. No data loss - history preserved in both locations during transition

---

## Completion Checklist

Migration is complete when:

- [x] All 6 active language repositories migrated with preserved history
- [x] ABAP placeholder created
- [x] Astgen-regression framework added
- [x] Root README created
- [x] All 14 CI/CD workflows created and configured
- [x] Initial version tags created and pushed
- [x] CI workflow triggers tested and verified
- [x] Release workflow setup validated
- [x] Migration summary documented
- [ ] Old repositories archived (manual)
- [ ] External documentation updated (manual)
- [ ] First production release tested (manual)

**Estimated total time**: 4-6 hours for automated steps + manual validation time

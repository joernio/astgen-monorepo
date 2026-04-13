#!/usr/bin/env python3
"""
Local regression test runner for SwiftAstGen.

Builds the current branch and a base branch (default: master) in isolation,
then runs regression.py to compare their AST outputs.

Usage:
    python3 scripts/regression-local.py [--base-branch BRANCH]

The script:
1. Builds the current branch in debug mode
2. Creates a git worktree for the base branch
3. Builds the base branch in the worktree
4. Runs regression.py with both versions
5. Cleans up the worktree

Example:
    python3 scripts/regression-local.py
    python3 scripts/regression-local.py --base-branch main
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path


def run(cmd: list, cwd: str = None, capture: bool = False) -> subprocess.CompletedProcess:
    """Run a command and return CompletedProcess. Raises on failure."""
    kwargs = {"cwd": cwd, "check": True}
    if capture:
        kwargs["stdout"] = subprocess.PIPE
        kwargs["stderr"] = subprocess.PIPE
    return subprocess.run(cmd, **kwargs)


def find_repo_root():
    """Find the repository root directory."""
    try:
        result = run(["git", "rev-parse", "--show-toplevel"], capture=True)
        return Path(result.stdout.decode().strip())
    except subprocess.CalledProcessError:
        print("Error: Not in a git repository", file=sys.stderr)
        sys.exit(1)


def current_branch(repo_root: Path) -> str:
    """Get the current git branch name."""
    result = subprocess.run(
        ["git", "branch", "--show-current"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
        cwd=str(repo_root),
    )
    return result.stdout.decode().strip()


def git_short_sha(repo_root: Path, ref: str) -> str:
    """Get the git commit SHA (short form) for a given ref."""
    result = subprocess.run(
        ["git", "rev-parse", "--short", ref],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
        cwd=str(repo_root),
    )
    return result.stdout.decode().strip()


def find_swiftastgen_binary(build_dir):
    """
    Find the SwiftAstGen binary in the build directory.

    Checks:
    1. .build/debug/SwiftAstGen (standard path)
    2. .build/arm64-apple-macosx/debug/SwiftAstGen (architecture-specific)
    3. .build/x86_64-apple-macosx/debug/SwiftAstGen (architecture-specific)
    4. Other architecture-specific paths

    Returns the first match found.
    """
    candidates = [
        build_dir / ".build" / "debug" / "SwiftAstGen",
        build_dir / ".build" / "arm64-apple-macosx" / "debug" / "SwiftAstGen",
        build_dir / ".build" / "x86_64-apple-macosx" / "debug" / "SwiftAstGen",
    ]

    # Also check for any other arch-specific paths
    build_root = build_dir / ".build"
    if build_root.exists():
        for arch_dir in build_root.iterdir():
            if arch_dir.is_dir() and arch_dir.name not in ["release", "debug"]:
                candidate = arch_dir / "debug" / "SwiftAstGen"
                if candidate not in candidates:
                    candidates.append(candidate)

    for candidate in candidates:
        if candidate.exists() and candidate.is_file():
            return candidate

    print(f"Error: Could not find SwiftAstGen binary in {build_dir}", file=sys.stderr)
    print("Tried:", file=sys.stderr)
    for c in candidates:
        print(f"  {c}", file=sys.stderr)
    sys.exit(1)


def main():
    parser = argparse.ArgumentParser(
        description="Run local regression tests for SwiftAstGen",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Example:
    python3 scripts/regression-local.py
    python3 scripts/regression-local.py --base-branch main
        """,
    )
    parser.add_argument(
        "--base-branch",
        default="master",
        help="Base branch to compare against (default: master)",
    )
    args = parser.parse_args()

    repo_root = find_repo_root()
    pr_branch = current_branch(repo_root)
    base_branch = args.base_branch

    print(f"Repository: {repo_root}")
    print(f"PR branch: {pr_branch}")
    print(f"Base branch: {base_branch}")
    print()

    # Build PR version (current tree)
    print("Building PR version...")
    run(["swift", "build"], cwd=str(repo_root))
    pr_sha = git_short_sha(repo_root, "HEAD")
    pr_binary = find_swiftastgen_binary(repo_root)
    print(f"PR binary: {pr_binary}")
    print(f"PR SHA: {pr_sha}")
    print()

    # Create worktree for base branch
    worktree_path = repo_root / ".worktree-regression-base"
    if worktree_path.exists():
        print(f"Removing existing worktree: {worktree_path}")
        try:
            run(["git", "worktree", "remove", "--force", str(worktree_path)], cwd=str(repo_root))
        except subprocess.CalledProcessError:
            pass

    print(f"Creating worktree for {base_branch}...")
    run(["git", "worktree", "add", str(worktree_path), base_branch], cwd=str(repo_root))

    try:
        # Build base version in worktree
        print(f"Building base version ({base_branch})...")
        run(["swift", "build"], cwd=str(worktree_path))
        base_sha = git_short_sha(worktree_path, "HEAD")
        base_binary = find_swiftastgen_binary(worktree_path)
        print(f"Base binary: {base_binary}")
        print(f"Base SHA: {base_sha}")
        print()

        # Run regression.py
        print("Running regression tests...")
        regression_script = repo_root / "scripts" / "regression.py"

        cmd = [
            sys.executable,
            str(regression_script),
            "--base-dist", str(base_binary.parent),
            "--pr-dist", str(pr_binary.parent),
            "--base-ref", f"{base_branch}@{base_sha}",
            "--pr-ref", f"{pr_branch}@{pr_sha}",
        ]

        subprocess.run(cmd, cwd=str(repo_root), check=True)

    finally:
        # Clean up worktree
        print()
        print("Cleaning up worktree...")
        try:
            run(["git", "worktree", "remove", "--force", str(worktree_path)], cwd=str(repo_root))
        except subprocess.CalledProcessError:
            pass
        print("Done.")


if __name__ == "__main__":
    main()

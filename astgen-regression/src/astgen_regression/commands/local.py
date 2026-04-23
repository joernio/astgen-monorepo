"""Local command implementation."""

from pathlib import Path
import subprocess
import sys

from astgen_regression.config import load_config
from astgen_regression.worktree import (
    get_repo_root,
    get_short_sha,
    create_worktree,
    remove_worktree
)


def cmd_local(args) -> None:
    """Run local command with worktrees.

    Args:
        args: Parsed command-line arguments
    """
    # Load config
    try:
        config = load_config(args.config)
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)

    # Get repository info
    try:
        repo_root = get_repo_root()
    except subprocess.CalledProcessError:
        print("ERROR: Not in a git repository", file=sys.stderr)
        sys.exit(1)

    # Determine base branch
    base_branch = args.base_branch
    if base_branch is None:
        base_branch = config.get("github", {}).get("base_branch", "main")

    # Get current branch and SHA
    try:
        result = subprocess.run(
            ["git", "branch", "--show-current"],
            stdout=subprocess.PIPE,
            check=True,
            cwd=str(repo_root)
        )
        current_branch = result.stdout.decode().strip()
        pr_sha = get_short_sha(repo_root, "HEAD")
    except subprocess.CalledProcessError as e:
        print(f"ERROR: Failed to get branch info: {e}", file=sys.stderr)
        sys.exit(1)

    print(f"[local-regression] Current branch: {current_branch} @ {pr_sha}", file=sys.stderr)
    print(f"[local-regression] Base branch:    {base_branch}", file=sys.stderr)

    # Build PR version (current tree)
    print("[local-regression] Building PR version...", file=sys.stderr)
    dist_dir = config["build"]["dist_dir"]
    pr_dist = repo_root / dist_dir

    try:
        # Run install command if present
        if "install_command" in config["build"]:
            print(f"[local-regression] Running: {config['build']['install_command']}", file=sys.stderr)
            subprocess.run(
                config["build"]["install_command"],
                shell=True,
                check=True,
                cwd=str(repo_root)
            )

        # Run build command
        print(f"[local-regression] Running: {config['build']['build_command']}", file=sys.stderr)
        subprocess.run(
            config["build"]["build_command"],
            shell=True,
            check=True,
            cwd=str(repo_root)
        )
    except subprocess.CalledProcessError as e:
        print(f"ERROR: PR build failed: {e}", file=sys.stderr)
        sys.exit(1)

    # Validate that build created the dist directory
    if not pr_dist.exists():
        print(f"ERROR: Build did not create expected directory: {dist_dir}", file=sys.stderr)
        print(f"       Check that build.dist_dir is correctly configured in regression.yaml", file=sys.stderr)
        sys.exit(1)

    # Setup worktree for base branch
    worktree_path = repo_root / ".worktrees" / "regression-base"

    try:
        # Fetch base branch
        print(f"[local-regression] Fetching {base_branch}...", file=sys.stderr)
        subprocess.run(
            ["git", "fetch", "origin", base_branch],
            cwd=str(repo_root),
            check=True
        )

        base_ref = f"origin/{base_branch}"
        base_sha = get_short_sha(repo_root, base_ref)
        print(f"[local-regression] Base SHA: {base_sha}", file=sys.stderr)

        # Create worktree
        create_worktree(repo_root, worktree_path, base_ref)

        # Build base version
        print("[local-regression] Building base version...", file=sys.stderr)
        base_dist = worktree_path / dist_dir

        # Run install command if present
        if "install_command" in config["build"]:
            subprocess.run(
                config["build"]["install_command"],
                shell=True,
                check=True,
                cwd=str(worktree_path)
            )

        # Run build command
        subprocess.run(
            config["build"]["build_command"],
            shell=True,
            check=True,
            cwd=str(worktree_path)
        )

        # Validate that build created the dist directory
        if not base_dist.exists():
            print(f"ERROR: Base build did not create expected directory: {dist_dir}", file=sys.stderr)
            print(f"       Check that build.dist_dir is correctly configured in regression.yaml", file=sys.stderr)
            sys.exit(1)

        # Run compare command
        print("[local-regression] Running regression comparison...\n", file=sys.stderr)

        from astgen_regression.commands.compare import cmd_compare

        # Create mock args for compare command
        class CompareArgs:
            def __init__(self):
                self.base_dist = base_dist
                self.pr_dist = pr_dist
                self.config = args.config
                self.pr_number = None
                self.base_ref = f"{base_branch} @ {base_sha}"
                self.pr_ref = f"{current_branch} @ {pr_sha}"
                self.output_diffs = None

        cmd_compare(CompareArgs())

    finally:
        # Clean up worktree
        print("\n[local-regression] Cleaning up worktree...", file=sys.stderr)
        remove_worktree(repo_root, worktree_path)

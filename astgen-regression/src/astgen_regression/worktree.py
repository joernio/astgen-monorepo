"""Git worktree management functionality."""

from pathlib import Path
import subprocess
import sys


def get_repo_root() -> Path:
    """Get git repository root directory.

    Returns:
        Path to repository root

    Raises:
        subprocess.CalledProcessError: If not in a git repo
    """
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
    )
    return Path(result.stdout.decode().strip())


def get_short_sha(repo_root: Path, ref: str) -> str:
    """Get short commit SHA for a git ref.

    Args:
        repo_root: Repository root directory
        ref: Git ref (branch, tag, commit)

    Returns:
        Short SHA string
    """
    result = subprocess.run(
        ["git", "rev-parse", "--short", ref],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
        cwd=str(repo_root),
    )
    return result.stdout.decode().strip()


def create_worktree(repo_root: Path, worktree_path: Path, branch: str) -> None:
    """Create a git worktree.

    Args:
        repo_root: Repository root directory
        worktree_path: Path for new worktree
        branch: Branch to checkout in worktree
    """
    # Remove existing worktree if present
    if worktree_path.exists():
        print(
            f"[worktree] Removing existing worktree at {worktree_path}", file=sys.stderr
        )
        remove_worktree(repo_root, worktree_path)

    print(
        f"[worktree] Creating worktree for {branch} at {worktree_path}", file=sys.stderr
    )
    subprocess.run(
        ["git", "worktree", "add", str(worktree_path), branch],
        cwd=str(repo_root),
        check=True,
    )


def remove_worktree(repo_root: Path, worktree_path: Path) -> None:
    """Remove a git worktree.

    Args:
        repo_root: Repository root directory
        worktree_path: Path to worktree to remove
    """
    try:
        subprocess.run(
            ["git", "worktree", "remove", "--force", str(worktree_path)],
            cwd=str(repo_root),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )
    except subprocess.CalledProcessError:
        # Silently continue - cleanup failure is not critical
        pass

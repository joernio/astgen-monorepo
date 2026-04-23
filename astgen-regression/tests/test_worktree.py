import tempfile
from pathlib import Path
from unittest.mock import patch, MagicMock
import subprocess
from astgen_regression.worktree import (
    create_worktree,
    remove_worktree,
    get_repo_root,
    get_short_sha
)


def test_get_repo_root():
    """Test getting repository root."""
    with patch('subprocess.run') as mock_run:
        mock_run.return_value = MagicMock(
            stdout=b"/path/to/repo\n",
            returncode=0
        )

        root = get_repo_root()

        assert root == Path("/path/to/repo")
        mock_run.assert_called_once()
        args = mock_run.call_args[0][0]
        assert "git" in args
        assert "rev-parse" in args
        assert "--show-toplevel" in args


def test_get_short_sha():
    """Test getting short commit SHA."""
    with patch('subprocess.run') as mock_run:
        mock_run.return_value = MagicMock(
            stdout=b"abc1234\n",
            returncode=0
        )

        sha = get_short_sha(Path("/repo"), "main")

        assert sha == "abc1234"


def test_create_worktree():
    """Test creating git worktree."""
    repo_root = Path("/repo")
    worktree_path = Path("/repo/.worktrees/test")
    branch = "main"

    with patch('subprocess.run') as mock_run:
        mock_run.return_value = MagicMock(returncode=0)

        create_worktree(repo_root, worktree_path, branch)

        # Should call git worktree add
        assert mock_run.call_count >= 1
        call_args = mock_run.call_args[0][0]
        assert "git" in call_args
        assert "worktree" in call_args
        assert "add" in call_args


def test_remove_worktree():
    """Test removing git worktree."""
    repo_root = Path("/repo")
    worktree_path = Path("/repo/.worktrees/test")

    with patch('subprocess.run') as mock_run:
        mock_run.return_value = MagicMock(returncode=0)

        remove_worktree(repo_root, worktree_path)

        args = mock_run.call_args[0][0]
        assert "git" in args
        assert "worktree" in args
        assert "remove" in args
        assert "--force" in args


def test_remove_worktree_failure_silent():
    """Test worktree removal failure is silent."""
    repo_root = Path("/repo")
    worktree_path = Path("/repo/.worktrees/test")

    with patch('subprocess.run') as mock_run:
        mock_run.side_effect = subprocess.CalledProcessError(1, "git worktree remove")

        # Should not raise
        remove_worktree(repo_root, worktree_path)

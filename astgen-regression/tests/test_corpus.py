import tempfile
from pathlib import Path
from unittest.mock import patch, MagicMock
import subprocess
from astgen_regression.corpus import clone_corpus


def test_clone_corpus_success():
    """Test successful corpus cloning."""
    corpus_config = {
        "name": "test-corpus",
        "clone_url": "https://github.com/test/repo.git",
        "tag": "v1.0.0",
        "input_subdir": "src",
    }

    with tempfile.TemporaryDirectory() as tmpdir:
        temp_path = Path(tmpdir)

        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=0)

            result = clone_corpus(corpus_config, temp_path)

            assert result is not None
            assert result == temp_path / "corpus-test-corpus" / "src"

            # Verify git clone was called correctly
            mock_run.assert_called_once()
            args = mock_run.call_args[0][0]
            assert args[0] == "git"
            assert args[1] == "clone"
            assert "--depth" in args
            assert "1" in args
            assert "--branch" in args
            assert "v1.0.0" in args
            assert corpus_config["clone_url"] in args


def test_clone_corpus_no_input_subdir():
    """Test cloning corpus with empty input_subdir (use repo root)."""
    corpus_config = {
        "name": "test-corpus",
        "clone_url": "https://github.com/test/repo.git",
        "tag": "v1.0.0",
        "input_subdir": "",
    }

    with tempfile.TemporaryDirectory() as tmpdir:
        temp_path = Path(tmpdir)

        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=0)

            result = clone_corpus(corpus_config, temp_path)

            assert result == temp_path / "corpus-test-corpus"


def test_clone_corpus_failure():
    """Test clone failure handling."""
    corpus_config = {
        "name": "test-corpus",
        "clone_url": "https://github.com/test/nonexistent.git",
        "tag": "v1.0.0",
        "input_subdir": "src",
    }

    with tempfile.TemporaryDirectory() as tmpdir:
        temp_path = Path(tmpdir)

        with patch("subprocess.run") as mock_run:
            mock_run.side_effect = subprocess.CalledProcessError(
                128, "git clone", stderr=b"fatal: repository not found"
            )

            result = clone_corpus(corpus_config, temp_path)

            assert result is None


def test_clone_corpus_timeout():
    """Test clone timeout handling."""
    corpus_config = {
        "name": "test-corpus",
        "clone_url": "https://github.com/test/huge-repo.git",
        "tag": "v1.0.0",
        "input_subdir": "",
    }

    with tempfile.TemporaryDirectory() as tmpdir:
        temp_path = Path(tmpdir)

        with patch("subprocess.run") as mock_run:
            mock_run.side_effect = subprocess.TimeoutExpired("git clone", 300)

            result = clone_corpus(corpus_config, temp_path)

            assert result is None

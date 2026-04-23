import tempfile
from pathlib import Path
from unittest.mock import patch, MagicMock
import subprocess
from astgen_regression.executor import execute_astgen, render_command


def test_render_command():
    """Test command template rendering."""
    exec_config = {
        "command": "node {dist_dir}/astgen.js -i {input_dir} -o {output_dir}"
    }
    dist_dir = "/path/to/dist"
    input_dir = "/path/to/input"
    output_dir = "/path/to/output"

    cmd = render_command(exec_config, dist_dir, input_dir, output_dir)

    assert cmd == "node /path/to/dist/astgen.js -i /path/to/input -o /path/to/output"


def test_execute_astgen_success():
    """Test successful astgen execution."""
    exec_config = {
        "command": "echo test",
        "timeout": 600
    }

    with tempfile.TemporaryDirectory() as tmpdir:
        dist_dir = Path(tmpdir) / "dist"
        input_dir = Path(tmpdir) / "input"
        output_dir = Path(tmpdir) / "output"

        with patch('subprocess.run') as mock_run:
            mock_run.return_value = MagicMock(returncode=0)

            success, elapsed = execute_astgen(exec_config, dist_dir, input_dir, output_dir)

            assert success is True
            assert elapsed >= 0


def test_execute_astgen_nonzero_exit():
    """Test astgen execution with non-zero exit code."""
    exec_config = {
        "command": "false",
        "timeout": 600
    }

    with tempfile.TemporaryDirectory() as tmpdir:
        dist_dir = Path(tmpdir) / "dist"
        input_dir = Path(tmpdir) / "input"
        output_dir = Path(tmpdir) / "output"

        with patch('subprocess.run') as mock_run:
            mock_run.return_value = MagicMock(
                returncode=1,
                stderr=b"Error: something failed"
            )

            success, elapsed = execute_astgen(exec_config, dist_dir, input_dir, output_dir)

            assert success is False
            assert elapsed >= 0


def test_execute_astgen_timeout():
    """Test astgen execution timeout."""
    exec_config = {
        "command": "sleep 1000",
        "timeout": 1
    }

    with tempfile.TemporaryDirectory() as tmpdir:
        dist_dir = Path(tmpdir) / "dist"
        input_dir = Path(tmpdir) / "input"
        output_dir = Path(tmpdir) / "output"

        with patch('subprocess.run') as mock_run:
            mock_run.side_effect = subprocess.TimeoutExpired("sleep 1000", 1)

            success, elapsed = execute_astgen(exec_config, dist_dir, input_dir, output_dir)

            assert success is False
            assert elapsed >= 0

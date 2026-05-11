import tempfile
from pathlib import Path
from unittest.mock import patch, MagicMock
import subprocess

import pytest

from astgen_regression.executor import (
    execute_astgen,
    execute_astgen_repeated,
    render_command,
)


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
    exec_config = {"command": "echo test", "timeout": 600}

    with tempfile.TemporaryDirectory() as tmpdir:
        dist_dir = Path(tmpdir) / "dist"
        input_dir = Path(tmpdir) / "input"
        output_dir = Path(tmpdir) / "output"

        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=0)

            success, elapsed = execute_astgen(
                exec_config, dist_dir, input_dir, output_dir
            )

            assert success is True
            assert elapsed >= 0


def test_execute_astgen_nonzero_exit():
    """Test astgen execution with non-zero exit code."""
    exec_config = {"command": "false", "timeout": 600}

    with tempfile.TemporaryDirectory() as tmpdir:
        dist_dir = Path(tmpdir) / "dist"
        input_dir = Path(tmpdir) / "input"
        output_dir = Path(tmpdir) / "output"

        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(
                returncode=1, stderr=b"Error: something failed"
            )

            success, elapsed = execute_astgen(
                exec_config, dist_dir, input_dir, output_dir
            )

            assert success is False
            assert elapsed >= 0


def test_execute_astgen_timeout():
    """Test astgen execution timeout."""
    exec_config = {"command": "sleep 1000", "timeout": 1}

    with tempfile.TemporaryDirectory() as tmpdir:
        dist_dir = Path(tmpdir) / "dist"
        input_dir = Path(tmpdir) / "input"
        output_dir = Path(tmpdir) / "output"

        with patch("subprocess.run") as mock_run:
            mock_run.side_effect = subprocess.TimeoutExpired("sleep 1000", 1)

            success, elapsed = execute_astgen(
                exec_config, dist_dir, input_dir, output_dir
            )

            assert success is False
            assert elapsed >= 0


def test_execute_astgen_repeated_returns_median():
    """Repeated execution returns the median of the timed runs."""
    exec_config = {"command": "echo test", "timeout": 600}

    with tempfile.TemporaryDirectory() as tmpdir:
        dist_dir = Path(tmpdir) / "dist"
        input_dir = Path(tmpdir) / "input"
        output_dir = Path(tmpdir) / "output"

        # 1 warmup + 5 timed; the timed runs return 0.5, 1.0, 1.5, 2.0, 2.5 → median 1.5.
        elapsed_returns = iter([9.9, 0.5, 1.0, 1.5, 2.0, 2.5])

        def fake_execute_astgen(_cfg, _dist, _input, output):
            output.mkdir(parents=True, exist_ok=True)
            return True, next(elapsed_returns)

        with patch(
            "astgen_regression.executor.execute_astgen", side_effect=fake_execute_astgen
        ):
            success, median, times = execute_astgen_repeated(
                exec_config,
                dist_dir,
                input_dir,
                output_dir,
                iterations=5,
                warmup=1,
            )

        assert success is True
        assert median == pytest.approx(1.5)
        assert times == [0.5, 1.0, 1.5, 2.0, 2.5]


def test_execute_astgen_repeated_zero_warmup():
    """warmup=0 works and just runs the timed iterations."""
    exec_config = {"command": "echo test", "timeout": 600}

    with tempfile.TemporaryDirectory() as tmpdir:
        dist_dir = Path(tmpdir) / "dist"
        input_dir = Path(tmpdir) / "input"
        output_dir = Path(tmpdir) / "output"

        elapsed_returns = iter([0.5, 1.5, 2.5])

        def fake_execute_astgen(_cfg, _dist, _input, output):
            output.mkdir(parents=True, exist_ok=True)
            return True, next(elapsed_returns)

        with patch(
            "astgen_regression.executor.execute_astgen", side_effect=fake_execute_astgen
        ) as mock_one:
            success, median, times = execute_astgen_repeated(
                exec_config,
                dist_dir,
                input_dir,
                output_dir,
                iterations=3,
                warmup=0,
            )

        assert success is True
        assert mock_one.call_count == 3
        assert median == pytest.approx(1.5)
        assert times == [0.5, 1.5, 2.5]


def test_execute_astgen_repeated_stops_on_failure():
    """First failed run aborts further iterations and reports failure."""
    exec_config = {"command": "echo test", "timeout": 600}

    with tempfile.TemporaryDirectory() as tmpdir:
        dist_dir = Path(tmpdir) / "dist"
        input_dir = Path(tmpdir) / "input"
        output_dir = Path(tmpdir) / "output"

        results = iter(
            [
                (True, 0.1),  # warmup
                (True, 1.0),  # iteration 1
                (False, 0.5),  # iteration 2 fails
                (True, 9.9),  # would-be iteration 3, must not be reached
            ]
        )

        def fake_execute_astgen(_cfg, _dist, _input, output):
            output.mkdir(parents=True, exist_ok=True)
            return next(results)

        with patch(
            "astgen_regression.executor.execute_astgen", side_effect=fake_execute_astgen
        ) as mock_one:
            success, median, times = execute_astgen_repeated(
                exec_config,
                dist_dir,
                input_dir,
                output_dir,
                iterations=3,
                warmup=1,
            )

        assert success is False
        assert mock_one.call_count == 3  # 1 warmup + iter 1 + iter 2 (failed)
        assert times == [1.0]
        assert median == pytest.approx(1.0)


def test_execute_astgen_repeated_invalid_iterations():
    """iterations < 1 is rejected."""
    with pytest.raises(ValueError):
        execute_astgen_repeated(
            {"command": "echo test", "timeout": 1},
            Path("/d"),
            Path("/i"),
            Path("/o"),
            iterations=0,
        )


def test_execute_astgen_repeated_invalid_warmup():
    """Negative warmup is rejected."""
    with pytest.raises(ValueError):
        execute_astgen_repeated(
            {"command": "echo test", "timeout": 1},
            Path("/d"),
            Path("/i"),
            Path("/o"),
            iterations=1,
            warmup=-1,
        )


def test_execute_astgen_repeated_wipes_output_between_runs():
    """Each run starts from an empty output directory."""
    exec_config = {"command": "echo test", "timeout": 600}

    with tempfile.TemporaryDirectory() as tmpdir:
        dist_dir = Path(tmpdir) / "dist"
        input_dir = Path(tmpdir) / "input"
        output_dir = Path(tmpdir) / "output"

        seen_existing = []

        def fake_execute_astgen(_cfg, _dist, _input, output):
            # Whether the output directory existed at the start of the run.
            seen_existing.append(output.exists() and any(output.iterdir()))
            output.mkdir(parents=True, exist_ok=True)
            (output / "marker.txt").write_text("data")
            return True, 1.0

        with patch(
            "astgen_regression.executor.execute_astgen", side_effect=fake_execute_astgen
        ):
            execute_astgen_repeated(
                exec_config,
                dist_dir,
                input_dir,
                output_dir,
                iterations=2,
                warmup=1,
            )

        # The directory had stale contents from the previous run before each call,
        # but execute_astgen_repeated wiped it. So the start-of-run snapshot is
        # always empty.
        assert seen_existing == [False, False, False]

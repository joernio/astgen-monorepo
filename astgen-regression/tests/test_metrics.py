import tempfile
from pathlib import Path
from astgen_regression.metrics import collect_metrics


def test_collect_metrics_empty_dir():
    """Test metrics for empty output directory."""
    artifacts_config = [
        {"name": "ast", "pattern": "*.json"},
        {"name": "typemap", "pattern": "*.typemap"},
    ]

    with tempfile.TemporaryDirectory() as tmpdir:
        output_dir = Path(tmpdir)

        metrics = collect_metrics(output_dir, artifacts_config)

        assert metrics == {
            "ast": {"file_count": 0, "total_size": 0},
            "typemap": {"file_count": 0, "total_size": 0},
        }


def test_collect_metrics_with_files():
    """Test metrics collection with actual files."""
    artifacts_config = [
        {"name": "ast", "pattern": "*.json"},
        {"name": "typemap", "pattern": "*.typemap"},
    ]

    with tempfile.TemporaryDirectory() as tmpdir:
        output_dir = Path(tmpdir)

        # Create test files
        (output_dir / "file1.json").write_text('{"key": "value"}')
        (output_dir / "file2.json").write_text('{"another": "json"}')
        (output_dir / "file1.typemap").write_text("type mapping data")

        subdir = output_dir / "nested"
        subdir.mkdir()
        (subdir / "file3.json").write_text('{"nested": "file"}')

        metrics = collect_metrics(output_dir, artifacts_config)

        assert metrics["ast"]["file_count"] == 3
        assert metrics["ast"]["total_size"] > 0
        assert metrics["typemap"]["file_count"] == 1
        assert metrics["typemap"]["total_size"] > 0


def test_collect_metrics_no_artifacts_config():
    """Test metrics when no artifacts configured (default to JSON)."""
    with tempfile.TemporaryDirectory() as tmpdir:
        output_dir = Path(tmpdir)

        (output_dir / "file1.json").write_text('{"test": "data"}')
        (output_dir / "file2.txt").write_text("not json")

        # Default behavior: count all files
        metrics = collect_metrics(output_dir, None)

        assert "all" in metrics
        assert metrics["all"]["file_count"] == 2

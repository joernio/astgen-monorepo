"""Metrics collection functionality."""

from pathlib import Path
import fnmatch


def collect_metrics(output_dir: Path, artifacts_config: list[dict] | None) -> dict:
    """Collect file count and size metrics from output directory.

    Args:
        output_dir: Output directory to scan
        artifacts_config: List of artifact definitions with 'name' and 'pattern',
                         or None to collect all files under 'all'

    Returns:
        Dict mapping artifact name to {"file_count": int, "total_size": int}
    """
    if artifacts_config is None:
        # Default: collect all files
        artifacts_config = [{"name": "all", "pattern": "*"}]

    metrics = {}

    for artifact in artifacts_config:
        name = artifact["name"]
        pattern = artifact["pattern"]

        file_count = 0
        total_size = 0

        if output_dir.exists():
            for path in output_dir.rglob("*"):
                if path.is_file() and fnmatch.fnmatch(path.name, pattern):
                    file_count += 1
                    total_size += path.stat().st_size

        metrics[name] = {
            "file_count": file_count,
            "total_size": total_size
        }

    return metrics

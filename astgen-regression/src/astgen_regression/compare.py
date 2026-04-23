"""Output comparison and diffing functionality."""

from pathlib import Path
import json
import filecmp
import difflib
import fnmatch


def normalize_json(path: Path) -> list[str]:
    """Parse JSON file and re-serialize with stable formatting.

    Args:
        path: Path to JSON file

    Returns:
        List of lines (with newlines) for diffing
    """
    try:
        obj = json.loads(path.read_text(errors='replace'))
        normalized = json.dumps(obj, indent=2, sort_keys=True) + "\n"
        return normalized.splitlines(keepends=True)
    except Exception:
        # Fall back to raw text if JSON is malformed
        return path.read_text(errors='replace').splitlines(keepends=True)


def json_diff_summary(base_path: Path, pr_path: Path) -> str:
    """Generate human-readable summary of changes between two JSON files.

    Args:
        base_path: Path to base JSON file
        pr_path: Path to PR JSON file

    Returns:
        Summary string like "3 added, 1 removed, 7 changed" or
        "whitespace/ordering only" or "" on parse error
    """
    try:
        base_obj = json.loads(base_path.read_text(errors='replace'))
        pr_obj = json.loads(pr_path.read_text(errors='replace'))
    except Exception:
        return ""

    added = 0
    removed = 0
    changed = 0

    def walk(b, p, depth: int = 0):
        nonlocal added, removed, changed

        if depth > 100:
            return

        if isinstance(b, dict) and isinstance(p, dict):
            for k in set(b) | set(p):
                if k not in b:
                    added += 1
                elif k not in p:
                    removed += 1
                else:
                    walk(b[k], p[k], depth + 1)
        elif isinstance(b, list) and isinstance(p, list):
            for i in range(min(len(b), len(p))):
                walk(b[i], p[i], depth + 1)
            extra = len(p) - len(b)
            if extra > 0:
                added += extra
            elif extra < 0:
                removed += -extra
        else:
            if b != p:
                changed += 1

    walk(base_obj, pr_obj)

    parts = []
    if added:
        parts.append(f"{added} added")
    if removed:
        parts.append(f"{removed} removed")
    if changed:
        parts.append(f"{changed} changed")

    return ", ".join(parts) if parts else "whitespace/ordering only"


def compare_outputs(
    base_dir: Path,
    pr_dir: Path,
    artifacts_config: list[dict]
) -> dict:
    """Compare two output directories.

    Args:
        base_dir: Base output directory
        pr_dir: PR output directory
        artifacts_config: List of artifact definitions

    Returns:
        Dict with keys:
        - only_in_base: List of relative paths
        - only_in_pr: List of relative paths
        - diffs: Dict mapping artifact name to list of
                 (rel_path, diff_lines, summary) tuples
    """
    only_in_base = []
    only_in_pr = []
    diffs_by_artifact = {}

    # Initialize diffs dict for each artifact
    for artifact in artifacts_config:
        diffs_by_artifact[artifact["name"]] = []

    # Build file inventories
    base_files = {}
    pr_files = {}

    if base_dir.exists():
        for path in base_dir.rglob("*"):
            if path.is_file():
                rel = str(path.relative_to(base_dir))
                base_files[rel] = path

    if pr_dir.exists():
        for path in pr_dir.rglob("*"):
            if path.is_file():
                rel = str(path.relative_to(pr_dir))
                pr_files[rel] = path

    base_set = set(base_files)
    pr_set = set(pr_files)

    only_in_base = sorted(base_set - pr_set)
    only_in_pr = sorted(pr_set - base_set)

    # Compare common files
    for rel in sorted(base_set & pr_set):
        base_path = base_files[rel]
        pr_path = pr_files[rel]

        # Quick check: same size and identical bytes
        if (base_path.stat().st_size == pr_path.stat().st_size and
            filecmp.cmp(str(base_path), str(pr_path), shallow=False)):
            continue

        # Files differ - determine which artifact type
        artifact_name = None
        for artifact in artifacts_config:
            pattern = artifact["pattern"]
            if fnmatch.fnmatch(base_path.name, pattern):
                artifact_name = artifact["name"]
                break

        if artifact_name is None:
            continue  # File doesn't match any artifact pattern

        # Generate diff
        base_text = normalize_json(base_path)
        pr_text = normalize_json(pr_path)
        summary = json_diff_summary(base_path, pr_path)

        diff_lines = list(difflib.unified_diff(
            base_text,
            pr_text,
            fromfile=f"out-base/{rel}",
            tofile=f"out-pr/{rel}"
        ))

        if diff_lines:
            diffs_by_artifact[artifact_name].append((rel, diff_lines, summary))

    return {
        "only_in_base": only_in_base,
        "only_in_pr": only_in_pr,
        "diffs": diffs_by_artifact
    }

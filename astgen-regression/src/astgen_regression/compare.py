"""Output comparison and diffing functionality."""

from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import Any
import difflib
import filecmp
import fnmatch
import json
import os
import re

from deepdiff import DeepDiff


# Skip emitting a full unified diff once the combined normalized line count
# exceeds this threshold. Prevents pathological SequenceMatcher slowdowns on
# very large AST changes; the structural summary is still reported.
MAX_DIFF_LINES = 20000


def _normalize_obj_to_lines(obj: Any) -> list[str]:
    """Serialize a parsed object with stable formatting into diff-ready lines."""
    normalized = json.dumps(obj, indent=2, sort_keys=True) + "\n"
    return normalized.splitlines(keepends=True)


def _diff_summary_from_objs(base_obj: Any, pr_obj: Any) -> str:
    """Compute a human-readable summary of structural changes between two parsed objects.

    Uses DeepDiff for structural comparison, providing accurate counts of
    added, removed, and changed elements regardless of depth or complexity.
    """
    diff = DeepDiff(base_obj, pr_obj, ignore_order=False, verbose_level=2)

    if not diff:
        return "no changes"

    added = 0
    removed = 0
    changed = 0

    # Count dictionary item changes
    if "dictionary_item_added" in diff:
        added += len(diff["dictionary_item_added"])
    if "dictionary_item_removed" in diff:
        removed += len(diff["dictionary_item_removed"])

    # Count list item changes
    if "iterable_item_added" in diff:
        added += len(diff["iterable_item_added"])
    if "iterable_item_removed" in diff:
        removed += len(diff["iterable_item_removed"])

    # Count value changes
    if "values_changed" in diff:
        changed += len(diff["values_changed"])

    # Count type changes as value changes
    if "type_changes" in diff:
        changed += len(diff["type_changes"])

    parts = []
    if added:
        parts.append(f"{added} added")
    if removed:
        parts.append(f"{removed} removed")
    if changed:
        parts.append(f"{changed} changed")

    return ", ".join(parts) if parts else "no structural changes"


def normalize_json(path: Path) -> list[str]:
    """Parse JSON file and re-serialize with stable formatting.

    Args:
        path: Path to JSON file

    Returns:
        List of lines (with newlines) for diffing
    """
    try:
        obj = json.loads(path.read_bytes())
        return _normalize_obj_to_lines(obj)
    except Exception:
        return path.read_text(errors="replace").splitlines(keepends=True)


def json_diff_summary(base_path: Path, pr_path: Path) -> str:
    """Generate human-readable summary of changes between two JSON files.

    Args:
        base_path: Path to base JSON file
        pr_path: Path to PR JSON file

    Returns:
        Summary string like "3 added, 1 removed, 7 changed",
        "no changes" if identical, or "" on parse error
    """
    try:
        base_obj = json.loads(base_path.read_bytes())
        pr_obj = json.loads(pr_path.read_bytes())
    except Exception:
        return ""

    return _diff_summary_from_objs(base_obj, pr_obj)


def _compare_one(
    rel: str,
    base_path: Path,
    pr_path: Path,
    artifact_matchers: list[tuple[str, re.Pattern[str]]],
) -> tuple[str, str, list[str], str] | None:
    """Compare a single pair of files.

    Returns None if the files are identical or don't match any artifact pattern;
    otherwise returns (rel, artifact_name, diff_lines, summary).
    """
    # filecmp.cmp(shallow=False) already stats both files and short-circuits on
    # size mismatch, so an explicit pre-stat would just duplicate that work.
    if filecmp.cmp(str(base_path), str(pr_path), shallow=False):
        return None

    artifact_name: str | None = None
    name = base_path.name
    for art_name, matcher in artifact_matchers:
        if matcher.match(name):
            artifact_name = art_name
            break

    if artifact_name is None:
        return None

    base_bytes = base_path.read_bytes()
    pr_bytes = pr_path.read_bytes()

    try:
        base_obj = json.loads(base_bytes)
        pr_obj = json.loads(pr_bytes)
        base_lines = _normalize_obj_to_lines(base_obj)
        pr_lines = _normalize_obj_to_lines(pr_obj)
        summary = _diff_summary_from_objs(base_obj, pr_obj)
    except Exception:
        base_lines = base_bytes.decode(errors="replace").splitlines(keepends=True)
        pr_lines = pr_bytes.decode(errors="replace").splitlines(keepends=True)
        summary = ""

    if len(base_lines) + len(pr_lines) > MAX_DIFF_LINES:
        diff_lines = [
            f"--- out-base/{rel}\n",
            f"+++ out-pr/{rel}\n",
            f"@@ diff omitted: {len(base_lines)} vs {len(pr_lines)} lines exceeds {MAX_DIFF_LINES} @@\n",
        ]
    else:
        diff_lines = list(
            difflib.unified_diff(
                base_lines,
                pr_lines,
                fromfile=f"out-base/{rel}",
                tofile=f"out-pr/{rel}",
            )
        )

    if not diff_lines:
        return None

    return (rel, artifact_name, diff_lines, summary)


def compare_outputs(base_dir: Path, pr_dir: Path, artifacts_config: list[dict]) -> dict:
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
    diffs_by_artifact: dict[str, list[tuple[str, list[str], str]]] = {
        artifact["name"]: [] for artifact in artifacts_config
    }

    # Precompile artifact glob patterns once instead of calling fnmatch.fnmatch
    # per (file, pattern) pair inside the inner loop.
    artifact_matchers: list[tuple[str, re.Pattern[str]]] = [
        (artifact["name"], re.compile(fnmatch.translate(artifact["pattern"])))
        for artifact in artifacts_config
    ]

    base_files: dict[str, Path] = {}
    pr_files: dict[str, Path] = {}

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

    common = base_set & pr_set
    if common:
        max_workers = min(32, (os.cpu_count() or 4) * 2)

        def _work(rel: str) -> tuple[str, str, list[str], str] | None:
            return _compare_one(rel, base_files[rel], pr_files[rel], artifact_matchers)

        with ThreadPoolExecutor(max_workers=max_workers) as ex:
            for result in ex.map(_work, common):
                if result is None:
                    continue
                rel, artifact_name, diff_lines, summary = result
                diffs_by_artifact[artifact_name].append((rel, diff_lines, summary))

        # Restore deterministic ordering after parallel execution.
        for entries in diffs_by_artifact.values():
            entries.sort(key=lambda t: t[0])

    return {
        "only_in_base": only_in_base,
        "only_in_pr": only_in_pr,
        "diffs": diffs_by_artifact,
    }

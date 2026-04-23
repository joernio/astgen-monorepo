#!/usr/bin/env python3
"""
Regression harness for SwiftAstGen.

Usage:
    python3 scripts/regression.py --base-dist ./base/.build/debug --pr-dist ./.build/debug
"""

import argparse
import difflib
import filecmp
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import time

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

MAX_STDERR_DISPLAY = 500  # characters to display in error messages

# ---------------------------------------------------------------------------
# Corpus definitions
# ---------------------------------------------------------------------------

CORPORA = [
    {
        "name": "alamofire",
        "label": "Alamofire/Alamofire@5.9.1",
        "clone_url": "https://github.com/Alamofire/Alamofire.git",
        "tag": "5.9.1",
        "input_subdir": "Source",
    },
    {
        "name": "vapor",
        "label": "vapor/vapor@4.99.3",
        "clone_url": "https://github.com/vapor/vapor.git",
        "tag": "4.99.3",
        "input_subdir": "Sources/Vapor",
    },
    {
        "name": "rxswift",
        "label": "ReactiveX/RxSwift@6.7.1",
        "clone_url": "https://github.com/ReactiveX/RxSwift.git",
        "tag": "6.7.1",
        "input_subdir": "RxSwift",
    },
]

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def human_size(n_bytes: int) -> str:
    """Return a human-readable file size string."""
    for unit in ("B", "KB", "MB", "GB"):
        if n_bytes < 1024.0 or unit == "GB":
            return f"{n_bytes:.1f} {unit}"
        n_bytes /= 1024.0


def pct_delta(base: float, pr: float) -> str:
    """Return a signed percentage delta string."""
    if base == 0:
        return "+0.0%" if pr == 0 else "+∞%"
    delta = (pr - base) / base * 100.0
    sign = "+" if delta >= 0 else ""
    return f"{sign}{delta:.1f}%"


def signed_int(delta: int) -> str:
    """Return a signed integer string."""
    if delta > 0:
        return f"+{delta}"
    return str(delta)


def collect_metrics(out_dir: pathlib.Path) -> dict:
    """Walk out_dir and collect file counts / sizes."""
    json_count = 0
    json_size = 0

    if out_dir.exists():
        for entry in out_dir.rglob("*"):
            if entry.is_file() and entry.suffix == ".json":
                json_count += 1
                json_size += entry.stat().st_size

    return {
        "json_count": json_count,
        "json_size": json_size,
    }


def run_swiftastgen(dist_dir: str, input_dir: str, output_dir: str) -> tuple[bool, float]:
    """
    Run SwiftAstGen and return (success, elapsed_seconds).
    Stdout/stderr are captured and not printed to the terminal.
    """
    swiftastgen_bin = os.path.join(dist_dir, "SwiftAstGen")

    # Verify binary exists and is executable
    if not os.path.isfile(swiftastgen_bin):
        print(f"ERROR: SwiftAstGen binary not found: {swiftastgen_bin}", file=sys.stderr)
        return False, 0.0
    if not os.access(swiftastgen_bin, os.X_OK):
        print(f"ERROR: SwiftAstGen binary not executable: {swiftastgen_bin}", file=sys.stderr)
        return False, 0.0

    cmd = [swiftastgen_bin, "--src", input_dir, "--output", output_dir]

    os.makedirs(output_dir, exist_ok=True)
    t0 = time.monotonic()
    try:
        result = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=600,  # 10 minutes - enough for large corpora
        )
        elapsed = time.monotonic() - t0
        if result.returncode != 0:
            stderr_text = result.stderr.decode(errors='replace')
            if len(stderr_text) > MAX_STDERR_DISPLAY:
                stderr_text = stderr_text[:MAX_STDERR_DISPLAY] + "... (truncated)"
            print(
                f"WARNING: SwiftAstGen exited {result.returncode} for {output_dir}\n"
                f"  stderr: {stderr_text}",
                file=sys.stderr,
            )
            return False, elapsed
        return True, elapsed
    except subprocess.TimeoutExpired:
        elapsed = time.monotonic() - t0
        print(f"ERROR: SwiftAstGen timed out after {elapsed:.1f}s for {output_dir}", file=sys.stderr)
        return False, elapsed
    except Exception as exc:
        elapsed = time.monotonic() - t0
        print(f"WARNING: failed to run SwiftAstGen for {output_dir}: {exc}", file=sys.stderr)
        return False, elapsed


def _normalize_json(path: pathlib.Path) -> list[str]:
    """Parse a JSON file and re-serialize it with stable formatting for diffing."""
    try:
        obj = json.loads(path.read_text(errors="replace"))
        return (json.dumps(obj, indent=2, sort_keys=True) + "\n").splitlines(keepends=True)
    except Exception:
        # Fall back to raw text if JSON is malformed
        return path.read_text(errors="replace").splitlines(keepends=True)


def _json_diff_summary(base_path: pathlib.Path, pr_path: pathlib.Path) -> str:
    """
    Return a one-line human-readable summary of what changed between two JSON files,
    e.g. '3 added, 1 removed, 7 changed'.
    Returns 'ERROR: parse failed' if JSON is malformed.
    """
    try:
        base_obj = json.loads(base_path.read_text(errors="replace"))
        pr_obj = json.loads(pr_path.read_text(errors="replace"))
    except Exception:
        return "ERROR: parse failed"

    added = removed = changed = 0

    def _walk(b, p, depth: int = 0) -> None:
        nonlocal added, removed, changed
        if depth > 100:  # Increased from 20 to handle deeply nested Swift ASTs
            return
        if isinstance(b, dict) and isinstance(p, dict):
            for k in set(b) | set(p):
                if k not in b:
                    added += 1
                elif k not in p:
                    removed += 1
                else:
                    _walk(b[k], p[k], depth + 1)
        elif isinstance(b, list) and isinstance(p, list):
            # Note: This compares by parallel index. For AST children arrays,
            # insertions/deletions may cause alignment shifts and count position
            # changes as content changes. This is acceptable for a regression
            # harness - the goal is to flag that something changed, not to
            # provide perfect semantic diff of tree structures.
            for i in range(min(len(b), len(p))):
                _walk(b[i], p[i], depth + 1)
            extra = len(p) - len(b)
            if extra > 0:
                added += extra
            elif extra < 0:
                removed += -extra
        else:
            if b != p:
                changed += 1

    _walk(base_obj, pr_obj)

    parts = []
    if added:
        parts.append(f"{added} added")
    if removed:
        parts.append(f"{removed} removed")
    if changed:
        parts.append(f"{changed} changed")
    return ", ".join(parts) if parts else "whitespace/ordering only"


def compare_outputs(base_dir: pathlib.Path, pr_dir: pathlib.Path) -> dict:
    """
    Compare two output directories.

    Returns:
        {
            "only_in_base": [relative_path, ...],
            "only_in_pr": [relative_path, ...],
            "ast_diffs": [(rel_path, diff_lines, summary), ...],
        }
    """
    only_in_base = []
    only_in_pr = []
    ast_diffs = []

    # Build sets of relative paths
    base_files: dict[str, pathlib.Path] = {}
    pr_files: dict[str, pathlib.Path] = {}

    if base_dir.exists():
        for p in base_dir.rglob("*"):
            if p.is_file() and p.suffix == ".json":
                rel = str(p.relative_to(base_dir))
                base_files[rel] = p

    if pr_dir.exists():
        for p in pr_dir.rglob("*"):
            if p.is_file() and p.suffix == ".json":
                rel = str(p.relative_to(pr_dir))
                pr_files[rel] = p

    base_set = set(base_files)
    pr_set = set(pr_files)

    only_in_base = sorted(base_set - pr_set)
    only_in_pr = sorted(pr_set - base_set)

    for rel in sorted(base_set & pr_set):
        bp = base_files[rel]
        pp = pr_files[rel]
        if bp.stat().st_size == pp.stat().st_size and filecmp.cmp(str(bp), str(pp), shallow=False):
            continue

        base_text = _normalize_json(bp)
        pr_text = _normalize_json(pp)
        summary = _json_diff_summary(bp, pp)

        diff_lines = list(
            difflib.unified_diff(
                base_text,
                pr_text,
                fromfile=f"out-base/{rel}",
                tofile=f"out-pr/{rel}",
            )
        )

        if diff_lines:
            ast_diffs.append((rel, diff_lines, summary))

    return {
        "only_in_base": only_in_base,
        "only_in_pr": only_in_pr,
        "ast_diffs": ast_diffs,
    }


def build_diff_details(diffs: list, kind: str, max_total_lines: int = 200) -> str:
    """
    Render a collapsible <details> block with truncated normalized diffs.
    kind is 'AST'.
    Each entry in diffs is a (rel_path, diff_lines, summary) tuple.
    """
    n = len(diffs)
    if n == 0:
        return ""

    lines_used = 0
    diff_text_parts = []

    for idx, (rel, diff_lines, summary) in enumerate(diffs):
        if lines_used >= max_total_lines:
            remaining = n - idx  # Fixed: use index not parts length
            diff_text_parts.append(f"\n... ({remaining} more files not shown)\n")
            break
        header = f"# {rel}"
        if summary:
            header += f"  [{summary}]"
        diff_text_parts.append(header + "\n")
        chunk = diff_lines[: max_total_lines - lines_used]
        lines_used += len(chunk)
        diff_text_parts.append("".join(chunk))
        if lines_used >= max_total_lines:
            diff_text_parts.append("\n... (truncated)\n")

    inner = "".join(diff_text_parts)
    return (
        f"<details><summary>{n} {kind} diffs</summary>\n\n"
        f"```diff\n{inner}```\n\n"
        f"</details>\n"
    )


def format_table_row(metric: str, base_val: str, pr_val: str, delta: str) -> str:
    return f"| {metric:<24} | {base_val:>11} | {pr_val:>8} | {delta:>8} |"


def render_corpus_section(name: str, label: str, result: dict, pr_label: str = "PR") -> str:
    """Render the Markdown section for one corpus."""
    bm = result["base_metrics"]
    pm = result["pr_metrics"]
    bt = result["base_time"]
    pt = result["pr_time"]
    cmp = result["comparison"]

    ast_diff_count = len(cmp["ast_diffs"])

    rows = [
        format_table_row(
            "JSON files generated",
            str(bm["json_count"]),
            str(pm["json_count"]),
            signed_int(pm["json_count"] - bm["json_count"]),
        ),
        format_table_row(
            "Total JSON size",
            human_size(bm["json_size"]),
            human_size(pm["json_size"]),
            pct_delta(bm["json_size"], pm["json_size"]),
        ),
        format_table_row(
            "Wall-clock time",
            f"{bt:.1f} s",
            f"{pt:.1f} s",
            pct_delta(bt, pt),
        ),
        format_table_row(
            "Files with AST diffs",
            "—",
            str(ast_diff_count),
            "",
        ),
    ]

    header = (
        f"### {name} ({label})\n\n"
        f"| {'Metric':<24} | {'base (main)':>11} | {pr_label:>8} | {'Delta':>8} |\n"
        f"|{'-'*26}|{'-'*13}|{'-'*10}|{'-'*10}|"
    )

    table = header + "\n" + "\n".join(rows) + "\n"

    ast_details = build_diff_details(cmp["ast_diffs"], "AST")

    parts = [table]
    if ast_details:
        parts.append(ast_details)

    return "\n".join(parts)


def write_diff_files(diffs_dir: pathlib.Path, corpus_results: list) -> None:
    """
    Write full (untruncated) diff content to files in diffs_dir.

    For each corpus that has diffs, creates:
      <corpus-name>-ast.diff  — if AST diffs exist
    """
    diffs_dir.mkdir(parents=True, exist_ok=True)
    for result in corpus_results:
        name = result["name"]
        cmp = result["comparison"]

        diffs = cmp["ast_diffs"]
        if not diffs:
            continue
        parts = []
        for rel, diff_lines, summary in diffs:
            header = f"# {rel}"
            if summary:
                header += f"  [{summary}]"
            parts.append(header + "\n")
            parts.append("".join(diff_lines))
        try:
            (diffs_dir / f"{name}-ast.diff").write_text("".join(parts), encoding='utf-8')
        except IOError as e:
            print(f"WARNING: Failed to write diff file {name}-ast.diff: {e}", file=sys.stderr)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main():
    """Main entry point for the regression harness."""
    parser = argparse.ArgumentParser(description="Regression harness for SwiftAstGen")
    parser.add_argument(
        "--base-dist",
        required=True,
        help="Path to base SwiftAstGen distribution (e.g., ./base/.build/debug)",
    )
    parser.add_argument(
        "--pr-dist",
        required=True,
        help="Path to PR SwiftAstGen distribution (e.g., ./.build/debug)",
    )
    parser.add_argument(
        "--pr-number",
        type=int,
        help="PR number (for report metadata)",
    )
    parser.add_argument(
        "--base-ref",
        default="main",
        help="Base ref name (e.g., main, master) for report metadata",
    )
    parser.add_argument(
        "--pr-ref",
        help="PR ref name (e.g., feature-branch) for report metadata",
    )
    parser.add_argument(
        "--output-diffs",
        metavar="DIR",
        help="Write full diff files to this directory (e.g., ./regression-diffs)",
    )

    args = parser.parse_args()

    # Determine PR label for report
    pr_label = "PR"
    if args.pr_number:
        pr_label = f"PR #{args.pr_number}"
    elif args.pr_ref:
        pr_label = args.pr_ref

    temp_dir = None
    try:
        temp_dir = tempfile.mkdtemp(prefix="swiftastgen-regression-")
        print(f"Using temp directory: {temp_dir}", file=sys.stderr)

        corpus_results = []

        for corpus in CORPORA:
            name = corpus["name"]
            label = corpus["label"]
            clone_url = corpus["clone_url"]
            tag = corpus["tag"]
            input_subdir = corpus["input_subdir"]

            print(f"Processing corpus: {name} ({label})", file=sys.stderr)

            # Clone corpus
            clone_dir = os.path.join(temp_dir, name)
            try:
                subprocess.run(
                    ["git", "clone", "--depth", "1", "--branch", tag, clone_url, clone_dir],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    timeout=300,
                    check=True,
                )
            except subprocess.CalledProcessError as exc:
                print(
                    f"WARNING: failed to clone {clone_url}: "
                    f"{exc.stderr.decode(errors='replace')[:500]}",
                    file=sys.stderr,
                )
                corpus_results.append({
                    "name": name,
                    "label": f"{label} [CLONE FAILED]",
                    "base_metrics": {"json_count": 0, "json_size": 0},
                    "pr_metrics": {"json_count": 0, "json_size": 0},
                    "base_time": 0.0,
                    "pr_time": 0.0,
                    "comparison": {"only_in_base": [], "only_in_pr": [], "ast_diffs": []},
                })
                continue
            except subprocess.TimeoutExpired:
                print(f"WARNING: failed to clone {clone_url}: timeout", file=sys.stderr)
                corpus_results.append({
                    "name": name,
                    "label": f"{label} [CLONE FAILED]",
                    "base_metrics": {"json_count": 0, "json_size": 0},
                    "pr_metrics": {"json_count": 0, "json_size": 0},
                    "base_time": 0.0,
                    "pr_time": 0.0,
                    "comparison": {"only_in_base": [], "only_in_pr": [], "ast_diffs": []},
                })
                continue
            except Exception as exc:
                print(f"WARNING: failed to clone {clone_url}: {exc}", file=sys.stderr)
                corpus_results.append({
                    "name": name,
                    "label": f"{label} [CLONE FAILED]",
                    "base_metrics": {"json_count": 0, "json_size": 0},
                    "pr_metrics": {"json_count": 0, "json_size": 0},
                    "base_time": 0.0,
                    "pr_time": 0.0,
                    "comparison": {"only_in_base": [], "only_in_pr": [], "ast_diffs": []},
                })
                continue

            input_dir = os.path.join(clone_dir, input_subdir)

            # Run base SwiftAstGen
            base_out_dir = os.path.join(temp_dir, f"{name}-base-out")
            print(f"  Running base SwiftAstGen...", file=sys.stderr)
            base_success, base_time = run_swiftastgen(args.base_dist, input_dir, base_out_dir)
            if not base_success:
                print(f"WARNING: Base SwiftAstGen failed for {name}", file=sys.stderr)
                corpus_results.append({
                    "name": name,
                    "label": f"{label} [BASE RUN FAILED]",
                    "base_metrics": {"json_count": 0, "json_size": 0},
                    "pr_metrics": {"json_count": 0, "json_size": 0},
                    "base_time": base_time,
                    "pr_time": 0.0,
                    "comparison": {"only_in_base": [], "only_in_pr": [], "ast_diffs": []},
                })
                continue

            base_metrics = collect_metrics(pathlib.Path(base_out_dir))

            # Run PR SwiftAstGen
            pr_out_dir = os.path.join(temp_dir, f"{name}-pr-out")
            print(f"  Running PR SwiftAstGen...", file=sys.stderr)
            pr_success, pr_time = run_swiftastgen(args.pr_dist, input_dir, pr_out_dir)
            if not pr_success:
                print(f"WARNING: PR SwiftAstGen failed for {name}", file=sys.stderr)
                corpus_results.append({
                    "name": name,
                    "label": f"{label} [PR RUN FAILED]",
                    "base_metrics": base_metrics,
                    "pr_metrics": {"json_count": 0, "json_size": 0},
                    "base_time": base_time,
                    "pr_time": pr_time,
                    "comparison": {"only_in_base": [], "only_in_pr": [], "ast_diffs": []},
                })
                continue

            pr_metrics = collect_metrics(pathlib.Path(pr_out_dir))

            # Compare outputs
            print(f"  Comparing outputs...", file=sys.stderr)
            comparison = compare_outputs(pathlib.Path(base_out_dir), pathlib.Path(pr_out_dir))

            corpus_results.append({
                "name": name,
                "label": label,
                "base_metrics": base_metrics,
                "pr_metrics": pr_metrics,
                "base_time": base_time,
                "pr_time": pr_time,
                "comparison": comparison,
            })

            print(f"  Done: {name}", file=sys.stderr)

    finally:
        if temp_dir:
            shutil.rmtree(temp_dir, ignore_errors=True)

    # Build Markdown report
    report_parts = [
        "<!-- swiftastgen-regression -->",
        "## SwiftAstGen Regression Report",
        "",
    ]

    # Provenance
    provenance_lines = []
    if args.pr_number:
        provenance_lines.append(f"**PR**: #{args.pr_number}")
    if args.pr_ref:
        provenance_lines.append(f"**PR ref**: `{args.pr_ref}`")
    provenance_lines.append(f"**Base ref**: `{args.base_ref}`")

    report_parts.append("\n".join(provenance_lines) + "\n")

    if not corpus_results:
        report_parts.append("\n**No corpus results available** (all clones or runs failed)\n")
    else:
        for result in corpus_results:
            report_parts.append("\n" + render_corpus_section(
                result["name"],
                result["label"],
                result,
                pr_label=pr_label,
            ))

    report = "\n".join(report_parts)

    # Write diff files if requested
    if args.output_diffs and corpus_results:
        diffs_dir = pathlib.Path(args.output_diffs)
        print(f"Writing diff files to: {diffs_dir}", file=sys.stderr)
        write_diff_files(diffs_dir, corpus_results)

    # Print report to stdout
    print(report)


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        print(f"WARNING: regression harness encountered an unexpected error: {exc}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
    sys.exit(0)

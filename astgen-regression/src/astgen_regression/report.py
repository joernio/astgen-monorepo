"""Markdown report generation functionality."""

from pathlib import Path


def human_size(n_bytes: int) -> str:
    """Format byte count as human-readable size.

    Args:
        n_bytes: Number of bytes

    Returns:
        Formatted string like "1.5 MB"
    """
    for unit in ("B", "KB", "MB", "GB"):
        if n_bytes < 1024.0 or unit == "GB":
            return f"{n_bytes:.1f} {unit}"
        n_bytes /= 1024.0
    return f"{n_bytes:.1f} GB"


def pct_delta(base: float, pr: float) -> str:
    """Calculate and format percentage delta.

    Args:
        base: Base value
        pr: PR value

    Returns:
        Formatted string like "+10.5%" or "-5.2%"
    """
    if base == 0:
        return "+0.0%" if pr == 0 else "+∞%"
    delta = (pr - base) / base * 100.0
    sign = "+" if delta >= 0 else ""
    return f"{sign}{delta:.1f}%"


def signed_int(delta: int) -> str:
    """Format integer with sign.

    Args:
        delta: Integer value

    Returns:
        Formatted string like "+5" or "-3" or "0"
    """
    if delta > 0:
        return f"+{delta}"
    return str(delta)


def format_table_row(metric: str, base_val: str, pr_val: str, delta: str) -> str:
    """Format a Markdown table row.

    Args:
        metric: Metric name
        base_val: Base value string
        pr_val: PR value string
        delta: Delta value string

    Returns:
        Formatted Markdown table row
    """
    return f"| {metric:<24} | {base_val:>11} | {pr_val:>8} | {delta:>8} |"


def build_diff_details(
    diffs: list[tuple[str, list[str], str]],
    kind: str,
    max_total_lines: int = 200
) -> str:
    """Build collapsible diff details block.

    Args:
        diffs: List of (rel_path, diff_lines, summary) tuples
        kind: Artifact kind name (e.g., "AST", "typemap")
        max_total_lines: Maximum total lines to include inline

    Returns:
        Markdown string with <details> block, or empty string if no diffs
    """
    n = len(diffs)
    if n == 0:
        return ""

    lines_used = 0
    diff_text_parts = []

    for idx, (rel, diff_lines, summary) in enumerate(diffs):
        if lines_used >= max_total_lines:
            remaining = n - idx
            diff_text_parts.append(f"\n... ({remaining} more files not shown)\n")
            break

        header = f"# {rel}"
        if summary:
            header += f"  [{summary}]"
        diff_text_parts.append(header + "\n")

        chunk = diff_lines[:max_total_lines - lines_used]
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


def render_corpus_section(
    name: str,
    label: str,
    result: dict,
    artifacts_config: list[dict],
    pr_label: str = "PR"
) -> str:
    """Render Markdown section for one corpus.

    Args:
        name: Corpus name
        label: Corpus label (e.g., "org/repo@tag")
        result: Result dict with base_metrics, pr_metrics, times, comparison
        artifacts_config: List of artifact definitions
        pr_label: Label for PR column

    Returns:
        Markdown section string
    """
    bm = result["base_metrics"]
    pm = result["pr_metrics"]
    bt = result["base_time"]
    pt = result["pr_time"]
    cmp = result["comparison"]

    rows = []

    # Rows for each artifact type
    for artifact in artifacts_config:
        art_name = artifact["name"]
        art_label = artifact["name"].upper()

        base_count = bm.get(art_name, {}).get("file_count", 0)
        pr_count = pm.get(art_name, {}).get("file_count", 0)
        base_size = bm.get(art_name, {}).get("total_size", 0)
        pr_size = pm.get(art_name, {}).get("total_size", 0)

        rows.append(format_table_row(
            f"{art_label} files generated",
            str(base_count),
            str(pr_count),
            signed_int(pr_count - base_count)
        ))

        rows.append(format_table_row(
            f"Total {art_label} size",
            human_size(base_size),
            human_size(pr_size),
            pct_delta(base_size, pr_size)
        ))

        diff_count = len(cmp["diffs"].get(art_name, []))
        rows.append(format_table_row(
            f"Files with {art_label} diffs",
            "—",
            str(diff_count),
            ""
        ))

    # Wall-clock time row
    rows.append(format_table_row(
        "Wall-clock time",
        f"{bt:.1f} s",
        f"{pt:.1f} s",
        pct_delta(bt, pt)
    ))

    header = (
        f"### {name} ({label})\n\n"
        f"| {'Metric':<24} | {'base (main)':>11} | {pr_label:>8} | {'Delta':>8} |\n"
        f"|{'-'*26}|{'-'*13}|{'-'*10}|{'-'*10}|"
    )

    table = header + "\n" + "\n".join(rows) + "\n"

    # Add diff details for each artifact
    parts = [table]
    for artifact in artifacts_config:
        art_name = artifact["name"]
        diffs = cmp["diffs"].get(art_name, [])
        if diffs:
            details = build_diff_details(diffs, artifact["name"].upper())
            parts.append(details)

    return "\n".join(parts)


def render_report(
    corpus_results: list[dict],
    artifacts_config: list[dict],
    metadata: dict
) -> str:
    """Render complete Markdown report.

    Args:
        corpus_results: List of corpus result dicts
        artifacts_config: List of artifact definitions
        metadata: Report metadata (project_name, base_ref, pr_ref, pr_number)

    Returns:
        Complete Markdown report string
    """
    project_name = metadata["project_name"]
    base_ref = metadata.get("base_ref")
    pr_ref = metadata.get("pr_ref")
    pr_number = metadata.get("pr_number")

    pr_label = f"PR (#{pr_number})" if pr_number else "PR"

    report_parts = [
        "<!-- astgen-regression -->",
        f"## {project_name} Regression Report",
        ""
    ]

    # Provenance line
    provenance = []
    if base_ref:
        provenance.append(f"**Base:** `{base_ref}`")
    if pr_ref:
        provenance.append(f"**PR:** `{pr_ref}`")

    if provenance:
        report_parts.append(" | ".join(provenance))
        report_parts.append("")

    # Corpus sections
    for result in corpus_results:
        section = render_corpus_section(
            result["name"],
            result["label"],
            result,
            artifacts_config,
            pr_label
        )
        report_parts.append(section)
        report_parts.append("")

    return "\n".join(report_parts)


def write_diff_files(
    diffs_dir: Path,
    corpus_results: list[dict],
    artifacts_config: list[dict]
) -> None:
    """Write full untruncated diff files.

    Args:
        diffs_dir: Directory to write diff files
        corpus_results: List of corpus result dicts
        artifacts_config: List of artifact definitions
    """
    diffs_dir.mkdir(parents=True, exist_ok=True)

    for result in corpus_results:
        name = result["name"]
        cmp = result["comparison"]

        for artifact in artifacts_config:
            art_name = artifact["name"]
            diffs = cmp["diffs"].get(art_name, [])

            if not diffs:
                continue

            parts = []
            for rel, diff_lines, summary in diffs:
                header = f"# {rel}"
                if summary:
                    header += f"  [{summary}]"
                parts.append(header + "\n")
                parts.append("".join(diff_lines))

            diff_file = diffs_dir / f"{name}-{art_name}.diff"
            try:
                diff_file.write_text("".join(parts), encoding='utf-8')
            except IOError as e:
                import sys
                print(
                    f"WARNING: Failed to write {diff_file}: {e}",
                    file=sys.stderr
                )

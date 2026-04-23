import tempfile
from pathlib import Path

from astgen_regression.report import (
    build_diff_details,
    format_table_row,
    human_size,
    pct_delta,
    render_corpus_section,
    render_report,
    signed_int,
    write_diff_files,
)


def test_human_size():
    """Test human-readable file size formatting."""
    assert human_size(0) == "0.0 B"
    assert human_size(1023) == "1023.0 B"
    assert human_size(1024) == "1.0 KB"
    assert human_size(1024 * 1024) == "1.0 MB"
    assert human_size(1024 * 1024 * 1024) == "1.0 GB"
    assert human_size(1536) == "1.5 KB"


def test_pct_delta():
    """Test percentage delta formatting."""
    assert pct_delta(100.0, 110.0) == "+10.0%"
    assert pct_delta(100.0, 90.0) == "-10.0%"
    assert pct_delta(100.0, 100.0) == "+0.0%"
    assert pct_delta(0.0, 0.0) == "+0.0%"
    assert pct_delta(0.0, 100.0) == "+∞%"


def test_signed_int():
    """Test signed integer formatting."""
    assert signed_int(0) == "0"
    assert signed_int(5) == "+5"
    assert signed_int(-5) == "-5"
    assert signed_int(100) == "+100"


def test_format_table_row():
    """Test Markdown table row formatting."""
    row = format_table_row("Test Metric", "100", "110", "+10%")

    assert "Test Metric" in row
    assert "100" in row
    assert "110" in row
    assert "+10%" in row
    assert row.startswith("|")
    assert row.endswith("|")


def test_build_diff_details_empty():
    """Test rendering with no diffs."""
    diffs = []
    result = build_diff_details(diffs, "AST", 200)
    assert result == ""


def test_build_diff_details_with_diffs():
    """Test rendering diff details block."""
    diffs = [
        ("file1.json", ["@@ -1 +1 @@\n", "-old\n", "+new\n"], "1 changed"),
        ("file2.json", ["@@ -1 +1 @@\n", "-foo\n", "+bar\n"], "1 changed"),
    ]

    result = build_diff_details(diffs, "AST", 200)

    assert "<details>" in result
    assert "<summary>2 AST diffs</summary>" in result
    assert "file1.json" in result
    assert "1 changed" in result
    assert "```diff" in result


def test_build_diff_details_truncation():
    """Test diff truncation at max lines."""
    # Create many diff lines
    diff_lines = [f"line {i}\n" for i in range(300)]
    diffs = [("file.json", diff_lines, "many changes")]

    result = build_diff_details(diffs, "AST", max_total_lines=50)

    assert "truncated" in result or "more files not shown" in result


def test_render_corpus_section():
    """Test rendering a corpus section."""
    result_data = {
        "name": "test-corpus",
        "label": "test/test@1.0.0",
        "base_metrics": {"ast": {"file_count": 10, "total_size": 1024000}},
        "pr_metrics": {"ast": {"file_count": 11, "total_size": 1048576}},
        "base_time": 5.5,
        "pr_time": 6.0,
        "comparison": {
            "only_in_base": [],
            "only_in_pr": ["new.json"],
            "diffs": {"ast": [("changed.json", ["diff line\n"], "1 changed")]},
        },
    }

    artifacts_config = [{"name": "ast", "pattern": "*.json"}]

    section = render_corpus_section(
        "test-corpus", "test/test@1.0.0", result_data, artifacts_config, "PR"
    )

    assert "### test-corpus" in section
    assert "test/test@1.0.0" in section
    assert "| Metric" in section
    assert "+1" in section  # file count delta
    assert "<details>" in section  # diff details


def test_render_report():
    """Test rendering complete report."""
    corpus_results = [
        {
            "name": "corpus1",
            "label": "org/repo@1.0",
            "base_metrics": {"ast": {"file_count": 10, "total_size": 1024}},
            "pr_metrics": {"ast": {"file_count": 11, "total_size": 2048}},
            "base_time": 1.0,
            "pr_time": 1.5,
            "comparison": {"only_in_base": [], "only_in_pr": [], "diffs": {"ast": []}},
        }
    ]

    artifacts_config = [{"name": "ast", "pattern": "*.json"}]

    metadata = {
        "project_name": "test-astgen",
        "base_ref": "main @ abc123",
        "pr_ref": "feature @ def456",
        "pr_number": None,
    }

    report = render_report(corpus_results, artifacts_config, metadata)

    assert "<!-- astgen-regression -->" in report
    assert "test-astgen Regression Report" in report
    assert "main @ abc123" in report
    assert "feature @ def456" in report
    assert "corpus1" in report


def test_write_diff_files():
    """Test writing diff files to directory."""
    corpus_results = [
        {
            "name": "corpus1",
            "label": "org/repo@1.0",
            "comparison": {
                "diffs": {
                    "ast": [("file1.json", ["diff content\n"], "summary")],
                    "typemap": [("file1.typemap", ["diff content\n"], "summary")],
                }
            },
        }
    ]

    artifacts_config = [
        {"name": "ast", "pattern": "*.json"},
        {"name": "typemap", "pattern": "*.typemap"},
    ]

    with tempfile.TemporaryDirectory() as tmpdir:
        diffs_dir = Path(tmpdir)

        write_diff_files(diffs_dir, corpus_results, artifacts_config)

        ast_diff = diffs_dir / "corpus1-ast.diff"
        typemap_diff = diffs_dir / "corpus1-typemap.diff"

        assert ast_diff.exists()
        assert typemap_diff.exists()

        ast_content = ast_diff.read_text()
        assert "file1.json" in ast_content
        assert "diff content" in ast_content

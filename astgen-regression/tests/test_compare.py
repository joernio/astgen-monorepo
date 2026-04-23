import tempfile
from pathlib import Path
import json
from astgen_regression.compare import normalize_json, json_diff_summary


def test_normalize_json_valid():
    """Test normalizing valid JSON file."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump({"z": 3, "a": 1, "b": 2}, f)
        path = Path(f.name)

    try:
        lines = normalize_json(path)
        text = ''.join(lines)

        # Should be sorted and indented
        parsed = json.loads(text)
        assert parsed == {"a": 1, "b": 2, "z": 3}
        assert '"a": 1' in text
        assert text.startswith('{')
    finally:
        path.unlink()


def test_normalize_json_malformed():
    """Test normalizing malformed JSON falls back to raw text."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        f.write("{invalid json")
        path = Path(f.name)

    try:
        lines = normalize_json(path)
        text = ''.join(lines)

        assert text == "{invalid json"
    finally:
        path.unlink()


def test_json_diff_summary_added_removed_changed():
    """Test diff summary for added/removed/changed keys."""
    base_json = {"a": 1, "b": 2, "c": {"d": 3}}
    pr_json = {"a": 1, "b": 999, "e": 5}

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(base_json, f)
        base_path = Path(f.name)

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(pr_json, f)
        pr_path = Path(f.name)

    try:
        summary = json_diff_summary(base_path, pr_path)

        # b changed (999 vs 2)
        # c removed (with nested d)
        # e added
        assert "added" in summary
        assert "removed" in summary
        assert "changed" in summary
    finally:
        base_path.unlink()
        pr_path.unlink()


def test_json_diff_summary_identical():
    """Test diff summary for identical JSON."""
    data = {"a": 1, "b": 2}

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(data, f)
        base_path = Path(f.name)

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(data, f)
        pr_path = Path(f.name)

    try:
        summary = json_diff_summary(base_path, pr_path)

        assert summary == "whitespace/ordering only"
    finally:
        base_path.unlink()
        pr_path.unlink()


def test_json_diff_summary_parse_error():
    """Test diff summary when JSON is malformed."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        f.write("{invalid")
        base_path = Path(f.name)

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        f.write("{also invalid")
        pr_path = Path(f.name)

    try:
        summary = json_diff_summary(base_path, pr_path)

        assert summary == ""
    finally:
        base_path.unlink()
        pr_path.unlink()


import difflib
from astgen_regression.compare import compare_outputs


def test_compare_outputs_identical():
    """Test comparing identical output directories."""
    artifacts_config = [{"name": "ast", "pattern": "*.json"}]

    with tempfile.TemporaryDirectory() as tmpdir:
        base_dir = Path(tmpdir) / "base"
        pr_dir = Path(tmpdir) / "pr"
        base_dir.mkdir()
        pr_dir.mkdir()

        # Create identical files
        (base_dir / "file1.json").write_text('{"a": 1}')
        (pr_dir / "file1.json").write_text('{"a": 1}')

        result = compare_outputs(base_dir, pr_dir, artifacts_config)

        assert result["only_in_base"] == []
        assert result["only_in_pr"] == []
        assert result["diffs"]["ast"] == []


def test_compare_outputs_only_in_base():
    """Test files only in base directory."""
    artifacts_config = [{"name": "ast", "pattern": "*.json"}]

    with tempfile.TemporaryDirectory() as tmpdir:
        base_dir = Path(tmpdir) / "base"
        pr_dir = Path(tmpdir) / "pr"
        base_dir.mkdir()
        pr_dir.mkdir()

        (base_dir / "removed.json").write_text('{"deleted": true}')
        (base_dir / "kept.json").write_text('{"kept": true}')
        (pr_dir / "kept.json").write_text('{"kept": true}')

        result = compare_outputs(base_dir, pr_dir, artifacts_config)

        assert "removed.json" in result["only_in_base"]
        assert result["only_in_pr"] == []


def test_compare_outputs_only_in_pr():
    """Test files only in PR directory."""
    artifacts_config = [{"name": "ast", "pattern": "*.json"}]

    with tempfile.TemporaryDirectory() as tmpdir:
        base_dir = Path(tmpdir) / "base"
        pr_dir = Path(tmpdir) / "pr"
        base_dir.mkdir()
        pr_dir.mkdir()

        (base_dir / "kept.json").write_text('{"kept": true}')
        (pr_dir / "kept.json").write_text('{"kept": true}')
        (pr_dir / "new.json").write_text('{"added": true}')

        result = compare_outputs(base_dir, pr_dir, artifacts_config)

        assert result["only_in_base"] == []
        assert "new.json" in result["only_in_pr"]


def test_compare_outputs_with_diffs():
    """Test comparing files with differences."""
    artifacts_config = [{"name": "ast", "pattern": "*.json"}]

    with tempfile.TemporaryDirectory() as tmpdir:
        base_dir = Path(tmpdir) / "base"
        pr_dir = Path(tmpdir) / "pr"
        base_dir.mkdir()
        pr_dir.mkdir()

        (base_dir / "changed.json").write_text('{"old": "value"}')
        (pr_dir / "changed.json").write_text('{"new": "value"}')

        result = compare_outputs(base_dir, pr_dir, artifacts_config)

        assert len(result["diffs"]["ast"]) == 1
        rel_path, diff_lines, summary = result["diffs"]["ast"][0]
        assert rel_path == "changed.json"
        assert len(diff_lines) > 0
        assert "added" in summary or "removed" in summary


def test_compare_outputs_multiple_artifacts():
    """Test comparing with multiple artifact types."""
    artifacts_config = [
        {"name": "ast", "pattern": "*.json"},
        {"name": "typemap", "pattern": "*.typemap"}
    ]

    with tempfile.TemporaryDirectory() as tmpdir:
        base_dir = Path(tmpdir) / "base"
        pr_dir = Path(tmpdir) / "pr"
        base_dir.mkdir()
        pr_dir.mkdir()

        (base_dir / "file.json").write_text('{"base": 1}')
        (pr_dir / "file.json").write_text('{"pr": 2}')

        (base_dir / "types.typemap").write_text("base types")
        (pr_dir / "types.typemap").write_text("pr types")

        result = compare_outputs(base_dir, pr_dir, artifacts_config)

        assert len(result["diffs"]["ast"]) == 1
        assert len(result["diffs"]["typemap"]) == 1

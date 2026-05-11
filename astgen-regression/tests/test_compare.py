import json
import tempfile
from pathlib import Path

from astgen_regression.compare import compare_outputs, json_diff_summary, normalize_json


def test_normalize_json_valid():
    """Test normalizing valid JSON file."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump({"z": 3, "a": 1, "b": 2}, f)
        path = Path(f.name)

    try:
        lines = normalize_json(path)
        text = "".join(lines)

        # Should be sorted and indented
        parsed = json.loads(text)
        assert parsed == {"a": 1, "b": 2, "z": 3}
        assert '"a": 1' in text
        assert text.startswith("{")
    finally:
        path.unlink()


def test_normalize_json_malformed():
    """Test normalizing malformed JSON falls back to raw text."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        f.write("{invalid json")
        path = Path(f.name)

    try:
        lines = normalize_json(path)
        text = "".join(lines)

        assert text == "{invalid json"
    finally:
        path.unlink()


def test_json_diff_summary_added_removed_changed():
    """Test diff summary for added/removed/changed keys."""
    base_json = {"a": 1, "b": 2, "c": {"d": 3}}
    pr_json = {"a": 1, "b": 999, "e": 5}

    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(base_json, f)
        base_path = Path(f.name)

    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
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

    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(data, f)
        base_path = Path(f.name)

    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(data, f)
        pr_path = Path(f.name)

    try:
        summary = json_diff_summary(base_path, pr_path)

        assert summary == "no changes"
    finally:
        base_path.unlink()
        pr_path.unlink()


def test_json_diff_summary_parse_error():
    """Test diff summary when JSON is malformed."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        f.write("{invalid")
        base_path = Path(f.name)

    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        f.write("{also invalid")
        pr_path = Path(f.name)

    try:
        summary = json_diff_summary(base_path, pr_path)

        assert summary == ""
    finally:
        base_path.unlink()
        pr_path.unlink()


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
        # DeepDiff sees this as a structural change (whole dict changed)
        assert "changed" in summary


def test_compare_outputs_multiple_artifacts():
    """Test comparing with multiple artifact types."""
    artifacts_config = [
        {"name": "ast", "pattern": "*.json"},
        {"name": "typemap", "pattern": "*.typemap"},
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


def test_compare_outputs_deterministic_order_under_threading():
    """Diffs must come back sorted by rel even though comparison is parallelized."""
    artifacts_config = [{"name": "ast", "pattern": "*.json"}]

    with tempfile.TemporaryDirectory() as tmpdir:
        base_dir = Path(tmpdir) / "base"
        pr_dir = Path(tmpdir) / "pr"
        base_dir.mkdir()
        pr_dir.mkdir()

        # 50 identical files to give the thread pool real work to chew through.
        for i in range(50):
            payload = json.dumps({"id": i})
            (base_dir / f"same_{i:02d}.json").write_text(payload)
            (pr_dir / f"same_{i:02d}.json").write_text(payload)

        # Differing files whose names sort non-trivially; intentionally created
        # in an order that does not match their sorted order.
        diff_names = [
            "zeta.json",
            "alpha.json",
            "mango.json",
            "beta.json",
            "kappa.json",
        ]
        for name in diff_names:
            (base_dir / name).write_text(json.dumps({"v": "base"}))
            (pr_dir / name).write_text(json.dumps({"v": "pr"}))

        result = compare_outputs(base_dir, pr_dir, artifacts_config)

        diff_rels = [rel for rel, _, _ in result["diffs"]["ast"]]
        assert diff_rels == sorted(diff_names)
        assert result["only_in_base"] == []
        assert result["only_in_pr"] == []


def test_compare_outputs_oversized_diff_guard(monkeypatch):
    """Files whose normalized line count exceeds MAX_DIFF_LINES should skip the unified diff."""
    artifacts_config = [{"name": "ast", "pattern": "*.json"}]

    # Lower the threshold so the test stays fast and self-contained.
    monkeypatch.setattr("astgen_regression.compare.MAX_DIFF_LINES", 20)

    # 30 keys * (key line + value line patterns) easily clears 20 normalized lines per side.
    big_base = {f"k{i:03d}": i for i in range(30)}
    big_pr = {f"k{i:03d}": i + 1 for i in range(30)}

    with tempfile.TemporaryDirectory() as tmpdir:
        base_dir = Path(tmpdir) / "base"
        pr_dir = Path(tmpdir) / "pr"
        base_dir.mkdir()
        pr_dir.mkdir()

        (base_dir / "huge.json").write_text(json.dumps(big_base))
        (pr_dir / "huge.json").write_text(json.dumps(big_pr))

        result = compare_outputs(base_dir, pr_dir, artifacts_config)

        assert len(result["diffs"]["ast"]) == 1
        rel_path, diff_lines, summary = result["diffs"]["ast"][0]
        assert rel_path == "huge.json"
        joined = "".join(diff_lines)
        assert "diff omitted" in joined
        assert "exceeds 20" in joined
        # Summary is still computed from the parsed objects.
        assert "changed" in summary

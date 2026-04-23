import tempfile
from pathlib import Path
from unittest.mock import patch, MagicMock
import pytest
from astgen_regression.commands.compare import cmd_compare


def test_cmd_compare_exits_with_code_1_on_regressions():
    """Test that cmd_compare exits with code 1 when regressions are detected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        base_dir = Path(tmpdir) / "base"
        pr_dir = Path(tmpdir) / "pr"
        base_dir.mkdir()
        pr_dir.mkdir()

        # Create mock args
        args = MagicMock()
        args.config = None
        args.base_dist = base_dir
        args.pr_dist = pr_dir
        args.base_ref = "main"
        args.pr_ref = "PR"
        args.pr_number = None
        args.output_diffs = None

        # Mock config with minimal valid structure
        mock_config = {
            "project": {"name": "test"},
            "execute": {
                "command": "echo {input_dir} {output_dir}",
                "timeout": 60
            },
            "artifacts": [{"name": "ast", "pattern": "*.json"}],
            "corpora": [
                {
                    "name": "test-corpus",
                    "label": "test@1.0",
                    "clone_url": "https://example.com/repo.git",
                    "tag": "1.0.0",
                    "input_subdir": ""
                }
            ]
        }

        # Mock corpus results with regressions (diffs present)
        mock_corpus_results = [
            {
                "name": "test-corpus",
                "label": "test@1.0",
                "base_metrics": {},
                "pr_metrics": {},
                "base_time": 1.0,
                "pr_time": 1.0,
                "comparison": {
                    "diffs": {
                        "ast": [
                            ("file.json", ["- old line", "+ new line"], "1 added, 1 removed")
                        ]
                    }
                }
            }
        ]

        with patch('astgen_regression.commands.compare.load_config', return_value=mock_config):
            with patch('astgen_regression.commands.compare.clone_corpus', return_value=Path("/tmp/corpus")):
                with patch('astgen_regression.commands.compare.execute_astgen', return_value=(True, 1.0)):
                    with patch('astgen_regression.commands.compare.collect_metrics', return_value={}):
                        with patch('astgen_regression.commands.compare.compare_outputs',
                                  return_value={"diffs": {"ast": [("file.json", ["diff"], "summary")]}}):
                            with patch('astgen_regression.commands.compare.render_report', return_value="# Report"):
                                with pytest.raises(SystemExit) as exc_info:
                                    cmd_compare(args)

                                assert exc_info.value.code == 1


def test_cmd_compare_exits_with_code_0_when_no_regressions():
    """Test that cmd_compare exits with code 0 when no regressions are detected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        base_dir = Path(tmpdir) / "base"
        pr_dir = Path(tmpdir) / "pr"
        base_dir.mkdir()
        pr_dir.mkdir()

        # Create mock args
        args = MagicMock()
        args.config = None
        args.base_dist = base_dir
        args.pr_dist = pr_dir
        args.base_ref = "main"
        args.pr_ref = "PR"
        args.pr_number = None
        args.output_diffs = None

        # Mock config with minimal valid structure
        mock_config = {
            "project": {"name": "test"},
            "execute": {
                "command": "echo {input_dir} {output_dir}",
                "timeout": 60
            },
            "artifacts": [{"name": "ast", "pattern": "*.json"}],
            "corpora": [
                {
                    "name": "test-corpus",
                    "label": "test@1.0",
                    "clone_url": "https://example.com/repo.git",
                    "tag": "1.0.0",
                    "input_subdir": ""
                }
            ]
        }

        # Mock corpus results with NO regressions (empty diffs)
        mock_corpus_results = [
            {
                "name": "test-corpus",
                "label": "test@1.0",
                "base_metrics": {},
                "pr_metrics": {},
                "base_time": 1.0,
                "pr_time": 1.0,
                "comparison": {
                    "diffs": {
                        "ast": []  # No diffs
                    }
                }
            }
        ]

        with patch('astgen_regression.commands.compare.load_config', return_value=mock_config):
            with patch('astgen_regression.commands.compare.clone_corpus', return_value=Path("/tmp/corpus")):
                with patch('astgen_regression.commands.compare.execute_astgen', return_value=(True, 1.0)):
                    with patch('astgen_regression.commands.compare.collect_metrics', return_value={}):
                        with patch('astgen_regression.commands.compare.compare_outputs',
                                  return_value={"diffs": {"ast": []}}):
                            with patch('astgen_regression.commands.compare.render_report', return_value="# Report"):
                                # Should complete without raising SystemExit
                                # (or if it does, code should be 0)
                                try:
                                    cmd_compare(args)
                                    # If no exception, that's success (exit code 0)
                                except SystemExit as e:
                                    # If it exits, should be with code 0 or None
                                    assert e.code in (0, None)

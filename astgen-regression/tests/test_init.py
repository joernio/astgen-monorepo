"""Tests for init command."""

import tempfile
from pathlib import Path
from unittest.mock import patch
import pytest
from astgen_regression.commands.init import cmd_init, LANGUAGE_DEFAULTS


def test_language_defaults_contains_seven_languages():
    """Verify LANGUAGE_DEFAULTS has exactly seven supported languages."""
    expected_languages = {"javascript", "rust", "swift", "dotnet", "ruby", "go", "abap"}
    assert set(LANGUAGE_DEFAULTS.keys()) == expected_languages


def test_language_defaults_no_typescript():
    """Verify typescript is removed."""
    assert "typescript" not in LANGUAGE_DEFAULTS


def test_language_defaults_no_csharp():
    """Verify csharp is renamed to dotnet."""
    assert "csharp" not in LANGUAGE_DEFAULTS
    assert "dotnet" in LANGUAGE_DEFAULTS


def test_language_defaults_has_go():
    """Verify go is present with correct structure."""
    assert "go" in LANGUAGE_DEFAULTS
    go_config = LANGUAGE_DEFAULTS["go"]
    assert go_config["project_name"] == "goastgen"
    assert go_config["language"] == "go"
    assert go_config["build_command"] == "go build -o build/goastgen"
    assert go_config["dist_dir"] == "build"
    assert (
        go_config["exec_command"] == "{dist_dir}/goastgen -out {output_dir} {input_dir}"
    )


def test_language_defaults_has_abap():
    """Verify abap is present (even with TODOs)."""
    assert "abap" in LANGUAGE_DEFAULTS
    abap_config = LANGUAGE_DEFAULTS["abap"]
    assert abap_config["project_name"] == "abapastgen"
    assert abap_config["language"] == "abap"
    # TODOs are intentional placeholders
    assert abap_config["build_command"] == "TODO"


def test_dotnet_language_field_updated():
    """Verify dotnet entry has language field set to 'dotnet' not 'csharp'."""
    assert "dotnet" in LANGUAGE_DEFAULTS
    dotnet_config = LANGUAGE_DEFAULTS["dotnet"]
    assert dotnet_config["language"] == "dotnet"
    assert dotnet_config["project_name"] == "DotNetAstGen"


def test_cmd_init_with_go_language():
    """Test init command generates valid config for go language."""
    with tempfile.TemporaryDirectory() as tmpdir:
        config_path = Path(tmpdir) / "regression.yaml"
        workflow_path = Path(tmpdir) / ".github" / "workflows" / "regression.yml"

        # Mock args
        from unittest.mock import MagicMock

        args = MagicMock()
        args.config = config_path
        args.language = "go"
        args.force = False

        # Mock workflow path resolution
        with patch("astgen_regression.commands.init.Path") as mock_path_class:
            # Make Path() return our temporary paths
            def path_side_effect(path_str):
                if ".github/workflows" in path_str:
                    return workflow_path
                return Path(path_str)

            mock_path_class.side_effect = path_side_effect

            # Run init
            cmd_init(args)

            # Verify config file created
            assert config_path.exists()
            content = config_path.read_text()
            assert 'language: "go"' in content
            assert "go build -o build/goastgen" in content


def test_cmd_init_with_dotnet_language():
    """Test init command works with dotnet (not csharp)."""
    with tempfile.TemporaryDirectory() as tmpdir:
        config_path = Path(tmpdir) / "regression.yaml"
        workflow_path = Path(tmpdir) / ".github" / "workflows" / "regression.yml"

        from unittest.mock import MagicMock

        args = MagicMock()
        args.config = config_path
        args.language = "dotnet"
        args.force = False

        with patch("astgen_regression.commands.init.Path") as mock_path_class:
            # Make Path() return our temporary paths
            def path_side_effect(path_str):
                if ".github/workflows" in path_str:
                    return workflow_path
                return Path(path_str)

            mock_path_class.side_effect = path_side_effect

            cmd_init(args)

            assert config_path.exists()
            content = config_path.read_text()
            assert 'language: "dotnet"' in content


def test_cmd_init_rejects_csharp_language():
    """Test init command rejects removed 'csharp' language."""
    with tempfile.TemporaryDirectory() as tmpdir:
        config_path = Path(tmpdir) / "regression.yaml"

        from unittest.mock import MagicMock

        args = MagicMock()
        args.config = config_path
        args.language = "csharp"
        args.force = False

        # Should exit with error
        with pytest.raises(SystemExit) as exc_info:
            cmd_init(args)

        assert exc_info.value.code == 1


def test_cmd_init_rejects_typescript_language():
    """Test init command rejects removed 'typescript' language."""
    with tempfile.TemporaryDirectory() as tmpdir:
        config_path = Path(tmpdir) / "regression.yaml"

        from unittest.mock import MagicMock

        args = MagicMock()
        args.config = config_path
        args.language = "typescript"
        args.force = False

        # Should exit with error
        with pytest.raises(SystemExit) as exc_info:
            cmd_init(args)

        assert exc_info.value.code == 1

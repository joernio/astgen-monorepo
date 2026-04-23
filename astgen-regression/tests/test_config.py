import tempfile
from pathlib import Path
import pytest
from astgen_regression.config import load_config, ConfigError


def test_load_config_valid():
    """Test loading a valid config file."""
    config_yaml = """
project:
  name: "test-astgen"
  language: "javascript"

build:
  build_command: "yarn build"
  dist_dir: "dist"

execute:
  command: "node {dist_dir}/astgen.js -i {input_dir} -o {output_dir}"
  timeout: 600

artifacts:
  - name: "AST"
    pattern: "*.json"

corpora:
  - name: "test-corpus"
    label: "test/test@1.0.0"
    clone_url: "https://github.com/test/test.git"
    tag: "1.0.0"
    input_subdir: "src"
"""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.yaml', delete=False) as f:
        f.write(config_yaml)
        config_path = Path(f.name)

    try:
        config = load_config(config_path)
        assert config["project"]["name"] == "test-astgen"
        assert config["project"]["language"] == "javascript"
        assert config["build"]["build_command"] == "yarn build"
        assert config["execute"]["timeout"] == 600
        assert len(config["artifacts"]) == 1
        assert len(config["corpora"]) == 1
    finally:
        config_path.unlink()


def test_load_config_missing_file():
    """Test loading non-existent config file."""
    with pytest.raises(ConfigError, match="Config file not found"):
        load_config(Path("/nonexistent/config.yaml"))


def test_validate_config_missing_required_field():
    """Test validation fails for missing required fields."""
    from astgen_regression.config import validate_config

    config = {
        "project": {"name": "test"},
        # Missing build, execute, corpora
    }

    with pytest.raises(ConfigError, match="Missing required field: build.build_command"):
        validate_config(config)


def test_validate_config_invalid_command_template():
    """Test validation fails for command template without placeholders."""
    from astgen_regression.config import validate_config

    config = {
        "project": {"name": "test", "language": "js"},
        "build": {"build_command": "build", "dist_dir": "dist"},
        "execute": {
            "command": "run astgen",  # Missing {input_dir} and {output_dir}
            "timeout": 600
        },
        "artifacts": [{"name": "AST", "pattern": "*.json"}],
        "corpora": [
            {
                "name": "test",
                "label": "test@1.0",
                "clone_url": "https://github.com/test/test.git",
                "tag": "1.0",
                "input_subdir": ""
            }
        ]
    }

    with pytest.raises(ConfigError, match="Command template must contain.*input_dir.*output_dir"):
        validate_config(config)

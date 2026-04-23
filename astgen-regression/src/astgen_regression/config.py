"""Configuration loading and validation."""

from pathlib import Path
from typing import Any
import yaml


class ConfigError(Exception):
    """Configuration validation error."""
    pass


def load_config(config_path: Path) -> dict[str, Any]:
    """Load and validate configuration from YAML file.

    Args:
        config_path: Path to regression.yaml

    Returns:
        Validated config dict

    Raises:
        ConfigError: If file not found or validation fails
    """
    if not config_path.exists():
        raise ConfigError(f"Config file not found: {config_path}")

    try:
        with open(config_path, 'r', encoding='utf-8') as f:
            config = yaml.safe_load(f)
    except yaml.YAMLError as e:
        raise ConfigError(f"Invalid YAML: {e}")

    validate_config(config)
    return config


def validate_config(config: dict[str, Any]) -> None:
    """Validate configuration structure and required fields.

    Args:
        config: Configuration dict

    Raises:
        ConfigError: If validation fails
    """
    # Check required top-level sections
    if "project" not in config:
        raise ConfigError("Missing required field: project")
    if "name" not in config.get("project", {}):
        raise ConfigError("Missing required field: project.name")

    if "build_command" not in config.get("build", {}):
        raise ConfigError("Missing required field: build.build_command")
    if "dist_dir" not in config.get("build", {}):
        raise ConfigError("Missing required field: build.dist_dir")

    if "command" not in config.get("execute", {}):
        raise ConfigError("Missing required field: execute.command")

    # Validate command template
    cmd = config.get("execute", {}).get("command", "")
    if "{input_dir}" not in cmd or "{output_dir}" not in cmd:
        raise ConfigError(
            "Command template must contain {input_dir} and {output_dir} placeholders.\n"
            f"Got: {cmd}\n\n"
            "Example:\n"
            '  command: "node {dist_dir}/astgen.js -i {input_dir} -o {output_dir}"'
        )

    if "corpora" not in config or not config["corpora"]:
        raise ConfigError("Missing required field: corpora (must have at least one corpus)")

    # Validate each corpus
    for i, corpus in enumerate(config["corpora"]):
        required = ["name", "label", "clone_url", "tag"]
        for field in required:
            if field not in corpus:
                raise ConfigError(f"Missing required field in corpora[{i}]: {field}")

    # Validate artifacts if present
    if "artifacts" in config:
        for i, artifact in enumerate(config["artifacts"]):
            if "name" not in artifact:
                raise ConfigError(f"Missing required field in artifacts[{i}]: name")
            if "pattern" not in artifact:
                raise ConfigError(f"Missing required field in artifacts[{i}]: pattern")

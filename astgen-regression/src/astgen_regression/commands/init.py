"""Init command implementation."""

from pathlib import Path
import sys

from jinja2 import Environment, PackageLoader, select_autoescape


LANGUAGE_DEFAULTS = {
    "javascript": {
        "project_name": "astgen",
        "language": "javascript",
        "install_command": "yarn install",
        "build_command": "yarn build",
        "dist_dir": "dist",
        "exec_command": "node {dist_dir}/astgen.js -i {input_dir} -o {output_dir} -t ts",
        "artifacts": [
            {"name": "AST", "pattern": "*.json", "description": "Abstract Syntax Tree files"},
            {"name": "typemap", "pattern": "*.typemap", "description": "TypeScript type mapping files"}
        ],
        "base_branch": "main",
        "python_version": "3.12",
        "setup_action": {
            "uses": "actions/setup-node@v6",
            "with": {"node-version": "24", "cache": "yarn"}
        }
    },
    "rust": {
        "project_name": "rust_ast_gen",
        "language": "rust",
        "install_command": None,
        "build_command": "cargo build --release",
        "dist_dir": "target/release",
        "exec_command": "{dist_dir}/rust_ast_gen --input-dir {input_dir} --output-dir {output_dir}",
        "artifacts": [
            {"name": "AST", "pattern": "*.json", "description": "Abstract Syntax Tree files"}
        ],
        "base_branch": "main",
        "python_version": "3.12",
        "setup_action": {
            "uses": "actions-rust-lang/setup-rust-toolchain@v1",
            "with": None
        }
    },
    "swift": {
        "project_name": "SwiftAstGen",
        "language": "swift",
        "install_command": None,
        "build_command": "swift build",
        "dist_dir": ".build/debug",
        "exec_command": "{dist_dir}/SwiftAstGen --src {input_dir} --output {output_dir}",
        "artifacts": [
            {"name": "AST", "pattern": "*.json", "description": "Abstract Syntax Tree files"}
        ],
        "base_branch": "main",
        "python_version": "3.12",
        "runs_on": "macos-latest",
        "setup_action": {
            "uses": "SwiftyLab/setup-swift@latest",
            "with": {"swift-version": "6.1"}
        }
    },
    "dotnet": {
        "project_name": "DotNetAstGen",
        "language": "dotnet",
        "install_command": None,
        "build_command": "dotnet build -c Release",
        "dist_dir": "DotNetAstGen/bin/Release/net8.0",
        "exec_command": "dotnet run --project DotNetAstGen -i {input_dir} -o {output_dir}",
        "artifacts": [
            {"name": "AST", "pattern": "*.json", "description": "Abstract Syntax Tree files"},
            {"name": "Symbols", "pattern": "*_Symbols.json", "description": "Symbol summary files"}
        ],
        "base_branch": "main",
        "python_version": "3.12",
        "setup_action": {
            "uses": "actions/setup-dotnet@v4",
            "with": {"dotnet-version": "8.0.x"}
        }
    },
    "ruby": {
        "project_name": "ruby_ast_gen",
        "language": "ruby",
        "install_command": "bundle install",
        "build_command": "bundle exec rake build",
        "dist_dir": "exe",
        "exec_command": "{dist_dir}/ruby_ast_gen -i {input_dir} -o {output_dir}",
        "artifacts": [
            {"name": "AST", "pattern": "*.json", "description": "Abstract Syntax Tree files"}
        ],
        "base_branch": "main",
        "python_version": "3.12",
        "setup_action": {
            "uses": "ruby/setup-ruby@v1",
            "with": {"ruby-version": "3.3"}
        }
    },
    "go": {
        "project_name": "goastgen",
        "language": "go",
        "install_command": None,
        "build_command": "go build -o build/goastgen",
        "dist_dir": "build",
        "exec_command": "{dist_dir}/goastgen -out {output_dir} {input_dir}",
        "artifacts": [
            {"name": "AST", "pattern": "*.json", "description": "Abstract Syntax Tree files"}
        ],
        "base_branch": "main",
        "python_version": "3.12",
        "setup_action": {
            "uses": "actions/setup-go@v5",
            "with": {"go-version": "1.22"}
        }
    },
    "abap": {
        "project_name": "abapastgen",
        "language": "abap",
        "install_command": "TODO",
        "build_command": "TODO",
        "dist_dir": "TODO",
        "exec_command": "TODO {dist_dir}/abapastgen -i {input_dir} -o {output_dir}",
        "artifacts": [
            {"name": "AST", "pattern": "*.json", "description": "Abstract Syntax Tree files"}
        ],
        "base_branch": "main",
        "python_version": "3.12",
        "setup_action": {
            "uses": "TODO",
            "with": None
        }
    }
}


def cmd_init(args) -> None:
    """Run init command.

    Args:
        args: Parsed command-line arguments
    """
    config_path = args.config
    workflow_path = Path(".github/workflows/regression.yml")

    # Check if files exist
    if config_path.exists() and not args.force:
        print(f"ERROR: {config_path} already exists. Use --force to overwrite.", file=sys.stderr)
        sys.exit(1)

    if workflow_path.exists() and not args.force:
        print(f"ERROR: {workflow_path} already exists. Use --force to overwrite.", file=sys.stderr)
        sys.exit(1)

    # Get language defaults
    language = args.language or "javascript"
    if language not in LANGUAGE_DEFAULTS:
        print(f"ERROR: Unknown language '{language}'. Supported: {', '.join(LANGUAGE_DEFAULTS.keys())}", file=sys.stderr)
        sys.exit(1)

    defaults = LANGUAGE_DEFAULTS[language]

    # Setup Jinja2 environment
    env = Environment(
        loader=PackageLoader("astgen_regression", "templates"),
        autoescape=select_autoescape()
    )

    # Render config template
    config_template = env.get_template("regression.yaml.j2")
    config_content = config_template.render(**defaults)

    # Write config file
    config_path.parent.mkdir(parents=True, exist_ok=True)
    config_path.write_text(config_content, encoding='utf-8')
    print(f"Created {config_path}")

    # Render workflow template
    workflow_template = env.get_template("workflow.yml.j2")
    workflow_content = workflow_template.render(**defaults)

    # Write workflow file
    workflow_path.parent.mkdir(parents=True, exist_ok=True)
    workflow_path.write_text(workflow_content, encoding='utf-8')
    print(f"Created {workflow_path}")

    print("\nNext steps:")
    print(f"  1. Edit {config_path} to define your corpora")
    print("  2. Run: astgen-regression local")

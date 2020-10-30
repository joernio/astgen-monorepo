# AST generator

This script creates Abstract Syntax Tree (AST) of all the files for supported languages in JSON format. AST is created by using the bundled babel parser (for JavaScript, TypeScript) or by invoking appropriate external tools.

## Supported languages

| Language   | Tool used |
| ---------- | --------- |
| JavaScript | babel     |
| TypeScript | babel     |
| JSX        | babel     |
| TSX        | babel     |

## Usage

## Installing

```bash
sudo npm install -g @appthreat/astgen
```

## Getting Help

```bash
bin/astgen -h
Options:
  -v, --version  Print version number                                  [boolean]
  -i, --src      Source directory                                 [default: "."]
  -o, --output   Output directory for generated AST json files
                                                            [default: "ast_out"]
  -t, --type     Project type. Default auto-detect
  -r, --recurse  Recurse mode suitable for mono-repos  [boolean] [default: true]
  -h             Show help                                             [boolean]
```

## Example

Navigate to the project and run `astgen` command.

```bash
cd <path to project>
astgen
```

To specify the project type and the path to the project.

```bash
astgen -t nodejs -i <path to project>
```

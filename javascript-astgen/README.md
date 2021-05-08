# AST generator

This script creates Abstract Syntax Tree (AST) of all the files for supported languages in JSON format. AST is created by using the bundled babel parser (for JavaScript, TypeScript) or by invoking appropriate external tools (See table below).

## Supported languages

| Language   | Tool used             |
| ---------- | --------------------- |
| Bash       | shfmt (cli)           |
| JavaScript | babel                 |
| TypeScript | babel                 |
| Vue        | vue-template-compiler |
| JSX        | babel                 |
| TSX        | babel                 |
| Dockerfile | dockerfile-ast        |

## Usage

## Installing

```bash
sudo npm install -g @joernio/astgen
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
astgen -t vue -i <path containing .vue files>
```

## Troubleshooting

`shfmt` is bundled with astgen and should work for linux amd64. When using astgen on any other platform and architecture, please download and place the [cli](https://github.com/mvdan/sh/releases) in the PATH.

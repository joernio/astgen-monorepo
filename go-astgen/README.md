# Go AST Generator

Generates JSON Abstract Syntax Trees (ASTs) for `.go` source files using the Go standard library.

If you pass the root folder of a Go project, it iterates through all `.go` files in the project directory and generates an AST in JSON format for each file.

## Supported languages

| Language | Tool used          | Notes |
| -------- | ------------------ | ----- |
| Go       | Go standard library |      |

## Building

```bash
go build -o build/goastgen
```

This generates a native binary for your local machine in the `build` folder.

## Testing

```bash
go test
```

## Usage

```
Usage:
    goastgen [flags] <source location>

Flags:
  -exclude string
        regex to exclude files
  -help
        print the usage
  -include string
        regex to include files
  -include-packages string
        ',' separated list of only package folders (e.g. "/pkg/, /cmd/")
  -out string
        Output location of ast (default ".ast")
  -version
        print the version
```

## Example

### Single file

Without `-out` the AST is written next to the source in a `.ast/` directory:

```bash
$ goastgen /path/src/hello.go
# -> /path/src/.ast/hello.go.json
```

With `-out` the output directory is overridden:

```bash
$ goastgen -out /tmp/randompath /path/src/hello.go
# -> /tmp/randompath/hello.go.json
```

### Project directory

Given a project tree:

```
/path/repository
      - hello.go
      - anotherfile.go
      - somepkg
            - somelib.go
```

Without `-out`:

```bash
$ goastgen /path/repository
# -> /path/repository/.ast/{hello.go.json, anotherfile.go.json, somepkg/somelib.go.json}
```

With `-out`:

```bash
$ goastgen -out /tmp/out/ /path/repository
# -> /tmp/out/{hello.go.json, anotherfile.go.json, somepkg/somelib.go.json}
```

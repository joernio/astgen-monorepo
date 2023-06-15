# AST generator

This utility generates Abstract Syntax Tree (AST) for .go files in JSON format. 

If you pass the root folder of the go project, it will iterate through all `.go` files from project directory 
and generate ast in JSON format for each `.go` file.

## Usage

## Building

Run below command from within root folder of the cloned repository.

```bash
 go build -o build/goastgen
```

This will generate native binary for your local machine inside `build` folder

## Getting Help

```bash
build/goastgen -help

Usage:
	goastgen [falgs] <source location>

Flags:
  -help
    	print the usage
  -out string
    	Out put location of ast (default ".ast")
  -version
    	print the version
```

## Example

### Single file
1. Generate AST with single `.go` file path without passing `-out` flag to indicate ast json out location.

```bash
$ goastgen <filepath>/<go filename>

e.g
$ goastgen /path/src/hello.go 

It should generate the AST in JSON format at 

/path/src/.ast/hello.go.json
```

2. Generate AST with single `.go` file with `-out` flag 

```bash
$ goastgen -out <output location> <filepath>/<go filename>

e.g
$ goastgen -out /tmp/randompath /path/src/hello.go 

It should generate the AST in JSON format at 

/tmp/randompath/hello.go.json
```

### Complete project directory

```bash
/path/repository
      - hello.go
      - anotherfile.go
      - somepkg
            - somelib.go
```
1. Generate AST with above root directory of the go project without passing `-out` flag
```bash
$ goastgen <root directory location of go project>

e.g.
$ goastgen /path/repository

It should generate AST in JSON fromat for each .go file at following location

/path/repository/.ast
                  - hello.go.json
                  - anotherfile.go.json
                  - somepkg
                        - somelib.go.json      
```

2. Generate AST with above root directory of the go project with `-out` flag

```bash
$ goastgen -out <output location> <root directory location of go project> 

e.g.
$ goastgen -out /temp/out/ /path/repository

It should generate AST in JSON fromat for each .go file at following location

/temp/out/
          - hello.go.json
          - anotherfile.go.json
          - somepkg
                - somelib.go.json      
```
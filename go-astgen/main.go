package main

import "C"
import (
	"fmt"
	"privado.ai/goastgen/goastgen"
)

//export ParseAstFromSource
func ParseAstFromSource(filename *C.char, src *C.char) *C.char {
	resultJson := goastgen.ParseAstFromSource(C.GoString(filename), C.GoString(src))
	return C.CString(resultJson)
}

//export ParseAstFromDir
func ParseAstFromDir(dir *C.char) *C.char {
	resultJson := goastgen.ParseAstFromDir(C.GoString(dir))
	return C.CString(resultJson)
}

//export ParseAstFromFile
func ParseAstFromFile(file *C.char) *C.char {
	resultJson := goastgen.ParseAstFromFile(C.GoString(file))
	return C.CString(resultJson)
}

func main() {
	fmt.Println("TODO: > Create a CLI interface to take input as source code folder location and generate the AST in JSON format")
}

// build

// go build -buildmode=c-shared -o lib-goastgen.dylib main.go

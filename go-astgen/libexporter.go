package main

import "C"
import (
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

//export Add
func Add(a int, b int) int {
	return a + b
}

func main() {}

// build

// go build -buildmode=c-shared -o lib-goastgen.dylib libexporter.go

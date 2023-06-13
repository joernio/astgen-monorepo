package main

import (
	"fmt"
	"os"
	"privado.ai/goastgen/goastgen"
)

func main() {
	args := os.Args[1:]
	path := args[0]
	resultJson := goastgen.ParseAstFromFile(path)
	fmt.Println(resultJson)
}

// go build -buildmode=c-shared -o lib-goastgen.dylib main.go

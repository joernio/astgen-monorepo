package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"privado.ai/goastgen/goastgen"
	"strings"
)

func main() {
	out, inputPath := parseArguments()
	processRequest(out, inputPath)
}

func processRequest(out string, inputPath string) {
	if strings.HasSuffix(inputPath, ".go") {
		fileInfo, err := os.Stat(inputPath)
		if err != nil {
			fmt.Println("Failed to get file info:", err)
			return
		}
		directory := filepath.Dir(inputPath)
		var outFile = ""
		if out == ".ast" {
			outFile = filepath.Join(directory, out, fileInfo.Name()+".json")
		} else {
			outFile = filepath.Join(out, fileInfo.Name()+".json")
		}
		writeFileContents(outFile, goastgen.ParseAstFromFile(inputPath))
	} else {
		err := filepath.Walk(inputPath, func(path string, info os.FileInfo, err error) error {
			if err != nil {
				fmt.Printf("Error accessing path %s: %v\n", path, err)
				return err
			}
			if !info.IsDir() && strings.HasSuffix(info.Name(), ".go") {
				var outFile = ""
				directory := filepath.Dir(path)
				if out == ".ast" {
					outFile = filepath.Join(inputPath, out, strings.ReplaceAll(directory, inputPath, ""), info.Name()+".json")
				} else {
					outFile = filepath.Join(out, strings.ReplaceAll(directory, inputPath, ""), info.Name()+".json")
				}
				writeFileContents(outFile, goastgen.ParseAstFromFile(path))
			}
			return nil
		})

		if err != nil {
			fmt.Printf("Error walking the path %s: %v\n", inputPath, err)
		}
	}
}

func parseArguments() (string, string) {
	var (
		out       string
		inputPath string = ""
	)
	flag.StringVar(&out, "out", ".ast", "Out put location of ast")
	flag.Parse()
	// Check if positional arguments exist
	if flag.NArg() > 0 {
		// Retrieve positional arguments
		inputPath = flag.Arg(0)
	}
	if inputPath == "" {
		fmt.Println("Usage:")
		fmt.Println("\tgoastgen [falgs] <source location>")
		fmt.Println()
		fmt.Println("Flags:")
		flag.PrintDefaults()
		os.Exit(1)
	}
	return out, inputPath
}

func writeFileContents(location string, contents string) {
	// Open the file for writing (creates a new file if it doesn't exist)
	dir := filepath.Dir(location)

	// Create all directories recursively
	err := os.MkdirAll(dir, 0755)
	if err != nil {
		fmt.Println("Failed to create file:", err)
		return
	}
	file, err := os.Create(location)
	if err != nil {
		fmt.Println("Failed to create file:", err)
		return
	}
	defer file.Close()

	// Write the contents to the file
	_, err = file.WriteString(contents)
	if err != nil {
		fmt.Println("Failed to write to file:", err)
		return
	}
}

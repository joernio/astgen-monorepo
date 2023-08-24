package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"privado.ai/goastgen/goastgen"
	"runtime"
	"strings"
)

var Version = "dev"

func main() {
	out, inputPath := parseArguments()
	processRequest(out, inputPath)
}

func processFile(out string, inputPath string, path string, info os.FileInfo, resultErr chan error, sem chan int) {
	sem <- 1
	defer func() {
		<-sem
	}()
	var outFile = ""
	var jsonResult string
	var err error
	directory := filepath.Dir(path)
	if out == ".ast" {
		outFile = filepath.Join(inputPath, out, strings.ReplaceAll(directory, inputPath, ""), info.Name()+".json")
	} else {
		outFile = filepath.Join(out, strings.ReplaceAll(directory, inputPath, ""), info.Name()+".json")
	}
	if strings.HasSuffix(info.Name(), ".go") {
		jsonResult, err = goastgen.ParseAstFromFile(path)
	} else if strings.HasSuffix(info.Name(), ".mod") {
		jsonResult, err = goastgen.ParseModFromFile(path)
	}
	if err != nil {
		fmt.Printf("Failed to generate AST for %s \n", path)
	} else {
		err = writeFileContents(outFile, jsonResult)
		if err != nil {
			fmt.Printf("Error writing AST to output location '%s'\n", outFile)
		} else {
			fmt.Printf("Converted AST for %s to %s \n", path, outFile)
		}
	}
	resultErr <- err
}

func processRequest(out string, inputPath string) {
	if strings.HasSuffix(inputPath, ".go") {
		fileInfo, err := os.Stat(inputPath)
		if err != nil {
			log.SetPrefix("[ERROR]")
			log.Println("Failed to get file info:", err)
			fmt.Printf("Error accessing path '%s'\n", inputPath)
			return
		}
		directory := filepath.Dir(inputPath)
		var outFile = ""
		if out == ".ast" {
			outFile = filepath.Join(directory, out, fileInfo.Name()+".json")
		} else {
			outFile = filepath.Join(out, fileInfo.Name()+".json")
		}
		jsonResult, perr := goastgen.ParseAstFromFile(inputPath)
		if perr != nil {
			fmt.Printf("Failed to generate AST for %s\n", inputPath)
			return
		} else {
			err = writeFileContents(outFile, jsonResult)
			if err != nil {
				fmt.Printf("Error writing AST to output location '%s'\n", outFile)
			} else {
				fmt.Printf("Converted AST for %s to %s\n", inputPath, outFile)
			}
			return
		}
	} else {
		concurrency := runtime.NumCPU()
		var successCount int = 0
		var failCount int = 0
		resultErrChan := make(chan error)
		sem := make(chan int, concurrency)
		var totalSentForProcessing = 0
		err := filepath.Walk(inputPath, func(path string, info os.FileInfo, err error) error {
			if err != nil {
				log.SetPrefix("[ERROR]")
				log.Printf("Error accessing path %s: %v\n", path, err)
				fmt.Printf("Error accessing path '%s'\n", path)
				return err
			}
			if !info.IsDir() && (strings.HasSuffix(info.Name(), ".go") || strings.HasSuffix(info.Name(), ".mod")) {
				totalSentForProcessing++
				go processFile(out, inputPath, path, info, resultErrChan, sem)
			}
			return nil
		})
		for i := 0; i < totalSentForProcessing; i++ {
			err = <-resultErrChan
			if err != nil {
				failCount++
			} else {
				successCount++
			}
		}

		//println("\n\n\n\n Without error -> ", successCount, ", With Error -> ", failCount)
		//println("total files sent for processing ----> ", totalSentForProcessing)
		//println("No of CPUs --->", concurrency)
		if err != nil {
			log.SetPrefix("[ERROR]")
			log.Printf("Error walking the path %s: %v\n", inputPath, err)
		}
	}
}

func parseArguments() (string, string) {
	var (
		out       string
		inputPath string = ""
		version   bool
		help      bool
	)
	flag.StringVar(&out, "out", ".ast", "Out put location of ast")
	flag.BoolVar(&version, "version", false, "print the version")
	flag.BoolVar(&help, "help", false, "print the usage")
	flag.Parse()
	if version {
		fmt.Println(Version)
		os.Exit(0)
	}
	// Check if positional arguments exist
	if flag.NArg() > 0 {
		// Retrieve positional arguments
		inputPath = flag.Arg(0)
	}
	if inputPath == "" || help {
		fmt.Println("Usage:")
		fmt.Println("\tgoastgen [falgs] <source location>")
		fmt.Println()
		fmt.Println("Flags:")
		flag.PrintDefaults()
		os.Exit(1)
	}
	return out, inputPath
}

func writeFileContents(location string, contents string) error {
	// Open the file for writing (creates a new file if it doesn't exist)
	dir := filepath.Dir(location)

	// Create all directories recursively
	err := os.MkdirAll(dir, 0755)
	if err != nil {
		log.SetPrefix("[ERROR]")
		log.Println("Failed to create file:", err)
		return err
	}
	file, err := os.Create(location)
	if err != nil {
		log.SetPrefix("[ERROR]")
		log.Println("Failed to create file:", err)
		return err
	}
	defer file.Close()

	// Write the contents to the file
	_, err = file.WriteString(contents)
	if err != nil {
		log.SetPrefix("[ERROR]")
		log.Println("Failed to write to file:", err)
		return err
	}
	return nil
}

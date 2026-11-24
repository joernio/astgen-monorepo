package main

import (
	"fmt"
	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"os"
	"path/filepath"
	"testing"
)

func TestProcessRequestWithSingleFileUseCase(t *testing.T) {
	// Get the temporary directory path
	tempDir := os.TempDir()

	// Create a new folder in the temporary directory
	newFolder := filepath.Join(tempDir, uuid.New().String())
	err := os.Mkdir(newFolder, 0755)
	if err != nil {
		fmt.Println("Failed to create folder:", err)
		return
	}
	srcFile := filepath.Join(newFolder, "hello.go")
	file, errf := os.Create(srcFile)
	if errf != nil {
		fmt.Println("Failed to create file:", errf)
		return
	}
	code := "package main \n" +
		"import \"fmt\"\n" +
		"func main() {\n" +
		"fmt.Println(\"Hello World\")\n" +
		"}"
	file.WriteString(code)
	processRequest(InputConfig{out: ".ast", inputPath: srcFile, includeFiles: "", excludeFiles: ""})
	expectedJsonFileLocation := filepath.Join(newFolder, ".ast", "hello.go.json")
	_, err = os.Stat(expectedJsonFileLocation)
	assert.Nil(t, err, "check the ast output is generated at expected location")

	diffOutLocation := filepath.Join(tempDir, uuid.New().String())
	processRequest(InputConfig{out: diffOutLocation, inputPath: srcFile, includeFiles: "", excludeFiles: ""})
	expectedJsonFileLocation = filepath.Join(diffOutLocation, "hello.go.json")
	_, err = os.Stat(expectedJsonFileLocation)
	assert.Nil(t, err, "check the ast output is generated at expected location")
}

func TestProcessRequestWithMultipleFileDiffFolderStructureUsecase(t *testing.T) {
	// Get the temporary directory path
	tempDir := os.TempDir()

	// Create a new folder in the temporary directory
	newFolder := filepath.Join(tempDir, uuid.New().String())
	err := os.Mkdir(newFolder, 0755)
	if err != nil {
		fmt.Println("Failed to create folder:", err)
		return
	}
	srcFile := filepath.Join(newFolder, "hello.go")
	file, errf := os.Create(srcFile)
	if errf != nil {
		fmt.Println("Failed to create file:", errf)
		return
	}
	code := "package main \n" +
		"import \"fmt\"\n" +
		"func main() {\n" +
		"fmt.Println(\"Hello World\")\n" +
		"}"
	file.WriteString(code)
	subDir := filepath.Join(newFolder, "subdir")
	err = os.Mkdir(subDir, 0755)
	if err != nil {
		fmt.Println("Failed to create folder:", err)
		return
	}
	subSrcFile := filepath.Join(subDir, "hellosub.go")
	file, errf = os.Create(subSrcFile)
	if errf != nil {
		fmt.Println("Failed to create file:", errf)
		return
	}
	file.WriteString(code)
	processRequest(InputConfig{out: ".ast", inputPath: newFolder, includeFiles: "", excludeFiles: ""})
	expectedJsonFileLocationone := filepath.Join(newFolder, ".ast", "hello.go.json")
	_, err = os.Stat(expectedJsonFileLocationone)
	assert.Nil(t, err, "check the ast output is generated at expected location")
	expectedJsonFileLocationtwo := filepath.Join(newFolder, ".ast", "subdir", "hellosub.go.json")
	_, err = os.Stat(expectedJsonFileLocationtwo)
	assert.Nil(t, err, "check the ast output is generated at expected location")

	diffOutLocation := filepath.Join(tempDir, uuid.New().String())
	processRequest(InputConfig{out: diffOutLocation, inputPath: newFolder, includeFiles: "", excludeFiles: ""})
	expectedJsonFileLocationone = filepath.Join(diffOutLocation, "hello.go.json")
	_, err = os.Stat(expectedJsonFileLocationone)
	assert.Nil(t, err, "check the ast output is generated at expected location")
	expectedJsonFileLocationtwo = filepath.Join(diffOutLocation, "subdir", "hellosub.go.json")
	_, err = os.Stat(expectedJsonFileLocationtwo)
	assert.Nil(t, err, "check the ast output is generated at expected location")
}

func TestWindowsGetIncludePackageFolders(t *testing.T) {
	// Get the temporary directory path
	tempDir := os.TempDir()
	// Create a new folder in the temporary directory
	newFolder := filepath.Join(tempDir, uuid.New().String())
	results := getIncludePackageFolders(InputConfig{out: ".ast", inputPath: newFolder, includeFiles: "", excludeFiles: "", includePackages: "/, /pkg, /cmd"})
	assert.Equal(t, 3, results.Size(), "result size as expected")
	assert.Equal(t, true, results.Contains(filepath.Join(newFolder, "")), "first result")
	assert.Equal(t, true, results.Contains(filepath.Join(newFolder, "pkg")), "second result")
	assert.Equal(t, true, results.Contains(filepath.Join(newFolder, "cmd")), "third result")

	results = getIncludePackageFolders(InputConfig{out: ".ast", inputPath: newFolder, includeFiles: "", excludeFiles: "", includePackages: "/, /pkg/, /cmd/"})
	assert.Equal(t, 3, results.Size(), "result size as expected")
	assert.Equal(t, true, results.Contains(filepath.Join(newFolder, "")), "first result")
	assert.Equal(t, true, results.Contains(filepath.Join(newFolder, "pkg")), "second result")
	assert.Equal(t, true, results.Contains(filepath.Join(newFolder, "cmd")), "third result")
}

func convertWithExclude(t *testing.T, excludeFiles string) (mainConverted bool, libConverted bool) {
	srcDir := filepath.Join(t.TempDir(), "m")
	assert.Nil(t, os.MkdirAll(filepath.Join(srcDir, "pkg", "a"), 0755))
	assert.Nil(t, os.WriteFile(filepath.Join(srcDir, "main.go"), []byte("package main\nfunc main() {}\n"), 0644))
	assert.Nil(t, os.WriteFile(filepath.Join(srcDir, "pkg", "a", "a.go"), []byte("package a\nfunc Foo() int { return 1 }\n"), 0644))
	outDir := t.TempDir()
	processRequest(InputConfig{out: outDir, inputPath: srcDir, includeFiles: "", excludeFiles: excludeFiles})
	_, mainErr := os.Stat(filepath.Join(outDir, "main.go.json"))
	_, libErr := os.Stat(filepath.Join(outDir, "pkg", "a", "a.go.json"))
	return mainErr == nil, libErr == nil
}

func TestExcludeFullyMatchingRelativePath(t *testing.T) {
	mainConverted, libConverted := convertWithExclude(t, `main\.go`)
	assert.False(t, mainConverted)
	assert.True(t, libConverted)

	mainConverted, libConverted = convertWithExclude(t, `pkg/a/.*`)
	assert.True(t, mainConverted)
	assert.False(t, libConverted)
}

func TestExcludeDoesNotMatchSubstring(t *testing.T) {
	mainConverted, libConverted := convertWithExclude(t, `ain`)
	assert.True(t, mainConverted)
	assert.True(t, libConverted)

	mainConverted, libConverted = convertWithExclude(t, `kg/a`)
	assert.True(t, mainConverted)
	assert.True(t, libConverted)
}

func TestExcludeDoesNotMatchFileNameOnly(t *testing.T) {
	mainConverted, libConverted := convertWithExclude(t, `a\.go`)
	assert.True(t, mainConverted)
	assert.True(t, libConverted)
}

func TestExcludeDoesNotMatchAbsolutePath(t *testing.T) {
	mainConverted, libConverted := convertWithExclude(t, `.*/main\.go`)
	assert.True(t, mainConverted)
	assert.True(t, libConverted)
}

func TestExcludeWithInvalidRegexExcludesNothing(t *testing.T) {
	mainConverted, libConverted := convertWithExclude(t, `(?!pkg/a/.*$).*\.go`)
	assert.True(t, mainConverted)
	assert.True(t, libConverted)
}

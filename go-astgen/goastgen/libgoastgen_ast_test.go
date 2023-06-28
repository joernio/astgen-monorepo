package goastgen

import (
	"fmt"
	"github.com/stretchr/testify/assert"
	"testing"
)

type RecursivePtrType struct {
	Id      int
	Name    string
	NodePtr interface{}
}

func TestRecursivePointerCheck(t *testing.T) {
	lastNodeId := 1
	var nodeAddressMap = make(map[uintptr]interface{})

	recursivePtrType := RecursivePtrType{Id: 10, Name: "Gajraj"}
	recursivePtrType.NodePtr = &recursivePtrType
	result := serilizeToMap(&recursivePtrType, nil, &lastNodeId, nodeAddressMap)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 10
	expectedResult["Name"] = "Gajraj"
	expectedResult["node_type"] = "goastgen.RecursivePtrType"
	expectedResult["node_id"] = 1
	expectedPtrResult := make(map[string]interface{})
	expectedPtrResult["node_type"] = "goastgen.RecursivePtrType"
	expectedPtrResult["node_id"] = 2
	expectedPtrResult["node_reference_id"] = 1
	expectedResult["NodePtr"] = expectedPtrResult

	assert.Equal(t, expectedResult, result, "Recursive type processed to map should match with expected result")
}

func TestFirst(t *testing.T) {
	code := "package main \n" +
		"import \"fmt\"\n" +
		"func main() {\n" +
		"fmt.Println(\"Hello World\")\n" +
		"}"

	result, _ := ParseAstFromSource("helloworld.go", code)
	fmt.Println(result)

}

package goastgen

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestArrayWithnillPointerCheck(t *testing.T) {
	lastNodeId := 1
	var nodeAddressMap = make(map[uintptr]interface{})

	var nilStr *string
	var nilObj *Phone
	var nilMap *map[string]Phone
	arrayWithnil := [4]interface{}{"valid string", nilStr, nilObj, nilMap}
	result := processArrayOrSlice(arrayWithnil, nil, &lastNodeId, nodeAddressMap)
	expectedResult := []interface{}{"valid string"}

	assert.Equal(t, expectedResult, result, "It should process valid values of the array successfully")
}

func TestSimpleInterfaceWithArray(t *testing.T) {
	lastNodeId := 1
	var nodeAddressMap = make(map[uintptr]interface{})

	arrayType := [2]interface{}{"first", "second"}
	result := processArrayOrSlice(arrayType, nil, &lastNodeId, nodeAddressMap)
	expectedResult := []interface{}{"first", "second"}
	assert.Equal(t, expectedResult, result, "Array of interface containing string pointers should match with expected results")
}

func TestSimpleInterfaceWithArrayOfPointersType(t *testing.T) {
	lastNodeId := 1
	var nodeAddressMap = make(map[uintptr]interface{})

	first := "first"
	second := "second"
	arrayType := [2]interface{}{&first, &second}
	result := processArrayOrSlice(arrayType, nil, &lastNodeId, nodeAddressMap)
	expectedResult := []interface{}{"first", "second"}
	assert.Equal(t, expectedResult, result, "Array of interface containing string pointers should match with expected results")
}

func TestObjectInterfaceWithArrayOfPointers(t *testing.T) {
	lastNodeId := 1
	var nodeAddressMap = make(map[uintptr]interface{})

	phone1 := Phone{PhoneNo: "1234567890", Type: "Home"}
	phone2 := Phone{PhoneNo: "0987654321", Type: "Office"}
	arrayType := [2]interface{}{&phone1, &phone2}
	result := processArrayOrSlice(arrayType, nil, &lastNodeId, nodeAddressMap)
	firstPhoneItem := make(map[string]interface{})
	firstPhoneItem["PhoneNo"] = "1234567890"
	firstPhoneItem["Type"] = "Home"
	firstPhoneItem["node_type"] = "goastgen.Phone"
	firstPhoneItem["node_id"] = 1
	secondPhoneItem := make(map[string]interface{})
	secondPhoneItem["PhoneNo"] = "0987654321"
	secondPhoneItem["Type"] = "Office"
	secondPhoneItem["node_type"] = "goastgen.Phone"
	secondPhoneItem["node_id"] = 2
	expectedResult := []interface{}{firstPhoneItem, secondPhoneItem}
	assert.Equal(t, expectedResult, result, "Simple Array type result should match with expected result Array")
}

func TestSliceObjctPtrType(t *testing.T) {
	lastNodeId := 1
	var nodeAddressMap = make(map[uintptr]interface{})

	phone1 := Phone{PhoneNo: "1234567890", Type: "Home"}
	phone2 := Phone{PhoneNo: "0987654321", Type: "Office"}
	objArrayType := SliceObjPtrType{Id: 20, PhoneList: []*Phone{&phone1, &phone2}}
	result := serilizeToMap(objArrayType, nil, &lastNodeId, nodeAddressMap)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 20
	expectedResult["node_type"] = "goastgen.SliceObjPtrType"
	expectedResult["node_id"] = 1
	firstPhoneItem := make(map[string]interface{})
	firstPhoneItem["PhoneNo"] = "1234567890"
	firstPhoneItem["Type"] = "Home"
	firstPhoneItem["node_type"] = "goastgen.Phone"
	firstPhoneItem["node_id"] = 2
	secondPhoneItem := make(map[string]interface{})
	secondPhoneItem["PhoneNo"] = "0987654321"
	secondPhoneItem["Type"] = "Office"
	secondPhoneItem["node_type"] = "goastgen.Phone"
	secondPhoneItem["node_id"] = 3
	expectedResult["PhoneList"] = []interface{}{firstPhoneItem, secondPhoneItem}

	assert.Equal(t, expectedResult, result, "Slice of Object pointers type result Map should match with expected result Map")
}

func TestArrayPtrType(t *testing.T) {
	lastNodeId := 1
	var nodeAddressMap = make(map[uintptr]interface{})

	firstStr := "First"
	secondStr := "Second"
	thirdStr := "Third"
	arrayType := ArrayPtrType{Id: 10, NameList: [3]*string{&firstStr, &secondStr, &thirdStr}}
	result := serilizeToMap(arrayType, nil, &lastNodeId, nodeAddressMap)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 10
	expectedResult["node_type"] = "goastgen.ArrayPtrType"
	expectedResult["NameList"] = []interface{}{firstStr, secondStr, thirdStr}
	expectedResult["node_id"] = 1

	assert.Equal(t, expectedResult, result, "Simple Array type result Map should match with expected result Map")
}

func TestObjectSliceType(t *testing.T) {
	lastNodeId := 1
	var nodeAddressMap = make(map[uintptr]interface{})

	objArrayType := ObjectSliceType{Id: 20, PhoneList: []Phone{{PhoneNo: "1234567890", Type: "Home"}, {PhoneNo: "0987654321", Type: "Office"}}}
	result := serilizeToMap(objArrayType, nil, &lastNodeId, nodeAddressMap)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 20
	expectedResult["node_type"] = "goastgen.ObjectSliceType"
	expectedResult["node_id"] = 1
	firstPhoneItem := make(map[string]interface{})
	firstPhoneItem["PhoneNo"] = "1234567890"
	firstPhoneItem["Type"] = "Home"
	firstPhoneItem["node_type"] = "goastgen.Phone"
	firstPhoneItem["node_id"] = 2
	secondPhoneItem := make(map[string]interface{})
	secondPhoneItem["PhoneNo"] = "0987654321"
	secondPhoneItem["Type"] = "Office"
	secondPhoneItem["node_type"] = "goastgen.Phone"
	secondPhoneItem["node_id"] = 3
	expectedResult["PhoneList"] = []interface{}{firstPhoneItem, secondPhoneItem}

	assert.Equal(t, expectedResult, result, "Simple Slice type result Map should match with expected result Map")
}

func TestArrayType(t *testing.T) {
	lastNodeId := 1
	var nodeAddressMap = make(map[uintptr]interface{})

	arrayType := ArrayType{Id: 10, NameList: [3]string{"First", "Second", "Third"}}
	result := serilizeToMap(arrayType, nil, &lastNodeId, nodeAddressMap)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 10
	expectedResult["NameList"] = []interface{}{"First", "Second", "Third"}
	expectedResult["node_type"] = "goastgen.ArrayType"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Simple Array type result Map should match with expected result Map")
}

func TestSliceType(t *testing.T) {
	lastNodeId := 1
	var nodeAddressMap = make(map[uintptr]interface{})
	arrayType := SliceType{Id: 10, NameList: []string{"First", "Second"}}
	result := serilizeToMap(arrayType, nil, &lastNodeId, nodeAddressMap)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 10
	expectedResult["NameList"] = []interface{}{"First", "Second"}
	expectedResult["node_type"] = "goastgen.SliceType"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Simple Slice type result Map should match with expected result Map")
}

func TestSimpleArrayType(t *testing.T) {
	lastNodeId := 1
	var nodeAddressMap = make(map[uintptr]interface{})

	phone1 := Phone{PhoneNo: "1234567890", Type: "Home"}
	phone2 := Phone{PhoneNo: "0987654321", Type: "Office"}
	simplePtrStr := "Simple PTR String"
	arrayType := []interface{}{&phone1, phone2, "Simple String", 90, &simplePtrStr}
	result := serilizeToMap(arrayType, nil, &lastNodeId, nodeAddressMap)

	firstPhone := make(map[string]interface{})
	firstPhone["PhoneNo"] = "1234567890"
	firstPhone["Type"] = "Home"
	firstPhone["node_type"] = "goastgen.Phone"
	firstPhone["node_id"] = 1
	secondPhone := make(map[string]interface{})
	secondPhone["PhoneNo"] = "0987654321"
	secondPhone["Type"] = "Office"
	secondPhone["node_type"] = "goastgen.Phone"
	secondPhone["node_id"] = 2

	expectedResult := []interface{}{firstPhone, secondPhone, "Simple String", 90, "Simple PTR String"}

	assert.Equal(t, expectedResult, result, "Array type with combination array elements should match with expected result")
}

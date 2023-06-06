package goastgen

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestArrayWithnillPointerCheck(t *testing.T) {
	var nilStr *string
	var nilObj *Phone
	var nilMap *map[string]Phone
	arrayWithnil := [4]interface{}{"valid string", nilStr, nilObj, nilMap}
	result := processArrayOrSlice(arrayWithnil)
	expectedResult := []interface{}{"valid string"}

	assert.Equal(t, expectedResult, result, "It should process valid values of the array successfully")
}

func TestSimpleInterfaceWithArray(t *testing.T) {
	arrayType := [2]interface{}{"first", "second"}
	result := processArrayOrSlice(arrayType)
	expectedResult := []interface{}{"first", "second"}
	assert.Equal(t, expectedResult, result, "Array of interface containing string pointers should match with expected results")
}

func TestSimpleInterfaceWithArrayOfPointersType(t *testing.T) {
	first := "first"
	second := "second"
	arrayType := [2]interface{}{&first, &second}
	result := processArrayOrSlice(arrayType)
	expectedResult := []interface{}{"first", "second"}
	assert.Equal(t, expectedResult, result, "Array of interface containing string pointers should match with expected results")
}

func TestObjectInterfaceWithArrayOfPointers(t *testing.T) {
	phone1 := Phone{PhoneNo: "1234567890", Type: "Home"}
	phone2 := Phone{PhoneNo: "0987654321", Type: "Office"}
	arrayType := [2]interface{}{&phone1, &phone2}
	result := processArrayOrSlice(arrayType)
	firstPhoneItem := make(map[string]interface{})
	firstPhoneItem["PhoneNo"] = "1234567890"
	firstPhoneItem["Type"] = "Home"

	secondPhoneItem := make(map[string]interface{})
	secondPhoneItem["PhoneNo"] = "0987654321"
	secondPhoneItem["Type"] = "Office"
	expectedResult := []interface{}{firstPhoneItem, secondPhoneItem}
	assert.Equal(t, expectedResult, result, "Simple Array type result should match with expected result Array")
}

func TestSliceObjctPtrType(t *testing.T) {
	phone1 := Phone{PhoneNo: "1234567890", Type: "Home"}
	phone2 := Phone{PhoneNo: "0987654321", Type: "Office"}
	objArrayType := SliceObjPtrType{Id: 20, PhoneList: []*Phone{&phone1, &phone2}}
	result := serilizeToMap(objArrayType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 20
	firstPhoneItem := make(map[string]interface{})
	firstPhoneItem["PhoneNo"] = "1234567890"
	firstPhoneItem["Type"] = "Home"

	secondPhoneItem := make(map[string]interface{})
	secondPhoneItem["PhoneNo"] = "0987654321"
	secondPhoneItem["Type"] = "Office"
	expectedResult["PhoneList"] = []interface{}{firstPhoneItem, secondPhoneItem}

	assert.Equal(t, expectedResult, result, "Slice of Object pointers type result Map should match with expected result Map")
}

func TestArrayPtrType(t *testing.T) {
	firstStr := "First"
	secondStr := "Second"
	thirdStr := "Third"
	arrayType := ArrayPtrType{Id: 10, NameList: [3]*string{&firstStr, &secondStr, &thirdStr}}
	result := serilizeToMap(arrayType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 10
	expectedResult["NameList"] = []interface{}{firstStr, secondStr, thirdStr}

	assert.Equal(t, expectedResult, result, "Simple Array type result Map should match with expected result Map")
	jsonResult := serilizeToJsonStr(result)
	expectedJsonResult := "{\n  \"Id\": 10,\n  \"NameList\": [\n    \"First\",\n    \"Second\",\n    \"Third\"\n  ]\n}"
	assert.Equal(t, expectedJsonResult, jsonResult, "Array of Pointer type result json should match with expected result")
}

func TestObjectSliceType(t *testing.T) {
	objArrayType := ObjectSliceType{Id: 20, PhoneList: []Phone{{PhoneNo: "1234567890", Type: "Home"}, {PhoneNo: "0987654321", Type: "Office"}}}
	result := serilizeToMap(objArrayType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 20
	firstPhoneItem := make(map[string]interface{})
	firstPhoneItem["PhoneNo"] = "1234567890"
	firstPhoneItem["Type"] = "Home"

	secondPhoneItem := make(map[string]interface{})
	secondPhoneItem["PhoneNo"] = "0987654321"
	secondPhoneItem["Type"] = "Office"
	expectedResult["PhoneList"] = []interface{}{firstPhoneItem, secondPhoneItem}

	assert.Equal(t, expectedResult, result, "Simple Slice type result Map should match with expected result Map")
}

func TestArrayType(t *testing.T) {
	arrayType := ArrayType{Id: 10, NameList: [3]string{"First", "Second", "Third"}}
	result := serilizeToMap(arrayType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 10
	expectedResult["NameList"] = []interface{}{"First", "Second", "Third"}

	assert.Equal(t, expectedResult, result, "Simple Array type result Map should match with expected result Map")
	jsonResult := serilizeToJsonStr(result)
	expectedJsonResult := "{\n  \"Id\": 10,\n  \"NameList\": [\n    \"First\",\n    \"Second\",\n    \"Third\"\n  ]\n}"
	assert.Equal(t, expectedJsonResult, jsonResult, "Simple Array type result json should match with expected result")
}

func TestSliceType(t *testing.T) {
	arrayType := SliceType{Id: 10, NameList: []string{"First", "Second"}}
	result := serilizeToMap(arrayType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 10
	expectedResult["NameList"] = []interface{}{"First", "Second"}

	assert.Equal(t, expectedResult, result, "Simple Slice type result Map should match with expected result Map")
}

func TestSimpleArrayType(t *testing.T) {
	phone1 := Phone{PhoneNo: "1234567890", Type: "Home"}
	phone2 := Phone{PhoneNo: "0987654321", Type: "Office"}
	simplePtrStr := "Simple PTR String"
	arrayType := []interface{}{&phone1, phone2, "Simple String", 90, &simplePtrStr}
	result := serilizeToMap(arrayType)

	firstPhone := make(map[string]interface{})
	firstPhone["PhoneNo"] = "1234567890"
	firstPhone["Type"] = "Home"
	secondPhone := make(map[string]interface{})
	secondPhone["PhoneNo"] = "0987654321"
	secondPhone["Type"] = "Office"

	expectedResult := []interface{}{firstPhone, secondPhone, "Simple String", 90, "Simple PTR String"}

	assert.Equal(t, expectedResult, result, "Array type with combination array elements should match with expected result")
}

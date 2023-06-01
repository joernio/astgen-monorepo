package goastgen

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

type Person struct {
	Name    string
	Address *Address
}

type Address struct {
	Addone string
	Addtwo *string
}

type Phone struct {
	Type    string
	PhoneNo string
}

type ObjectSliceType struct {
	Id        int
	PhoneList []Phone
}

type SliceType struct {
	Id       int
	NameList []string
}

type ArrayType struct {
	Id       int
	NameList [3]string
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

	assert.Equal(t, expectedResult, result, "Simple Array type result Map should match with expected result Map")
}

func TestArrayType(t *testing.T) {
	arrayType := ArrayType{Id: 10, NameList: [3]string{"First", "Second", "Third"}}
	result := serilizeToMap(arrayType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 10
	expectedResult["NameList"] = [3]string{"First", "Second", "Third"}

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
	expectedResult["NameList"] = []string{"First", "Second"}

	assert.Equal(t, expectedResult, result, "Simple Slice type result Map should match with expected result Map")
}

func TestSimpleTypeWithNullValue(t *testing.T) {
	address := Address{Addone: "First line address"}
	result := serilizeToMap(address)
	expectedResult := make(map[string]interface{})
	expectedResult["Addone"] = "First line address"

	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")

	phone := Phone{PhoneNo: "1234567890"}
	result = serilizeToMap(phone)
	expectedResult = make(map[string]interface{})
	expectedResult["PhoneNo"] = "1234567890"
	expectedResult["Type"] = ""

	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")
}

func TestSimpleType(t *testing.T) {
	phone := Phone{PhoneNo: "1234567890", Type: "Home"}
	result := serilizeToMap(phone)
	expectedResult := make(map[string]interface{})
	expectedResult["PhoneNo"] = "1234567890"
	expectedResult["Type"] = "Home"

	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")
}

func TestSimplePointerType(t *testing.T) {
	addtwo := "Second line address"
	var p *Address
	p = &Address{Addone: "First line address", Addtwo: &addtwo}
	result := serilizeToMap(p)
	expectedResult := make(map[string]interface{})
	expectedResult["Addone"] = "First line address"
	expectedResult["Addtwo"] = "Second line address"
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")

	jsonResult := serilizeToJsonStr(result)
	expectedJsonResult := "{\n  \"Addone\": \"First line address\",\n  \"Addtwo\": \"Second line address\"\n}"
	assert.Equal(t, expectedJsonResult, jsonResult, "Simple type result json should match with expected result")
}

func TestSecondLevelType(t *testing.T) {
	addtwo := "Second line address"
	var a *Address
	a = &Address{Addone: "First line address", Addtwo: &addtwo}

	var p *Person
	p = &Person{Name: "Sample Name", Address: a}
	result := serilizeToMap(p)
	expectedResult := make(map[string]interface{})
	expectedResult["Name"] = "Sample Name"
	addressResult := make(map[string]interface{})
	addressResult["Addone"] = "First line address"
	addressResult["Addtwo"] = "Second line address"
	expectedResult["Address"] = addressResult
	assert.Equal(t, expectedResult, result, "Second level type result Map should match with expected result map")

	jsonResult := serilizeToJsonStr(result)
	expectedJsonResult := "{\n  \"Address\": {\n    \"Addone\": \"First line address\",\n    \"Addtwo\": \"Second line address\"\n  },\n  \"Name\": \"Sample Name\"\n}"
	assert.Equal(t, expectedJsonResult, jsonResult, "Second level type result json should match with expected result")
}

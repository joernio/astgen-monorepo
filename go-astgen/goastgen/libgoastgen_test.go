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

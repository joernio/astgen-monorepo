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

type ArrayPtrType struct {
	Id       int
	NameList [3]*string
}

type SliceObjPtrType struct {
	Id        int
	PhoneList []*Phone
}

type MapObjType struct {
	Id     int
	Phones map[string]Phone
}

type MapIntType struct {
	Id    int
	Names map[string]int
}

type MapType struct {
	Id    int
	Names map[string]string
}

type MapStrPtrType struct {
	Id    int
	Names map[string]*string
}

type MapObjPtrType struct {
	Id     int
	Phones map[string]*Phone
}

func TestMapObjPtrType(t *testing.T) {
	first := Phone{PhoneNo: "1234567890", Type: "Home"}
	second := Phone{PhoneNo: "0987654321", Type: "Office"}
	phones := make(map[string]*Phone)
	phones["first"] = &first
	phones["second"] = &second

	mapType := MapObjPtrType{Id: 90, Phones: phones}
	result := serilizeToMap(mapType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 90
	expectedPhones := make(map[string]interface{})
	firstPhone := make(map[string]interface{})
	firstPhone["PhoneNo"] = "1234567890"
	firstPhone["Type"] = "Home"
	secondPhone := make(map[string]interface{})
	secondPhone["PhoneNo"] = "0987654321"
	secondPhone["Type"] = "Office"
	expectedPhones["first"] = firstPhone
	expectedPhones["second"] = secondPhone
	expectedResult["Phones"] = expectedPhones

	assert.Equal(t, expectedResult, result, "Map with Object Pointer type result Map should match with expected result Map")

	jsonResult := serilizeToJsonStr(result)
	expectedJsonResult := "{\n  \"Id\": 90,\n  \"Phones\": {\n    \"first\": {\n      \"PhoneNo\": \"1234567890\",\n      \"Type\": \"Home\"\n    },\n    \"second\": {\n      \"PhoneNo\": \"0987654321\",\n      \"Type\": \"Office\"\n    }\n  }\n}"
	assert.Equal(t, expectedJsonResult, jsonResult, "Map with Object type result json should match with expected result")
}

func TestMapStrPtrType(t *testing.T) {
	first := "firstvalue"
	second := "secondvalue"
	names := make(map[string]*string)
	names["firstname"] = &first
	names["secondname"] = &second
	mapType := MapStrPtrType{Id: 30, Names: names}
	result := serilizeToMap(mapType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 30
	expectedNames := make(map[string]interface{})
	expectedNames["firstname"] = "firstvalue"
	expectedNames["secondname"] = "secondvalue"
	expectedResult["Names"] = expectedNames
	assert.Equal(t, expectedResult, result, "Map String pointer type result Map should match with expected result Map")

	jsonResult := serilizeToJsonStr(result)
	expectedJsonResult := "{\n  \"Id\": 30,\n  \"Names\": {\n    \"firstname\": \"firstvalue\",\n    \"secondname\": \"secondvalue\"\n  }\n}"
	assert.Equal(t, expectedJsonResult, jsonResult, "Map with String pointer type result json should match with expected result")
}

func TestMapObjType(t *testing.T) {
	phones := make(map[string]Phone)
	phones["first"] = Phone{PhoneNo: "1234567890", Type: "Home"}
	phones["second"] = Phone{PhoneNo: "0987654321", Type: "Office"}

	mapType := MapObjType{Id: 90, Phones: phones}
	result := serilizeToMap(mapType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 90
	expectedPhones := make(map[string]interface{})
	firstPhone := make(map[string]interface{})
	firstPhone["PhoneNo"] = "1234567890"
	firstPhone["Type"] = "Home"
	secondPhone := make(map[string]interface{})
	secondPhone["PhoneNo"] = "0987654321"
	secondPhone["Type"] = "Office"
	expectedPhones["first"] = firstPhone
	expectedPhones["second"] = secondPhone
	expectedResult["Phones"] = expectedPhones

	assert.Equal(t, expectedResult, result, "Map with Object type result Map should match with expected result Map")

	jsonResult := serilizeToJsonStr(result)
	expectedJsonResult := "{\n  \"Id\": 90,\n  \"Phones\": {\n    \"first\": {\n      \"PhoneNo\": \"1234567890\",\n      \"Type\": \"Home\"\n    },\n    \"second\": {\n      \"PhoneNo\": \"0987654321\",\n      \"Type\": \"Office\"\n    }\n  }\n}"
	assert.Equal(t, expectedJsonResult, jsonResult, "Map with Object type result json should match with expected result")
}

func TestMapIntType(t *testing.T) {
	names := make(map[string]int)
	names["firstname"] = 1000
	names["secondname"] = 2000
	mapType := MapIntType{Id: 30, Names: names}
	result := serilizeToMap(mapType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 30
	expectedNames := make(map[string]int)
	expectedNames["firstname"] = 1000
	expectedNames["secondname"] = 2000
	expectedResult["Names"] = expectedNames
	assert.Equal(t, expectedResult, result, "Simple Map type result Map should match with expected result Map")

	jsonResult := serilizeToJsonStr(result)
	expectedJsonResult := "{\n  \"Id\": 30,\n  \"Names\": {\n    \"firstname\": 1000,\n    \"secondname\": 2000\n  }\n}"
	assert.Equal(t, expectedJsonResult, jsonResult, "Simple Map type result json should match with expected result")
}

func TestMapType(t *testing.T) {
	names := make(map[string]string)
	names["firstname"] = "firstvalue"
	names["secondname"] = "secondvalue"
	mapType := MapType{Id: 30, Names: names}
	result := serilizeToMap(mapType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 30
	expectedNames := make(map[string]string)
	expectedNames["firstname"] = "firstvalue"
	expectedNames["secondname"] = "secondvalue"
	expectedResult["Names"] = expectedNames
	assert.Equal(t, expectedResult, result, "Simple Map type result Map should match with expected result Map")
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

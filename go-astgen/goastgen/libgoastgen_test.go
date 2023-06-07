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

type InterfaceStrObjPtrType struct {
	Id    int
	Name  interface{}
	Phone interface{}
}

func TestInterfaceObjPtrType(t *testing.T) {
	phone := Phone{PhoneNo: "1234567890", Type: "Home"}
	interfaceObjPtrType := InterfaceStrObjPtrType{Id: 200, Phone: &phone}
	result := serilizeToMap(interfaceObjPtrType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 200
	expectedResult["node_type"] = "goastgen.InterfaceStrObjPtrType"
	phoneResult := make(map[string]interface{})
	phoneResult["PhoneNo"] = "1234567890"
	phoneResult["Type"] = "Home"
	phoneResult["node_type"] = "goastgen.Phone"
	expectedResult["Phone"] = phoneResult

	assert.Equal(t, expectedResult, result, "Struct type with interface{} containing pointer to object should match with expected result")
}

func TestInterfaceStrPtrType(t *testing.T) {
	sampleStr := "Sample"
	interfaceStrPtrType := InterfaceStrObjPtrType{Id: 100, Name: &sampleStr}
	result := serilizeToMap(interfaceStrPtrType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 100
	expectedResult["Name"] = "Sample"
	expectedResult["node_type"] = "goastgen.InterfaceStrObjPtrType"
	assert.Equal(t, expectedResult, result, "Struct type with interface{} containing pointer to string should match with expected result")
}

func TestObjectWithNullValueCheck(t *testing.T) {
	type SimpleObj struct {
		Id   int
		Name *string
	}

	simpleObj := SimpleObj{Id: 10}
	result := serilizeToMap(simpleObj)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 10
	expectedResult["node_type"] = "goastgen.SimpleObj"
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

	type SimpleObjObj struct {
		Id    int
		Phone *Phone
	}

	simpleObjObj := SimpleObjObj{Id: 20}
	result = serilizeToMap(simpleObjObj)
	expectedResult = make(map[string]interface{})
	expectedResult["Id"] = 20
	expectedResult["node_type"] = "goastgen.SimpleObjObj"
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

	type SimpleObjMap struct {
		Id       int
		Document *map[string]interface{}
	}

	simpleObjMap := SimpleObjObj{Id: 30}
	result = serilizeToMap(simpleObjMap)
	expectedResult = make(map[string]interface{})
	expectedResult["Id"] = 30
	expectedResult["node_type"] = "goastgen.SimpleObjObj"
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

	type SimpleObjArray struct {
		Id    int
		Array *[2]string
		Slice *[]string
	}

	simpleObjArray := SimpleObjArray{Id: 40}
	result = serilizeToMap(simpleObjArray)
	expectedResult = make(map[string]interface{})
	expectedResult["Id"] = 40
	expectedResult["node_type"] = "goastgen.SimpleObjArray"
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

}

func TestSimpleTypeWithNullValue(t *testing.T) {
	address := Address{Addone: "First line address"}
	result := serilizeToMap(address)
	expectedResult := make(map[string]interface{})
	expectedResult["Addone"] = "First line address"
	expectedResult["node_type"] = "goastgen.Address"
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")

	phone := Phone{PhoneNo: "1234567890"}
	result = serilizeToMap(phone)
	expectedResult = make(map[string]interface{})
	expectedResult["PhoneNo"] = "1234567890"
	expectedResult["Type"] = ""
	expectedResult["node_type"] = "goastgen.Phone"
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")
}

func TestSimpleType(t *testing.T) {
	phone := Phone{PhoneNo: "1234567890", Type: "Home"}
	result := serilizeToMap(phone)
	expectedResult := make(map[string]interface{})
	expectedResult["PhoneNo"] = "1234567890"
	expectedResult["Type"] = "Home"
	expectedResult["node_type"] = "goastgen.Phone"
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
	expectedResult["node_type"] = "goastgen.Address"
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")

	jsonResult := serilizeToJsonStr(result)
	expectedJsonResult := "{\n  \"Addone\": \"First line address\",\n  \"Addtwo\": \"Second line address\",\n  \"node_type\": \"goastgen.Address\"\n}"
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
	expectedResult["node_type"] = "goastgen.Person"
	addressResult := make(map[string]interface{})
	addressResult["Addone"] = "First line address"
	addressResult["Addtwo"] = "Second line address"
	addressResult["node_type"] = "goastgen.Address"
	expectedResult["Address"] = addressResult
	assert.Equal(t, expectedResult, result, "Second level type result Map should match with expected result map")

	jsonResult := serilizeToJsonStr(result)
	expectedJsonResult := "{\n  \"Address\": {\n    \"Addone\": \"First line address\",\n    \"Addtwo\": \"Second line address\",\n    \"node_type\": \"goastgen.Address\"\n  },\n  \"Name\": \"Sample Name\",\n  \"node_type\": \"goastgen.Person\"\n}"
	assert.Equal(t, expectedJsonResult, jsonResult, "Second level type result json should match with expected result")
}

func TestSimplePrimitive(t *testing.T) {
	result := serilizeToMap("Hello")
	assert.Equal(t, "Hello", result, "Simple string test should return same value")

	message := "Hello another message"
	result = serilizeToMap(&message)

	assert.Equal(t, "Hello another message", result, "Simple string pointer test should return same value string")
}

func TestSimpleNullCheck(t *testing.T) {
	var emptyStr string

	result := serilizeToMap(emptyStr)
	assert.Equal(t, "", result, "result should be empty string")

	var nilValue *string = nil
	nilResult := serilizeToMap(nilValue)
	assert.Nil(t, nilResult, "Null value should return null")

	var nillObj *Phone

	nilResult = serilizeToMap(nillObj)
	assert.Nil(t, nilResult, "Null object should return null")

	var nillMap *map[string]interface{}
	nilResult = serilizeToMap(nillMap)
	assert.Nil(t, nilResult, "Null map should return null")

	var nilSlice *[]string
	nilResult = serilizeToMap(nilSlice)
	assert.Nil(t, nilResult, "Null Slice should return null")

	var nilArray *[2]string
	nilResult = serilizeToMap(nilArray)
	assert.Nil(t, nilResult, "Null Array should return null")

}

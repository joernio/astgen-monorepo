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
	lastNodeId = 1

	phone := Phone{PhoneNo: "1234567890", Type: "Home"}
	interfaceObjPtrType := InterfaceStrObjPtrType{Id: 200, Phone: &phone}
	result := serilizeToMap(interfaceObjPtrType, nil)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 201
	expectedResult["node_type"] = "goastgen.InterfaceStrObjPtrType"
	expectedResult["node_id"] = 1
	phoneResult := make(map[string]interface{})
	phoneResult["PhoneNo"] = "1234567890"
	phoneResult["Type"] = "Home"
	phoneResult["node_type"] = "goastgen.Phone"
	phoneResult["node_id"] = 2
	expectedResult["Phone"] = phoneResult

	assert.Equal(t, expectedResult, result, "Struct type with interface{} containing pointer to object should match with expected result")
}

func TestInterfaceStrPtrType(t *testing.T) {
	lastNodeId = 1

	sampleStr := "Sample"
	interfaceStrPtrType := InterfaceStrObjPtrType{Id: 100, Name: &sampleStr}
	result := serilizeToMap(interfaceStrPtrType, nil)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 100
	expectedResult["Name"] = "Sample"
	expectedResult["node_type"] = "goastgen.InterfaceStrObjPtrType"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Struct type with interface{} containing pointer to string should match with expected result")
}

func TestObjectWithNullValueCheck(t *testing.T) {
	lastNodeId = 1

	type SimpleObj struct {
		Id   int
		Name *string
	}

	simpleObj := SimpleObj{Id: 10}
	result := serilizeToMap(simpleObj, nil)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 10
	expectedResult["node_type"] = "goastgen.SimpleObj"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

	lastNodeId = 1

	type SimpleObjObj struct {
		Id    int
		Phone *Phone
	}

	simpleObjObj := SimpleObjObj{Id: 20}
	result = serilizeToMap(simpleObjObj, nil)
	expectedResult = make(map[string]interface{})
	expectedResult["Id"] = 20
	expectedResult["node_type"] = "goastgen.SimpleObjObj"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

	lastNodeId = 1
	type SimpleObjMap struct {
		Id       int
		Document *map[string]interface{}
	}

	simpleObjMap := SimpleObjObj{Id: 30}
	result = serilizeToMap(simpleObjMap, nil)
	expectedResult = make(map[string]interface{})
	expectedResult["Id"] = 30
	expectedResult["node_type"] = "goastgen.SimpleObjObj"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

	lastNodeId = 1
	type SimpleObjArray struct {
		Id    int
		Array *[2]string
		Slice *[]string
	}

	simpleObjArray := SimpleObjArray{Id: 40}
	result = serilizeToMap(simpleObjArray, nil)
	expectedResult = make(map[string]interface{})
	expectedResult["Id"] = 40
	expectedResult["node_type"] = "goastgen.SimpleObjArray"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

}

func TestSimpleTypeWithNullValue(t *testing.T) {
	lastNodeId = 1

	address := Address{Addone: "First line address"}
	result := serilizeToMap(address, nil)
	expectedResult := make(map[string]interface{})
	expectedResult["Addone"] = "First line address"
	expectedResult["node_type"] = "goastgen.Address"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")

	lastNodeId = 1
	phone := Phone{PhoneNo: "1234567890"}
	result = serilizeToMap(phone, nil)
	expectedResult = make(map[string]interface{})
	expectedResult["PhoneNo"] = "1234567890"
	expectedResult["Type"] = ""
	expectedResult["node_type"] = "goastgen.Phone"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")
}

func TestSimpleType(t *testing.T) {
	lastNodeId = 1

	phone := Phone{PhoneNo: "1234567890", Type: "Home"}
	result := serilizeToMap(phone, nil)
	expectedResult := make(map[string]interface{})
	expectedResult["PhoneNo"] = "1234567890"
	expectedResult["Type"] = "Home"
	expectedResult["node_type"] = "goastgen.Phone"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")
}

func TestSimplePointerType(t *testing.T) {
	lastNodeId = 1

	addtwo := "Second line address"
	var p *Address
	p = &Address{Addone: "First line address", Addtwo: &addtwo}
	result := serilizeToMap(p, nil)
	expectedResult := make(map[string]interface{})
	expectedResult["Addone"] = "First line address"
	expectedResult["Addtwo"] = "Second line address"
	expectedResult["node_type"] = "goastgen.Address"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")

}

func TestSecondLevelType(t *testing.T) {
	lastNodeId = 1

	addtwo := "Second line address"
	var a *Address
	a = &Address{Addone: "First line address", Addtwo: &addtwo}

	var p *Person
	p = &Person{Name: "Sample Name", Address: a}
	result := serilizeToMap(p, nil)
	expectedResult := make(map[string]interface{})
	expectedResult["Name"] = "Sample Name"
	expectedResult["node_type"] = "goastgen.Person"
	expectedResult["node_id"] = 1
	addressResult := make(map[string]interface{})
	addressResult["Addone"] = "First line address"
	addressResult["Addtwo"] = "Second line address"
	addressResult["node_type"] = "goastgen.Address"
	addressResult["node_id"] = 2
	expectedResult["Address"] = addressResult
	assert.Equal(t, expectedResult, result, "Second level type result Map should match with expected result map")

}

func TestSimplePrimitive(t *testing.T) {
	result := serilizeToMap("Hello", nil)
	assert.Equal(t, "Hello", result, "Simple string test should return same value")

	message := "Hello another message"
	result = serilizeToMap(&message, nil)

	assert.Equal(t, "Hello another message", result, "Simple string pointer test should return same value string")
}

func TestSimpleNullCheck(t *testing.T) {
	var emptyStr string

	result := serilizeToMap(emptyStr, nil)
	assert.Equal(t, "", result, "result should be empty string")

	var nilValue *string = nil
	nilResult := serilizeToMap(nilValue, nil)
	assert.Nil(t, nilResult, "Null value should return null")

	var nillObj *Phone

	nilResult = serilizeToMap(nillObj, nil)
	assert.Nil(t, nilResult, "Null object should return null")

	var nillMap *map[string]interface{}
	nilResult = serilizeToMap(nillMap, nil)
	assert.Nil(t, nilResult, "Null map should return null")

	var nilSlice *[]string
	nilResult = serilizeToMap(nilSlice, nil)
	assert.Nil(t, nilResult, "Null Slice should return null")

	var nilArray *[2]string
	nilResult = serilizeToMap(nilArray, nil)
	assert.Nil(t, nilResult, "Null Array should return null")

}

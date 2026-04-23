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
	var goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	phone := Phone{PhoneNo: "1234567890", Type: "Home"}
	interfaceObjPtrType := InterfaceStrObjPtrType{Id: 200, Phone: &phone}
	result := goFile.serilizeToMap(interfaceObjPtrType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 200
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
	var goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	sampleStr := "Sample"
	interfaceStrPtrType := InterfaceStrObjPtrType{Id: 100, Name: &sampleStr}
	result := goFile.serilizeToMap(interfaceStrPtrType)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 100
	expectedResult["Name"] = "Sample"
	expectedResult["node_type"] = "goastgen.InterfaceStrObjPtrType"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Struct type with interface{} containing pointer to string should match with expected result")
}

func TestObjectWithNullValueCheck(t *testing.T) {
	var goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	type SimpleObj struct {
		Id   int
		Name *string
	}

	simpleObj := SimpleObj{Id: 10}
	result := goFile.serilizeToMap(simpleObj)
	expectedResult := make(map[string]interface{})
	expectedResult["Id"] = 10
	expectedResult["node_type"] = "goastgen.SimpleObj"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

	goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	type SimpleObjObj struct {
		Id    int
		Phone *Phone
	}

	simpleObjObj := SimpleObjObj{Id: 20}
	result = goFile.serilizeToMap(simpleObjObj)
	expectedResult = make(map[string]interface{})
	expectedResult["Id"] = 20
	expectedResult["node_type"] = "goastgen.SimpleObjObj"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

	goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	type SimpleObjMap struct {
		Id       int
		Document *map[string]interface{}
	}

	simpleObjMap := SimpleObjObj{Id: 30}
	result = goFile.serilizeToMap(simpleObjMap)
	expectedResult = make(map[string]interface{})
	expectedResult["Id"] = 30
	expectedResult["node_type"] = "goastgen.SimpleObjObj"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

	goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	type SimpleObjArray struct {
		Id    int
		Array *[2]string
		Slice *[]string
	}

	simpleObjArray := SimpleObjArray{Id: 40}
	result = goFile.serilizeToMap(simpleObjArray)
	expectedResult = make(map[string]interface{})
	expectedResult["Id"] = 40
	expectedResult["node_type"] = "goastgen.SimpleObjArray"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "It should not process those fields which contains nil pointer, rest of the fields should be processed")

}

func TestSimpleTypeWithNullValue(t *testing.T) {
	var goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	address := Address{Addone: "First line address"}
	result := goFile.serilizeToMap(address)
	expectedResult := make(map[string]interface{})
	expectedResult["Addone"] = "First line address"
	expectedResult["node_type"] = "goastgen.Address"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")

	goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	phone := Phone{PhoneNo: "1234567890"}
	result = goFile.serilizeToMap(phone)
	expectedResult = make(map[string]interface{})
	expectedResult["PhoneNo"] = "1234567890"
	expectedResult["Type"] = ""
	expectedResult["node_type"] = "goastgen.Phone"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")
}

func TestSimpleType(t *testing.T) {
	var goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	phone := Phone{PhoneNo: "1234567890", Type: "Home"}
	result := goFile.serilizeToMap(phone)
	expectedResult := make(map[string]interface{})
	expectedResult["PhoneNo"] = "1234567890"
	expectedResult["Type"] = "Home"
	expectedResult["node_type"] = "goastgen.Phone"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")
}

func TestSimplePointerType(t *testing.T) {
	var goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	addtwo := "Second line address"
	var p *Address
	p = &Address{Addone: "First line address", Addtwo: &addtwo}
	result := goFile.serilizeToMap(p)
	expectedResult := make(map[string]interface{})
	expectedResult["Addone"] = "First line address"
	expectedResult["Addtwo"] = "Second line address"
	expectedResult["node_type"] = "goastgen.Address"
	expectedResult["node_id"] = 1
	assert.Equal(t, expectedResult, result, "Simple type result Map should match with expected result Map")

}

func TestSecondLevelType(t *testing.T) {
	var goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	addtwo := "Second line address"
	var a *Address
	a = &Address{Addone: "First line address", Addtwo: &addtwo}

	var p *Person
	p = &Person{Name: "Sample Name", Address: a}
	result := goFile.serilizeToMap(p)
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
	var goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	result := goFile.serilizeToMap("Hello")
	assert.Equal(t, "Hello", result, "Simple string test should return same value")

	message := "Hello another message"
	result = goFile.serilizeToMap(&message)

	assert.Equal(t, "Hello another message", result, "Simple string pointer test should return same value string")
}

func TestSimpleNullCheck(t *testing.T) {
	var goFile = GoFile{File: "", lastNodeId: 1, nodeAddressMap: make(map[uintptr]interface{})}
	var emptyStr string

	result := goFile.serilizeToMap(emptyStr)
	assert.Equal(t, "", result, "result should be empty string")

	var nilValue *string = nil
	nilResult := goFile.serilizeToMap(nilValue)
	assert.Nil(t, nilResult, "Null value should return null")

	var nillObj *Phone

	nilResult = goFile.serilizeToMap(nillObj)
	assert.Nil(t, nilResult, "Null object should return null")

	var nillMap *map[string]interface{}
	nilResult = goFile.serilizeToMap(nillMap)
	assert.Nil(t, nilResult, "Null map should return null")

	var nilSlice *[]string
	nilResult = goFile.serilizeToMap(nilSlice)
	assert.Nil(t, nilResult, "Null Slice should return null")

	var nilArray *[2]string
	nilResult = goFile.serilizeToMap(nilArray)
	assert.Nil(t, nilResult, "Null Array should return null")

}

package goastgen

import "C"
import (
	"encoding/json"
	"fmt"
	"go/parser"
	"go/token"
	"log"
	"reflect"
)

//export ExternallyCalled
func ExternallyCalled() *C.char {
	result := "John"
	return C.CString(result)
}

//export Add
func Add(a int, b int) int {
	return a + b
}

func astParser() {
	fset := token.NewFileSet()
	//file, err := parser.ParseDir(fset, "/Users/pandurang/projects/golang/helloworld/", nil, 0)
	file, err := parser.ParseFile(fset, "/Users/pandurang/projects/golang/helloworld/hello.go", nil, 0)
	if err != nil {
		log.Fatal(err)
	}

	result := serilizeToMap(file)
	resultJson := serilizeToJsonStr(result)
	fmt.Println(resultJson)
}

func serilizeToJsonStr(objectMap map[string]interface{}) string {
	jsonStr, _ := json.MarshalIndent(objectMap, "", "  ")
	return string(jsonStr)
}

func processMap(object interface{}) interface{} {
	var result interface{}
	value := reflect.ValueOf(object)
	mapValueTypeKind := value.Type().Elem().Kind()
	switch mapValueTypeKind {
	case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		result = object
	case reflect.Struct:
		objMap := make(map[string]interface{})
		for _, key := range value.MapKeys() {
			objMap[key.String()] = serilizeToMap(value.MapIndex(key).Interface())
		}
		result = objMap
	case reflect.Pointer:
		mapValuePtrKind := value.Type().Elem().Elem().Kind()
		switch mapValuePtrKind {
		case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
			objMap := make(map[string]interface{})
			for _, key := range value.MapKeys() {
				objMap[key.String()] = value.MapIndex(key).Elem().Interface()
			}
			result = objMap
		case reflect.Struct:
			objMap := make(map[string]interface{})
			for _, key := range value.MapKeys() {
				objMap[key.String()] = serilizeToMap(value.MapIndex(key).Elem().Interface())
			}
			result = objMap
		}
	}
	return result
}

func processArrayOrSlice(object interface{}) interface{} {
	value := reflect.ValueOf(object)
	var nodeList []interface{}
	for j := 0; j < value.Len(); j++ {
		arrayElementValue := value.Index(j)
		elementKind := arrayElementValue.Kind()
		if elementKind == reflect.Interface {
			arrayElementValue = arrayElementValue.Elem()
			elementKind = arrayElementValue.Kind()
		}
		switch elementKind {
		case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
			nodeList = append(nodeList, arrayElementValue.Interface())
		case reflect.Struct:
			nodeList = append(nodeList, serilizeToMap(arrayElementValue.Interface()))
		case reflect.Map:
			nodeList = append(nodeList, processMap(arrayElementValue.Interface()))
		case reflect.Pointer:
			arrayElementValuePtrKind := arrayElementValue.Elem().Kind()
			switch arrayElementValuePtrKind {
			case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
				nodeList = append(nodeList, arrayElementValue.Elem().Interface())
			case reflect.Struct:
				nodeList = append(nodeList, serilizeToMap(arrayElementValue.Elem().Interface()))
			case reflect.Map:
				nodeList = append(nodeList, processMap(arrayElementValue.Elem().Interface()))
			}
		}
	}
	return nodeList
}

func serilizeToMap(node interface{}) map[string]interface{} {
	objectMap := make(map[string]interface{})
	var elementType reflect.Type
	var elementValueObj reflect.Value

	pointerType := reflect.TypeOf(node)
	if pointerType.Kind() == reflect.Pointer {
		// NOTE: This handles only one level of pointer. At this moment we don't expect to get pointer to pointer.
		elementValueObj = reflect.ValueOf(node).Elem()
		elementType = pointerType.Elem()
	} else {
		elementValueObj = reflect.ValueOf(node)
		elementType = pointerType
	}

	for i := 0; i < elementType.NumField(); i++ {
		field := elementType.Field(i)
		value := elementValueObj.Field(i)
		fieldKind := value.Type().Kind()

		if fieldKind == reflect.Pointer {
			// NOTE: This handles only one level of pointer. At this moment we don't expect to get pointer to pointer.
			fieldKind = value.Type().Elem().Kind()
			value = value.Elem()
		}
		switch fieldKind {
		case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
			if value.IsValid() {
				objectMap[field.Name] = value.Interface()
			}
		case reflect.Struct:
			if value.IsValid() {
				objectMap[field.Name] = serilizeToMap(value.Interface())
			}
		case reflect.Map:
			if value.IsValid() {
				objectMap[field.Name] = processMap(value.Interface())
			}
		case reflect.Array, reflect.Slice:
			if value.IsValid() {
				objectMap[field.Name] = processArrayOrSlice(value.Interface())
			}
		}
	}
	return objectMap
}

func main() {

}

// build

//  go build -buildmode=c-shared -o lib-sample.dylib sample.go

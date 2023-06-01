package goastgen

import "C"
import (
	"encoding/json"
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

func serilizeToJsonStr(objectMap map[string]interface{}) string {
	jsonStr, _ := json.MarshalIndent(objectMap, "", "  ")
	return string(jsonStr)
}

func serilizeToMap(node interface{}) map[string]interface{} {
	objectMap := make(map[string]interface{})
	var elementType reflect.Type
	var elementValueObj reflect.Value

	pointerType := reflect.TypeOf(node)
	if pointerType.Kind() == reflect.Ptr {
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
		fieldKind := field.Type.Kind()

		if fieldKind == reflect.Ptr {
			// NOTE: This handles only one level of pointer. At this moment we don't expect to get pointer to pointer.
			fieldKind = field.Type.Elem().Kind()
			value = value.Elem()
		}
		switch fieldKind {
		case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
			if value.IsValid() {
				objectMap[field.Name] = value.Interface()
			}
		case reflect.Struct:
			objectMap[field.Name] = serilizeToMap(value.Interface())
		case reflect.Array, reflect.Slice:
			objectMap[field.Name] = value.Interface()
		}
	}
	return objectMap
}

func main() {

}

// build

//  go build -buildmode=c-shared -o lib-sample.dylib sample.go

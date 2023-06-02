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
			objectMap[field.Name] = serilizeToMap(value.Interface())
		case reflect.Map:
			mapValueTypeKind := value.Type().Elem().Kind()
			switch mapValueTypeKind {
			case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
				objectMap[field.Name] = value.Interface()
			case reflect.Struct:
				objMap := make(map[string]interface{})
				for _, key := range value.MapKeys() {
					objMap[key.String()] = serilizeToMap(value.MapIndex(key).Interface())
				}
				objectMap[field.Name] = objMap
			}

		case reflect.Array, reflect.Slice:
			arrayValueTypeKind := value.Type().Elem().Kind()
			switch arrayValueTypeKind {
			case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
				objectMap[field.Name] = value.Interface()
			case reflect.Struct:
				var nodeList []interface{}
				for j := 0; j < value.Len(); j++ {
					fieldArrayNode := value.Index(j).Interface()
					nodeList = append(nodeList, serilizeToMap(fieldArrayNode))
				}
				objectMap[field.Name] = nodeList
			case reflect.Pointer:
				arrayValuePtrKind := value.Type().Elem().Elem().Kind()
				switch arrayValuePtrKind {
				case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
					// Create an array of type int with a length of 5
					arrayType := reflect.SliceOf(value.Type().Elem().Elem())
					arrayValue := reflect.MakeSlice(arrayType, value.Len(), value.Len())
					for j := 0; j < value.Len(); j++ {
						arrayValue.Index(j).Set(value.Index(j).Elem())
					}
					objectMap[field.Name] = arrayValue.Interface()
				case reflect.Struct:
					var nodeList []interface{}
					for j := 0; j < value.Len(); j++ {
						fieldArrayNode := value.Index(j).Elem().Interface()
						nodeList = append(nodeList, serilizeToMap(fieldArrayNode))
					}
					objectMap[field.Name] = nodeList
				}
			}
		}
	}
	return objectMap
}

func main() {

}

// build

//  go build -buildmode=c-shared -o lib-sample.dylib sample.go

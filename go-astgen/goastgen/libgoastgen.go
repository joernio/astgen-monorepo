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
	file, err := parser.ParseDir(fset, "/Users/pandurang/projects/golang/helloworld/", nil, 0)
	//file, err := parser.ParseFile(fset, "/Users/pandurang/projects/golang/helloworld/hello.go", nil, 0)
	if err != nil {
		log.Fatal(err)
	}
	result := serilizeToMap(file)
	resultJson := serilizeToJsonStr(result)
	fmt.Println(resultJson)
}

func serilizeToJsonStr(objectMap interface{}) string {
	jsonStr, _ := json.MarshalIndent(objectMap, "", "  ")
	return string(jsonStr)
}

/**
  Process Map type objects. In order to process the contents of the map's value object.
  If the value object is of type 'struct' then we are converting it to map[string]interface{} and using it.
*/
func processMap(object interface{}) interface{} {
	value := reflect.ValueOf(object)
	objMap := make(map[string]interface{})
	for _, key := range value.MapKeys() {
		objValue := value.MapIndex(key)
		// Checking the reflect.Kind of value object and if its pointer
		// then fetching the reflect.Value of the object pointed to by this pointer
		if objValue.Kind() == reflect.Pointer {
			objValue = objValue.Elem()
		}
		if objValue.IsValid() {
			switch objValue.Kind() {
			case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
				objMap[key.String()] = objValue.Interface()
			case reflect.Struct:
				objMap[key.String()] = processStruct(objValue.Interface())
			default:
				log.SetPrefix("[WARNING]")
				log.Println(getLogPrefix(), objValue.Kind(), "- not handled")
			}
		}
	}
	return objMap
}

/**
  This will process the Array or Slice (Dynamic Array).
  It will identify the type/reflect.Kind of each array element and process the array element according.
*/
func processArrayOrSlice(object interface{}) interface{} {
	value := reflect.ValueOf(object)
	var nodeList []interface{}
	for j := 0; j < value.Len(); j++ {
		arrayElementValue := value.Index(j)
		elementKind := arrayElementValue.Kind()
		// If you create an array interface{} and assign pointer as elements into this array.
		// when we try to identify the reflect.Kind of such element it will be of type reflect.Interface.
		// In such case we need to call .elem() to fetch the original reflect.Value of the array element.
		// Refer test case - TestSimpleInterfaceWithArrayOfPointersType for the same.
		if elementKind == reflect.Interface {
			arrayElementValue = arrayElementValue.Elem()
			elementKind = arrayElementValue.Kind()
		}
		switch elementKind {
		case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
			nodeList = append(nodeList, arrayElementValue.Interface())
		case reflect.Struct:
			nodeList = append(nodeList, processStruct(arrayElementValue.Interface()))
		case reflect.Map:
			nodeList = append(nodeList, processMap(arrayElementValue.Interface()))
		case reflect.Pointer:
			if arrayElementValue.Elem().IsValid() {
				arrayElementValuePtrKind := arrayElementValue.Elem().Kind()
				switch arrayElementValuePtrKind {
				case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
					nodeList = append(nodeList, arrayElementValue.Elem().Interface())
				case reflect.Struct:
					nodeList = append(nodeList, processStruct(arrayElementValue.Elem().Interface()))
				case reflect.Map:
					nodeList = append(nodeList, processMap(arrayElementValue.Elem().Interface()))
				default:
					log.SetPrefix("[WARNING]")
					log.Println(getLogPrefix(), arrayElementValuePtrKind, "- not handled for array pointer element")
				}
			}
		default:
			log.SetPrefix("[WARNING]")
			log.Println(getLogPrefix(), elementKind, "- not handled for array element")
		}
	}
	return nodeList
}

/**
  This will process object of 'struct' type and convert it into document / map[string]interface{}.
  It will process each field of this object, if it contains further child objects, arrays or maps.
  Then it will get those respective field objects processed through respective processors.
  e.g. if the field object is of type 'struct' then it will call function processStruct recursively
*/
func processStruct(node interface{}) interface{} {
	objectMap := make(map[string]interface{})
	var elementType reflect.Type
	var elementValueObj reflect.Value

	pointerType := reflect.TypeOf(node)

	// If the first object itself is the pointer then get the underlying object 'Value' and process it.
	if pointerType.Kind() == reflect.Pointer {
		// NOTE: This handles only one level of pointer. At this moment we don't expect to get pointer to pointer.
		//This will get 'reflect.Value' object pointed to by this pointer.
		elementValueObj = reflect.ValueOf(node).Elem()
		//This will get 'reflect.Type' object pointed to by this pointer
		elementType = pointerType.Elem()
	} else {
		elementValueObj = reflect.ValueOf(node)
		elementType = pointerType
	}

	// We will iterate through each field process each field according to its reflect.Kind type.
	for i := 0; i < elementType.NumField(); i++ {
		field := elementType.Field(i)
		value := elementValueObj.Field(i)
		fieldKind := value.Type().Kind()

		if fieldKind == reflect.Pointer {
			// NOTE: This handles only one level of pointer. At this moment we don't expect to get pointer to pointer.

			// This will fetch the reflect.Kind of object pointed to by this field pointer
			fieldKind = value.Type().Elem().Kind()
			// This will fetch the reflect.Value of object pointed to by this field pointer.
			value = value.Elem()
		}
		switch fieldKind {
		case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
			if value.IsValid() {
				objectMap[field.Name] = value.Interface()
			}
		case reflect.Struct:
			if value.IsValid() {
				objectMap[field.Name] = processStruct(value.Interface())
			}
		case reflect.Map:
			if value.IsValid() {
				objectMap[field.Name] = processMap(value.Interface())
			}
		case reflect.Array, reflect.Slice:
			if value.IsValid() {
				objectMap[field.Name] = processArrayOrSlice(value.Interface())
			}
		default:
			log.SetPrefix("[WARNING]")
			log.Println(getLogPrefix(), field.Name, "- of Kind ->", fieldKind, "- not handled")
		}
	}
	return objectMap
}

/*
 First step to convert the given object to Map, in order to export into JSON format.

 This function will check if the given passed object is of primitive, struct, map, array or slice (Dynamic array) type
 and process object accordingly to convert the same to map[string]interface

 In case the object itself is of primitive data type, it will not convert it to map, rather it will just return the same object as is.

 So possible return value types could be primitive type, map (map[string]interface{}) or slice ([]interface{})

*/
func serilizeToMap(node interface{}) interface{} {
	var elementType reflect.Type
	var elementValue reflect.Value
	nodeType := reflect.TypeOf(node)
	nodeValue := reflect.ValueOf(node)

	// If the first object itself is the pointer then get the underlying object 'Value' and process it.
	if nodeType.Kind() == reflect.Pointer {
		// NOTE: This handles only one level of pointer. At this moment we don't expect to get pointer to pointer.
		elementType = nodeType.Elem()
		elementValue = nodeValue.Elem()
	} else {
		elementType = nodeType
		elementValue = nodeValue
	}
	switch elementType.Kind() {
	case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		if elementValue.IsValid() {
			return elementValue.Interface()
		}
		return nil
	case reflect.Struct:
		if elementValue.IsValid() {
			return processStruct(elementValue.Interface())
		}
		return nil
	case reflect.Map:
		if elementValue.IsValid() {
			return processMap(elementValue.Interface())
		}
		return nil
	case reflect.Array, reflect.Slice:
		if elementValue.IsValid() {
			return processArrayOrSlice(elementValue.Interface())
		}
		return nil
	default:
		log.SetPrefix("[WARNING]")
		log.Println(getLogPrefix(), elementType.Kind(), " - not handled")
		return elementValue.Interface()
	}
}

func main() {

}

// build

//  go build -buildmode=c-shared -o lib-sample.dylib sample.go

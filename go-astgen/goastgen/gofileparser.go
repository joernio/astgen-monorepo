package goastgen

import (
	"go/ast"
	"go/parser"
	"go/token"
	"log"
	"reflect"
	"unsafe"
)

type GoFile struct {
	File string
	// Last node id reference
	lastNodeId int
	//fset: *token.FileSet - As this library is primarily designed to generate AST JSON. This parameter facilitate adding line, column no and File details to the node.
	fset *token.FileSet
	// We maintain the cache of processed object pointers mapped to their respective node_id
	nodeAddressMap map[uintptr]interface{}
}

//Parse
/*
 It will parse the given File and generate AST in JSON format

 Parameters:
  File: absolute File path to be parsed

 Returns:
  If given File is a valid go code then it will generate AST in JSON format otherwise will return "" string.
*/
func (goFile *GoFile) Parse() (string, error) {
	goFile.fset = token.NewFileSet()
	goFile.lastNodeId = 1
	goFile.nodeAddressMap = make(map[uintptr]interface{})
	// NOTE: Haven't explore much of mode parameter. Default value has been passed as 0
	parsedAst, err := parser.ParseFile(goFile.fset, goFile.File, nil, 0)
	if err != nil {
		log.SetPrefix("[ERROR]")
		log.Println("Error while parsing source File -> '", goFile.File, ",")
		log.Print(err)
		return "", err
	}
	// We maintain the cache of processed object pointers mapped to their respective node_id
	// Last node id reference
	result := goFile.serilizeToMap(parsedAst)
	return serilizeToJsonStr(result)
}

// ParseAstFromSource
/*
 It will parse given source code and generate AST in JSON format

 Parameters:
  filename: Filename used for generating AST metadata
  src: string, []byte, or io.Reader - Source code

 Returns:
  If given source is valid go source then it will generate AST in JSON format other will return "" string.
*/
func (goFile *GoFile) ParseAstFromSource(src any) (string, error) {
	goFile.fset = token.NewFileSet()
	goFile.lastNodeId = 1
	goFile.nodeAddressMap = make(map[uintptr]interface{})
	parsedAst, err := parser.ParseFile(goFile.fset, goFile.File, src, 0)
	if err != nil {
		// TODO: convert this to just warning error log.
		log.SetPrefix("[ERROR]")
		log.Println("Error while parsing source from source File -> '", goFile.File, "'")
		log.Print(err)
		return "", err
	}
	result := goFile.serilizeToMap(parsedAst)
	return serilizeToJsonStr(result)
}

/*
 First step to convert the given object to Map, in order to export into JSON format.

 This function will check if the given passed object is of primitive, struct, map, array or slice (Dynamic array) type
 and process object accordingly to convert the same to map[string]interface

 In case the object itself is of primitive data type, it will not convert it to map, rather it will just return the same object as is.

 Parameters:
  node: any object
  fset: *token.FileSet - As this library is primarily designed to generate AST JSON. This parameter facilitate adding line, column no and File details to the node.

 Returns:
  possible return value types could be primitive type, map (map[string]interface{}) or slice ([]interface{})

*/
func (goFile *GoFile) serilizeToMap(node interface{}) interface{} {
	var elementType reflect.Type
	var elementValue reflect.Value
	var ptrValue reflect.Value
	nodeType := reflect.TypeOf(node)
	nodeValue := reflect.ValueOf(node)
	// If the first object itself is the pointer then get the underlying object 'Value' and process it.
	if nodeType.Kind() == reflect.Pointer {
		// NOTE: This handles only one level of pointer. At this moment we don't expect to get pointer to pointer.
		//This will get 'reflect.Value' object pointed to by this pointer.
		elementType = nodeType.Elem()
		//This will get 'reflect.Type' object pointed to by this pointer
		elementValue = nodeValue.Elem()
		ptrValue = nodeValue
	} else {
		elementType = nodeType
		elementValue = nodeValue
	}

	// In case the node is pointer, it will check if given Value contains valid pointer address.
	if elementValue.IsValid() {
		switch elementType.Kind() {
		case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
			return elementValue.Interface()
		case reflect.Struct:
			return goFile.processStruct(elementValue.Interface(), ptrValue)
		case reflect.Map:
			return goFile.processMap(elementValue.Interface())
		case reflect.Array, reflect.Slice:
			return goFile.processArrayOrSlice(elementValue.Interface())
		default:
			log.SetPrefix("[WARNING]")
			log.Println(getLogPrefix(), elementType.Kind(), " - not handled")
			return elementValue.Interface()
		}
	}
	return nil
}

/*
 This will process object of 'struct' type and convert it into document / map[string]interface{}.
 It will process each field of this object, if it contains further child objects, arrays or maps.
 Then it will get those respective field objects processed through respective processors.
 e.g. if the field object is of type 'struct' then it will call function processStruct recursively

 Parameters:
  node: Object of struct
  objPtrValue: reflect.Value - As we cannot get the pointer information from reflect.Value object.
               If its a pointer that is getting processed, the caller will pass the reflect.Value of pointer.
               So that it can be used for checking the cache if the given object pointed by the same pointer is already processed or not.

 Returns:
  It will return object of map[string]interface{} by converting all the child fields recursively into map

*/
func (goFile *GoFile) processStruct(node interface{}, objPtrValue reflect.Value) interface{} {
	objectMap := make(map[string]interface{})
	elementType := reflect.TypeOf(node)
	elementValueObj := reflect.ValueOf(node)

	process := true
	var objAddress uintptr
	// We are checking if the given object is already processed.
	// We are doing that by maintaining map of processed object pointers set with node_id.
	// If object is already processed then we will not process it again.
	// Instead we wil just add its node_id as reference id

	// NOTE: Important point to understand we are not maintaining every object in this cache.
	// We are only maintaining those objects which are referenced as a pointer. In that case objPtrValue.Kind() will be of reflect.Pointer type
	if objPtrValue.Kind() == reflect.Pointer {
		ptr := unsafe.Pointer(objPtrValue.Pointer()) // Get the pointer address as an unsafe.Pointer
		objAddress = uintptr(ptr)                    // Convert unsafe.Pointer to uintptr
		refNodeId, ok := goFile.nodeAddressMap[objAddress]
		if ok {
			process = false
			//if the given object is already processed, then we are adding its respective node_id as a reference_id in this node.
			objectMap["node_reference_id"] = refNodeId
		}
		// Reading and setting column no, line no and File details.
		if astNode, ok := objPtrValue.Interface().(ast.Node); ok && goFile.fset != nil {
			if pos := astNode.Pos(); pos.IsValid() {
				position := goFile.fset.Position(pos)
				//Add File information only inside ast.File node which is the root node for a File AST.
				if elementValueObj.Type().String() == "ast.File" {
					objectMap["node_filename"] = position.Filename
				}
				objectMap["node_line_no"] = position.Line
				objectMap["node_col_no"] = position.Column
			}
			if epos := astNode.End(); epos.IsValid() {
				position := goFile.fset.Position(epos)
				objectMap["node_line_no_end"] = position.Line
				objectMap["node_col_no_end"] = position.Column
			}
		}
	}

	objectMap["node_id"] = goFile.lastNodeId
	goFile.lastNodeId++
	objectMap["node_type"] = elementValueObj.Type().String()

	if process {
		if objPtrValue.Kind() == reflect.Pointer {
			goFile.nodeAddressMap[objAddress] = objectMap["node_id"]
		}
		// We will iterate through each field process each field according to its reflect.Kind type.
		for i := 0; i < elementType.NumField(); i++ {
			field := elementType.Field(i)

			value := elementValueObj.Field(i)
			fieldKind := value.Type().Kind()

			// If object is defined with field type as interface{} and assigned with pointer value.
			// We need to first fetch the element from the interface
			if fieldKind == reflect.Interface {
				fieldKind = value.Elem().Kind()
				value = value.Elem()
			}

			var ptrValue reflect.Value

			if fieldKind == reflect.Pointer {
				// NOTE: This handles only one level of pointer. At this moment we don't expect to get pointer to pointer.
				// This will fetch the reflect.Kind of object pointed to by this field pointer
				fieldKind = value.Type().Elem().Kind()
				// This will fetch the reflect.Value of object pointed to by this field pointer.
				ptrValue = value
				// capturing the reflect.Value of the pointer if it's a pointer to be passed to recursive processStruct method.
				value = value.Elem()
			}
			// In case the node is pointer, it will check if given Value contains valid pointer address.
			if value.IsValid() {
				switch fieldKind {
				case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
					if value.Type().String() == "token.Token" {
						objectMap[field.Name] = value.Interface().(token.Token).String()
					} else {
						objectMap[field.Name] = value.Interface()
					}
				case reflect.Struct:
					objectMap[field.Name] = goFile.processStruct(value.Interface(), ptrValue)
				case reflect.Map:
					objectMap[field.Name] = goFile.processMap(value.Interface())
				case reflect.Array, reflect.Slice:
					objectMap[field.Name] = goFile.processArrayOrSlice(value.Interface())
				default:
					log.SetPrefix("[WARNING]")
					log.Println(getLogPrefix(), field.Name, "- of Kind ->", fieldKind, "- not handled")
				}
			}
		}
	}
	return objectMap
}

/*
 This will process the Array or Slice (Dynamic Array).
 It will identify the type/reflect.Kind of each array element and process the array element according.

 Parameters:
  object: []interface{} - expected to pass object of Array or Slice

 Returns:
  It will return []map[string]interface{}
*/
func (goFile *GoFile) processArrayOrSlice(object interface{}) interface{} {
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
		ptrValue := arrayElementValue

		switch elementKind {
		case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
			nodeList = append(nodeList, arrayElementValue.Interface())
		case reflect.Struct:
			nodeList = append(nodeList, goFile.processStruct(arrayElementValue.Interface(), ptrValue))
		case reflect.Map:
			nodeList = append(nodeList, goFile.processMap(arrayElementValue.Interface()))
		case reflect.Pointer:
			// In case the node is pointer, it will check if given Value contains valid pointer address.
			if arrayElementValue.Elem().IsValid() {
				arrayElementValuePtrKind := arrayElementValue.Elem().Kind()
				switch arrayElementValuePtrKind {
				case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
					nodeList = append(nodeList, arrayElementValue.Elem().Interface())
				case reflect.Struct:
					nodeList = append(nodeList, goFile.processStruct(arrayElementValue.Elem().Interface(), ptrValue))
				case reflect.Map:
					nodeList = append(nodeList, goFile.processMap(arrayElementValue.Elem().Interface()))
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

/*
Process Map type objects. In order to process the contents of the map's value object.
If the value object is of type 'struct' then we are converting it to map[string]interface{} and using it.

Parameters:
 object: expects map[string] any


Returns:
 It returns and object of map[string]interface{} by converting any 'Struct' type value field to map
*/
func (goFile *GoFile) processMap(object interface{}) interface{} {
	value := reflect.ValueOf(object)
	objMap := make(map[string]interface{})
	for _, key := range value.MapKeys() {
		objValue := value.MapIndex(key)

		// If the map is created to accept valye of any type i.e. map[string]interface{}.
		// Then it's value's reflect.Kind is of type reflect.Interface.
		// We need to fetch original objects reflect.Value by calling .Elem() on it.
		if objValue.Kind() == reflect.Interface {
			objValue = objValue.Elem()
		}

		var ptrValue reflect.Value
		// Checking the reflect.Kind of value object and if its pointer
		// then fetching the reflect.Value of the object pointed to by this pointer
		if objValue.Kind() == reflect.Pointer {
			objValue = objValue.Elem()
			ptrValue = objValue
		}

		if objValue.IsValid() {
			switch objValue.Kind() {
			case reflect.String, reflect.Int, reflect.Bool, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Float32, reflect.Float64, reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
				objMap[key.String()] = objValue.Interface()
			case reflect.Struct:
				objMap[key.String()] = goFile.processStruct(objValue.Interface(), ptrValue)
			default:
				log.SetPrefix("[WARNING]")
				log.Println(getLogPrefix(), objValue.Kind(), "- not handled")
			}
		}
	}
	return objMap
}

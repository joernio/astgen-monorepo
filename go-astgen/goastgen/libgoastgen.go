package goastgen

import (
	"encoding/json"
	"log"
)

/*
 Independent function which handles serialisation of map[string]interface{} in to JSON

 Parameters:
  objectMap: Mostly it will be object of map[string]interface{}

 Returns:
  JSON string
*/
func serilizeToJsonStr(objectMap interface{}) (string, error) {
	jsonStr, err := json.MarshalIndent(objectMap, "", "  ")
	if err != nil {
		log.SetPrefix("[ERROR]")
		log.Println("Error while generating the AST JSON")
		log.Println(err)
		return "", err
	}
	return string(jsonStr), nil
}

package goastgen

import (
	"golang.org/x/mod/modfile"
	"io/ioutil"
	"log"
)

type ModFile struct {
	File string
}

//Parse
/*
 It will parse the .mod File and generate module and dependency information in JSON format

 Parameters:
  File: absolute File path to be parsed

 Returns:
  If given File is a valid .mod File then it will generate the module and dependency information in JSON format
*/
func (mod *ModFile) Parse() (string, error) {
	objMap := make(map[string]interface{})
	contents, err := ioutil.ReadFile(mod.File)
	if err != nil {
		log.SetPrefix("[ERROR]")
		log.Printf("Error while processing '%s' \n", mod.File)
		log.Println(err)
		return "", err
	}
	modFile, err := modfile.Parse(mod.File, contents, nil)
	if err != nil {
		log.SetPrefix("[ERROR]")
		log.Printf("Error while processing '%s' \n", mod.File)
		log.Println(err)
		return "", err
	}
	objMap["node_filename"] = mod.File
	module := make(map[string]interface{})
	if modFile.Module != nil {
		module["Name"] = modFile.Module.Mod.Path
		module["node_line_no"] = modFile.Module.Syntax.Start.Line
		module["node_col_no"] = modFile.Module.Syntax.Start.LineRune
		module["node_line_no_end"] = modFile.Module.Syntax.End.Line
		module["node_col_no_end"] = modFile.Module.Syntax.End.LineRune
		module["node_type"] = "mod.Module"
	} else {
		module["Name"] = mod.File
		module["node_line_no"] = 0
		module["node_col_no"] = 0
		module["node_line_no_end"] = 0
		module["node_col_no_end"] = 0
		module["node_type"] = "mod.Module"
	}
	objMap["Module"] = module
	dependencies := []interface{}{}
	for _, req := range modFile.Require {
		node := make(map[string]interface{})
		node["Module"] = req.Mod.Path
		node["Version"] = req.Mod.Version
		node["Indirect"] = req.Indirect
		node["node_line_no"] = req.Syntax.Start.Line
		node["node_col_no"] = req.Syntax.Start.LineRune
		node["node_line_no_end"] = req.Syntax.End.Line
		node["node_col_no_end"] = req.Syntax.End.LineRune
		node["node_type"] = "mod.Dependency"
		dependencies = append(dependencies, node)
	}
	objMap["dependencies"] = dependencies
	return serilizeToJsonStr(objMap)
}

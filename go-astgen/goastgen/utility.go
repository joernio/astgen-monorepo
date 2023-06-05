package goastgen

import "runtime"

// getLogPrefix returns a formatted string with the method name
func getLogPrefix() string {
	pc, _, _, _ := runtime.Caller(1)
	method := runtime.FuncForPC(pc).Name()
	return "[" + method + "]"
}

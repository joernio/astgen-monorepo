package goastgen

import "runtime"

// StringSet is a custom type based on map to implement set functionality.
type StringSet map[string]struct{}

// Add adds an element to the set.
func (s StringSet) Add(element string) {
	s[element] = struct{}{}
}

// Contains checks if an element is in the set.
func (s StringSet) Contains(element string) bool {
	_, exists := s[element]
	return exists
}

// Remove removes an element from the set (optional).
func (s StringSet) Remove(element string) {
	delete(s, element)
}

func (s StringSet) Size() int {
	return len(s)
}

// getLogPrefix returns a formatted string with the method name
func getLogPrefix() string {
	pc, _, _, _ := runtime.Caller(1)
	method := runtime.FuncForPC(pc).Name()
	return "[" + method + "]"
}

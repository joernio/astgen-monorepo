package goastgen

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

type Person struct {
	Name string
	Age  *int
}

func TestSimpleType(t *testing.T) {
	age := 30
	var p *Person
	p = &Person{Name: "John", Age: &age}
	result := serilizeToJson(p)
	expectedResult := "{\n  \"Age\": 30,\n  \"Name\": \"John\"\n}"
	assert.Equal(t, expectedResult, result, "The two words should be the same.")
}

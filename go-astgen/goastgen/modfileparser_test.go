package goastgen

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
)

// go.mod files written by newer Go toolchains carry directives that goastgen
// does not need (godebug since Go 1.23, tool since 1.24, ignore since 1.25).
// They must not make the whole go.mod unparseable.
func TestModFileWithNewerDirectives(t *testing.T) {
	dir := t.TempDir()
	modPath := filepath.Join(dir, "go.mod")
	content := `module example.com/app

go 1.25.0

toolchain go1.25.1

godebug default=go1.21

require github.com/google/uuid v1.3.0

tool golang.org/x/tools/cmd/stringer

ignore ./web/node_modules
`
	assert.Nil(t, os.WriteFile(modPath, []byte(content), 0o644))

	mod := ModFile{File: modPath}
	jsonStr, err := mod.Parse()
	assert.Nil(t, err)

	var result map[string]interface{}
	assert.Nil(t, json.Unmarshal([]byte(jsonStr), &result))
	module := result["Module"].(map[string]interface{})
	assert.Equal(t, "example.com/app", module["Name"])
	deps := result["dependencies"].([]interface{})
	assert.Equal(t, 1, len(deps))
	dep := deps[0].(map[string]interface{})
	assert.Equal(t, "github.com/google/uuid", dep["Module"])
	assert.Equal(t, "v1.3.0", dep["Version"])
}

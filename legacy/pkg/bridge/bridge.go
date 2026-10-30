package bridge

import (
	"github.com/ecocee/kode-go/internal/parser"
	"github.com/ecocee/kode-go/pkg/ast"
)

// ParseSource parses Kode source code and returns the AST statements
func ParseSource(filePath string, sourceCode string) ([]ast.Statement, error) {
	p, err := parser.NewParser(filePath, sourceCode)
	if err != nil {
		return nil, err
	}
	return p.Parse()
}

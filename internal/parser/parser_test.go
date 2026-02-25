package parser

import (
	"testing"

	"github.com/ecocee/kode-go/pkg/ast"
)

func TestParser(t *testing.T) {
	tests := []struct {
		input    string
		checkAST func([]ast.Statement) bool
	}{
		{
			input: `x := 42
y := x + 1`,
			checkAST: func(stmts []ast.Statement) bool {
				if len(stmts) != 2 {
					return false
				}
				// Check first statement
				if let1, ok := stmts[0].(ast.LetStmt); !ok || let1.Name != "x" || let1.Type != nil {
					return false
				}
				// Check second statement
				if let2, ok := stmts[1].(ast.LetStmt); !ok || let2.Name != "y" || let2.Type != nil {
					return false
				}
				return true
			},
		},
	}

	for _, test := range tests {
		parser, err := NewParser("test.kode", test.input)
		if err != nil {
			t.Errorf("Parser creation error: %v", err)
			continue
		}

		stmts, err := parser.Parse()
		if err != nil {
			t.Errorf("Parse error: %v", err)
			continue
		}

		if !test.checkAST(stmts) {
			t.Errorf("AST check failed for input: %s", test.input)
		}
	}
}

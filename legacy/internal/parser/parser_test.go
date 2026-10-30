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
				if let1, ok := stmts[0].(ast.LetStmt); !ok || let1.Name != "x" {
					return false
				}
				if let2, ok := stmts[1].(ast.LetStmt); !ok || let2.Name != "y" {
					return false
				}
				return true
			},
		},
		// arithmetic precedence
		{
			input: `let z = 1;`,
			checkAST: func(stmts []ast.Statement) bool {
				if len(stmts) != 1 {
					return false
				}
				if let1, ok := stmts[0].(ast.LetStmt); !ok || let1.Name != "z" {
					return false
				}
				return true
			},
		},
		// if statement
		{
			input: `if (a > b) { let x = 1; } else { let x = 2; }`,
			checkAST: func(stmts []ast.Statement) bool {
				if len(stmts) != 1 {
					return false
				}
				if _, ok := stmts[0].(ast.IfStmt); !ok {
					return false
				}
				return true
			},
		},
		// function definition
		{
			input: `func add(a: int, b: int) int { return a + b; }`,
			checkAST: func(stmts []ast.Statement) bool {
				if len(stmts) != 1 {
					return false
				}
				if fn, ok := stmts[0].(ast.FunctionDefStmt); !ok || fn.Name != "add" {
					return false
				}
				return true
			},
		},
		// nested blocks
		{
			input: `{
    let a = 1;
    {
        let b = 2;
    }
}`,
			checkAST: func(stmts []ast.Statement) bool { return true },
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
			t.Errorf("Parse error for %q: %v", test.input, err)
			continue
		}

		if !test.checkAST(stmts) {
			t.Errorf("AST check failed for input: %s", test.input)
		}
	}
}

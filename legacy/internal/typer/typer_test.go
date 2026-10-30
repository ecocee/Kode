package typer

import (
	"testing"

	"github.com/ecocee/kode-go/internal/lexer"
	"github.com/ecocee/kode-go/internal/parser"
	"github.com/ecocee/kode-go/pkg/ast"
)

func TestTyper(t *testing.T) {
	tests := []struct {
		input string
		valid bool
	}{
		{input: "x := 42", valid: true},
		{input: `x := 42
y := x + 1`, valid: true},
		{input: "let x: int = true", valid: false},
		{input: "let x = 3.14\nx = x + 1.0", valid: true},
		// named function declaration
		{input: "func f(a: int) int { return a + 1 }\nlet y = f(5)", valid: true},
		// anonymous function literals are not yet supported and thus treated
		// as syntax/typing errors; we avoid testing them here.
		{input: "if (true) { let x = 1 }", valid: true},
		{input: "if (1) { }", valid: false},
		// current typer is static; reassignment to different type should error
		{input: "let a = 1\na = \"no\"", valid: false},
	}

	for _, test := range tests {
		p, err := parser.NewParser("test.kode", test.input)
		if err != nil {
			t.Errorf("Parser creation error for %q: %v", test.input, err)
			continue
		}

		stmts, err := p.Parse()
		if err != nil {
			// print tokens for debugging
			if lex, lexErr := lexer.NewLexer(test.input).Tokenize(); lexErr == nil {
				t.Logf("tokens: %v", lex)
			}
			t.Errorf("Parse error for %q: %v", test.input, err)
			continue
		}

		program := ast.Program{Statements: stmts}
		typer := NewTyper()
		err = typer.CheckProgram(program)

		if test.valid && err != nil {
			t.Errorf("Expected valid, got error: %v", err)
		} else if !test.valid && err == nil {
			t.Errorf("Expected invalid for %q, but no error", test.input)
		}
	}
}

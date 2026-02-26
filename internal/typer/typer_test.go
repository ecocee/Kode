package typer

import (
	"testing"

	"github.com/ecocee/kode-go/internal/parser"
	"github.com/ecocee/kode-go/pkg/ast"
)

func TestTyper(t *testing.T) {
	tests := []struct {
		input string
		valid bool
	}{
		{
			input: "x := 42",
			valid: true,
		},
		{
			input: `x := 42
y := x + 1`,
			valid: true,
		},
		{
			input: "x: int = true", // Type mismatch
			valid: false,
		},
	}

	for _, test := range tests {
		p, err := parser.NewParser("test.kode", test.input)
		if err != nil {
			t.Errorf("Parser error: %v", err)
			continue
		}

		stmts, err := p.Parse()
		if err != nil {
			t.Errorf("Parse error: %v", err)
			continue
		}

		program := ast.Program{Statements: stmts}
		typer := NewTyper()
		err = typer.CheckProgram(program)

		if test.valid && err != nil {
			t.Errorf("Expected valid, got error: %v", err)
		} else if !test.valid && err == nil {
			t.Errorf("Expected invalid, but no error")
		}
	}
}

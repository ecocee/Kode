package compiler_test

import (
	"testing"

	"github.com/ecocee/kode-go/internal/compiler"
	"github.com/ecocee/kode-go/internal/parser"
	"github.com/ecocee/kode-go/pkg/ast"
)

func compileProgram(t *testing.T, src string) *compiler.Compiler {
	p, err := parser.NewParser("test.kode", src)
	if err != nil {
		t.Fatalf("parser error: %v", err)
	}
	stmts, err := p.Parse()
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}
	c := compiler.NewCompiler()
	_, err = c.Compile(ast.Program{Statements: stmts})
	if err != nil {
		t.Fatalf("compile error: %v", err)
	}
	return c
}

func TestCompilerGlobalsAndFunctions(t *testing.T) {
	src := `let a = 1
let b = 2
func add(x: int, y: int) int { return x + y }
let c = add(a, b)`
	compileProgram(t, src)
}

func TestCompilerArithmeticOrder(t *testing.T) {
	src := `let x = 1 + 2 * 3` // ensures precedence
	compileProgram(t, src)
}

func TestCompilerControlFlow(t *testing.T) {
	src := `func main() {
    if (true) { print(1) } else { print(2) }
}`
	compileProgram(t, src)
}

func TestCompilerFailure(t *testing.T) {
	src := `let x = 1 + "hi"` // should fail typing
	p, _ := parser.NewParser("test.kode", src)
	stmts, _ := p.Parse()
	c := compiler.NewCompiler()
	if _, err := c.Compile(ast.Program{Statements: stmts}); err == nil {
		t.Errorf("expected compile error for mismatched types")
	}
}

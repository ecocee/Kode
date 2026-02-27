package runtime_test

import (
	"bytes"
	"fmt"
	"os"
	"testing"

	"github.com/ecocee/kode-go/internal/compiler"
	"github.com/ecocee/kode-go/internal/parser"
	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/runtime"
)

// captureStdout temporarily redirects os.Stdout and returns whatever was written.
func captureStdout(f func()) string {
	orig := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	f()

	w.Close()
	var buf bytes.Buffer
	_, _ = buf.ReadFrom(r)
	os.Stdout = orig
	return buf.String()
}

func TestGlobalLetAndPrint(t *testing.T) {
	src := "let a = 1\nlet b = 2\nprint(a + b)"
	p, err := parser.NewParser("test.kode", src)
	if err != nil {
		t.Fatal(err)
	}
	stmts, err := p.Parse()
	if err != nil {
		t.Fatal(err)
	}
	c := compiler.NewCompiler()
	ir, err := c.Compile(ast.Program{Statements: stmts})
	if err != nil {
		t.Fatal(err)
	}
	rt := runtime.NewRuntime()
	out := captureStdout(func() {
		if err := rt.Execute(ir, "test.kode"); err != nil {
			t.Fatal(err)
		}
	})
	if out != "3\n" {
		t.Fatalf("expected 3\n, got %q", out)
	}
}

func TestIfElseAndLoop(t *testing.T) {
	old := runtime.VerboseRuntime
	runtime.VerboseRuntime = false
	defer func() { runtime.VerboseRuntime = old }()
	src := `let sum = 0
let i = 0
while (i < 5) {
    sum = sum + i
    i = i + 1
}
if (sum == 10) { print("ok") } else { print("bad") }`
	p, _ := parser.NewParser("test.kode", src)
	stmts, _ := p.Parse()
	c := compiler.NewCompiler()
	ir, err := c.Compile(ast.Program{Statements: stmts})
	if err != nil {
		t.Fatal(err)
	}
	if runtime.VerboseRuntime {
		fmt.Printf("IR: %+v\n", ir)
	}
	rt := runtime.NewRuntime()
	out := captureStdout(func() { rt.Execute(ir, "test.kode") })
	if out != "ok\n" {
		t.Fatalf("expected ok, got %q", out)
	}
}

func TestRecursion(t *testing.T) {
	old := runtime.VerboseRuntime
	runtime.VerboseRuntime = false
	defer func() { runtime.VerboseRuntime = old }()
	src := `func fact(n: int) int {
    if (n <= 1) { return 1 }
    return n * fact(n - 1)
}
print(fact(5))`
	p, _ := parser.NewParser("test.kode", src)
	stmts, _ := p.Parse()
	c := compiler.NewCompiler()
	ir, err := c.Compile(ast.Program{Statements: stmts})
	if err != nil {
		t.Fatal(err)
	}
	if runtime.VerboseRuntime {
		fmt.Printf("IR: %+v\n", ir)
	}
	rt := runtime.NewRuntime()
	out := captureStdout(func() { rt.Execute(ir, "test.kode") })
	if out != "120\n" {
		t.Fatalf("expected 120, got %q", out)
	}
}

func TestRuntimeError(t *testing.T) {
	src := `print(1 / 0)`
	p, _ := parser.NewParser("test.kode", src)
	stmts, _ := p.Parse()
	c := compiler.NewCompiler()
	ir, err := c.Compile(ast.Program{Statements: stmts})
	if err != nil {
		t.Fatal(err)
	}
	rt := runtime.NewRuntime()
	err = rt.Execute(ir, "test.kode")
	if err == nil {
		t.Fatalf("expected runtime error, got nil")
	}
}

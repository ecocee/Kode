package codegen

import (
	"fmt"
	"strings"

	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/ir"
)

// GoCodeGenerator generates Go code from Kode IR
type GoCodeGenerator struct {
	ir         *ir.IR
	sourceFile string
}

// NewGoCodeGenerator creates a new Go code generator
func NewGoCodeGenerator(irProgram *ir.IR) *GoCodeGenerator {
	return &GoCodeGenerator{ir: irProgram}
}

// SetSourceFile sets the source file path for the generator
func (g *GoCodeGenerator) SetSourceFile(path string) {
	g.sourceFile = path
}

// Generate generates Go source code
func (g *GoCodeGenerator) Generate() (string, error) {
	var code strings.Builder

	// Write package and imports
	code.WriteString("package main\n\n")
	code.WriteString("import (\n")
	code.WriteString("\t\"fmt\"\n")
	code.WriteString("\t\"os\"\n")
	code.WriteString("\t\"path/filepath\"\n")
	code.WriteString("\t\"github.com/ecocee/kode-go/pkg/ast\"\n")
	code.WriteString("\t\"github.com/ecocee/kode-go/pkg/bridge\"\n")
	code.WriteString("\t\"github.com/ecocee/kode-go/pkg/ir\"\n")
	code.WriteString("\t\"github.com/ecocee/kode-go/pkg/runtime\"\n")
	code.WriteString(")\n\n")

	// Generate the AST statements directly as Go code
	code.WriteString("func main() {\n")
	code.WriteString("\t// Find and read the Kode source file\n")
	code.WriteString("\tsourceFileName := filepath.Base(\"" + g.sourceFile + "\")\n")
	code.WriteString("\t\n")
	code.WriteString("\t// Try to read from current directory\n")
	code.WriteString("\tsourceData, err := os.ReadFile(sourceFileName)\n")
	code.WriteString("\tif err != nil {\n")
	code.WriteString("\t\tfmt.Fprintf(os.Stderr, \"Error reading source file %s: %v\\n\", sourceFileName, err)\n")
	code.WriteString("\t\tos.Exit(1)\n")
	code.WriteString("\t}\n")
	code.WriteString("\t\n")
	code.WriteString("\t// Parse the source using the bridge\n")
	code.WriteString("\tstmts, err := bridge.ParseSource(sourceFileName, string(sourceData))\n")
	code.WriteString("\tif err != nil {\n")
	code.WriteString("\t\tfmt.Fprintf(os.Stderr, \"Parse error: %v\\n\", err)\n")
	code.WriteString("\t\tos.Exit(1)\n")
	code.WriteString("\t}\n")
	code.WriteString("\t\n")
	code.WriteString("\t// Execute the parsed statements\n")
	code.WriteString("\trt := runtime.NewRuntime()\n")
	code.WriteString("\tif err := rt.Execute(&ir.IR{AST: ast.Program{Statements: stmts}}); err != nil {\n")
	code.WriteString("\t\tfmt.Fprintf(os.Stderr, \"Runtime error: %v\\n\", err)\n")
	code.WriteString("\t\tos.Exit(1)\n")
	code.WriteString("\t}\n")
	code.WriteString("}\n\n")

	// Remove the getStatements function since we parse at runtime
	code.WriteString("// Statements are parsed from the source file at runtime\n")

	return code.String(), nil
}

// generateFunction generates a Go function
func (g *GoCodeGenerator) generateFunction(fn *ir.IRFunction) string {
	var code strings.Builder

	returnType := g.typeToGoCode(fn.ReturnType)
	paramStr := ""
	if len(fn.Params) > 0 {
		paramStr = strings.Join(fn.Params, ", ")
	}

	code.WriteString(fmt.Sprintf("func %s(%s) %s {\n", fn.Name, paramStr, returnType))

	// Generate function body
	for _, block := range fn.Body {
		code.WriteString(g.generateBlock(block))
	}

	// Add default return if needed
	if returnType != "" && returnType != "void" {
		code.WriteString(fmt.Sprintf("\treturn %s(0)\n", g.getZeroValue(fn.ReturnType)))
	}

	code.WriteString("}\n\n")
	return code.String()
}

// generateBlock generates a basic block
func (g *GoCodeGenerator) generateBlock(block *ir.IRBlock) string {
	var code strings.Builder

	for _, instr := range block.Instructions {
		code.WriteString(g.generateInstruction(instr))
	}

	return code.String()
}

// generateInstruction generates an instruction
func (g *GoCodeGenerator) generateInstruction(instr ir.IRInstruction) string {
	switch i := instr.(type) {
	case ir.IRCall:
		return g.generateCall(&i)
	case ir.IRReturn:
		return g.generateReturn(&i)
	case ir.IRBinaryOp:
		return g.generateBinaryOp(&i)
	default:
		return ""
	}
}

// generateCall generates a function call
func (g *GoCodeGenerator) generateCall(call *ir.IRCall) string {
	args := make([]string, len(call.Args))
	for i, arg := range call.Args {
		args[i] = g.valueToGoCode(arg)
	}

	if call.Function == "print" {
		argStr := strings.Join(args, ", ")
		return fmt.Sprintf("\tfmt.Println(%s)\n", argStr)
	}

	return fmt.Sprintf("\t%s(%s)\n", call.Function, strings.Join(args, ", "))
}

// generateReturn generates a return statement
func (g *GoCodeGenerator) generateReturn(ret *ir.IRReturn) string {
	if ret.Value != nil {
		return fmt.Sprintf("\treturn %s\n", g.valueToGoCode(ret.Value))
	}
	return "\treturn\n"
}

// generateBinaryOp generates a binary operation
func (g *GoCodeGenerator) generateBinaryOp(op *ir.IRBinaryOp) string {
	left := g.valueToGoCode(op.Left)
	right := g.valueToGoCode(op.Right)

	opStr := ""
	switch op.Op {
	case ast.OpAdd:
		opStr = "+"
	case ast.OpSubtract:
		opStr = "-"
	case ast.OpMultiply:
		opStr = "*"
	case ast.OpDivide:
		opStr = "/"
	case ast.OpEqual:
		opStr = "=="
	case ast.OpNotEqual:
		opStr = "!="
	case ast.OpLessThan:
		opStr = "<"
	case ast.OpGreaterThan:
		opStr = ">"
	case ast.OpLessThanOrEqual:
		opStr = "<="
	case ast.OpGreaterThanOrEqual:
		opStr = ">="
	case ast.OpAnd:
		opStr = "&&"
	case ast.OpOr:
		opStr = "||"
	default:
		opStr = "?"
	}

	return fmt.Sprintf("\t%s := %s %s %s\n", op.Result, left, opStr, right)
}

// valueToGoCode converts an IR value to Go code
func (g *GoCodeGenerator) valueToGoCode(val ir.IRValue) string {
	switch v := val.(type) {
	case ir.IRConstant:
		switch v.Value.(type) {
		case string:
			return fmt.Sprintf("%q", v.Value)
		default:
			return fmt.Sprintf("%v", v.Value)
		}
	case ir.IRVariable:
		return v.Name
	default:
		return "nil"
	}
}

// typeToGoCode converts an AST type to Go code
func (g *GoCodeGenerator) typeToGoCode(t ast.Type) string {
	if t == nil {
		return ""
	}

	switch t.(type) {
	case ast.IntType:
		return "int64"
	case ast.FloatType:
		return "float64"
	case ast.BoolType:
		return "bool"
	case ast.StringType:
		return "string"
	case ast.VoidType:
		return ""
	case ast.ArrayType:
		arrayType := t.(ast.ArrayType)
		return fmt.Sprintf("[]%s", g.typeToGoCode(arrayType.ElementType))
	default:
		return "interface{}"
	}
}

// getZeroValue returns the zero value for a type
func (g *GoCodeGenerator) getZeroValue(t ast.Type) string {
	switch t.(type) {
	case ast.IntType:
		return "0"
	case ast.FloatType:
		return "0.0"
	case ast.BoolType:
		return "false"
	case ast.StringType:
		return `""`
	default:
		return "nil"
	}
}

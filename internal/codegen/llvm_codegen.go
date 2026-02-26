package codegen

import (
	"fmt"
	"strings"

	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/ir"
)

// LLVMCodeGenerator generates LLVM IR from Kode IR
type LLVMCodeGenerator struct {
	ir          *ir.IR
	stringCount int
}

// NewLLVMCodeGenerator creates a new LLVM code generator
func NewLLVMCodeGenerator(irProgram *ir.IR) *LLVMCodeGenerator {
	return &LLVMCodeGenerator{
		ir:          irProgram,
		stringCount: 0,
	}
}

// Generate generates LLVM IR code
func (g *LLVMCodeGenerator) Generate() (string, error) {
	var code strings.Builder

	// Write LLVM header
	code.WriteString("; ModuleID = 'kode'\n")
	code.WriteString("source_filename = \"kode\"\n")
	code.WriteString("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n")
	code.WriteString("target triple = \"x86_64-unknown-linux-gnu\"\n\n")

	// Declare external functions
	code.WriteString("; External declarations\n")
	code.WriteString("declare i32 @printf(i8* noundef, ...) #1\n")
	code.WriteString("declare i32 @puts(i8* noundef) #1\n\n")

	// String constants
	code.WriteString("; String constants\n")
	stringMap := make(map[string]string)

	// Collect all string constants from globals and functions
	for _, global := range g.ir.Program.Globals {
		g.collectStrings(global.Value, stringMap, &code)
	}

	for _, fn := range g.ir.Program.Functions {
		for _, block := range fn.Body {
			for _, instr := range block.Instructions {
				g.collectStringsFromInstr(instr, stringMap, &code)
			}
		}
	}

	code.WriteString("\n")

	// Generate functions
	code.WriteString("; Function definitions\n")
	for _, fn := range g.ir.Program.Functions {
		code.WriteString(g.generateFunction(fn, stringMap))
	}

	// Attributes
	code.WriteString("\nattributes #1 = { \"frame-pointer\"=\"all\" \"no-trapping-math\"=\"true\" \"stack-protector-buffer-size\"=\"8\" \"target-cpu\"=\"x86-64\" \"target-features\"=\"+cx8,+fxsr,+mmx,+sse,+sse2,+x87\" \"tune-cpu\"=\"generic\" }\n")

	return code.String(), nil
}

// collectStrings collects string constants
func (g *LLVMCodeGenerator) collectStrings(val ir.IRValue, stringMap map[string]string, code *strings.Builder) {
	switch v := val.(type) {
	case ir.IRConstant:
		if str, ok := v.Value.(string); ok {
			if _, exists := stringMap[str]; !exists {
				varName := fmt.Sprintf("@str.%d", g.stringCount)
				stringMap[str] = varName
				g.stringCount++
				escaped := strings.ReplaceAll(str, "\"", "\\22")
				len := len(str)
				code.WriteString(fmt.Sprintf("%s = private unnamed_addr constant [%d x i8] c\"%s\\00\", align 1\n",
					varName, len+1, escaped))
			}
		}
	}
}

// collectStringsFromInstr collects strings from instructions
func (g *LLVMCodeGenerator) collectStringsFromInstr(instr ir.IRInstruction, stringMap map[string]string, code *strings.Builder) {
	switch i := instr.(type) {
	case ir.IRCall:
		for _, arg := range i.Args {
			g.collectStrings(arg, stringMap, code)
		}
	case ir.IRBinaryOp:
		g.collectStrings(i.Left, stringMap, code)
		g.collectStrings(i.Right, stringMap, code)
	}
}

// generateFunction generates an LLVM function
func (g *LLVMCodeGenerator) generateFunction(fn *ir.IRFunction, stringMap map[string]string) string {
	var code strings.Builder

	returnType := g.typeToLLVMType(fn.ReturnType)
	if returnType == "" {
		returnType = "void"
	}

	code.WriteString(fmt.Sprintf("define %s @%s(", returnType, fn.Name))

	// Generate parameters
	for i, param := range fn.Params {
		if i > 0 {
			code.WriteString(", ")
		}
		code.WriteString(fmt.Sprintf("i64 %%arg.%s", param))
	}

	code.WriteString(") #0 {\n")
	code.WriteString("entry:\n")

	// Generate function body
	varCounter := 0
	for _, block := range fn.Body {
		for _, instr := range block.Instructions {
			code.WriteString(g.generateInstruction(instr, stringMap, &varCounter))
		}
	}

	if returnType == "void" {
		code.WriteString("  ret void\n")
	} else {
		code.WriteString("  ret i64 0\n")
	}

	code.WriteString("}\n\n")
	return code.String()
}

// generateInstruction generates an LLVM instruction
func (g *LLVMCodeGenerator) generateInstruction(instr ir.IRInstruction, stringMap map[string]string, varCounter *int) string {
	switch i := instr.(type) {
	case ir.IRCall:
		return g.generateCall(&i, stringMap, varCounter)
	case ir.IRBinaryOp:
		return g.generateBinaryOp(&i, varCounter)
	case ir.IRReturn:
		return g.generateReturn(&i)
	default:
		return ""
	}
}

// generateCall generates an LLVM function call
func (g *LLVMCodeGenerator) generateCall(call *ir.IRCall, stringMap map[string]string, varCounter *int) string {
	var code strings.Builder

	if call.Function == "print" {
		for _, arg := range call.Args {
			code.WriteString(g.generatePrintCall(arg, stringMap, varCounter))
		}
		return code.String()
	}

	varName := fmt.Sprintf("%%v%d", *varCounter)
	*varCounter++

	code.WriteString(fmt.Sprintf("  %s = call i64 @%s(", varName, call.Function))
	for i, arg := range call.Args {
		if i > 0 {
			code.WriteString(", ")
		}
		code.WriteString(g.valueToLLVMValue(arg))
	}
	code.WriteString(")\n")

	return code.String()
}

// generatePrintCall generates LLVM code for print
func (g *LLVMCodeGenerator) generatePrintCall(val ir.IRValue, stringMap map[string]string, varCounter *int) string {
	var code strings.Builder

	switch v := val.(type) {
	case ir.IRConstant:
		if str, ok := v.Value.(string); ok {
			// Print string
			strVar := stringMap[str]
			code.WriteString(fmt.Sprintf("  %%ptr%d = getelementptr inbounds [%d x i8], ptr %s, i64 0, i64 0\n",
				*varCounter, len(str)+1, strVar))
			code.WriteString(fmt.Sprintf("  call i32 @puts(ptr %%ptr%d)\n", *varCounter))
			*varCounter++
		} else if num, ok := v.Value.(int64); ok {
			// Print number as string
			code.WriteString(fmt.Sprintf("  %%fmt.str = getelementptr inbounds [4 x i8], ptr @fmt.int, i64 0, i64 0\n"))
			code.WriteString(fmt.Sprintf("  call i32 (ptr, ...) @printf(ptr %%fmt.str, i64 %d)\n", num))
		}
	case ir.IRVariable:
		code.WriteString(fmt.Sprintf("  %%val%d = load i64, ptr %%%s\n", *varCounter, v.Name))
		code.WriteString(fmt.Sprintf("  %%fmt.str = getelementptr inbounds [4 x i8], ptr @fmt.int, i64 0, i64 0\n"))
		code.WriteString(fmt.Sprintf("  call i32 (ptr, ...) @printf(ptr %%fmt.str, i64 %%val%d)\n", *varCounter))
		*varCounter++
	}

	return code.String()
}

// generateBinaryOp generates an LLVM binary operation
func (g *LLVMCodeGenerator) generateBinaryOp(op *ir.IRBinaryOp, varCounter *int) string {
	var code strings.Builder

	left := g.valueToLLVMValue(op.Left)
	right := g.valueToLLVMValue(op.Right)
	varName := fmt.Sprintf("%%v%d", *varCounter)
	*varCounter++

	switch op.Op {
	case ast.OpAdd:
		code.WriteString(fmt.Sprintf("  %s = add i64 %s, %s\n", varName, left, right))
	case ast.OpSubtract:
		code.WriteString(fmt.Sprintf("  %s = sub i64 %s, %s\n", varName, left, right))
	case ast.OpMultiply:
		code.WriteString(fmt.Sprintf("  %s = mul i64 %s, %s\n", varName, left, right))
	case ast.OpDivide:
		code.WriteString(fmt.Sprintf("  %s = sdiv i64 %s, %s\n", varName, left, right))
	default:
		code.WriteString(fmt.Sprintf("  %s = add i64 %s, %s\n", varName, left, right))
	}

	return code.String()
}

// generateReturn generates an LLVM return statement
func (g *LLVMCodeGenerator) generateReturn(ret *ir.IRReturn) string {
	if ret.Value != nil {
		val := g.valueToLLVMValue(ret.Value)
		return fmt.Sprintf("  ret i64 %s\n", val)
	}
	return "  ret void\n"
}

// typeToLLVMType converts Kode type to LLVM type
func (g *LLVMCodeGenerator) typeToLLVMType(t ast.Type) string {
	if t == nil {
		return ""
	}

	switch t.(type) {
	case ast.IntType:
		return "i64"
	case ast.FloatType:
		return "double"
	case ast.BoolType:
		return "i1"
	case ast.StringType:
		return "i8*"
	case ast.VoidType:
		return "void"
	default:
		return "i64"
	}
}

// valueToLLVMValue converts an IR value to LLVM value
func (g *LLVMCodeGenerator) valueToLLVMValue(val ir.IRValue) string {
	switch v := val.(type) {
	case ir.IRConstant:
		switch v.Value.(type) {
		case int64:
			return fmt.Sprintf("i64 %v", v.Value)
		default:
			return "i64 0"
		}
	case ir.IRVariable:
		return fmt.Sprintf("i64 %%%s", v.Name)
	default:
		return "i64 0"
	}
}

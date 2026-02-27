package typer

import (
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"strings"

	"github.com/ecocee/kode-go/internal/parser"
	"github.com/ecocee/kode-go/pkg/ast"
)

// enable typer debug tracing (set to true during debugging)
// exported for tests or external control
var VerboseTyper = false

// KodeError represents an error with file and line information
type KodeError struct {
	File    string
	Line    int
	Message string
	Cause   error
}

func (e KodeError) Error() string {
	msg := e.Message
	if e.Cause != nil {
		msg = msg + ": " + e.Cause.Error()
	}
	if e.Line > 0 {
		return fmt.Sprintf("%s:%d: %s", e.File, e.Line, msg)
	}
	return fmt.Sprintf("%s: %s", e.File, msg)
}

func (e KodeError) Unwrap() error {
	return e.Cause
}

// wrapError wraps an error with file context
func wrapError(file string, line int, message string, cause error) error {
	return KodeError{
		File:    file,
		Line:    line,
		Message: message,
		Cause:   cause,
	}
}

// TypeVar represents a type variable for inference
type TypeVar struct {
	ID int
}

func (tv TypeVar) String() string { return fmt.Sprintf("T%d", tv.ID) }

// Constraint represents a type constraint
type Constraint struct {
	Left  ast.Type
	Right ast.Type
}

// Substitution map
type Substitution map[TypeVar]ast.Type

// Typer performs type checking and inference
type Typer struct {
	nextVarID   int
	env         map[string]ast.Type
	constraints []Constraint
}

// NewTyper creates a new type checker
func NewTyper() *Typer {
	typer := &Typer{
		nextVarID:   0,
		env:         make(map[string]ast.Type),
		constraints: []Constraint{},
	}

	// Add built-in functions
	typer.env["print"] = ast.FunctionType{
		ParamTypes: []ast.Type{ast.StringType{}},
		ReturnType: ast.VoidType{},
	}

	typer.env["input"] = ast.FunctionType{
		ParamTypes: []ast.Type{ast.StringType{}},
		ReturnType: ast.StringType{},
	}

	return typer
}

// CheckProgram type checks a program
func (t *Typer) CheckProgram(program ast.Program) error {
	// First pass: process imports to load module types
	for _, stmt := range program.Statements {
		if importStmt, ok := stmt.(ast.ImportStmt); ok {
			if err := t.loadModuleImport(importStmt); err != nil {
				return err
			}
		}
	}

	// Second pass: hoist all function definitions
	for _, stmt := range program.Statements {
		if fnStmt, ok := stmt.(ast.FunctionDefStmt); ok {
			// Add function to env without type checking the body yet
			paramTypes := make([]ast.Type, len(fnStmt.Params))
			for i, param := range fnStmt.Params {
				paramTypes[i] = param.Type
			}
			fnType := ast.FunctionType{ParamTypes: paramTypes, ReturnType: fnStmt.ReturnType}
			t.env[fnStmt.Name] = fnType
		}
	}

	// Second pass: type check all statements
	for _, stmt := range program.Statements {
		if err := t.checkStatement(stmt); err != nil {
			return err
		}
	}
	return t.solveConstraints()
}

// checkStatement type checks a statement
func (t *Typer) checkStatement(stmt ast.Statement) error {
	switch s := stmt.(type) {
	case ast.LetStmt:
		typ, err := t.inferExpression(s.Value)
		if err != nil {
			return err
		}
		if s.Type != nil {
			t.addConstraint(s.Type, typ)
		} else {
			// Infer type
			s.Type = typ
		}
		t.env[s.Name] = s.Type
	case ast.FunctionDefStmt:
		// Add function to env
		paramTypes := make([]ast.Type, len(s.Params))
		for i, param := range s.Params {
			paramTypes[i] = param.Type
		}
		fnType := ast.FunctionType{ParamTypes: paramTypes, ReturnType: s.ReturnType}
		t.env[s.Name] = fnType
		// Check body in new scope
		oldEnv := t.env
		t.env = make(map[string]ast.Type)
		for k, v := range oldEnv {
			t.env[k] = v
		}
		for _, param := range s.Params {
			t.env[param.Name] = param.Type
		}
		if s.IsExprBody {
			// Body is an Expression
			if _, err := t.inferExpression(s.Body.(ast.Expression)); err != nil {
				return err
			}
		} else {
			// Body is []Statement
			for _, bodyStmt := range s.Body.([]ast.Statement) {
				if err := t.checkStatement(bodyStmt); err != nil {
					return err
				}
			}
		}
		t.env = oldEnv
	case ast.AssignStmt:
		typ, err := t.inferExpression(s.Value)
		if err != nil {
			return err
		}
		if varType, ok := t.env[s.Name]; ok {
			t.addConstraint(varType, typ)
		} else {
			return fmt.Errorf("undefined variable: %s", s.Name)
		}
	case ast.ReturnStmt:
		// Assume return type from context
	case ast.IfStmt:
		condType, err := t.inferExpression(s.Condition)
		if err != nil {
			return err
		}
		t.addConstraint(ast.BoolType{}, condType)
		for _, stmt := range s.ThenBranch {
			if err := t.checkStatement(stmt); err != nil {
				return err
			}
		}
		for _, stmt := range s.ElseBranch {
			if err := t.checkStatement(stmt); err != nil {
				return err
			}
		}
	case ast.WhileStmt:
		condType, err := t.inferExpression(s.Condition)
		if err != nil {
			return err
		}
		t.addConstraint(ast.BoolType{}, condType)
		for _, stmt := range s.Body {
			if err := t.checkStatement(stmt); err != nil {
				return err
			}
		}
	case ast.ForStmt:
		if s.Init != nil {
			if stmt, ok := s.Init.(ast.Statement); ok {
				if err := t.checkStatement(stmt); err != nil {
					return err
				}
			} else if expr, ok := s.Init.(ast.Expression); ok {
				_, err := t.inferExpression(expr)
				if err != nil {
					return err
				}
			}
		}
		if s.Condition != nil {
			condType, err := t.inferExpression(s.Condition)
			if err != nil {
				return err
			}
			t.addConstraint(ast.BoolType{}, condType)
		}
		if s.Incr != nil {
			_, err := t.inferExpression(s.Incr)
			if err != nil {
				return err
			}
		}
		for _, stmt := range s.Body {
			if err := t.checkStatement(stmt); err != nil {
				return err
			}
		}
	case ast.StructDeclStmt:
		// Register struct type
		fields := make(map[string]ast.Type)
		for _, field := range s.Fields {
			fields[field.Name] = field.Type
		}
		structType := ast.StructType{Name: s.Name, Fields: fields}
		t.env[s.Name] = structType
	case ast.EnumDeclStmt:
		// Register enum type
		enumType := ast.EnumType{Name: s.Name, Variants: s.Variants}
		t.env[s.Name] = enumType
	case ast.PrintStmt:
		_, err := t.inferExpression(s.Value)
		return err
	case ast.ExprStmt:
		_, err := t.inferExpression(s.Expr)
		return err
		// Add more cases as needed
	}
	return nil
}

// inferExpression infers the type of an expression
func (t *Typer) inferExpression(expr ast.Expression) (ast.Type, error) {
	switch e := expr.(type) {
	case ast.NumberExpr:
		return ast.IntType{}, nil
	case ast.FloatExpr:
		return ast.FloatType{}, nil
	case ast.BoolExpr:
		return ast.BoolType{}, nil
	case ast.StringExpr:
		return ast.StringType{}, nil
	case ast.IdentifierExpr:
		if typ, ok := t.env[e.Name]; ok {
			return typ, nil
		}
		return nil, fmt.Errorf("undefined variable: %s", e.Name)
	case ast.BinaryExpr:

		leftType, err := t.inferExpression(e.Left)
		if err != nil {
			return nil, err
		}
		rightType, err := t.inferExpression(e.Right)
		if err != nil {
			return nil, err
		}
		switch e.Op {
		case ast.OpAdd, ast.OpSubtract, ast.OpMultiply, ast.OpDivide, ast.OpModulo:
			// Allow both int and float for arithmetic
			// Determine result type based on operands
			leftIsNumeric := t.isNumericType(leftType)
			rightIsNumeric := t.isNumericType(rightType)

			if !leftIsNumeric || !rightIsNumeric {
				return nil, fmt.Errorf("non-numeric operands for arithmetic")
			}

			// If either is float, result is float
			if _, isFloat := leftType.(ast.FloatType); isFloat {
				return ast.FloatType{}, nil
			}
			if _, isFloat := rightType.(ast.FloatType); isFloat {
				return ast.FloatType{}, nil
			}
			return ast.IntType{}, nil
		case ast.OpAssign:
			t.addConstraint(leftType, rightType)
			return rightType, nil
		case ast.OpEqual, ast.OpNotEqual, ast.OpLessThan, ast.OpGreaterThan, ast.OpLessThanOrEqual, ast.OpGreaterThanOrEqual:
			t.addConstraint(leftType, rightType)
			return ast.BoolType{}, nil
		case ast.OpAnd, ast.OpOr:
			// Allow any types for logical operators with coercion semantics
			// But still type-check the operands
			_, leftErr := t.inferExpression(e.Left)
			if leftErr != nil {
				return nil, leftErr
			}
			_, rightErr := t.inferExpression(e.Right)
			if rightErr != nil {
				return nil, rightErr
			}
			// Logical operators always return bool
			return ast.BoolType{}, nil
		case ast.OpBitAnd, ast.OpBitOr, ast.OpBitXor, ast.OpBitShl, ast.OpBitShr:
			// Bitwise operators require integer operands and return int
			if !t.isNumericType(leftType) || !t.isNumericType(rightType) {
				return nil, fmt.Errorf("bitwise operations require integer operands")
			}
			// Bitwise ops don't support float, so ensure both are int
			if _, isFloat := leftType.(ast.FloatType); isFloat {
				return nil, fmt.Errorf("bitwise operations do not support float operands")
			}
			if _, isFloat := rightType.(ast.FloatType); isFloat {
				return nil, fmt.Errorf("bitwise operations do not support float operands")
			}
			return ast.IntType{}, nil
		}
	case ast.CallExpr:
		calleeType, err := t.inferExpression(e.Callee)
		if err != nil {
			return nil, err
		}

		// Special handling for built-in functions
		if idExpr, ok := e.Callee.(ast.IdentifierExpr); ok {
			if idExpr.Name == "print" {
				// print accepts any type
				for _, arg := range e.Arguments {
					_, err := t.inferExpression(arg)
					if err != nil {
						return nil, err
					}
				}
				return ast.VoidType{}, nil
			}
			if idExpr.Name == "input" {
				// input accepts a string prompt and returns a string
				if len(e.Arguments) > 0 {
					_, err := t.inferExpression(e.Arguments[0])
					if err != nil {
						return nil, err
					}
				}
				return ast.StringType{}, nil
			}
		}

		if fnType, ok := calleeType.(ast.FunctionType); ok {
			if len(fnType.ParamTypes) != len(e.Arguments) {
				return nil, fmt.Errorf("wrong number of arguments")
			}
			for i, arg := range e.Arguments {
				argType, err := t.inferExpression(arg)
				if err != nil {
					return nil, err
				}
				t.addConstraint(fnType.ParamTypes[i], argType)
			}
			return fnType.ReturnType, nil
		}
		return nil, fmt.Errorf("not a function")
	case ast.ArrayExpr:
		if len(e.Elements) == 0 {
			return ast.ArrayType{ElementType: t.newTypeVar()}, nil
		}
		elemType, err := t.inferExpression(e.Elements[0])
		if err != nil {
			return nil, err
		}
		for _, elem := range e.Elements[1:] {
			et, err := t.inferExpression(elem)
			if err != nil {
				return nil, err
			}
			t.addConstraint(elemType, et)
		}
		return ast.ArrayType{ElementType: elemType}, nil
	case ast.UnaryExpr:
		operandType, err := t.inferExpression(e.Expr)
		if err != nil {
			return nil, err
		}
		switch e.Op {
		case ast.OpPostInc, ast.OpPostDec:
			// Ensure operand is numeric
			if !t.isNumericType(operandType) {
				return nil, fmt.Errorf("operand of ++/-- must be numeric")
			}
			return operandType, nil
		case ast.OpBitNot:
			// Bitwise NOT requires integer operand and returns int
			if !t.isNumericType(operandType) {
				return nil, fmt.Errorf("bitwise NOT requires integer operand")
			}
			if _, isFloat := operandType.(ast.FloatType); isFloat {
				return nil, fmt.Errorf("bitwise NOT does not support float operands")
			}
			return ast.IntType{}, nil
		}
		return operandType, nil // For other unary ops, return operand type
	case ast.ArrayAccessExpr:
		// Type check array and index
		arrayType, err := t.inferExpression(e.Array)
		if err != nil {
			return nil, err
		}
		indexType, err := t.inferExpression(e.Index)
		if err != nil {
			return nil, err
		}

		// Index must be numeric
		if !t.isNumericType(indexType) {
			return nil, fmt.Errorf("array index must be numeric, got %s", indexType)
		}

		// Array must be array type
		if arrType, ok := arrayType.(ast.ArrayType); ok {
			return arrType.ElementType, nil
		}
		return nil, fmt.Errorf("cannot index non-array type: %s", arrayType)
	case ast.MemberAccessExpr:
		// Type check member access (arr.len, obj.field, etc.)
		objType, err := t.inferExpression(e.Object)
		if err != nil {
			return nil, err
		}

		// Handle array methods/properties
		if arrType, ok := objType.(ast.ArrayType); ok {
			switch e.Member {
			case "len":
				// arr.len is a property that returns int
				return ast.IntType{}, nil
			case "push":
				// arr.push is a method taking element type
				return ast.FunctionType{
					ParamTypes: []ast.Type{arrType.ElementType},
					ReturnType: ast.VoidType{},
				}, nil
			case "pop":
				// arr.pop is a method returning element type
				return ast.FunctionType{
					ParamTypes: []ast.Type{},
					ReturnType: arrType.ElementType,
				}, nil
			default:
				return nil, fmt.Errorf("array has no member: %s", e.Member)
			}
		}

		// Handle struct field access
		if structType, ok := objType.(ast.StructType); ok {
			if fieldType, ok := structType.Fields[e.Member]; ok {
				return fieldType, nil
			}
			return nil, fmt.Errorf("struct %s has no field: %s", structType.Name, e.Member)
		}

		return nil, fmt.Errorf("cannot access member of type: %s", objType)
	case ast.StructLiteralExpr:
		// Type check struct literal
		structType, ok := t.env[e.StructName]
		if !ok {
			return nil, fmt.Errorf("undefined struct: %s", e.StructName)
		}

		if sType, ok := structType.(ast.StructType); ok {
			// Check all fields are provided and have correct types
			for fieldName, fieldType := range sType.Fields {
				if val, ok := e.Fields[fieldName]; ok {
					valType, err := t.inferExpression(val)
					if err != nil {
						return nil, err
					}
					t.addConstraint(fieldType, valType)
				} else {
					return nil, fmt.Errorf("missing field %s in struct literal %s", fieldName, e.StructName)
				}
			}
			// Check no extra fields provided
			for providedField := range e.Fields {
				if _, ok := sType.Fields[providedField]; !ok {
					return nil, fmt.Errorf("struct %s has no field %s", e.StructName, providedField)
				}
			}
			return sType, nil
		}
		return nil, fmt.Errorf("%s is not a struct type", e.StructName)
		// Add more cases
	}
	return t.newTypeVar(), nil // Default to type var
}

// addConstraint adds a type constraint
func (t *Typer) addConstraint(left, right ast.Type) {
	t.constraints = append(t.constraints, Constraint{Left: left, Right: right})
}

// newTypeVar creates a new type variable
func (t *Typer) newTypeVar() TypeVar {
	tv := TypeVar{ID: t.nextVarID}
	t.nextVarID++
	return tv
}

// solveConstraints solves the constraints (simplified unification)
func (t *Typer) solveConstraints() error {
	// Simplified: just check basic constraints
	for _, c := range t.constraints {
		// debug
		if VerboseTyper {
			fmt.Printf("constraint: %T %v  vs  %T %v\n", c.Left, c.Left, c.Right, c.Right)
		}
		if !t.unify(c.Left, c.Right) {
			return fmt.Errorf("type mismatch: %v vs %v", c.Left, c.Right)
		}
	}
	return nil
}

// unify checks if two types unify (simplified)
func (t *Typer) unify(a, b ast.Type) bool {
	switch at := a.(type) {
	case ast.IntType:
		_, ok := b.(ast.IntType)
		return ok
	case ast.BoolType:
		_, ok := b.(ast.BoolType)
		return ok
	case ast.StringType:
		_, ok := b.(ast.StringType)
		return ok
	case ast.FloatType:
		_, ok := b.(ast.FloatType)
		return ok
	case ast.ArrayType:
		if bt, ok := b.(ast.ArrayType); ok {
			return t.unify(at.ElementType, bt.ElementType)
		}
		return false
	case ast.FunctionType:
		if bt, ok := b.(ast.FunctionType); ok {
			if len(at.ParamTypes) != len(bt.ParamTypes) {
				return false
			}
			for i := range at.ParamTypes {
				if !t.unify(at.ParamTypes[i], bt.ParamTypes[i]) {
					return false
				}
			}
			return t.unify(at.ReturnType, bt.ReturnType)
		}
		return false
	case TypeVar:
		// For simplicity, assume type vars unify with anything
		return true
	}
	return false
}

// isNumericType checks if a type is numeric (int or float)
func (t *Typer) isNumericType(typ ast.Type) bool {
	switch typ.(type) {
	case ast.IntType, ast.FloatType:
		return true
	default:
		return false
	}
}

// resolveModulePathForTyper resolves an import path to an actual file path
func (t *Typer) resolveModulePathForTyper(importPath string) (string, error) {
	// If the path has an extension, use it as-is
	if filepath.Ext(importPath) != "" {
		if _, err := os.Stat(importPath); err == nil {
			return importPath, nil
		}
	}

	// Try with .kode extension
	kodePath := importPath + ".kode"
	if _, err := os.Stat(kodePath); err == nil {
		return kodePath, nil
	}

	// Try in examples directory
	examplesPath := filepath.Join("examples", importPath+".kode")
	if _, err := os.Stat(examplesPath); err == nil {
		return examplesPath, nil
	}

	// Try in current directory
	currentPath := filepath.Join(".", importPath+".kode")
	if _, err := os.Stat(currentPath); err == nil {
		return currentPath, nil
	}

	return "", fmt.Errorf("module not found: %s", importPath)
}

// extractModuleExports extracts exported functions and constants from module AST
func (t *Typer) extractModuleExports(statements []ast.Statement) map[string]ast.Type {
	exports := make(map[string]ast.Type)

	for _, stmt := range statements {
		switch s := stmt.(type) {
		case ast.ExportStmt:
			// Extract the wrapped statement
			switch wrapped := s.Statement.(type) {
			case ast.FunctionDefStmt:
				// Convert function to type
				paramTypes := make([]ast.Type, len(wrapped.Params))
				for i, param := range wrapped.Params {
					paramTypes[i] = param.Type
				}
				fnType := ast.FunctionType{
					ParamTypes: paramTypes,
					ReturnType: wrapped.ReturnType,
				}
				exports[wrapped.Name] = fnType
			case ast.ConstDeclStmt:
				// Constants get their type from the declaration
				exports[wrapped.Name] = wrapped.Type
			}
		}
	}

	return exports
}

// loadModuleImport loads a module and adds its exported symbols to the type environment
func (t *Typer) loadModuleImport(stmt ast.ImportStmt) error {
	// Resolve the module path
	modulePath, err := t.resolveModulePathForTyper(stmt.Path)
	if err != nil {
		return wrapError("", 0, fmt.Sprintf("module not found: %s", stmt.Path), err)
	}

	// Read the module file
	content, err := ioutil.ReadFile(modulePath)
	if err != nil {
		return wrapError(modulePath, 0, "failed to read module file", err)
	}

	// Parse the module directly from source
	p, err := parser.NewParser(modulePath, string(content))
	if err != nil {
		return wrapError(modulePath, 0, "failed to create parser for module", err)
	}

	statements, err := p.Parse()
	if err != nil {
		// Try to extract line number from parser error
		line := t.extractLineFromError(err.Error(), string(content))
		return wrapError(modulePath, line, "parse error in imported module", err)
	}

	// Create a temporary typer for the module and type-check it
	moduleTyper := NewTyper()
	moduleProgram := ast.Program{Statements: statements}
	if err := moduleTyper.CheckProgram(moduleProgram); err != nil {
		// Try to extract line number from type error
		line := t.extractLineFromError(err.Error(), string(content))
		return wrapError(modulePath, line, "type error in imported module", err)
	}

	// Extract exported symbols
	exports := t.extractModuleExports(statements)

	// Also hoist function definitions from the module for complete type info
	for _, s := range statements {
		if expStmt, ok := s.(ast.ExportStmt); ok {
			if fnStmt, ok := expStmt.Statement.(ast.FunctionDefStmt); ok {
				paramTypes := make([]ast.Type, len(fnStmt.Params))
				for i, param := range fnStmt.Params {
					paramTypes[i] = param.Type
				}
				fnType := ast.FunctionType{
					ParamTypes: paramTypes,
					ReturnType: fnStmt.ReturnType,
				}
				exports[fnStmt.Name] = fnType
			}
		}
	}

	// Add exported symbols to current environment based on import style
	if stmt.IsNamed {
		// Named import: import { add, subtract } from "math"
		for _, item := range stmt.Items {
			if typ, ok := exports[item]; ok {
				t.env[item] = typ
			} else {
				return wrapError("", 0, fmt.Sprintf("exported symbol '%s' not found in module %s", item, stmt.Path), nil)
			}
		}
	} else if stmt.Alias != "" {
		// Namespace import: import math from "math"
	} else {
		// Wildcard import: import * from "math"
		for name, typ := range exports {
			t.env[name] = typ
		}
	}

	return nil
}

// extractLineFromError attempts to extract line number from error message
func (t *Typer) extractLineFromError(_, source string) int {
	// Look for patterns like "at line X" or "line X"
	// For now, return 0 if we can't determine the line
	sourceLines := strings.Split(source, "\n")

	// Simple heuristic: if error mentions a specific token, find it in source
	// This is a basic implementation - could be improved
	_ = sourceLines // Avoid unused variable warning
	return 0
}

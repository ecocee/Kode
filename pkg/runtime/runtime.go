package runtime

import (
	"context"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"sync"

	"github.com/ecocee/kode-go/internal/parser"
	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/ir"
)

// enable runtime debug tracing (set to true during debugging)
// exported for tests or external control
var VerboseRuntime = false

// Scheduler manages goroutines
type Scheduler struct {
	wg sync.WaitGroup
}

// NewScheduler creates a new scheduler
func NewScheduler() *Scheduler {
	return &Scheduler{}
}

// Spawn spawns a goroutine
func (s *Scheduler) Spawn(fn func()) {
	s.wg.Add(1)
	go func() {
		defer s.wg.Done()
		fn()
	}()
}

// Wait waits for all goroutines to finish
func (s *Scheduler) Wait() {
	s.wg.Wait()
}

// Channel represents a communication channel
type Channel struct {
	ch chan interface{}
}

// NewChannel creates a new channel
func NewChannel() *Channel {
	return &Channel{ch: make(chan interface{})}
}

// Send sends a value
func (c *Channel) Send(ctx context.Context, value interface{}) error {
	select {
	case c.ch <- value:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

// Receive receives a value
func (c *Channel) Receive(ctx context.Context) (interface{}, error) {
	select {
	case val := <-c.ch:
		return val, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	}
}

// Close closes the channel
func (c *Channel) Close() {
	close(c.ch)
}

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
		msg = fmt.Sprintf("%s: %v", e.Message, e.Cause)
	}
	if e.Line > 0 {
		return fmt.Sprintf("%s:%d: %s", e.File, e.Line, msg)
	}
	return fmt.Sprintf("%s: %s", e.File, msg)
}

func (e KodeError) Unwrap() error {
	return e.Cause
}

// Runtime holds the execution environment
type Runtime struct {
	Scheduler   *Scheduler
	Channels    map[string]*Channel
	globals     map[string]interface{}
	locals      map[string]interface{}
	functions   map[string]ast.FunctionDefStmt
	currentFile string
}

// wrapError wraps an error with file context
func wrapRuntimeError(file string, line int, message string, cause error) error {
	return KodeError{
		File:    file,
		Line:    line,
		Message: message,
		Cause:   cause,
	}
}

// NewRuntime creates a new runtime
func NewRuntime() *Runtime {
	return &Runtime{
		Scheduler:   NewScheduler(),
		Channels:    make(map[string]*Channel),
		globals:     make(map[string]interface{}),
		locals:      make(map[string]interface{}),
		functions:   make(map[string]ast.FunctionDefStmt),
		currentFile: "",
	}
}

// getLineFromNode extracts the line number from any AST node
func getLineFromNode(node interface{}) int {
	switch n := node.(type) {
	case ast.LetStmt:
		return n.Line
	case ast.AssignStmt:
		return n.Line
	case ast.FunctionDefStmt:
		return n.Line
	case ast.ReturnStmt:
		return n.Line
	case ast.IfStmt:
		return n.Line
	case ast.WhileStmt:
		return n.Line
	case ast.ForStmt:
		return n.Line
	case ast.ExprStmt:
		return n.Line
	case ast.BlockStmt:
		return n.Line
	case ast.PrintStmt:
		return n.Line
	case ast.ImportStmt:
		return n.Line
	case ast.ExportStmt:
		return n.Line
	case ast.TryStmt:
		return n.Line
	case ast.GoStmt:
		return n.Line
	case ast.SelectStmt:
		return n.Line
	case ast.HttpStmt:
		return n.Line
	case ast.RouteStmt:
		return n.Line
	case ast.BreakStmt:
		return n.Line
	case ast.ContinueStmt:
		return n.Line
	case ast.SpawnStmt:
		return n.Line
	case ast.DeferStmt:
		return n.Line
	case ast.ImplDeclStmt:
		return n.Line
	case ast.ServiceDeclStmt:
		return n.Line
	case ast.ModuleDeclStmt:
		return n.Line
	default:
		return 0
	}
}

// Execute runs the program
func (r *Runtime) Execute(irProgram interface{}, fileName string) error {
	program, ok := irProgram.(*ir.IR)
	if !ok {
		return fmt.Errorf("invalid IR program")
	}
	// Set current file for error reporting
	r.currentFile = fileName
	// Always use AST interpreter for execution
	return r.executeAST(program.AST)
}

// executeAST executes the AST program
func (r *Runtime) executeAST(program ast.Program) error {
	for _, stmt := range program.Statements {
		if err := r.executeStatement(stmt); err != nil {
			return err
		}
	}
	return nil
}

// executeStatement executes a single statement
func (r *Runtime) executeStatement(stmt ast.Statement) error {
	switch s := stmt.(type) {
	case ast.LetStmt:
		val, err := r.evaluateExpression(s.Value)
		if err != nil {
			return wrapRuntimeError(r.currentFile, getLineFromNode(s), "error in let statement", err)
		}
		r.globals[s.Name] = val
	case ast.AssignStmt:
		val, err := r.evaluateExpression(s.Value)
		if err != nil {
			return wrapRuntimeError(r.currentFile, getLineFromNode(s), "error in assignment", err)
		}
		if _, ok := r.locals[s.Name]; ok {
			r.locals[s.Name] = val
		} else {
			r.globals[s.Name] = val
		}
	case ast.PrintStmt:
		val, err := r.evaluateExpression(s.Value)
		if err != nil {
			return wrapRuntimeError(r.currentFile, getLineFromNode(s), "error in print statement", err)
		}
		fmt.Println(val)
	case ast.IfStmt:
		cond, err := r.evaluateExpression(s.Condition)
		if err != nil {
			return wrapRuntimeError(r.currentFile, getLineFromNode(s), "error in if condition", err)
		}
		if cond.(bool) {
			for _, st := range s.ThenBranch {
				if err := r.executeStatement(st); err != nil {
					return err
				}
			}
		} else if s.ElseBranch != nil {
			for _, st := range s.ElseBranch {
				if err := r.executeStatement(st); err != nil {
					return err
				}
			}
		}
	case ast.WhileStmt:
		for {
			cond, err := r.evaluateExpression(s.Condition)
			if err != nil {
				return err
			}
			if !cond.(bool) {
				break
			}
			for _, st := range s.Body {
				if err := r.executeStatement(st); err != nil {
					return err
				}
			}
		}
	case ast.ForStmt:
		if s.Init != nil {
			if stmt, ok := s.Init.(ast.Statement); ok {
				if err := r.executeStatement(stmt); err != nil {
					return err
				}
			} else if expr, ok := s.Init.(ast.Expression); ok {
				_, err := r.evaluateExpression(expr)
				if err != nil {
					return err
				}
			}
		}
		for {
			if s.Condition != nil {
				cond, err := r.evaluateExpression(s.Condition)
				if err != nil {
					return err
				}
				if !cond.(bool) {
					break
				}
			}
			for _, st := range s.Body {
				if err := r.executeStatement(st); err != nil {
					return err
				}
			}
			if s.Incr != nil {
				_, err := r.evaluateExpression(s.Incr)
				if err != nil {
					return err
				}
			}
		}
	case ast.FunctionDefStmt:
		r.functions[s.Name] = s
	case ast.StructDeclStmt:
		// Struct declarations don't need runtime code, just type definitions
		// In a full implementation, we'd store struct metadata here
		return nil
	case ast.EnumDeclStmt:
		// Enum declarations don't need runtime code, just type definitions
		return nil
	case ast.ImportStmt:
		// Load and execute the module
		return r.loadAndExecuteModule(s)
	case ast.ExportStmt:
		// Export wraps another statement, execute the wrapped statement
		return r.executeStatement(s.Statement)
	case ast.ConstDeclStmt:
		// Const declarations are like let declarations at runtime
		val, err := r.evaluateExpression(s.Value)
		if err != nil {
			return err
		}
		r.globals[s.Name] = val
		return nil
	case ast.ExprStmt:
		_, err := r.evaluateExpression(s.Expr)
		return err
	}
	return nil
}

// evaluateExpression evaluates an expression
func (r *Runtime) evaluateExpression(expr ast.Expression) (interface{}, error) {
	switch e := expr.(type) {
	case ast.NumberExpr:
		return e.Value, nil
	case ast.StringExpr:
		return e.Value, nil
	case ast.BoolExpr:
		return e.Value, nil
	case ast.IdentifierExpr:
		if val, ok := r.locals[e.Name]; ok {
			return val, nil
		}
		if val, ok := r.globals[e.Name]; ok {
			return val, nil
		}
		return nil, wrapRuntimeError(r.currentFile, 0, fmt.Sprintf("undefined variable: %s", e.Name), nil)
	case ast.BinaryExpr:
		if e.Op == ast.OpAssign {
			// Special handling for assignment
			if id, ok := e.Left.(ast.IdentifierExpr); ok {
				val, err := r.evaluateExpression(e.Right)
				if err != nil {
					return nil, err
				}
				if _, ok := r.locals[id.Name]; ok {
					r.locals[id.Name] = val
				} else {
					r.globals[id.Name] = val
				}
				return val, nil
			}
			return nil, fmt.Errorf("invalid assignment target")
		}
		left, err := r.evaluateExpression(e.Left)
		if err != nil {
			return nil, err
		}
		right, err := r.evaluateExpression(e.Right)
		if err != nil {
			return nil, err
		}
		return r.evaluateBinaryOp(e.Op, left, right, e.Line)
	case ast.UnaryExpr:
		operand, err := r.evaluateExpression(e.Expr)
		if err != nil {
			return nil, err
		}
		switch e.Op {
		case ast.OpNot:
			// Logical NOT
			boolVal := r.toBool(operand)
			return !boolVal, nil
		case ast.OpBitNot:
			// Bitwise NOT
			if val, ok := operand.(int64); ok {
				return ^val, nil
			}
			return nil, fmt.Errorf("bitwise NOT requires integer operand")
		case ast.OpPostInc:
			if id, ok := e.Expr.(ast.IdentifierExpr); ok {
				if val, ok := operand.(int64); ok {
					oldVal := val
					newVal := val + 1
					if _, ok := r.locals[id.Name]; ok {
						r.locals[id.Name] = newVal
					} else {
						r.globals[id.Name] = newVal
					}
					return oldVal, nil
				}
			}
			return nil, fmt.Errorf("operand of ++ must be an integer variable")
		case ast.OpPostDec:
			if id, ok := e.Expr.(ast.IdentifierExpr); ok {
				if val, ok := operand.(int64); ok {
					oldVal := val
					newVal := val - 1
					if _, ok := r.locals[id.Name]; ok {
						r.locals[id.Name] = newVal
					} else {
						r.globals[id.Name] = newVal
					}
					return oldVal, nil
				}
			}
			return nil, fmt.Errorf("operand of -- must be an integer variable")
		}
		return operand, nil
	case ast.ArrayExpr:
		// Array literal [1, 2, 3, ...]
		arr := make([]interface{}, len(e.Elements))
		for i, elem := range e.Elements {
			val, err := r.evaluateExpression(elem)
			if err != nil {
				return nil, err
			}
			arr[i] = val
		}
		return arr, nil
	case ast.ArrayAccessExpr:
		// Array indexing arr[index]
		arrayVal, err := r.evaluateExpression(e.Array)
		if err != nil {
			return nil, err
		}
		indexVal, err := r.evaluateExpression(e.Index)
		if err != nil {
			return nil, err
		}
		arr, ok := arrayVal.([]interface{})
		if !ok {
			return nil, fmt.Errorf("cannot index non-array type")
		}
		idx, ok := indexVal.(int64)
		if !ok {
			// Try to convert float to int
			if f, ok := indexVal.(float64); ok {
				idx = int64(f)
			} else {
				return nil, fmt.Errorf("array index must be a number")
			}
		}
		if idx < 0 || idx >= int64(len(arr)) {
			return nil, fmt.Errorf("array index out of bounds: %d", idx)
		}
		return arr[idx], nil
	case ast.MemberAccessExpr:
		// Member access: obj.member or arr.len
		objVal, err := r.evaluateExpression(e.Object)
		if err != nil {
			return nil, err
		}

		// Handle array methods
		if arr, ok := objVal.([]interface{}); ok {
			switch e.Member {
			case "len":
				// Return the length as an integer
				return int64(len(arr)), nil
			case "push", "pop":
				// Return a method reference (would be called later)
				return map[string]interface{}{"_method": e.Member, "_array": arr}, nil
			default:
				return nil, fmt.Errorf("array has no method: %s", e.Member)
			}
		}

		// Handle struct field access
		if structVal, ok := objVal.(map[string]interface{}); ok {
			if val, ok := structVal[e.Member]; ok {
				return val, nil
			}
			return nil, fmt.Errorf("struct has no field: %s", e.Member)
		}

		return nil, fmt.Errorf("cannot access member of type %T", objVal)
	case ast.StructLiteralExpr:
		// Struct literal: StructName { field1: value1, field2: value2, ... }
		fields := make(map[string]interface{})
		for fieldName, fieldExpr := range e.Fields {
			val, err := r.evaluateExpression(fieldExpr)
			if err != nil {
				return nil, err
			}
			fields[fieldName] = val
		}
		// Add struct type metadata
		fields["_type"] = e.StructName
		return fields, nil
	case ast.EnumVariantExpr:
		// Enum variant: EnumName::Variant(value)
		var value interface{} = nil
		if e.Value != nil {
			val, err := r.evaluateExpression(e.Value)
			if err != nil {
				return nil, err
			}
			value = val
		}

		// Create enum variant as a map with metadata
		variant := map[string]interface{}{
			"_type":    e.EnumName,
			"_variant": e.Variant,
			"_value":   value,
		}
		return variant, nil
	case ast.CallExpr:
		// Assume callee is identifier
		callee := e.Callee.(ast.IdentifierExpr)
		if callee.Name == "print" {
			// Built-in print
			for _, arg := range e.Arguments {
				val, err := r.evaluateExpression(arg)
				if err != nil {
					return nil, err
				}
				fmt.Print(val)
			}
			fmt.Println()
			return nil, nil
		}
		if callee.Name == "input" {
			// Built-in input
			if len(e.Arguments) > 0 {
				prompt, err := r.evaluateExpression(e.Arguments[0])
				if err != nil {
					return nil, err
				}
				fmt.Print(prompt)
			}
			var input string
			fmt.Scanln(&input)
			return input, nil
		}
		fn, ok := r.functions[callee.Name]
		if !ok {
			return nil, wrapRuntimeError(r.currentFile, 0, fmt.Sprintf("undefined function: %s", callee.Name), nil)
		}
		// Evaluate args
		args := make([]interface{}, len(e.Arguments))
		for i, arg := range e.Arguments {
			val, err := r.evaluateExpression(arg)
			if err != nil {
				return nil, err
			}
			args[i] = val
		}
		// Call function
		return r.callFunction(fn, args)
	}
	return nil, nil
}

// callFunction calls a function
func (r *Runtime) callFunction(fn ast.FunctionDefStmt, args []interface{}) (interface{}, error) {
	oldLocals := r.locals
	oldFile := r.currentFile
	r.locals = make(map[string]interface{})
	r.currentFile = fn.FilePrefix // Set current file to function's source file
	for i, param := range fn.Params {
		r.locals[param.Name] = args[i]
	}
	var result interface{}
	var err error
	if fn.IsExprBody {
		result, err = r.evaluateExpression(fn.Body.(ast.Expression))
		if err != nil {
			r.locals = oldLocals
			r.currentFile = oldFile
			return nil, err
		}
	} else {
		stmts := fn.Body.([]ast.Statement)
		for _, stmt := range stmts {
			if ret, ok := stmt.(ast.ReturnStmt); ok {
				result, err = r.evaluateExpression(ret.Value)
				if err != nil {
					r.locals = oldLocals
					r.currentFile = oldFile
					return nil, err
				}
				break
			}
			err = r.executeStatement(stmt)
			if err != nil {
				r.locals = oldLocals
				r.currentFile = oldFile
				return nil, err
			}
		}
	}
	r.locals = oldLocals
	r.currentFile = oldFile
	return result, nil
}

// executeBlock executes a block
func (r *Runtime) executeBlock(block *ir.IRBlock) error {
	for _, instr := range block.Instructions {
		switch i := instr.(type) {
		case ir.IRBinaryOp:
			left := r.evaluateValue(i.Left)
			right := r.evaluateValue(i.Right)
			result, err := r.evaluateBinaryOp(i.Op, left, right, 0)
			if err != nil {
				return err
			}
			r.locals[i.Result] = result
		case ir.IRCall:
			if err := r.executeCall(&i); err != nil {
				return err
			}
		case ir.IRReturn:
			// Return for now (simplified)
			return nil
		}
	}
	return nil
}

// executeCall executes a function call
func (r *Runtime) executeCall(call *ir.IRCall) error {
	args := make([]interface{}, len(call.Args))
	for i, arg := range call.Args {
		args[i] = r.evaluateValue(arg)
	}

	switch call.Function {
	case "print":
		if len(args) > 0 {
			fmt.Println(args[0])
		}
		return nil
	default:
		return fmt.Errorf("unknown function: %s", call.Function)
	}
}

// evaluateValue evaluates an IR value
func (r *Runtime) evaluateValue(val ir.IRValue) interface{} {
	switch v := val.(type) {
	case ir.IRConstant:
		return v.Value
	case ir.IRVariable:
		if val, ok := r.locals[v.Name]; ok {
			if VerboseRuntime {
				fmt.Printf("lookup local %s -> %#v\n", v.Name, val)
			}
			return val
		}
		if val, ok := r.globals[v.Name]; ok {
			if VerboseRuntime {
				fmt.Printf("lookup global %s -> %#v\n", v.Name, val)
			}
			return val
		}
		if VerboseRuntime {
			fmt.Printf("lookup variable %s -> nil\n", v.Name)
		}
		return nil
	default:
		return nil
	}
}

// evaluateBinaryOp evaluates a binary operation
func (r *Runtime) evaluateBinaryOp(op ast.BinaryOp, left, right interface{}, line int) (interface{}, error) {
	if VerboseRuntime {
		fmt.Printf("binary op %v with left=%#v (%T) right=%#v (%T)\n", op, left, left, right, right)
	}
	switch op {
	case ast.OpAdd:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l + r, nil
			}
			if r, ok := right.(float64); ok {
				return float64(l) + r, nil
			}
		}
		if l, ok := left.(float64); ok {
			if r, ok := right.(float64); ok {
				return l + r, nil
			}
			if r, ok := right.(int64); ok {
				return l + float64(r), nil
			}
		}
		if l, ok := left.(string); ok {
			if r, ok := right.(string); ok {
				return l + r, nil
			}
		}
	case ast.OpSubtract:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l - r, nil
			}
			if r, ok := right.(float64); ok {
				return float64(l) - r, nil
			}
		}
		if l, ok := left.(float64); ok {
			if r, ok := right.(float64); ok {
				return l - r, nil
			}
			if r, ok := right.(int64); ok {
				return l - float64(r), nil
			}
		}
	case ast.OpMultiply:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l * r, nil
			}
			if r, ok := right.(float64); ok {
				return float64(l) * r, nil
			}
		}
		if l, ok := left.(float64); ok {
			if r, ok := right.(float64); ok {
				return l * r, nil
			}
			if r, ok := right.(int64); ok {
				return l * float64(r), nil
			}
		}
	case ast.OpDivide:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				if r == 0 {
					return nil, fmt.Errorf("division by zero")
				}
				return l / r, nil
			}
			if r, ok := right.(float64); ok {
				if r == 0 {
					return nil, fmt.Errorf("division by zero")
				}
				return float64(l) / r, nil
			}
		}
		if l, ok := left.(float64); ok {
			if r, ok := right.(float64); ok {
				if r == 0 {
					return nil, fmt.Errorf("division by zero")
				}
				return l / r, nil
			}
			if r, ok := right.(int64); ok {
				if r == 0 {
					return nil, fmt.Errorf("division by zero")
				}
				return l / float64(r), nil
			}
		}
	case ast.OpModulo:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				if r == 0 {
					return nil, fmt.Errorf("division by zero")
				}
				return l % r, nil
			}
		}
	case ast.OpAnd:
		// Logical AND - convert to bool first
		leftBool := r.toBool(left)
		rightBool := r.toBool(right)
		return leftBool && rightBool, nil
	case ast.OpOr:
		// Logical OR - convert to bool first
		leftBool := r.toBool(left)
		rightBool := r.toBool(right)
		return leftBool || rightBool, nil
	case ast.OpBitAnd:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l & r, nil
			}
		}
	case ast.OpBitOr:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l | r, nil
			}
		}
	case ast.OpBitXor:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l ^ r, nil
			}
		}
	case ast.OpBitShl:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l << uint(r), nil
			}
		}
	case ast.OpBitShr:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l >> uint(r), nil
			}
		}
	case ast.OpEqual:
		return left == right, nil
	case ast.OpNotEqual:
		return left != right, nil
	case ast.OpLessThan:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l < r, nil
			}
		}
	case ast.OpGreaterThan:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l > r, nil
			}
		}
	case ast.OpLessThanOrEqual:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l <= r, nil
			}
		}
	case ast.OpGreaterThanOrEqual:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l >= r, nil
			}
		}
	}
	return nil, wrapRuntimeError(r.currentFile, line, fmt.Sprintf("unsupported binary operation: %v", op), nil)
}

// resolveModulePath resolves an import path to an actual file path
func (r *Runtime) resolveModulePath(importPath string) (string, error) {
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

// extractExportedSymbols extracts exported functions and constants from module AST
func (r *Runtime) extractExportedSymbols(statements []ast.Statement) map[string]interface{} {
	exports := make(map[string]interface{})

	for _, stmt := range statements {
		switch s := stmt.(type) {
		case ast.ExportStmt:
			// Extract the wrapped statement
			switch wrapped := s.Statement.(type) {
			case ast.FunctionDefStmt:
				exports[wrapped.Name] = wrapped
			case ast.ConstDeclStmt:
				// Evaluate the constant value and add to exports
				if val, err := r.evaluateExpression(wrapped.Value); err == nil {
					exports[wrapped.Name] = val
				}
			}
		}
	}

	return exports
}

// loadAndExecuteModule loads and executes a module, making exported symbols available
func (r *Runtime) loadAndExecuteModule(stmt ast.ImportStmt) error {
	// Resolve the module path
	modulePath, err := r.resolveModulePath(stmt.Path)
	if err != nil {
		return wrapRuntimeError("", 0, fmt.Sprintf("module not found: %s", stmt.Path), err)
	}

	// Read the module file
	content, err := ioutil.ReadFile(modulePath)
	if err != nil {
		return wrapRuntimeError(modulePath, 0, "failed to read module file", err)
	}

	// Parse the module
	p, err := parser.NewParser(modulePath, string(content))
	if err != nil {
		return wrapRuntimeError(modulePath, 0, "failed to create parser for module", err)
	}

	statements, err := p.Parse()
	if err != nil {
		// Try to extract line number from parser error
		line := r.extractLineFromRuntimeError()
		return wrapRuntimeError(modulePath, line, "parse error in imported module", err)
	}

	// Create a temporary runtime for the module to collect exports
	moduleRuntime := NewRuntime()
	moduleRuntime.currentFile = modulePath

	// Execute the module to populate its functions and constants
	for i, s := range statements {
		if err := moduleRuntime.executeStatement(s); err != nil {
			// Try to extract line number from execution error
			line := r.extractLineFromRuntimeError()
			if line == 0 {
				line = i + 1 // Fallback to statement index
			}
			return wrapRuntimeError(modulePath, line, "execution error in imported module", err)
		}
	}

	// Extract exported symbols
	exports := moduleRuntime.extractExportedSymbols(statements)

	// Also need to get exported functions from moduleRuntime.functions
	for name, fn := range moduleRuntime.functions {
		// Check if this function was exported
		for _, stmt := range statements {
			if expStmt, ok := stmt.(ast.ExportStmt); ok {
				if fnStmt, ok := expStmt.Statement.(ast.FunctionDefStmt); ok && fnStmt.Name == name {
					exports[name] = fn
					break
				}
			}
		}
	}

	// Add exported symbols to current runtime based on import style
	if stmt.IsNamed {
		// Named import: import { add, subtract } from "math"
		for _, item := range stmt.Items {
			if val, ok := exports[item]; ok {
				r.globals[item] = val
				// If it's a function, also add to functions map
				if fn, ok := val.(ast.FunctionDefStmt); ok {
					r.functions[item] = fn
				}
			} else {
				return wrapRuntimeError("", 0, fmt.Sprintf("exported symbol '%s' not found in module %s", item, stmt.Path), nil)
			}
		}
	} else if stmt.Alias != "" {
		// Namespace import: import math from "math" -> creates math object
		namespace := make(map[string]interface{})
		for name, val := range exports {
			namespace[name] = val
		}
		r.globals[stmt.Alias] = namespace
	} else {
		// Wildcard import: import * from "math"
		for name, val := range exports {
			r.globals[name] = val
			if fn, ok := val.(ast.FunctionDefStmt); ok {
				r.functions[name] = fn
			}
		}
	}

	return nil
}

// toBool converts a value to boolean for logical operations
func (r *Runtime) toBool(val interface{}) bool {
	switch v := val.(type) {
	case bool:
		return v
	case int64:
		return v != 0
	case float64:
		return v != 0.0
	case string:
		return len(v) > 0
	case nil:
		return false
	default:
		return true // non-nil objects are truthy
	}
}

// extractLineFromRuntimeError attempts to extract line number from error message
func (r *Runtime) extractLineFromRuntimeError() int {
	// Look for patterns like "at line X" or "line X"
	// For now, return 0 if we can't determine the line
	// This is a basic implementation - could be improved with better parsing
	return 0
}

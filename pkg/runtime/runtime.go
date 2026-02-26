package runtime

import (
	"context"
	"fmt"
	"sync"

	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/ir"
)

// enable runtime debug tracing (set to true during debugging)
// exported for tests or external control
var VerboseRuntime = false

// internal alias used by code to avoid changing numerous references
var verboseRuntime = VerboseRuntime

// keep sync between variables whenever modified
func init() {
	// periodically sync if necessary
}

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

// Runtime holds the execution environment
type Runtime struct {
	Scheduler *Scheduler
	Channels  map[string]*Channel
	globals   map[string]interface{}
	locals    map[string]interface{}
	functions map[string]ast.FunctionDefStmt
}

// NewRuntime creates a new runtime
func NewRuntime() *Runtime {
	return &Runtime{
		Scheduler: NewScheduler(),
		Channels:  make(map[string]*Channel),
		globals:   make(map[string]interface{}),
		locals:    make(map[string]interface{}),
		functions: make(map[string]ast.FunctionDefStmt),
	}
}

// Execute runs the program
func (r *Runtime) Execute(irProgram interface{}) error {
	program, ok := irProgram.(*ir.IR)
	if !ok {
		return fmt.Errorf("invalid IR program")
	}
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
			return err
		}
		r.globals[s.Name] = val
	case ast.AssignStmt:
		val, err := r.evaluateExpression(s.Value)
		if err != nil {
			return err
		}
		if _, ok := r.locals[s.Name]; ok {
			r.locals[s.Name] = val
		} else {
			r.globals[s.Name] = val
		}
	case ast.PrintStmt:
		val, err := r.evaluateExpression(s.Value)
		if err != nil {
			return err
		}
		fmt.Println(val)
	case ast.IfStmt:
		cond, err := r.evaluateExpression(s.Condition)
		if err != nil {
			return err
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
		return nil, fmt.Errorf("undefined variable: %s", e.Name)
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
		return r.evaluateBinaryOp(e.Op, left, right)
	case ast.UnaryExpr:
		operand, err := r.evaluateExpression(e.Expr)
		if err != nil {
			return nil, err
		}
		switch e.Op {
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

		return nil, fmt.Errorf("cannot access member of non-array type")
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
			return nil, fmt.Errorf("undefined function: %s", callee.Name)
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
	r.locals = make(map[string]interface{})
	for i, param := range fn.Params {
		r.locals[param.Name] = args[i]
	}
	var result interface{}
	var err error
	if fn.IsExprBody {
		result, err = r.evaluateExpression(fn.Body.(ast.Expression))
		if err != nil {
			r.locals = oldLocals
			return nil, err
		}
	} else {
		stmts := fn.Body.([]ast.Statement)
		for _, stmt := range stmts {
			if ret, ok := stmt.(ast.ReturnStmt); ok {
				result, err = r.evaluateExpression(ret.Value)
				if err != nil {
					r.locals = oldLocals
					return nil, err
				}
				break
			}
			err = r.executeStatement(stmt)
			if err != nil {
				r.locals = oldLocals
				return nil, err
			}
		}
	}
	r.locals = oldLocals
	return result, nil
}

// executeFunction executes a function
func (r *Runtime) executeFunction(fn *ir.IRFunction, args []interface{}) error {
	// Create local scope
	oldLocals := r.locals
	r.locals = make(map[string]interface{})

	// Bind arguments to parameters
	for i, param := range fn.Params {
		if i < len(args) {
			r.locals[param] = args[i]
		}
	}

	// Execute blocks
	for _, block := range fn.Body {
		if err := r.executeBlock(block); err != nil {
			r.locals = oldLocals
			return err
		}
	}

	r.locals = oldLocals
	return nil
}

// executeBlock executes a block
func (r *Runtime) executeBlock(block *ir.IRBlock) error {
	for _, instr := range block.Instructions {
		switch i := instr.(type) {
		case ir.IRBinaryOp:
			left := r.evaluateValue(i.Left)
			right := r.evaluateValue(i.Right)
			result, err := r.evaluateBinaryOp(i.Op, left, right)
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
func (r *Runtime) evaluateBinaryOp(op ast.BinaryOp, left, right interface{}) (interface{}, error) {
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
	return nil, fmt.Errorf("unsupported binary operation: %v", op)
}

package runtime

import (
	"context"
	"fmt"
	"sync"

	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/ir"
)

// enable runtime debug tracing (set to true during debugging)
var verboseRuntime = false

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
}

// NewRuntime creates a new runtime
func NewRuntime() *Runtime {
	return &Runtime{
		Scheduler: NewScheduler(),
		Channels:  make(map[string]*Channel),
		globals:   make(map[string]interface{}),
		locals:    make(map[string]interface{}),
	}
}

// Execute runs the program
func (r *Runtime) Execute(irProgram interface{}) error {
	program, ok := irProgram.(*ir.IR)
	if !ok {
		return fmt.Errorf("invalid IR program")
	}

	// Process globals
	for _, g := range program.Program.Globals {
		r.globals[g.Name] = r.evaluateValue(g.Value)
	}

	// Execute main function if it exists
	for _, fn := range program.Program.Functions {
		if fn.Name == "main" {
			return r.executeFunction(fn, []interface{}{})
		}
	}

	return nil
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
			result := r.evaluateBinaryOp(i.Op, left, right)
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
			if verboseRuntime {
				fmt.Printf("lookup local %s -> %#v\n", v.Name, val)
			}
			return val
		}
		if val, ok := r.globals[v.Name]; ok {
			if verboseRuntime {
				fmt.Printf("lookup global %s -> %#v\n", v.Name, val)
			}
			return val
		}
		if verboseRuntime {
			fmt.Printf("lookup variable %s -> nil\n", v.Name)
		}
		return nil
	default:
		return nil
	}
}

// evaluateBinaryOp evaluates a binary operation
func (r *Runtime) evaluateBinaryOp(op ast.BinaryOp, left, right interface{}) interface{} {
	if verboseRuntime {
		fmt.Printf("binary op %v with left=%#v (%T) right=%#v (%T)\n", op, left, left, right, right)
	}
	switch op {
	case ast.OpAdd:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l + r
			}
			if r, ok := right.(float64); ok {
				return float64(l) + r
			}
		}
		if l, ok := left.(float64); ok {
			if r, ok := right.(float64); ok {
				return l + r
			}
			if r, ok := right.(int64); ok {
				return l + float64(r)
			}
		}
		if l, ok := left.(string); ok {
			if r, ok := right.(string); ok {
				return l + r
			}
		}
	case ast.OpSubtract:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l - r
			}
			if r, ok := right.(float64); ok {
				return float64(l) - r
			}
		}
		if l, ok := left.(float64); ok {
			if r, ok := right.(float64); ok {
				return l - r
			}
			if r, ok := right.(int64); ok {
				return l - float64(r)
			}
		}
	case ast.OpMultiply:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l * r
			}
			if r, ok := right.(float64); ok {
				return float64(l) * r
			}
		}
		if l, ok := left.(float64); ok {
			if r, ok := right.(float64); ok {
				return l * r
			}
			if r, ok := right.(int64); ok {
				return l * float64(r)
			}
		}
	case ast.OpDivide:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				if r == 0 {
					return nil
				}
				return l / r
			}
			if r, ok := right.(float64); ok {
				if r == 0 {
					return nil
				}
				return float64(l) / r
			}
		}
		if l, ok := left.(float64); ok {
			if r, ok := right.(float64); ok {
				if r == 0 {
					return nil
				}
				return l / r
			}
			if r, ok := right.(int64); ok {
				if r == 0 {
					return nil
				}
				return l / float64(r)
			}
		}
	case ast.OpEqual:
		return left == right
	case ast.OpNotEqual:
		return left != right
	case ast.OpLessThan:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l < r
			}
		}
	case ast.OpGreaterThan:
		if l, ok := left.(int64); ok {
			if r, ok := right.(int64); ok {
				return l > r
			}
		}
	}
	return nil
}

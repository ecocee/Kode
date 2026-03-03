package bytecode

import (
	"fmt"
	"math"
	"math/rand"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
)

// ClosureValue holds a compiled function name plus captured variable values.
// The captured values are baked in as hidden trailing parameters when the closure is called.
type ClosureValue struct {
	FuncName string
	Captured []interface{}
}

// CallFrame represents a function call frame
type CallFrame struct {
	pc     int                    // Return program counter
	bp     int                    // Return base pointer
	locals map[string]interface{} // Local variables for this frame
}

// LoopFrame represents a loop frame for break/continue
type LoopFrame struct {
	startPC     int // PC to jump to for continue
	breakTarget int // PC to jump to for break
}

// VM is a bytecode virtual machine
type VM struct {
	program     *Program
	stack       []interface{}
	globals     map[string]interface{}
	locals      []map[string]interface{} // Stack of local scopes
	pc          int                      // Program counter
	bp          int                      // Base pointer
	callStack   []CallFrame              // Call stack
	loopStack   []LoopFrame              // Loop stack for break/continue
	functions   map[string]*Program
	tryStack    []int   // Stack of catch block PCs for nested try/catch
	deferredPCs [][]int // Per-call-frame deferred code start PCs
}

// NewVM creates a new virtual machine
func NewVM(program *Program) *VM {
	return &VM{
		program:     program,
		stack:       make([]interface{}, 0, 256),
		globals:     make(map[string]interface{}),
		locals:      make([]map[string]interface{}, 0),
		callStack:   make([]CallFrame, 0),
		loopStack:   make([]LoopFrame, 0),
		tryStack:    make([]int, 0),
		deferredPCs: make([][]int, 0),
	}
}

// Run executes the program
func (vm *VM) Run() error {
	// Initialize globals
	for name := range vm.program.Globals {
		vm.globals[name] = nil
	}

	for vm.pc = 0; vm.pc < len(vm.program.Instructions); vm.pc++ {
		instr := vm.program.Instructions[vm.pc]

		switch instr.Op {
		case OpPush:
			if len(instr.Args) > 0 {
				constIdx := instr.Args[0].(int)
				vm.stack = append(vm.stack, vm.program.Constants[constIdx])
			}

		case OpPop:
			if len(vm.stack) > 0 {
				vm.stack = vm.stack[:len(vm.stack)-1]
			}

		case OpDup:
			if len(vm.stack) > 0 {
				vm.stack = append(vm.stack, vm.stack[len(vm.stack)-1])
			}

		case OpLoadGlobal:
			if len(instr.Args) > 0 {
				varIdx := instr.Args[0].(int)
				found := false
				// Find variable by index
				for name, idx := range vm.program.Globals {
					if idx == varIdx {
						vm.stack = append(vm.stack, vm.globals[name])
						found = true
						break
					}
				}
				if !found {
					// Variable not found — push nil to avoid stack corruption
					vm.stack = append(vm.stack, nil)
				}
			}

		case OpStoreGlobal:
			if len(instr.Args) > 0 && len(vm.stack) > 0 {
				varIdx := instr.Args[0].(int)
				val := vm.pop()
				// Find variable by index
				for name, idx := range vm.program.Globals {
					if idx == varIdx {
						vm.globals[name] = val
						break
					}
				}
			}

		case OpAdd:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				if left == nil || right == nil {
					runtimeErr := fmt.Errorf("cannot perform arithmetic on nil value")
					if !vm.tryHandleError(runtimeErr) {
						return runtimeErr
					}
					continue
				}
				result := vm.add(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpSub:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				if left == nil || right == nil {
					runtimeErr := fmt.Errorf("cannot perform arithmetic on nil value")
					if !vm.tryHandleError(runtimeErr) {
						return runtimeErr
					}
					continue
				}
				result := vm.subtract(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpMul:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				if left == nil || right == nil {
					runtimeErr := fmt.Errorf("cannot perform arithmetic on nil value")
					if !vm.tryHandleError(runtimeErr) {
						return runtimeErr
					}
					continue
				}
				result := vm.multiply(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpDiv:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				if left == nil || right == nil {
					runtimeErr := fmt.Errorf("cannot perform arithmetic on nil value")
					if !vm.tryHandleError(runtimeErr) {
						return runtimeErr
					}
					continue
				}
				if (isInt(right) && asInt(right) == 0) || (isFloat(right) && asFloat(right) == 0) {
					runtimeErr := fmt.Errorf("division by zero")
					if !vm.tryHandleError(runtimeErr) {
						return runtimeErr
					}
					continue
				}
				result := vm.divide(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpMod:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				if left == nil || right == nil {
					runtimeErr := fmt.Errorf("cannot perform arithmetic on nil value")
					if !vm.tryHandleError(runtimeErr) {
						return runtimeErr
					}
					continue
				}
				if isInt(right) && asInt(right) == 0 {
					runtimeErr := fmt.Errorf("modulo by zero")
					if !vm.tryHandleError(runtimeErr) {
						return runtimeErr
					}
					continue
				}
				result := vm.modulo(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpNeg:
			if len(vm.stack) > 0 {
				val := vm.pop()
				switch v := val.(type) {
				case int:
					vm.stack = append(vm.stack, -v)
				case float64:
					vm.stack = append(vm.stack, -v)
				}
			}

		case OpIncr:
			if len(vm.stack) > 0 {
				val := vm.pop()
				switch v := val.(type) {
				case int:
					vm.stack = append(vm.stack, v+1)
				case float64:
					vm.stack = append(vm.stack, v+1)
				}
			}

		case OpDecr:
			if len(vm.stack) > 0 {
				val := vm.pop()
				switch v := val.(type) {
				case int:
					vm.stack = append(vm.stack, v-1)
				case float64:
					vm.stack = append(vm.stack, v-1)
				}
			}

		case OpEq:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.equal(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpNe:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := !vm.equal(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpLt:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.lessThan(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpLte:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.lessThan(left, right) || vm.equal(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpGt:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := !vm.lessThan(left, right) && !vm.equal(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpGte:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := !vm.lessThan(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpAnd:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.isTruthy(left) && vm.isTruthy(right)
				vm.stack = append(vm.stack, result)
			}

		case OpOr:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.isTruthy(left) || vm.isTruthy(right)
				vm.stack = append(vm.stack, result)
			}

		case OpNot:
			if len(vm.stack) > 0 {
				val := vm.pop()
				vm.stack = append(vm.stack, !vm.isTruthy(val))
			}

		case OpPrint:
			if len(vm.stack) > 0 {
				val := vm.pop()
				fmt.Println(vm.valueToString(val))
			}
		case OpInput:
			var input string
			fmt.Print("")
			fmt.Scanln(&input)
			vm.stack = append(vm.stack, input)

		case OpInputWithPrompt:
			if len(vm.stack) > 0 {
				prompt := vm.pop()
				fmt.Print(vm.valueToString(prompt))
				var input string
				fmt.Scanln(&input)
				vm.stack = append(vm.stack, input)
			}

		case OpJmp:
			if len(instr.Args) > 0 {
				offset := instr.Args[0].(int)
				if offset < 0 {
					// Negative offset means jump back
					vm.pc += offset
				} else {
					// Positive offset means jump forward
					vm.pc += offset
				}
				vm.pc-- // -1 because pc will be increment in loop
			}

		case OpJmpIfFalse:
			if len(vm.stack) > 0 {
				val := vm.pop()
				if !vm.isTruthy(val) {
					if len(instr.Args) > 0 {
						offset := instr.Args[0].(int)
						vm.pc += offset - 1 // -1 because loop will increment
					}
				}
			}

		case OpJmpIfTrue:
			if len(vm.stack) > 0 {
				val := vm.pop()
				if vm.isTruthy(val) {
					if len(instr.Args) > 0 {
						offset := instr.Args[0].(int)
						vm.pc += offset - 1 // -1 because loop will increment
					}
				}
			}

		case OpHalt:
			return nil

		case OpBitAnd:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.bitwiseAnd(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpBitOr:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.bitwiseOr(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpBitXor:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.bitwiseXor(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpBitNot:
			if len(vm.stack) > 0 {
				val := vm.pop()
				result := vm.bitwiseNot(val)
				vm.stack = append(vm.stack, result)
			}

		case OpBitShl:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.bitwiseShl(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpBitShr:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.bitwiseShr(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpArrayCreate:
			if len(instr.Args) > 0 {
				if count, ok := instr.Args[0].(int); ok {
					// Pop 'count' elements from stack and create array
					if len(vm.stack) >= count {
						arr := make([]interface{}, count)
						// Pop in reverse order (LIFO)
						for i := count - 1; i >= 0; i-- {
							arr[i] = vm.pop()
						}
						vm.stack = append(vm.stack, arr)
					}
				}
			}

		case OpArrayAccess:
			if len(vm.stack) >= 2 {
				index := vm.pop()
				array := vm.pop()
				if arr, ok := array.([]interface{}); ok {
					var idx int
					switch i := index.(type) {
					case int:
						idx = i
					case int64:
						idx = int(i)
					default:
						runtimeErr := fmt.Errorf("array index must be an integer, got %T", index)
						if !vm.tryHandleError(runtimeErr) {
							return runtimeErr
						}
						continue
					}
					if idx < 0 || idx >= len(arr) {
						runtimeErr := fmt.Errorf("index out of bounds: index %d, length %d", idx, len(arr))
						if !vm.tryHandleError(runtimeErr) {
							return runtimeErr
						}
						continue
					}
					vm.stack = append(vm.stack, arr[idx])
				} else {
					runtimeErr := fmt.Errorf("cannot index into non-array type %T", array)
					if !vm.tryHandleError(runtimeErr) {
						return runtimeErr
					}
					continue
				}
			}

		case OpArrayStore:
			if len(vm.stack) >= 3 {
				value := vm.pop()
				index := vm.pop()
				array := vm.pop()
				if arr, ok := array.([]interface{}); ok {
					var idx int
					switch i := index.(type) {
					case int:
						idx = i
					case int64:
						idx = int(i)
					}
					if idx >= 0 && idx < len(arr) {
						arr[idx] = value
					}
				}
				vm.stack = append(vm.stack, array)
			}

		case OpArrayLen:
			if len(vm.stack) > 0 {
				array := vm.pop()
				if arr, ok := array.([]interface{}); ok {
					vm.stack = append(vm.stack, int(len(arr)))
				} else {
					vm.stack = append(vm.stack, 0)
				}
			}

		case OpArrayPush:
			// Stack: [..., value, array] — pops array and value, pushes new array
			if len(vm.stack) >= 2 {
				arr := vm.pop()
				val := vm.pop()
				switch a := arr.(type) {
				case []interface{}:
					newArr := make([]interface{}, len(a)+1)
					copy(newArr, a)
					newArr[len(a)] = val
					vm.stack = append(vm.stack, newArr)
				default:
					vm.stack = append(vm.stack, []interface{}{val})
				}
			}

		case OpArrayPop:
			// Stack: [..., array] — pops array, pushes popped_value then new_array on top
			if len(vm.stack) > 0 {
				arr := vm.pop()
				switch a := arr.(type) {
				case []interface{}:
					if len(a) == 0 {
						vm.stack = append(vm.stack, nil)
						vm.stack = append(vm.stack, []interface{}{})
					} else {
						popped := a[len(a)-1]
						newArr := make([]interface{}, len(a)-1)
						copy(newArr, a[:len(a)-1])
						vm.stack = append(vm.stack, popped)
						vm.stack = append(vm.stack, newArr)
					}
				default:
					vm.stack = append(vm.stack, nil)
					vm.stack = append(vm.stack, []interface{}{})
				}
			}

		case OpTryBegin:
			// Register catch block offset
			if len(instr.Args) > 0 {
				catchOffset := instr.Args[0].(int)
				catchPC := vm.pc + catchOffset
				vm.tryStack = append(vm.tryStack, catchPC)
			}

		case OpTryEnd:
			// Clear error handler
			if len(vm.tryStack) > 0 {
				vm.tryStack = vm.tryStack[:len(vm.tryStack)-1]
			}

		case OpDefer:
			// Register deferred code start and jump over the block
			if len(instr.Args) > 0 {
				jumpDist := instr.Args[0].(int)
				deferredPC := vm.pc + 1 // first instruction of deferred block
				depth := len(vm.callStack)
				for len(vm.deferredPCs) <= depth {
					vm.deferredPCs = append(vm.deferredPCs, []int{})
				}
				vm.deferredPCs[depth] = append(vm.deferredPCs[depth], deferredPC)
				vm.pc += jumpDist // jump over deferred block
			}

		case OpThrow:
			var errVal interface{}
			if len(vm.stack) > 0 {
				errVal = vm.pop()
			}
			if len(vm.tryStack) > 0 {
				catchPC := vm.tryStack[len(vm.tryStack)-1]
				vm.tryStack = vm.tryStack[:len(vm.tryStack)-1]
				vm.stack = append(vm.stack, errVal)
				vm.pc = catchPC - 1
			} else {
				return fmt.Errorf("unhandled error: %v", errVal)
			}

		case OpMemberAccess:
			// Member access: get a member from object
			if len(instr.Args) > 0 && len(vm.stack) > 0 {
				memberIdx := instr.Args[0].(int)
				member := vm.program.Constants[memberIdx].(string)
				obj := vm.pop()

				// Handle array methods and properties
				if arr, ok := obj.([]interface{}); ok {
					switch member {
					case "len":
						// Return array length as int (normalised from int64)
						vm.stack = append(vm.stack, int(len(arr)))
					case "push":
						// Return a method reference for arr.push(val)
						vm.stack = append(vm.stack, map[string]interface{}{"_method": "push", "_array": arr})
					case "pop":
						// Return a method reference for arr.pop()
						vm.stack = append(vm.stack, map[string]interface{}{"_method": "pop", "_array": arr})
					default:
						return fmt.Errorf("array has no method: %s", member)
					}
				} else if structVal, ok := obj.(map[string]interface{}); ok {
					// Handle struct field access
					if val, ok := structVal[member]; ok {
						vm.stack = append(vm.stack, val)
					} else {
						return fmt.Errorf("struct has no field: %s", member)
					}
				} else if obj == nil {
					return fmt.Errorf("cannot access member '%s' of nil", member)
				} else {
					return fmt.Errorf("cannot access member of type %T", obj)
				}
			}

		case OpStructCreate:
			// Create struct from stack values
			if len(instr.Args) >= 2 {
				structIdx := instr.Args[0].(int)
				fieldCount := instr.Args[1].(int)
				structName := vm.program.Constants[structIdx].(string)

				// Pop field values and names from stack
				fields := make(map[string]interface{})
				for i := 0; i < fieldCount; i++ {
					if len(vm.stack) < 2 {
						return fmt.Errorf("not enough values on stack for struct creation")
					}
					fieldName := vm.pop().(string)
					fieldValue := vm.pop()
					fields[fieldName] = fieldValue
				}

				// Create struct as a map with metadata
				structData := map[string]interface{}{
					"_type": structName,
				}
				for k, v := range fields {
					structData[k] = v
				}

				vm.stack = append(vm.stack, structData)
			}

		case OpEnumVariant:
			// Create enum variant
			if len(instr.Args) >= 2 {
				enumIdx := instr.Args[0].(int)
				variantIdx := instr.Args[1].(int)
				enumName := vm.program.Constants[enumIdx].(string)
				variantName := vm.program.Constants[variantIdx].(string)

				// Only pop a value from the stack if the compiler emitted one
				var value interface{} = nil
				hasValue := false
				if len(instr.Args) >= 3 {
					if hv, ok := instr.Args[2].(bool); ok {
						hasValue = hv
					}
				}
				if hasValue && len(vm.stack) > 0 {
					value = vm.pop()
				}

				// Create enum variant as a map with metadata
				variantData := map[string]interface{}{
					"_type":    enumName,
					"_variant": variantName,
					"_value":   value,
				}

				vm.stack = append(vm.stack, variantData)
			}

		case OpLoad:
			// Load local variable
			if len(instr.Args) > 0 {
				varIdx := instr.Args[0].(int)
				found := false
				if len(vm.locals) > 0 {
					// Search through local scopes from innermost to outermost
					for i := len(vm.locals) - 1; i >= 0; i-- {
						if val, ok := vm.locals[i][fmt.Sprintf("_var_%d", varIdx)]; ok {
							vm.stack = append(vm.stack, val)
							found = true
							break
						}
					}
				}
				if !found {
					// Local not found — push nil to avoid stack corruption
					vm.stack = append(vm.stack, nil)
				}
			}

		case OpStore:
			// Store to local variable
			if len(instr.Args) > 0 && len(vm.stack) > 0 && len(vm.locals) > 0 {
				varIdx := instr.Args[0].(int)
				val := vm.pop()
				vm.locals[len(vm.locals)-1][fmt.Sprintf("_var_%d", varIdx)] = val
			}

		case OpCall:
			// Call function (args: funcName string, argCount int)
			if len(instr.Args) >= 2 {
				funcName, _ := instr.Args[0].(string)
				argCount, _ := instr.Args[1].(int)

				if !vm.dispatchCall(funcName, argCount) {
					// Pop args for builtin call
					args := make([]interface{}, argCount)
					for i := argCount - 1; i >= 0; i-- {
						if len(vm.stack) > 0 {
							args[i] = vm.pop()
						}
					}
					result := vm.callBuiltin(funcName, args)
					vm.stack = append(vm.stack, result)
				}
			}

		case OpCallDynamic:
			// Call a closure/function whose reference is on the stack (below the args).
			// Stack layout (bottom→top based on push order): fn_ref, arg0..argN-1
			argCount := 0
			if len(instr.Args) > 0 {
				argCount, _ = instr.Args[0].(int)
			}
			// Pop args first (on top), then pop fn_ref
			args := make([]interface{}, argCount)
			for i := argCount - 1; i >= 0; i-- {
				if len(vm.stack) > 0 {
					args[i] = vm.pop()
				}
			}
			fnRef := vm.pop()
			// Re-push args so dispatchCall can pop them
			for _, a := range args {
				vm.stack = append(vm.stack, a)
			}
			if !vm.dispatchCall(fnRef, argCount) {
				// Fallback builtin / unknown
				if fnName, isStr := fnRef.(string); isStr {
					// rebuild args (already popped)
					a2 := make([]interface{}, argCount)
					for i := argCount - 1; i >= 0; i-- {
						if len(vm.stack) > 0 {
							a2[i] = vm.pop()
						}
					}
					result := vm.callBuiltin(fnName, a2)
					vm.stack = append(vm.stack, result)
				} else {
					// pop the re-pushed args and push nil
					for i := 0; i < argCount; i++ {
						if len(vm.stack) > 0 {
							vm.pop()
						}
					}
					vm.stack = append(vm.stack, nil)
				}
			}

		case OpMakeClosure:
			// Create a ClosureValue capturing the top `captureCount` stack values.
			// Args: [funcName string, captureCount int]
			funcName, _ := instr.Args[0].(string)
			captureCount := 0
			if len(instr.Args) > 1 {
				captureCount, _ = instr.Args[1].(int)
			}
			captured := make([]interface{}, captureCount)
			for i := captureCount - 1; i >= 0; i-- {
				if len(vm.stack) > 0 {
					captured[i] = vm.pop()
				}
			}
			vm.stack = append(vm.stack, ClosureValue{FuncName: funcName, Captured: captured})

		case OpMethodCall:
			// Call method on a receiver value.
			// Args: [methodName string, argCount int]
			// Stack (bottom→top): receiver, arg0, arg1 …  argN-1
			methodName, _ := instr.Args[0].(string)
			argCount := 0
			if len(instr.Args) > 1 {
				argCount, _ = instr.Args[1].(int)
			}
			// Pop args
			args := make([]interface{}, argCount)
			for i := argCount - 1; i >= 0; i-- {
				if len(vm.stack) > 0 {
					args[i] = vm.pop()
				}
			}
			// Pop receiver
			receiver := vm.pop()
			result := vm.callMethod(receiver, methodName, args)
			vm.stack = append(vm.stack, result)

		case OpReturn:
			// Execute deferred calls for this frame
			vm.runDeferred(len(vm.callStack))
			// Return without value - pop frame
			if len(vm.callStack) > 0 {
				frame := vm.callStack[len(vm.callStack)-1]
				vm.callStack = vm.callStack[:len(vm.callStack)-1]
				if len(vm.locals) > 0 {
					vm.locals = vm.locals[:len(vm.locals)-1]
				}
				vm.pc = frame.pc - 1 // -1 because loop will increment
			} else {
				// Top-level return
				return nil
			}

		case OpReturnValue:
			// Execute deferred calls for this frame
			vm.runDeferred(len(vm.callStack))
			// Return with value - value is already on stack
			if len(vm.callStack) > 0 {
				frame := vm.callStack[len(vm.callStack)-1]
				vm.callStack = vm.callStack[:len(vm.callStack)-1]
				if len(vm.locals) > 0 {
					vm.locals = vm.locals[:len(vm.locals)-1]
				}
				vm.pc = frame.pc - 1 // -1 because loop will increment
			} else {
				// Top-level return
				return nil
			}

		case OpNoop:
			// No operation

		case OpBreak:
			// Break: jump to target PC stored in the loop context
			// With bounds checking to prevent invalid jumps
			if len(instr.Args) > 0 {
				offset := instr.Args[0].(int)
				newPC := vm.pc + offset - 1 // -1 because loop will increment
				if newPC >= 0 && newPC < len(vm.program.Instructions) {
					vm.pc = newPC
				} else {
					// Invalid offset; just continue to next instruction
					// (Proper fix requires compiler to emit correct loop targets)
				}
			}
		case OpContinue:
			// Continue: jump to loop start with bounds checking
			if len(instr.Args) > 0 {
				offset := instr.Args[0].(int)
				newPC := vm.pc + offset - 1 // -1 because loop will increment
				if newPC >= 0 && newPC < len(vm.program.Instructions) {
					vm.pc = newPC
				} else {
					// Invalid offset; just continue to next instruction
					// (Proper fix requires compiler to emit correct loop targets)
				}
			}
		default:
			return fmt.Errorf("unknown opcode: %d", instr.Op)
		}
	}

	return nil
}

// Helper functions
func (vm *VM) pop() interface{} {
	if len(vm.stack) > 0 {
		val := vm.stack[len(vm.stack)-1]
		vm.stack = vm.stack[:len(vm.stack)-1]
		return val
	}
	return nil
}

// runDeferred executes all deferred calls registered at the given call depth
func (vm *VM) runDeferred(depth int) {
	if depth >= len(vm.deferredPCs) {
		return
	}
	deferred := vm.deferredPCs[depth]
	// Execute in LIFO order
	for i := len(deferred) - 1; i >= 0; i-- {
		savedPC := vm.pc
		vm.pc = deferred[i]
		// Run until we hit OpNoop (end of deferred block) or OpReturn
		for vm.pc < len(vm.program.Instructions) {
			instr := vm.program.Instructions[vm.pc]
			if instr.Op == OpNoop {
				break
			}
			// Execute instruction (simplified - just advance PC for now)
			vm.pc++
		}
		vm.pc = savedPC
	}
	vm.deferredPCs[depth] = nil
}

// callFunction sets up a new call frame and jumps to entryPC.
// Arguments must already be on the stack (argCount of them, pushed in order).
// extraArgs are appended after the popped args (used for closure captures).
func (vm *VM) callFunction(entryPC, argCount int, extraArgs ...interface{}) {
	// Pop arguments from stack in reverse order (last arg is on top)
	args := make([]interface{}, argCount)
	for i := argCount - 1; i >= 0; i-- {
		if len(vm.stack) > 0 {
			args[i] = vm.pop()
		}
	}
	// Append captured values (for closures)
	allArgs := args
	if len(extraArgs) > 0 {
		allArgs = append(args, extraArgs...)
	}
	// Push a new call frame
	frame := CallFrame{
		pc:     vm.pc + 1, // return address: instruction after OpCall/OpCallDynamic
		bp:     vm.bp,
		locals: make(map[string]interface{}),
	}
	vm.callStack = append(vm.callStack, frame)
	// Set up locals for the new scope
	newLocals := make(map[string]interface{})
	for i, arg := range allArgs {
		newLocals[fmt.Sprintf("_var_%d", i)] = arg
	}
	vm.locals = append(vm.locals, newLocals)
	vm.bp = len(vm.stack)
	// Jump (loop will increment, so subtract 1)
	vm.pc = entryPC - 1
}

// dispatchCall handles calling a value (string funcName or ClosureValue) with argCount stack args.
// Returns true if call was dispatched internally (no builtin fallback needed).
func (vm *VM) dispatchCall(callee interface{}, argCount int) bool {
	switch cv := callee.(type) {
	case ClosureValue:
		if entryPC, ok := vm.program.Functions[cv.FuncName]; ok {
			vm.callFunction(entryPC, argCount, cv.Captured...)
			return true
		}
	case string:
		// Try to resolve as a global closure variable first
		if ref, exists := vm.globals[cv]; exists {
			if inner, ok := ref.(ClosureValue); ok {
				if entryPC, ok2 := vm.program.Functions[inner.FuncName]; ok2 {
					vm.callFunction(entryPC, argCount, inner.Captured...)
					return true
				}
			}
			// Also handle: global var holds a plain string function name (no-capture closure)
			if innerName, ok := ref.(string); ok {
				if entryPC, ok2 := vm.program.Functions[innerName]; ok2 {
					vm.callFunction(entryPC, argCount)
					return true
				}
			}
		}
		if entryPC, ok := vm.program.Functions[cv]; ok {
			vm.callFunction(entryPC, argCount)
			return true
		}
	}
	return false
}

// tryHandleError attempts to route a runtime error to the nearest catch block.
// Returns true if the error was caught (vm.pc is set to catch PC - 1).
func (vm *VM) tryHandleError(err error) bool {
	if len(vm.tryStack) > 0 {
		catchPC := vm.tryStack[len(vm.tryStack)-1]
		vm.tryStack = vm.tryStack[:len(vm.tryStack)-1]
		vm.stack = append(vm.stack, err.Error())
		vm.pc = catchPC - 1 // will be incremented by the loop
		return true
	}
	return false
}

// Type checking helpers
func isInt(v interface{}) bool {
	_, ok := v.(int)
	return ok
}

func isFloat(v interface{}) bool {
	_, ok := v.(float64)
	return ok
}

func asInt(v interface{}) int {
	if i, ok := v.(int); ok {
		return i
	}
	if f, ok := v.(float64); ok {
		return int(f)
	}
	return 0
}

func asFloat(v interface{}) float64 {
	if f, ok := v.(float64); ok {
		return f
	}
	if i, ok := v.(int); ok {
		return float64(i)
	}
	return 0
}

func (vm *VM) add(left, right interface{}) interface{} {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			return l + r
		case float64:
			return float64(l) + r
		}
	case float64:
		switch r := right.(type) {
		case int:
			return l + float64(r)
		case float64:
			return l + r
		}
	case string:
		return l + vm.valueToString(right)
	}
	return nil
}

func (vm *VM) subtract(left, right interface{}) interface{} {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			return l - r
		case float64:
			return float64(l) - r
		}
	case float64:
		switch r := right.(type) {
		case int:
			return l - float64(r)
		case float64:
			return l - r
		}
	}
	return nil
}

func (vm *VM) multiply(left, right interface{}) interface{} {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			return l * r
		case float64:
			return float64(l) * r
		}
	case float64:
		switch r := right.(type) {
		case int:
			return l * float64(r)
		case float64:
			return l * r
		}
	}
	return nil
}

func (vm *VM) divide(left, right interface{}) interface{} {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			if r == 0 {
				return 0
			}
			return l / r
		case float64:
			if r == 0 {
				return 0.0
			}
			return float64(l) / r
		}
	case float64:
		switch r := right.(type) {
		case int:
			if r == 0 {
				return 0.0
			}
			return l / float64(r)
		case float64:
			if r == 0 {
				return 0.0
			}
			return l / r
		}
	}
	return nil
}

func (vm *VM) modulo(left, right interface{}) interface{} {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			if r == 0 {
				return 0
			}
			return l % r
		}
	}
	return nil
}

func (vm *VM) equal(left, right interface{}) bool {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			return l == r
		case int64:
			return int64(l) == r
		case float64:
			return float64(l) == r
		}
	case int64:
		switch r := right.(type) {
		case int:
			return l == int64(r)
		case int64:
			return l == r
		case float64:
			return float64(l) == r
		}
	case float64:
		switch r := right.(type) {
		case int:
			return l == float64(r)
		case int64:
			return l == float64(r)
		case float64:
			return l == r
		}
	case string:
		if r, ok := right.(string); ok {
			return l == r
		}
	case bool:
		if r, ok := right.(bool); ok {
			return l == r
		}
	}
	return false
}

func (vm *VM) lessThan(left, right interface{}) bool {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			return l < r
		case int64:
			return int64(l) < r
		case float64:
			return float64(l) < r
		}
	case int64:
		switch r := right.(type) {
		case int:
			return l < int64(r)
		case int64:
			return l < r
		case float64:
			return float64(l) < r
		}
	case float64:
		switch r := right.(type) {
		case int:
			return l < float64(r)
		case int64:
			return l < float64(r)
		case float64:
			return l < r
		}
	}
	return false
}

func (vm *VM) isTruthy(val interface{}) bool {
	switch v := val.(type) {
	case bool:
		return v
	case int:
		return v != 0
	case float64:
		return v != 0
	case string:
		return v != ""
	case nil:
		return false
	default:
		return true
	}
}

func (vm *VM) valueToString(val interface{}) string {
	switch v := val.(type) {
	case int:
		return strconv.FormatInt(int64(v), 10)
	case float64:
		return strconv.FormatFloat(v, 'f', -1, 64)
	case string:
		return v
	case bool:
		if v {
			return "true"
		}
		return "false"
	case nil:
		return ""
	default:
		return fmt.Sprintf("%v", v)
	}
}

func (vm *VM) bitwiseAnd(left, right interface{}) interface{} {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			return l & r
		}
	}
	return nil
}

func (vm *VM) bitwiseOr(left, right interface{}) interface{} {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			return l | r
		}
	}
	return nil
}

func (vm *VM) bitwiseXor(left, right interface{}) interface{} {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			return l ^ r
		}
	}
	return nil
}

func (vm *VM) bitwiseNot(val interface{}) interface{} {
	switch v := val.(type) {
	case int:
		return ^v
	}
	return nil
}

func (vm *VM) bitwiseShl(left, right interface{}) interface{} {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			return l << uint(r)
		}
	}
	return nil
}

func (vm *VM) bitwiseShr(left, right interface{}) interface{} {
	switch l := left.(type) {
	case int:
		switch r := right.(type) {
		case int:
			return l >> uint(r)
		}
	}
	return nil
}

func (vm *VM) callBuiltin(name string, args []interface{}) interface{} {
	switch name {

	// ── Output ─────────────────────────────────────────────────────────────
	case "print":
		for _, arg := range args {
			fmt.Println(vm.valueToString(arg))
		}
		return nil
	case "println":
		parts := make([]string, len(args))
		for i, a := range args {
			parts[i] = vm.valueToString(a)
		}
		fmt.Println(strings.Join(parts, " "))
		return nil
	case "printf":
		if len(args) > 0 {
			if fmtStr, ok := args[0].(string); ok {
				iArgs := make([]interface{}, len(args)-1)
				copy(iArgs, args[1:])
				fmt.Printf(fmtStr, iArgs...)
			}
		}
		return nil

	// ── Type inspection ────────────────────────────────────────────────────
	case "type":
		if len(args) > 0 {
			switch args[0].(type) {
			case int:
				return "int"
			case float64:
				return "float"
			case bool:
				return "bool"
			case string:
				return "string"
			case []interface{}:
				return "array"
			case map[string]interface{}:
				return "struct"
			case ClosureValue:
				return "function"
			case nil:
				return "nil"
			default:
				return fmt.Sprintf("%T", args[0])
			}
		}
		return "nil"

	// ── Type conversions ──────────────────────────────────────────────────
	case "int":
		if len(args) > 0 {
			switch v := args[0].(type) {
			case int:
				return v
			case float64:
				return int(v)
			case string:
				if i, err := strconv.ParseInt(strings.TrimSpace(v), 10, 64); err == nil {
					return int(i)
				}
				if f, err := strconv.ParseFloat(strings.TrimSpace(v), 64); err == nil {
					return int(f)
				}
			case bool:
				if v {
					return 1
				}
				return 0
			}
		}
		return 0
	case "float":
		if len(args) > 0 {
			switch v := args[0].(type) {
			case int:
				return float64(v)
			case float64:
				return v
			case string:
				if f, err := strconv.ParseFloat(strings.TrimSpace(v), 64); err == nil {
					return f
				}
			case bool:
				if v {
					return 1.0
				}
				return 0.0
			}
		}
		return 0.0
	case "string":
		if len(args) > 0 {
			return vm.valueToString(args[0])
		}
		return ""
	case "bool":
		if len(args) > 0 {
			return vm.isTruthy(args[0])
		}
		return false

	// ── Lengths ────────────────────────────────────────────────────────────
	case "len":
		if len(args) > 0 {
			switch v := args[0].(type) {
			case string:
				return len(v)
			case []interface{}:
				return len(v)
			}
		}
		return 0

	// ── Range ─────────────────────────────────────────────────────────────
	case "range":
		switch len(args) {
		case 1:
			n := vm.toInt(args[0])
			out := make([]interface{}, n)
			for i := 0; i < n; i++ {
				out[i] = i
			}
			return out
		case 2:
			start, end := vm.toInt(args[0]), vm.toInt(args[1])
			out := make([]interface{}, 0, end-start)
			for i := start; i < end; i++ {
				out = append(out, i)
			}
			return out
		case 3:
			start, end, step := vm.toInt(args[0]), vm.toInt(args[1]), vm.toInt(args[2])
			if step == 0 {
				step = 1
			}
			var out []interface{}
			for i := start; step > 0 && i < end || step < 0 && i > end; i += step {
				out = append(out, i)
			}
			return out
		}
		return []interface{}{}

	// ── Math (flat names) ────────────────────────────────────────────────
	case "abs", "math.abs":
		if len(args) > 0 {
			switch v := args[0].(type) {
			case int:
				if v < 0 {
					return -v
				}
				return v
			case float64:
				return math.Abs(v)
			}
		}
		return 0
	case "sqrt", "math.sqrt":
		if len(args) > 0 {
			return math.Sqrt(vm.toFloat(args[0]))
		}
		return 0.0
	case "pow", "math.pow":
		if len(args) >= 2 {
			return math.Pow(vm.toFloat(args[0]), vm.toFloat(args[1]))
		}
		return 1.0
	case "floor", "math.floor":
		if len(args) > 0 {
			return int(math.Floor(vm.toFloat(args[0])))
		}
		return 0
	case "ceil", "math.ceil":
		if len(args) > 0 {
			return int(math.Ceil(vm.toFloat(args[0])))
		}
		return 0
	case "round", "math.round":
		if len(args) > 0 {
			return int(math.Round(vm.toFloat(args[0])))
		}
		return 0
	case "min", "math.min":
		if len(args) >= 2 {
			a, b := vm.toFloat(args[0]), vm.toFloat(args[1])
			if a < b {
				return a
			}
			return b
		}
		return 0
	case "max", "math.max":
		if len(args) >= 2 {
			a, b := vm.toFloat(args[0]), vm.toFloat(args[1])
			if a > b {
				return a
			}
			return b
		}
		return 0
	case "random", "math.random":
		return rand.Float64()
	case "math.pi":
		return math.Pi
	case "math.e":
		return math.E

	// ── String helpers (flat names) ──────────────────────────────────────
	case "split":
		if len(args) >= 2 {
			s := vm.valueToString(args[0])
			sep := vm.valueToString(args[1])
			parts := strings.Split(s, sep)
			out := make([]interface{}, len(parts))
			for i, p := range parts {
				out[i] = p
			}
			return out
		}
		return []interface{}{}
	case "trim":
		if len(args) > 0 {
			return strings.TrimSpace(vm.valueToString(args[0]))
		}
		return ""
	case "upper":
		if len(args) > 0 {
			return strings.ToUpper(vm.valueToString(args[0]))
		}
		return ""
	case "lower":
		if len(args) > 0 {
			return strings.ToLower(vm.valueToString(args[0]))
		}
		return ""
	case "contains":
		if len(args) >= 2 {
			return strings.Contains(vm.valueToString(args[0]), vm.valueToString(args[1]))
		}
		return false
	case "startsWith":
		if len(args) >= 2 {
			return strings.HasPrefix(vm.valueToString(args[0]), vm.valueToString(args[1]))
		}
		return false
	case "endsWith":
		if len(args) >= 2 {
			return strings.HasSuffix(vm.valueToString(args[0]), vm.valueToString(args[1]))
		}
		return false
	case "replace":
		if len(args) >= 3 {
			return strings.ReplaceAll(vm.valueToString(args[0]), vm.valueToString(args[1]), vm.valueToString(args[2]))
		}
		return ""
	case "indexOf":
		if len(args) >= 2 {
			return strings.Index(vm.valueToString(args[0]), vm.valueToString(args[1]))
		}
		return -1
	case "repeat":
		if len(args) >= 2 {
			return strings.Repeat(vm.valueToString(args[0]), vm.toInt(args[1]))
		}
		return ""

	// ── File I/O ──────────────────────────────────────────────────────────
	case "readFile":
		if len(args) > 0 {
			path := vm.valueToString(args[0])
			data, err := os.ReadFile(path)
			if err != nil {
				return nil
			}
			return string(data)
		}
		return nil
	case "writeFile":
		if len(args) >= 2 {
			path := vm.valueToString(args[0])
			content := vm.valueToString(args[1])
			err := os.WriteFile(path, []byte(content), 0644)
			return err == nil
		}
		return false
	case "appendFile":
		if len(args) >= 2 {
			path := vm.valueToString(args[0])
			content := vm.valueToString(args[1])
			f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
			if err != nil {
				return false
			}
			defer f.Close()
			_, err = f.WriteString(content)
			return err == nil
		}
		return false
	case "fileExists":
		if len(args) > 0 {
			_, err := os.Stat(vm.valueToString(args[0]))
			return err == nil
		}
		return false
	case "readDir":
		if len(args) > 0 {
			entries, err := os.ReadDir(vm.valueToString(args[0]))
			if err != nil {
				return []interface{}{}
			}
			out := make([]interface{}, len(entries))
			for i, e := range entries {
				out[i] = e.Name()
			}
			return out
		}
		return []interface{}{}
	case "joinPath":
		parts := make([]string, len(args))
		for i, a := range args {
			parts[i] = vm.valueToString(a)
		}
		return filepath.Join(parts...)

	// ── Array helpers (flat names) ────────────────────────────────────────
	case "sort":
		if len(args) > 0 {
			if arr, ok := args[0].([]interface{}); ok {
				return vm.sortArray(arr)
			}
		}
		return []interface{}{}
	case "reverse":
		if len(args) > 0 {
			if arr, ok := args[0].([]interface{}); ok {
				cp := make([]interface{}, len(arr))
				copy(cp, arr)
				for i, j := 0, len(cp)-1; i < j; i, j = i+1, j-1 {
					cp[i], cp[j] = cp[j], cp[i]
				}
				return cp
			}
		}
		return []interface{}{}
	case "join":
		if len(args) >= 2 {
			if arr, ok := args[0].([]interface{}); ok {
				sep := vm.valueToString(args[1])
				parts := make([]string, len(arr))
				for i, v := range arr {
					parts[i] = vm.valueToString(v)
				}
				return strings.Join(parts, sep)
			}
		}
		return ""
	case "keys":
		if len(args) > 0 {
			if m, ok := args[0].(map[string]interface{}); ok {
				ks := make([]interface{}, 0, len(m))
				for k := range m {
					ks = append(ks, k)
				}
				return ks
			}
		}
		return []interface{}{}
	case "values":
		if len(args) > 0 {
			if m, ok := args[0].(map[string]interface{}); ok {
				vs := make([]interface{}, 0, len(m))
				for _, v := range m {
					vs = append(vs, v)
				}
				return vs
			}
		}
		return []interface{}{}
	case "has":
		if len(args) >= 2 {
			if m, ok := args[0].(map[string]interface{}); ok {
				_, exists := m[vm.valueToString(args[1])]
				return exists
			}
			if arr, ok := args[0].([]interface{}); ok {
				needle := args[1]
				for _, v := range arr {
					if fmt.Sprintf("%v", v) == fmt.Sprintf("%v", needle) {
						return true
					}
				}
				return false
			}
		}
		return false

	default:
		return nil
	}
}

// callMethod dispatches a method call on a receiver value (string, array, or math namespace).
func (vm *VM) callMethod(receiver interface{}, method string, args []interface{}) interface{} {
	switch rv := receiver.(type) {

	case string:
		switch method {
		case "len":
			return len(rv)
		case "upper":
			return strings.ToUpper(rv)
		case "lower":
			return strings.ToLower(rv)
		case "trim":
			return strings.TrimSpace(rv)
		case "trimLeft":
			if len(args) > 0 {
				return strings.TrimLeft(rv, vm.valueToString(args[0]))
			}
			return strings.TrimLeft(rv, " \t\n")
		case "trimRight":
			if len(args) > 0 {
				return strings.TrimRight(rv, vm.valueToString(args[0]))
			}
			return strings.TrimRight(rv, " \t\n")
		case "split":
			sep := ""
			if len(args) > 0 {
				sep = vm.valueToString(args[0])
			}
			parts := strings.Split(rv, sep)
			out := make([]interface{}, len(parts))
			for i, p := range parts {
				out[i] = p
			}
			return out
		case "contains":
			if len(args) > 0 {
				return strings.Contains(rv, vm.valueToString(args[0]))
			}
			return false
		case "startsWith", "hasPrefix":
			if len(args) > 0 {
				return strings.HasPrefix(rv, vm.valueToString(args[0]))
			}
			return false
		case "endsWith", "hasSuffix":
			if len(args) > 0 {
				return strings.HasSuffix(rv, vm.valueToString(args[0]))
			}
			return false
		case "replace":
			if len(args) >= 2 {
				return strings.ReplaceAll(rv, vm.valueToString(args[0]), vm.valueToString(args[1]))
			}
			return rv
		case "indexOf", "index":
			if len(args) > 0 {
				return strings.Index(rv, vm.valueToString(args[0]))
			}
			return -1
		case "repeat":
			if len(args) > 0 {
				return strings.Repeat(rv, vm.toInt(args[0]))
			}
			return rv
		case "slice", "substr":
			if len(args) >= 2 {
				start, end := vm.toInt(args[0]), vm.toInt(args[1])
				runes := []rune(rv)
				n := len(runes)
				if start < 0 {
					start = 0
				}
				if end > n {
					end = n
				}
				return string(runes[start:end])
			} else if len(args) == 1 {
				start := vm.toInt(args[0])
				runes := []rune(rv)
				if start < 0 || start >= len(runes) {
					return ""
				}
				return string(runes[start:])
			}
			return rv
		case "toInt":
			if i, err := strconv.ParseInt(strings.TrimSpace(rv), 10, 64); err == nil {
				return int(i)
			}
			return 0
		case "toFloat":
			if f, err := strconv.ParseFloat(strings.TrimSpace(rv), 64); err == nil {
				return f
			}
			return 0.0
		case "chars":
			runes := []rune(rv)
			out := make([]interface{}, len(runes))
			for i, r := range runes {
				out[i] = string(r)
			}
			return out
		}

	case []interface{}:
		switch method {
		case "len":
			return len(rv)
		case "contains":
			if len(args) > 0 {
				needle := fmt.Sprintf("%v", args[0])
				for _, v := range rv {
					if fmt.Sprintf("%v", v) == needle {
						return true
					}
				}
				return false
			}
			return false
		case "join":
			sep := ""
			if len(args) > 0 {
				sep = vm.valueToString(args[0])
			}
			parts := make([]string, len(rv))
			for i, v := range rv {
				parts[i] = vm.valueToString(v)
			}
			return strings.Join(parts, sep)
		case "sort":
			return vm.sortArray(rv)
		case "reverse":
			cp := make([]interface{}, len(rv))
			copy(cp, rv)
			for i, j := 0, len(cp)-1; i < j; i, j = i+1, j-1 {
				cp[i], cp[j] = cp[j], cp[i]
			}
			return cp
		case "slice":
			if len(args) >= 2 {
				start, end := vm.toInt(args[0]), vm.toInt(args[1])
				if start < 0 {
					start = 0
				}
				if end > len(rv) {
					end = len(rv)
				}
				cp := make([]interface{}, end-start)
				copy(cp, rv[start:end])
				return cp
			} else if len(args) == 1 {
				start := vm.toInt(args[0])
				if start < 0 || start > len(rv) {
					return []interface{}{}
				}
				cp := make([]interface{}, len(rv)-start)
				copy(cp, rv[start:])
				return cp
			}
			return rv
		case "map":
			if len(args) > 0 {
				result := make([]interface{}, len(rv))
				for i, v := range rv {
					vm.stack = append(vm.stack, v)
					if vm.dispatchCall(args[0], 1) {
						// dispatchCall set up call frame; result will be on stack after return
						// We need a different approach: run the sub-call synchronously
						// For simplicity, use callBuiltinFn if it's a string
						result[i] = vm.callBuiltin("__noop__", nil)
					}
					// Simpler: if args[0] is a ClosureValue or string, call it directly
					result[i] = vm.applyFn(args[0], []interface{}{v})
				}
				return result
			}
			return rv
		case "filter":
			if len(args) > 0 {
				var result []interface{}
				for _, v := range rv {
					res := vm.applyFn(args[0], []interface{}{v})
					if vm.isTruthy(res) {
						result = append(result, v)
					}
				}
				if result == nil {
					return []interface{}{}
				}
				return result
			}
			return rv
		case "reduce":
			if len(args) >= 2 {
				acc := args[1]
				for _, v := range rv {
					acc = vm.applyFn(args[0], []interface{}{acc, v})
				}
				return acc
			}
			return nil
		case "indexOf", "index":
			if len(args) > 0 {
				needle := fmt.Sprintf("%v", args[0])
				for i, v := range rv {
					if fmt.Sprintf("%v", v) == needle {
						return i
					}
				}
				return -1
			}
			return -1
		case "first":
			if len(rv) > 0 {
				return rv[0]
			}
			return nil
		case "last":
			if len(rv) > 0 {
				return rv[len(rv)-1]
			}
			return nil
		case "flat":
			var result []interface{}
			for _, v := range rv {
				if inner, ok := v.([]interface{}); ok {
					result = append(result, inner...)
				} else {
					result = append(result, v)
				}
			}
			if result == nil {
				return []interface{}{}
			}
			return result
		}

	// math namespace (receiver is the string "math" used as an identifier)
	case nil:
		// method on nil — return nil

	}

	// Fallback: try calling as a builtin with the method name
	allArgs := append([]interface{}{receiver}, args...)
	return vm.callBuiltin(method, allArgs)
}

// applyFn calls a function (ClosureValue or string name) with the given args
// synchronously by temporarily running the VM. Used for map/filter/reduce.
func (vm *VM) applyFn(fn interface{}, args []interface{}) interface{} {
	for _, a := range args {
		vm.stack = append(vm.stack, a)
	}
	origPC := vm.pc
	handled := vm.dispatchCall(fn, len(args))
	if !handled {
		for i := 0; i < len(args); i++ {
			if len(vm.stack) > 0 {
				vm.pop()
			}
		}
		return nil
	}
	// Run until we return from the called frame (depth decreases)
	targetDepth := len(vm.callStack)
	for vm.pc < len(vm.program.Instructions) && len(vm.callStack) >= targetDepth {
		instr := vm.program.Instructions[vm.pc]
		if instr.Op == OpReturnValue || instr.Op == OpReturn {
			vm.runDeferred(len(vm.callStack))
			if len(vm.callStack) > 0 {
				frame := vm.callStack[len(vm.callStack)-1]
				vm.callStack = vm.callStack[:len(vm.callStack)-1]
				if len(vm.locals) > 0 {
					vm.locals = vm.locals[:len(vm.locals)-1]
				}
				vm.pc = frame.pc - 1
			}
			break
		}
		// Run the instruction via the main dispatch
		_ = vm.execOne(instr)
		vm.pc++
	}
	_ = origPC
	if len(vm.stack) > 0 {
		return vm.pop()
	}
	return nil
}

// execOne runs a single instruction without incrementing the PC.
// Used by applyFn for inline sub-execution.
func (vm *VM) execOne(instr Instruction) error {
	// Minimal subset needed for map/filter/reduce callbacks
	switch instr.Op {
	case OpPush:
		if len(instr.Args) > 0 {
			idx := instr.Args[0].(int)
			vm.stack = append(vm.stack, vm.program.Constants[idx])
		}
	case OpLoad:
		if len(instr.Args) > 0 {
			varIdx := instr.Args[0].(int)
			if len(vm.locals) > 0 {
				localMap := vm.locals[len(vm.locals)-1]
				key := fmt.Sprintf("_var_%d", varIdx)
				if val, ok := localMap[key]; ok {
					vm.stack = append(vm.stack, val)
					return nil
				}
			}
			vm.stack = append(vm.stack, nil)
		}
	case OpAdd:
		if len(vm.stack) >= 2 {
			right := vm.pop()
			left := vm.pop()
			vm.stack = append(vm.stack, vm.add(left, right))
		}
	case OpSub:
		if len(vm.stack) >= 2 {
			right := vm.pop()
			left := vm.pop()
			vm.stack = append(vm.stack, vm.subtract(left, right))
		}
	case OpMul:
		if len(vm.stack) >= 2 {
			right := vm.pop()
			left := vm.pop()
			vm.stack = append(vm.stack, vm.multiply(left, right))
		}
	}
	return nil
}

// sortArray returns a sorted copy of an interface{} slice.
func (vm *VM) sortArray(arr []interface{}) []interface{} {
	cp := make([]interface{}, len(arr))
	copy(cp, arr)
	sort.Slice(cp, func(i, j int) bool {
		ai := vm.valueToString(cp[i])
		aj := vm.valueToString(cp[j])
		// Numeric sort if both are numbers
		if ni, ok1 := cp[i].(int); ok1 {
			if nj, ok2 := cp[j].(int); ok2 {
				return ni < nj
			}
		}
		if fi, ok1 := cp[i].(float64); ok1 {
			if fj, ok2 := cp[j].(float64); ok2 {
				return fi < fj
			}
		}
		return ai < aj
	})
	return cp
}

// toInt converts any value to int.
func (vm *VM) toInt(v interface{}) int {
	switch n := v.(type) {
	case int:
		return n
	case float64:
		return int(n)
	case string:
		if i, err := strconv.ParseInt(strings.TrimSpace(n), 10, 64); err == nil {
			return int(i)
		}
	case bool:
		if n {
			return 1
		}
	}
	return 0
}

// toFloat converts any value to float64.
func (vm *VM) toFloat(v interface{}) float64 {
	switch n := v.(type) {
	case int:
		return float64(n)
	case float64:
		return n
	case string:
		if f, err := strconv.ParseFloat(strings.TrimSpace(n), 64); err == nil {
			return f
		}
	case bool:
		if n {
			return 1.0
		}
	}
	return 0.0
}

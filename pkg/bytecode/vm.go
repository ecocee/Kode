package bytecode

import (
	"fmt"
	"strconv"
)

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

				// If name is not a direct function, check if a global variable
				// holds a closure reference (a string naming a compiled function).
				resolvedName := funcName
				if _, ok := vm.program.Functions[funcName]; !ok {
					if ref, exists := vm.globals[funcName]; exists {
						if nameStr, isStr := ref.(string); isStr {
							if _, isFn := vm.program.Functions[nameStr]; isFn {
								resolvedName = nameStr
							}
						}
					}
				}

				if entryPC, ok := vm.program.Functions[resolvedName]; ok {
					vm.callFunction(entryPC, argCount)
				} else {
					// Built-in / unknown
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
			// Call a closure/function whose name is on the stack.
			// Stack layout (bottom→top): fn_ref, arg0, arg1, ..., argN-1
			// instr.Args[0] = argCount
			argCount := 0
			if len(instr.Args) > 0 {
				argCount, _ = instr.Args[0].(int)
			}
			// Pop args first (they are on top), then pop the fn_ref below them.
			args := make([]interface{}, argCount)
			for i := argCount - 1; i >= 0; i-- {
				if len(vm.stack) > 0 {
					args[i] = vm.pop()
				}
			}
			fnRef := vm.pop() // the closure handle
			if fnName, isStr := fnRef.(string); isStr {
				if entryPC, ok := vm.program.Functions[fnName]; ok {
					// Re-push the args so callFunction can pop them
					for _, a := range args {
						vm.stack = append(vm.stack, a)
					}
					vm.callFunction(entryPC, argCount)
				} else {
					result := vm.callBuiltin(fnName, args)
					vm.stack = append(vm.stack, result)
				}
			} else {
				// Not callable — push nil
				vm.stack = append(vm.stack, nil)
			}

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
func (vm *VM) callFunction(entryPC, argCount int) {
	// Pop arguments from stack in reverse order (last arg is on top)
	args := make([]interface{}, argCount)
	for i := argCount - 1; i >= 0; i-- {
		if len(vm.stack) > 0 {
			args[i] = vm.pop()
		}
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
	for i, arg := range args {
		newLocals[fmt.Sprintf("_var_%d", i)] = arg
	}
	vm.locals = append(vm.locals, newLocals)
	vm.bp = len(vm.stack)
	// Jump (loop will increment, so subtract 1)
	vm.pc = entryPC - 1
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
	case "print":
		for _, arg := range args {
			fmt.Println(vm.valueToString(arg))
		}
		return nil
	case "len":
		if len(args) > 0 {
			switch v := args[0].(type) {
			case string:
				return int(len(v))
			case []interface{}:
				return int(len(v))
			}
		}
		return 0
	case "int":
		if len(args) > 0 {
			switch v := args[0].(type) {
			case int:
				return v
			case float64:
				return int(v)
			case string:
				if i, err := strconv.ParseInt(v, 10, 64); err == nil {
					return int(i)
				}
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
				if f, err := strconv.ParseFloat(v, 64); err == nil {
					return f
				}
			}
		}
		return 0.0
	case "string":
		if len(args) > 0 {
			return vm.valueToString(args[0])
		}
		return ""
	default:
		return nil
	}
}

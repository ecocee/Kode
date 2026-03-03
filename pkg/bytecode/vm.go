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
	program   *Program
	stack     []interface{}
	globals   map[string]interface{}
	locals    []map[string]interface{} // Stack of local scopes
	pc        int                      // Program counter
	bp        int                      // Base pointer
	callStack []CallFrame              // Call stack
	loopStack []LoopFrame              // Loop stack for break/continue
	functions map[string]*Program
}

// NewVM creates a new virtual machine
func NewVM(program *Program) *VM {
	return &VM{
		program:   program,
		stack:     make([]interface{}, 0, 256),
		globals:   make(map[string]interface{}),
		locals:    make([]map[string]interface{}, 0),
		callStack: make([]CallFrame, 0),
		loopStack: make([]LoopFrame, 0),
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
					return fmt.Errorf("cannot perform arithmetic on nil value")
				}
				result := vm.add(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpSub:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				if left == nil || right == nil {
					return fmt.Errorf("cannot perform arithmetic on nil value")
				}
				result := vm.subtract(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpMul:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				if left == nil || right == nil {
					return fmt.Errorf("cannot perform arithmetic on nil value")
				}
				result := vm.multiply(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpDiv:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				if left == nil || right == nil {
					return fmt.Errorf("cannot perform arithmetic on nil value")
				}
				if (isInt(right) && asInt(right) == 0) || (isFloat(right) && asFloat(right) == 0) {
					return fmt.Errorf("division by zero")
				}
				result := vm.divide(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpMod:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				if left == nil || right == nil {
					return fmt.Errorf("cannot perform arithmetic on nil value")
				}
				if isInt(right) && asInt(right) == 0 {
					return fmt.Errorf("modulo by zero")
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
						return fmt.Errorf("array index must be an integer, got %T", index)
					}
					if idx < 0 || idx >= len(arr) {
						return fmt.Errorf("index out of bounds: index %d, length %d", idx, len(arr))
					}
					vm.stack = append(vm.stack, arr[idx])
				} else {
					return fmt.Errorf("cannot index into non-array type %T", array)
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
			// Call function
			if len(instr.Args) >= 2 {
				// First arg is function name (string)
				// Second arg is argument count
				var funcName string
				if nameVal, ok := instr.Args[0].(string); ok {
					funcName = nameVal
				}

				argCount := 0
				if countVal, ok := instr.Args[1].(int); ok {
					argCount = countVal
				}

				// Look up function entry point
				if entryPC, ok := vm.program.Functions[funcName]; ok {
					// Pop arguments from stack in reverse order
					args := make([]interface{}, argCount)
					for i := argCount - 1; i >= 0; i-- {
						if len(vm.stack) > 0 {
							args[i] = vm.pop()
						}
					}

					// Create call frame
					frame := CallFrame{
						pc:     vm.pc + 1, // Save instr after OpCall; loop will increment to it
						bp:     vm.bp,
						locals: make(map[string]interface{}),
					}
					vm.callStack = append(vm.callStack, frame)

					// Create locals map for this function call
					newLocals := make(map[string]interface{})
					// Map parameters to local indices
					for i, arg := range args {
						newLocals[fmt.Sprintf("_var_%d", i)] = arg
					}
					vm.locals = append(vm.locals, newLocals)

					// Set base pointer to stack length
					vm.bp = len(vm.stack)

					// Jump to function entry point
					// Set to entryPC - 1 because loop will increment it
					vm.pc = entryPC - 1
				} else {
					// Built-in function
					args := make([]interface{}, argCount)
					for i := argCount - 1; i >= 0; i-- {
						if len(vm.stack) > 0 {
							args[i] = vm.pop()
						}
					}

					// Check if it's a built-in function
					result := vm.callBuiltin(funcName, args)
					vm.stack = append(vm.stack, result)
				}
			}

		case OpReturn:
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

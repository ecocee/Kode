package bytecode

import (
	"fmt"
	"strconv"
)

// VM is a bytecode virtual machine
type VM struct {
	program   *Program
	stack     []interface{}
	globals   map[string]interface{}
	locals    []map[string]interface{} // Stack of local scopes
	pc        int                      // Program counter
	bp        int                      // Base pointer
	functions map[string]*Program
}

// NewVM creates a new virtual machine
func NewVM(program *Program) *VM {
	return &VM{
		program: program,
		stack:   make([]interface{}, 0, 256),
		globals: make(map[string]interface{}),
		locals:  make([]map[string]interface{}, 0),
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
				// Find variable by index
				for name, idx := range vm.program.Globals {
					if idx == varIdx {
						vm.stack = append(vm.stack, vm.globals[name])
						break
					}
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
				result := vm.add(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpSub:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.subtract(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpMul:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.multiply(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpDiv:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
				result := vm.divide(left, right)
				vm.stack = append(vm.stack, result)
			}

		case OpMod:
			if len(vm.stack) >= 2 {
				right := vm.pop()
				left := vm.pop()
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
					if idx, ok := index.(int64); ok && idx >= 0 && idx < int64(len(arr)) {
						vm.stack = append(vm.stack, arr[idx])
					} else if idx, ok := index.(int); ok && idx >= 0 && idx < len(arr) {
						vm.stack = append(vm.stack, arr[idx])
					} else {
						vm.stack = append(vm.stack, nil) // Out of bounds
					}
				}
			}

		case OpArrayStore:
			if len(vm.stack) >= 3 {
				value := vm.pop()
				index := vm.pop()
				array := vm.pop()
				if arr, ok := array.([]interface{}); ok {
					if idx, ok := index.(int64); ok && idx >= 0 && idx < int64(len(arr)) {
						arr[int(idx)] = value
					} else if idx, ok := index.(int); ok && idx >= 0 && idx < len(arr) {
						arr[idx] = value
					}
				}
				vm.stack = append(vm.stack, array)
			}

		case OpArrayLen:
			if len(vm.stack) > 0 {
				array := vm.pop()
				if arr, ok := array.([]interface{}); ok {
					vm.stack = append(vm.stack, int64(len(arr)))
				} else {
					vm.stack = append(vm.stack, int64(0))
				}
			}

		case OpMemberAccess:
			// Member access: get a member from object
			if len(instr.Args) > 0 && len(vm.stack) > 0 {
				memberIdx := instr.Args[0].(int)
				member := vm.program.Constants[memberIdx].(string)
				obj := vm.pop()

				// Handle array methods
				if arr, ok := obj.([]interface{}); ok {
					switch member {
					case "len":
						// Push a callable that returns array length
						// For now, return the length directly
						vm.stack = append(vm.stack, int64(len(arr)))
					case "push":
						// Create a push method (needs to be called)
						vm.stack = append(vm.stack, map[string]interface{}{"_method": "push", "_array": arr})
					case "pop":
						// Return method reference
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
				
				// Pop value from stack if present
				var value interface{} = nil
				if len(vm.stack) > 0 {
					// Check if there's a value to pop
					// For now, we'll keep the stack as is
				}
				
				// Create enum variant as a map with metadata
				variantData := map[string]interface{}{
					"_type":    enumName,
					"_variant": variantName,
					"_value":   value,
				}
				
				vm.stack = append(vm.stack, variantData)
			}

		case OpBreak:
			// Break: set pc to after loop (handled by loop jump patching)
			// For now, just halt execution (improve later for nested loops)
			return nil
		case OpContinue:
			// Continue: jump to loop start (handled by loop jump patching)
			// For now, just halt execution (improve later for nested loops)
			return nil
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
		case float64:
			return float64(l) == r
		}
	case float64:
		switch r := right.(type) {
		case int:
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
		case float64:
			return float64(l) < r
		}
	case float64:
		switch r := right.(type) {
		case int:
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

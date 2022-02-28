package stack

import "core:fmt"

Value :: union {
	i64,
	bool,
}

ExecuteOps :: proc(ops: []Op) -> []Value {
	stack: [dynamic]Value
	ip := 0
	for {
		op := ops[ip]
		switch data in op.data {
		case ExitOp:
			return stack[:]
		case JumpOp:
			ip += data.relative_ip
			continue
		case JumpFalseOp:
			condition := pop(&stack).(bool)
			if !condition {
				ip += data.relative_ip
				continue
			}
		case IntegerPushOp:
			append(&stack, data.value)
		case IntegerDropOp:
			_ = pop(&stack).(i64)
		case IntegerDupOp:
			value := pop(&stack).(i64)
			append(&stack, value, value)
		case IntegerAddOp:
			b := pop(&stack).(i64)
			a := pop(&stack).(i64)
			append(&stack, a + b)
		case IntegerSubtractOp:
			b := pop(&stack).(i64)
			a := pop(&stack).(i64)
			append(&stack, a - b)
		case IntegerMultiplyOp:
			b := pop(&stack).(i64)
			a := pop(&stack).(i64)
			append(&stack, a * b)
		case IntegerDivideOp:
			b := pop(&stack).(i64)
			a := pop(&stack).(i64)
			append(&stack, a / b)
		case IntegerModulusOp:
			b := pop(&stack).(i64)
			a := pop(&stack).(i64)
			append(&stack, a % b)
		case IntegerLessThanOp:
			b := pop(&stack).(i64)
			a := pop(&stack).(i64)
			append(&stack, a < b)
		case IntegerGreaterThanOp:
			b := pop(&stack).(i64)
			a := pop(&stack).(i64)
			append(&stack, a > b)
		case IntegerEqualOp:
			b := pop(&stack).(i64)
			a := pop(&stack).(i64)
			append(&stack, a == b)
		case IntegerPrintOp:
			fmt.println(pop(&stack).(i64))
		case BoolPushOp:
			append(&stack, data.value)
		case BoolDropOp:
			_ = pop(&stack).(bool)
		case BoolDupOp:
			value := pop(&stack).(bool)
			append(&stack, value, value)
		case BoolNotOp:
			value := pop(&stack).(bool)
			append(&stack, !value)
		case BoolEqualOp:
			b := pop(&stack).(bool)
			a := pop(&stack).(bool)
			append(&stack, a == b)
		case BoolPrintOp:
			fmt.println(pop(&stack).(bool))
		case:
			unreachable()
		}
		ip += 1
	}
}

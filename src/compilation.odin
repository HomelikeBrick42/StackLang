package stack

import "core:fmt"

@(private = "file")
Scope :: union {
	IfConditionScope,
	IfScope,
	ElseScope,
}

@(private = "file")
IfConditionScope :: struct {
	location: SourceLocation,
}

@(private = "file")
IfScope :: struct {
	location:        SourceLocation,
	jump_false_ip:   int,
	condition_scope: IfConditionScope,
	stack_before:    []Type,
}

@(private = "file")
ElseScope :: struct {
	location:        SourceLocation,
	jump_past_if_ip: int,
	if_scope:        IfScope,
	stack_after_if:  []Type,
}

CompileOps :: proc(filepath, source: string) -> (
	ops: [dynamic]Op,
	error: Maybe(CompileError),
) {
	lexer := Lexer_Create(filepath, source)

	type_stack: [dynamic]Type
	defer delete(type_stack)

	scopes: [dynamic]Scope
	defer delete(scopes)

	for {
		token := Lexer_NextToken(&lexer)
		switch token.kind {
		case .Invalid:
			unreachable()
		case .EOF:
			if len(type_stack) > 0 {
				error = CompileError {
					location = token.location,
					message  = fmt.aprintf("Expected the stack to be empty at the end of the program"),
				}
				return
			}
			append(&ops, Op{location = token.location, data = ExitOp{}})
			return
		case .Integer:
			value := token.data.(u128)
			if value > u128(max(i64)) {
				error = CompileError {
					location = token.location,
					message  = fmt.aprintf("The integer literal %i is too big for an i64", value),
				}
				return
			}
			append(&ops, Op{location = token.location, data = IntegerPushOp{value = i64(value)}})
			append(&type_stack, IntegerType{})
		case .Name:
			unimplemented()
		case .OpenParenthesis:
			unimplemented()
		case .CloseParenthesis:
			unimplemented()
		case .OpenBrace:
			if _, ok := scopes[len(scopes) - 1].(IfConditionScope); ok {
				condition_scope := pop(&scopes).(IfConditionScope)
				ExpectTypes(&type_stack, token.location, {BoolType{}}) or_return

				stack_copy := make([]Type, len(type_stack))
				copy(stack_copy, type_stack[:])

				append(
					&scopes,
					IfScope{
						location = token.location,
						jump_false_ip = len(ops),
						condition_scope = condition_scope,
						stack_before = stack_copy,
					},
				)
				append(&ops, Op{location = token.location, data = JumpFalseOp{}})
			} else {
				unimplemented()
			}
		case .CloseBrace:
			scope := pop(&scopes)
			#partial switch scope in scope {
			case IfScope:
				jump_false_op := &ops[scope.jump_false_ip].data.(JumpFalseOp)
				jump_false_op.relative_ip = len(ops) - scope.jump_false_ip

				token = Lexer_PeekToken(lexer)
				if token.kind == .Else {
					token = Lexer_NextToken(&lexer)

					stack_copy := make([]Type, len(type_stack))
					copy(stack_copy, type_stack[:])

					clear(&type_stack)
					append(&type_stack, ..scope.stack_before)

					append(
						&scopes,
						ElseScope{
							location = token.location,
							if_scope = scope,
							jump_past_if_ip = len(ops),
							stack_after_if = stack_copy,
						},
					)

					append(&ops, Op{location = token.location, data = JumpOp{}})
					jump_false_op.relative_ip = len(ops) - scope.jump_false_ip

					token = Lexer_NextToken(&lexer)
					if token.kind != .OpenBrace {
						error = CompileError {
							location = token.location,
							message  = fmt.aprintf("Expected {{ after else"),
						}
						return
					}
				} else {
					if len(scope.stack_before) != len(type_stack) {
						error = CompileError {
							location = token.location,
							message  = fmt.aprintf("if cannot change the number of elements on the stack"),
						}
					}

					for _, i in scope.stack_before {
						if !TypesEqual(scope.stack_before[i], type_stack[i]) {
							error = CompileError {
								location = token.location,
								message  = fmt.aprintf("If cannot change the types of the elements on the stack"),
							}
							return
						}
					}

					delete(scope.stack_before)
				}
			case ElseScope:
				jump_op := &ops[scope.jump_past_if_ip].data.(JumpOp)
				jump_op.relative_ip = len(ops) - scope.jump_past_if_ip

				if len(scope.stack_after_if) != len(type_stack) {
					error = CompileError {
						location = token.location,
						message  = fmt.aprintf(
							"Expected the same number of elements from the if and else branches",
						),
					}
					return
				}

				for _, i in scope.stack_after_if {
					if !TypesEqual(scope.stack_after_if[i], type_stack[i]) {
						error = CompileError {
							location = token.location,
							message  = fmt.aprintf("Expected the same types from the if and else branches"),
						}
						return
					}
				}

				delete(scope.if_scope.stack_before)
				delete(scope.stack_after_if)
			case:
				unimplemented()
			}
		case .Add:
			ExpectTypes(&type_stack, token.location, {IntegerType{}, IntegerType{}}) or_return
			append(&ops, Op{location = token.location, data = IntegerAddOp{}})
			append(&type_stack, IntegerType{})
		case .Subtract:
			ExpectTypes(&type_stack, token.location, {IntegerType{}, IntegerType{}}) or_return
			append(&ops, Op{location = token.location, data = IntegerSubtractOp{}})
			append(&type_stack, IntegerType{})
		case .Multiply:
			ExpectTypes(&type_stack, token.location, {IntegerType{}, IntegerType{}}) or_return
			append(&ops, Op{location = token.location, data = IntegerMultiplyOp{}})
			append(&type_stack, IntegerType{})
		case .Divide:
			ExpectTypes(&type_stack, token.location, {IntegerType{}, IntegerType{}}) or_return
			append(&ops, Op{location = token.location, data = IntegerDivideOp{}})
			append(&type_stack, IntegerType{})
		case .Equal:
			ExpectTypeCount(&type_stack, token.location, 2) or_return
			type := pop(&type_stack)
			if integer_type, ok := type.(IntegerType); ok {
				ExpectTypes(&type_stack, token.location, {IntegerType{}}) or_return
				append(&ops, Op{location = token.location, data = IntegerEqualOp{}})
			} else if bool_type, ok := type.(BoolType); ok {
				ExpectTypes(&type_stack, token.location, {BoolType{}}) or_return
				append(&ops, Op{location = token.location, data = BoolEqualOp{}})
			} else {
				error = CompileError {
					location = token.location,
					message  = fmt.aprintf("== cannot compare %v", type),
				}
				return
			}
			append(&type_stack, BoolType{})
		case .Assign:
			unimplemented()
		case .Print:
			ExpectTypeCount(&type_stack, token.location, 1) or_return
			type := pop(&type_stack)
			if integer_type, ok := type.(IntegerType); ok {
				append(&ops, Op{location = token.location, data = IntegerPrintOp{}})
			} else if bool_type, ok := type.(BoolType); ok {
				append(&ops, Op{location = token.location, data = BoolPrintOp{}})
			} else {
				error = CompileError {
					location = token.location,
					message  = fmt.aprintf("print connot print %v", type),
				}
				return
			}
		case .If:
			append(&scopes, IfConditionScope{location = token.location})
		case .Else:
			error = CompileError {
				location = token.location,
				message  = fmt.aprintf("else is not connected to an if"),
			}
			return
		case .Drop:
			ExpectTypeCount(&type_stack, token.location, 1) or_return
			type := pop(&type_stack)
			if integer_type, ok := type.(IntegerType); ok {
				append(&ops, Op{location = token.location, data = IntegerDropOp{}})
			} else if bool_type, ok := type.(BoolType); ok {
				append(&ops, Op{location = token.location, data = BoolDropOp{}})
			} else {
				error = CompileError {
					location = token.location,
					message  = fmt.aprintf("drop connot drop %v", type),
				}
				return
			}
		case .Dup:
			ExpectTypeCount(&type_stack, token.location, 1) or_return
			type := pop(&type_stack)
			if integer_type, ok := type.(IntegerType); ok {
				append(&ops, Op{location = token.location, data = IntegerDupOp{}})
				append(&type_stack, IntegerType{}, IntegerType{})
			} else if bool_type, ok := type.(BoolType); ok {
				append(&ops, Op{location = token.location, data = BoolDupOp{}})
				append(&type_stack, BoolType{}, BoolType{})
			} else {
				error = CompileError {
					location = token.location,
					message  = fmt.aprintf("dup connot duplicate %v", type),
				}
				return
			}
		case:
			unreachable()
		}
	}
}

@(private = "file")
ExpectTypeCount :: proc(
	type_stack: ^[dynamic]Type,
	location: SourceLocation,
	count: int,
) -> (
	error: Maybe(CompileError),
) {
	if len(type_stack) < count {
		error = CompileError {
			location = location,
			message  = fmt.aprintf(
				"Expected at least %i items on the stack, but got %i",
				count,
				len(type_stack),
			),
		}
		return
	}
	return nil
}

@(private = "file")
ExpectTypes :: proc(
	type_stack: ^[dynamic]Type,
	location: SourceLocation,
	types: []Type,
) -> (
	error: Maybe(CompileError),
) {
	ExpectTypeCount(type_stack, location, len(types)) or_return
	for expected_type, i in types {
		if type := type_stack[len(type_stack) -
		   len(types) +
		   i]; !TypesEqual(type, expected_type) {
			error = CompileError {
				location = location,
				message  = fmt.aprintf(
					"Expected type %v for argument %i, but got %v",
					expected_type,
					i,
					type,
				),
			}
			return
		}
	}
	for _ in types {
		pop(type_stack)
	}
	return nil
}

use std::{cell::Cell, collections::HashMap, rc::Rc};

use stack_lang::{Op::*, *};

fn main() {
    let builtins = HashMap::from([
        (
            "print_s64".to_string(),
            Value::BuiltinFunction(
                Type::Function {
                    arguments: vec![Type::S64],
                    return_values: vec![],
                },
                Rc::new(|stack| {
                    let value = stack.pop().unwrap();
                    match value {
                        Value::S64(value) => println!("{value}"),
                        _ => panic!("Expected a s64"),
                    }
                }),
            ),
        ),
        (
            "print_type".to_string(),
            Value::BuiltinFunction(
                Type::Function {
                    arguments: vec![Type::Type],
                    return_values: vec![],
                },
                Rc::new(|stack| {
                    let value = stack.pop().unwrap();
                    match value {
                        Value::Type(typ) => println!("{typ}"),
                        _ => panic!("Expected a type"),
                    }
                }),
            ),
        ),
    ]);

    let ops = [
        EnterScope,
        Push(Value::S64(5)),
        Push(Value::S64(42)),
        Push(Value::S64(6)),
        Over(1),
        Drop,
        Add,
        GetLocal("print_s64".into()),
        Load,
        Call,
        Push(Value::Type(Type::Function {
            arguments: vec![Type::S64, Type::S64],
            return_values: vec![Type::S64],
        })),
        GetLocal("print_type".into()),
        Load,
        Call,
        Push(Value::S64(34)),
        MakeFunction {
            typ: Type::Function {
                arguments: vec![Type::S64],
                return_values: vec![Type::Function {
                    arguments: vec![Type::S64],
                    return_values: vec![Type::S64],
                }],
            },
            ops: Rc::new(vec![
                EnterScope,
                CreateLocal("value".into()),
                MakeFunction {
                    typ: Type::Function {
                        arguments: vec![Type::S64],
                        return_values: vec![Type::S64],
                    },
                    ops: Rc::new(vec![
                        EnterScope,
                        GetLocal("value".into()),
                        Load,
                        Add,
                        ExitScope,
                        Return,
                    ]),
                },
                ExitScope,
                Return,
            ]),
        },
        Call,
        Push(Value::S64(35)),
        Over(1),
        Call,
        GetLocal("print_s64".into()),
        Load,
        Call,
        ExitScope,
        Return,
    ];

    let mut type_stack = vec![];
    type_check(
        &ops,
        &mut type_stack,
        builtins
            .iter()
            .map(|(name, value)| (name.clone(), value.get_type()))
            .collect(),
    );

    execute(
        &ops,
        &mut vec![],
        builtins
            .iter()
            .map(|(name, value)| (name.clone(), Rc::new(Cell::new(value.clone()))))
            .collect(),
    );
}

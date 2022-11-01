use std::{cell::Cell, collections::HashMap, io::Write, rc::Rc};

use stack_lang::*;

fn main() {
    let builtins = HashMap::from([]);

    let constants = HashMap::from([
        (
            "println".to_string(),
            vec![Value::BuiltinFunction(
                Type::Procedure {
                    arguments: vec![],
                    return_values: vec![],
                },
                Rc::new(|_| println!()),
            )],
        ),
        (
            "print_type".to_string(),
            vec![Value::BuiltinFunction(
                Type::Procedure {
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
            )],
        ),
        (
            "print_int".to_string(),
            vec![Value::BuiltinFunction(
                Type::Procedure {
                    arguments: vec![Type::Integer],
                    return_values: vec![],
                },
                Rc::new(|stack| {
                    let value = stack.pop().unwrap();
                    match value {
                        Value::Integer(value) => println!("{value}"),
                        _ => panic!("Expected an int"),
                    }
                }),
            )],
        ),
        (
            "print_string".to_string(),
            vec![Value::BuiltinFunction(
                Type::Procedure {
                    arguments: vec![Type::String],
                    return_values: vec![],
                },
                Rc::new(|stack| {
                    let value = stack.pop().unwrap();
                    match value {
                        Value::String(str) => {
                            print!("{str}");
                            std::io::stdout().flush().unwrap();
                        }
                        _ => panic!("Expected a string"),
                    }
                }),
            )],
        ),
        (
            "read_line".to_string(),
            vec![Value::BuiltinFunction(
                Type::Procedure {
                    arguments: vec![],
                    return_values: vec![Type::String],
                },
                Rc::new(|stack| {
                    stack.push(Value::String(
                        std::io::stdin().lines().next().unwrap().unwrap(),
                    ));
                }),
            )],
        ),
    ]);

    let mut args = std::env::args().skip(1);
    let filepath = args.next().expect("expected a filepath to read");
    let source = std::fs::read_to_string(filepath).expect("Unable to read file");
    let ops = compile_ops(&source, &builtins, constants);

    execute(
        &ops,
        &mut vec![],
        builtins
            .iter()
            .map(|(name, value)| (name.clone(), Rc::new(Cell::new(value.clone()))))
            .collect(),
    );
}

use std::{cell::Cell, collections::HashMap, rc::Rc};

use stack_lang::*;

fn main() {
    let builtins = HashMap::from([
        (
            "print_int".to_string(),
            Value::BuiltinFunction(
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
            ),
        ),
        (
            "print_type".to_string(),
            Value::BuiltinFunction(
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
            ),
        ),
    ]);

    let source = r"
5 42 6
over(2 1)
print_int load call
print_int load call
print_int load call

proc_type(int int) -> (int)
print_type load call

34
proc(int) -> (proc_type(int) -> (int)) {
    new_local value
    proc(int) -> (int) {
        value load add
    }
}
call
35 over(1) call
print_int load call
";

    let ops = compile_ops(source, &builtins);

    execute(
        &ops,
        &mut vec![],
        builtins
            .iter()
            .map(|(name, value)| (name.clone(), Rc::new(Cell::new(value.clone()))))
            .collect(),
    );
}

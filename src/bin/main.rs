use std::{cell::Cell, collections::HashMap, rc::Rc};

use stack_lang::*;

fn main() {
    let builtins = HashMap::from([
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
            "print_string".to_string(),
            Value::BuiltinFunction(
                Type::Procedure {
                    arguments: vec![Type::String],
                    return_values: vec![],
                },
                Rc::new(|stack| {
                    let value = stack.pop().unwrap();
                    match value {
                        Value::String(str) => println!("{str}"),
                        _ => panic!("Expected a string"),
                    }
                }),
            ),
        ),
    ]);

    let source = r#"
5 42 6
over(2 1)
get("print_int") load
dup over(2 2) call
dup over(2 2) call
dup over(2 2) call
drop

proc_type(int int) -> (int)
get("print_type") load call

34
proc(int) -> (proc_type(int) -> (int)) {
    var("value")
    proc(int) -> (int) {
        get("value") load add
    }
}
call
35 swap call
get("print_int") load call

"hello"
get("print_string") load call

5
var("test")

proc(int ref) -> () {
    dup load
    26 add
    swap
    store
}
get("test") swap call
get("print_int" "test") load swap load call
"#;

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

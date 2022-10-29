use std::{cell::Cell, collections::HashMap, rc::Rc};

use lazy_static::lazy_static;
use regex::Regex;

use crate::{execute, type_check, Op, Type, Value};

enum ParseScope {
    ProcTypeParameterTypes {
        old_ops: Vec<Op>,
    },
    ProcTypeReturnTypes {
        parameter_types: Vec<Type>,
        old_ops: Vec<Op>,
    },
    ProcParameterTypes {
        old_ops: Vec<Op>,
    },
    ProcReturnTypes {
        parameter_types: Vec<Type>,
        old_ops: Vec<Op>,
    },
    ProcBody {
        parameter_types: Vec<Type>,
        return_types: Vec<Type>,
        old_ops: Vec<Op>,
    },
}

pub fn compile_ops(mut source: &str, builtins: &HashMap<String, Value>) -> Vec<Op> {
    let builtin_types: HashMap<_, _> = builtins
        .iter()
        .map(|(name, value)| (name.clone(), value.get_type()))
        .collect();
    let builtin_values: HashMap<_, _> = builtins
        .iter()
        .map(|(name, value)| (name.clone(), Rc::new(Cell::new(value.clone()))))
        .collect();

    lazy_static! {
        static ref WHITESPACE: Regex = Regex::new(r"^\s+").unwrap();
        static ref NUMBER: Regex = Regex::new(r"^[0-9]+").unwrap();
        static ref IDENTIFIER: Regex = Regex::new(r"^[_A-Za-z][_0-9A-Za-z]*").unwrap();
        static ref OVER: Regex = Regex::new(r"^over[0-9]+$").unwrap();
        static ref PROCEDURE_ARROW: Regex = Regex::new(r"^\s*->\s*\(").unwrap();
    }

    let mut parse_scopes: Vec<ParseScope> = vec![];

    let mut ops = vec![Op::EnterScope];
    while source.len() > 0 {
        if let Some(m) = WHITESPACE.find(source) {
            source = &source[m.as_str().len()..];
        } else if let Some(m) = NUMBER.find(source) {
            let number = m.as_str();
            source = &source[number.len()..];
            ops.push(Op::Push(Value::Integer(number.parse().unwrap())));
        } else if let Some(m) = IDENTIFIER.find(source) {
            let identifier = m.as_str();
            source = &source[identifier.len()..];
            if OVER.is_match(identifier) {
                let value = m.as_str()[4..].parse::<usize>().unwrap();
                ops.push(Op::Over(value));
            } else {
                match identifier {
                    "int" => ops.push(Op::Push(Value::Type(Type::Integer))),
                    "drop" => ops.push(Op::Drop),
                    "add" => ops.push(Op::Add),
                    "sub" => ops.push(Op::Subtract),
                    "mul" => ops.push(Op::Multiply),
                    "divmod" => ops.push(Op::DivMod),
                    "load" => ops.push(Op::Load),
                    "store" => ops.push(Op::Store),
                    "call" => ops.push(Op::Call),
                    "return" => ops.push(Op::Return),
                    "new_local" => {
                        if let Some(m) = WHITESPACE.find(source) {
                            source = &source[m.as_str().len()..];
                        }
                        let name = IDENTIFIER.find(source).unwrap().as_str();
                        source = &source[name.len()..];
                        ops.push(Op::CreateLocal(name.into()));
                    }
                    "proc_type" => {
                        assert_eq!(source.chars().next().unwrap(), '(');
                        source = &source[1..];
                        parse_scopes.push(ParseScope::ProcTypeParameterTypes { old_ops: ops });
                        ops = vec![Op::EnterScope];
                    }
                    "proc" => {
                        assert_eq!(source.chars().next().unwrap(), '(');
                        source = &source[1..];
                        parse_scopes.push(ParseScope::ProcParameterTypes { old_ops: ops });
                        ops = vec![Op::EnterScope];
                    }
                    _ => ops.push(Op::GetLocal(identifier.into())),
                }
            }
        } else if source.chars().next().unwrap() == ')' && parse_scopes.len() > 0 {
            source = &source[1..];
            match parse_scopes.pop().unwrap() {
                ParseScope::ProcTypeParameterTypes { old_ops } => {
                    ops.push(Op::ExitScope);
                    ops.push(Op::Return);
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::Type,
                            "All elements left on the proc_type parameter type stack must be types"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_values.clone());
                    let parameter_types = values
                        .into_iter()
                        .map(|value| match value {
                            Value::Type(typ) => typ,
                            _ => unreachable!(),
                        })
                        .collect();
                    let arrow = PROCEDURE_ARROW.find(source).unwrap();
                    source = &source[arrow.as_str().len()..];
                    parse_scopes.push(ParseScope::ProcTypeReturnTypes {
                        parameter_types,
                        old_ops,
                    });
                    ops = vec![Op::EnterScope];
                }
                ParseScope::ProcTypeReturnTypes {
                    parameter_types,
                    old_ops,
                } => {
                    ops.push(Op::ExitScope);
                    ops.push(Op::Return);
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::Type,
                            "All elements left on the proc_type return type stack must be types"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_values.clone());
                    let return_types = values
                        .into_iter()
                        .map(|value| match value {
                            Value::Type(typ) => typ,
                            _ => unreachable!(),
                        })
                        .collect();
                    ops = old_ops;
                    ops.push(Op::Push(Value::Type(Type::Procedure {
                        arguments: parameter_types,
                        return_values: return_types,
                    })));
                }
                ParseScope::ProcParameterTypes { old_ops } => {
                    ops.push(Op::ExitScope);
                    ops.push(Op::Return);
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::Type,
                            "All elements left on the proc parameter type stack must be types"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_values.clone());
                    let parameter_types = values
                        .into_iter()
                        .map(|value| match value {
                            Value::Type(typ) => typ,
                            _ => unreachable!(),
                        })
                        .collect();
                    let arrow = PROCEDURE_ARROW.find(source).unwrap();
                    source = &source[arrow.as_str().len()..];
                    parse_scopes.push(ParseScope::ProcReturnTypes {
                        parameter_types,
                        old_ops,
                    });
                    ops = vec![Op::EnterScope];
                }
                ParseScope::ProcReturnTypes {
                    parameter_types,
                    old_ops,
                } => {
                    ops.push(Op::ExitScope);
                    ops.push(Op::Return);
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::Type,
                            "All elements left on the proc return type stack must be types"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_values.clone());
                    let return_types = values
                        .into_iter()
                        .map(|value| match value {
                            Value::Type(typ) => typ,
                            _ => unreachable!(),
                        })
                        .collect();
                    if let Some(m) = WHITESPACE.find(source) {
                        source = &source[m.as_str().len()..];
                    }
                    assert_eq!(source.chars().next().unwrap(), '{');
                    source = &source[1..];
                    parse_scopes.push(ParseScope::ProcBody {
                        parameter_types,
                        return_types,
                        old_ops,
                    });
                    ops = vec![Op::EnterScope];
                }
                ParseScope::ProcBody { .. } => panic!("Cannot use ')' to close a proc body"),
            }
        } else if source.chars().next().unwrap() == '}' && parse_scopes.len() > 0 {
            source = &source[1..];
            match parse_scopes.pop().unwrap() {
                ParseScope::ProcTypeParameterTypes { .. } => {
                    panic!("Cannot use '}}' to close a proc_type parameter type");
                }
                ParseScope::ProcTypeReturnTypes { .. } => {
                    panic!("Cannot use '}}' to close a proc_type return type");
                }
                ParseScope::ProcParameterTypes { .. } => {
                    panic!("Cannot use '}}' to close a proc parameter type");
                }
                ParseScope::ProcReturnTypes { .. } => {
                    panic!("Cannot use '}}' to close a proc return type");
                }
                ParseScope::ProcBody {
                    parameter_types,
                    return_types,
                    old_ops,
                } => {
                    ops.push(Op::ExitScope);
                    ops.push(Op::Return);
                    let new_ops = Rc::new(ops);
                    ops = old_ops;
                    ops.push(Op::MakeProcedure {
                        typ: Type::Procedure {
                            arguments: parameter_types,
                            return_values: return_types,
                        },
                        ops: new_ops,
                    });
                }
            }
        } else {
            panic!("Unexpected character {:?}", source.chars().next().unwrap());
        }
    }
    ops.push(Op::ExitScope);
    ops.push(Op::Return);
    type_check(&ops, &mut vec![], builtin_types);
    ops
}

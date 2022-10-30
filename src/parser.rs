use std::{cell::Cell, collections::HashMap, rc::Rc};

use lazy_static::lazy_static;
use regex::Regex;

use crate::{execute, type_check, Op, Type, Value};

enum ParseScope {
    Over {
        old_ops: Vec<Op>,
    },
    Var {
        old_ops: Vec<Op>,
    },
    Get {
        old_ops: Vec<Op>,
    },
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
        static ref STRING_LITERAL: Regex = Regex::new(r#"^"(.*?)""#).unwrap();
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
        } else if let Some(m) = STRING_LITERAL.find(source) {
            let str = m.as_str();
            source = &source[str.len()..];
            let captures = STRING_LITERAL.captures(str).unwrap();
            ops.push(Op::Push(Value::String(captures[1].into())));
        } else if let Some(m) = IDENTIFIER.find(source) {
            let identifier = m.as_str();
            source = &source[identifier.len()..];
            match identifier {
                "int" => ops.push(Op::Push(Value::Type(Type::Integer))),
                "dup" => ops.push(Op::Dup),
                "ref" => ops.push(Op::MakeReferenceType),
                "drop" => ops.push(Op::Drop),
                "over" => {
                    if let Some(m) = WHITESPACE.find(source) {
                        source = &source[m.as_str().len()..];
                    }
                    assert_eq!(source.chars().next().unwrap(), '(');
                    source = &source[1..];
                    parse_scopes.push(ParseScope::Over { old_ops: ops });
                    ops = vec![Op::EnterScope];
                }
                "add" => ops.push(Op::Add),
                "sub" => ops.push(Op::Subtract),
                "mul" => ops.push(Op::Multiply),
                "divmod" => ops.push(Op::DivMod),
                "load" => ops.push(Op::Load),
                "store" => ops.push(Op::Store),
                "call" => ops.push(Op::Call),
                "return" => ops.push(Op::Return),
                "swap" => ops.push(Op::Over(vec![1])),
                "var" => {
                    if let Some(m) = WHITESPACE.find(source) {
                        source = &source[m.as_str().len()..];
                    }
                    assert_eq!(source.chars().next().unwrap(), '(');
                    source = &source[1..];
                    parse_scopes.push(ParseScope::Var { old_ops: ops });
                    ops = vec![Op::EnterScope];
                }
                "get" => {
                    if let Some(m) = WHITESPACE.find(source) {
                        source = &source[m.as_str().len()..];
                    }
                    assert_eq!(source.chars().next().unwrap(), '(');
                    source = &source[1..];
                    parse_scopes.push(ParseScope::Get { old_ops: ops });
                    ops = vec![Op::EnterScope];
                }
                "proc_type" => {
                    if let Some(m) = WHITESPACE.find(source) {
                        source = &source[m.as_str().len()..];
                    }
                    assert_eq!(source.chars().next().unwrap(), '(');
                    source = &source[1..];
                    parse_scopes.push(ParseScope::ProcTypeParameterTypes { old_ops: ops });
                    ops = vec![Op::EnterScope];
                }
                "proc" => {
                    if let Some(m) = WHITESPACE.find(source) {
                        source = &source[m.as_str().len()..];
                    }
                    assert_eq!(source.chars().next().unwrap(), '(');
                    source = &source[1..];
                    parse_scopes.push(ParseScope::ProcParameterTypes { old_ops: ops });
                    ops = vec![Op::EnterScope];
                }
                _ => panic!("Unknown identifier '{identifier}'"),
            }
        } else if source.chars().next().unwrap() == ')' && parse_scopes.len() > 0 {
            source = &source[1..];
            match parse_scopes.pop().unwrap() {
                ParseScope::Over { old_ops } => {
                    ops.push(Op::ExitScope);
                    ops.push(Op::Return);
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::Integer,
                            "All elements left on the over offset stack must be integers"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_values.clone());
                    let offsets = values
                        .into_iter()
                        .map(|value| match value {
                            Value::Integer(value) => {
                                assert!(
                                    value >= 0,
                                    "all over offsets must be positive but got {value}"
                                );
                                value as usize
                            }
                            _ => unreachable!(),
                        })
                        .collect();
                    ops = old_ops;
                    ops.push(Op::Over(offsets));
                }
                ParseScope::Var { old_ops } => {
                    ops.push(Op::ExitScope);
                    ops.push(Op::Return);
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::String,
                            "All elements left on the var name stack must be strings"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_values.clone());
                    let names = values
                        .into_iter()
                        .map(|value| match value {
                            Value::String(value) => {
                                assert!(
                                    IDENTIFIER.is_match(&value),
                                    "Expected a valid identifier but got {value:?}"
                                );
                                value
                            }
                            _ => unreachable!(),
                        })
                        .collect();
                    ops = old_ops;
                    ops.push(Op::NewLocals(names));
                }
                ParseScope::Get { old_ops } => {
                    ops.push(Op::ExitScope);
                    ops.push(Op::Return);
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::String,
                            "All elements left on the get name stack must be strings"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_values.clone());
                    let names = values
                        .into_iter()
                        .map(|value| match value {
                            Value::String(value) => {
                                assert!(
                                    IDENTIFIER.is_match(&value),
                                    "Expected a valid identifier but got {value:?}"
                                );
                                value
                            }
                            _ => unreachable!(),
                        })
                        .collect();
                    ops = old_ops;
                    ops.push(Op::GetLocals(names));
                }
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
                ParseScope::Over { .. } => {
                    panic!("Cannot use '}}' to close an over");
                }
                ParseScope::Var { .. } => {
                    panic!("Cannot use '}}' to close a var");
                }
                ParseScope::Get { .. } => {
                    panic!("Cannot use '}}' to close a get");
                }
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

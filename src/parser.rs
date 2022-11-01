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
    IfCondition,
    IfThen {
        old_ops: Vec<Op>,
    },
    IfElse {
        then_ops: Vec<Op>,
        old_ops: Vec<Op>,
    },
    WhileCondition {
        old_ops: Vec<Op>,
    },
    WhileBody {
        condition_ops: Vec<Op>,
        old_ops: Vec<Op>,
    },
    Const {
        old_ops: Vec<Op>,
    },
}

pub fn compile_ops(
    mut source: &str,
    builtin_vars: &HashMap<String, Value>,
    constants: HashMap<String, Vec<Value>>,
) -> Vec<Op> {
    let builtin_var_types = builtin_vars
        .iter()
        .map(|(name, value)| (name.clone(), value.get_type()))
        .collect::<HashMap<_, _>>();
    let builtin_var_values = builtin_vars
        .iter()
        .map(|(name, value)| (name.clone(), Rc::new(Cell::new(value.clone()))))
        .collect::<HashMap<_, _>>();

    lazy_static! {
        static ref WHITESPACE: Regex = Regex::new(r"^\s+").unwrap();
        static ref NUMBER: Regex = Regex::new(r"^[0-9]+").unwrap();
        static ref IDENTIFIER: Regex = Regex::new(r"^[_A-Za-z][_0-9A-Za-z]*").unwrap();
        static ref STRING_LITERAL: Regex = Regex::new(r#"^"(.*?)""#).unwrap();
        static ref PROCEDURE_ARROW: Regex = Regex::new(r"^\s*->\s*\(").unwrap();
        static ref ELSE: Regex = Regex::new(r"^\s*else\s*\{").unwrap();
        static ref DUMP_TYPES: Regex = Regex::new(r"^\?\?\?").unwrap();
    }

    let mut parse_scopes: Vec<ParseScope> = vec![];

    let mut ops = vec![Op::EnterScope];
    let mut constants = vec![constants];

    while source.len() > 0 {
        if let Some(m) = WHITESPACE.find(source) {
            source = &source[m.as_str().len()..];
        } else if let Some(m) = DUMP_TYPES.find(source) {
            source = &source[m.as_str().len()..];
            ops.push(Op::DumpCurrentTypeStackInTypeChecking);
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
                    constants.push(HashMap::new());
                }
                "add" => ops.push(Op::Add),
                "sub" => ops.push(Op::Subtract),
                "mul" => ops.push(Op::Multiply),
                "divmod" => ops.push(Op::DivMod),
                "load" => ops.push(Op::Load),
                "store" => ops.push(Op::Store),
                "call" => ops.push(Op::Call),
                "swap" => ops.push(Op::Over(vec![1])),
                "var" => {
                    if let Some(m) = WHITESPACE.find(source) {
                        source = &source[m.as_str().len()..];
                    }
                    assert_eq!(source.chars().next().unwrap(), '(');
                    source = &source[1..];
                    parse_scopes.push(ParseScope::Var { old_ops: ops });
                    ops = vec![Op::EnterScope];
                    constants.push(HashMap::new());
                }
                "get" => {
                    if let Some(m) = WHITESPACE.find(source) {
                        source = &source[m.as_str().len()..];
                    }
                    assert_eq!(source.chars().next().unwrap(), '(');
                    source = &source[1..];
                    parse_scopes.push(ParseScope::Get { old_ops: ops });
                    ops = vec![Op::EnterScope];
                    constants.push(HashMap::new());
                }
                "proc_type" => {
                    if let Some(m) = WHITESPACE.find(source) {
                        source = &source[m.as_str().len()..];
                    }
                    assert_eq!(source.chars().next().unwrap(), '(');
                    source = &source[1..];
                    parse_scopes.push(ParseScope::ProcTypeParameterTypes { old_ops: ops });
                    ops = vec![Op::EnterScope];
                    constants.push(HashMap::new());
                }
                "proc" => {
                    if let Some(m) = WHITESPACE.find(source) {
                        source = &source[m.as_str().len()..];
                    }
                    assert_eq!(source.chars().next().unwrap(), '(');
                    source = &source[1..];
                    parse_scopes.push(ParseScope::ProcParameterTypes { old_ops: ops });
                    ops = vec![Op::EnterScope];
                    constants.push(HashMap::new());
                }
                "if" => {
                    parse_scopes.push(ParseScope::IfCondition);
                }
                "while" => {
                    parse_scopes.push(ParseScope::WhileCondition { old_ops: ops });
                    ops = vec![Op::EnterScope];
                    constants.push(HashMap::new());
                }
                "greater" => ops.push(Op::GreaterThan),
                "less" => ops.push(Op::LessThan),
                "const" => {
                    if let Some(m) = WHITESPACE.find(source) {
                        source = &source[m.as_str().len()..];
                    }
                    assert_eq!(source.chars().next().unwrap(), '(');
                    source = &source[1..];
                    parse_scopes.push(ParseScope::Const { old_ops: ops });
                    ops = vec![Op::EnterScope];
                    constants.push(HashMap::new());
                }
                _ => {
                    if let Some(values) = constants
                        .iter()
                        .rev()
                        .find_map(|scope| scope.get(identifier))
                    {
                        for value in values {
                            ops.push(Op::Push(value.clone()));
                        }
                    } else {
                        panic!("Unknown identifier '{identifier}'");
                    }
                }
            }
        } else if let (true, Some(ParseScope::IfCondition)) = (
            source.chars().next().unwrap() == '{' && parse_scopes.len() > 0,
            parse_scopes.last(),
        ) {
            source = &source[1..];
            parse_scopes.pop();
            parse_scopes.push(ParseScope::IfThen { old_ops: ops });
            ops = vec![Op::EnterScope];
            constants.push(HashMap::new());
        } else if let (true, Some(ParseScope::WhileCondition { .. })) = (
            source.chars().next().unwrap() == '{' && parse_scopes.len() > 0,
            parse_scopes.last(),
        ) {
            source = &source[1..];
            ops.push(Op::ExitScope);
            constants.pop();
            let old_ops =
                if let ParseScope::WhileCondition { old_ops } = parse_scopes.pop().unwrap() {
                    old_ops
                } else {
                    unreachable!()
                };
            parse_scopes.push(ParseScope::WhileBody {
                condition_ops: ops,
                old_ops,
            });
            ops = vec![Op::EnterScope];
            constants.push(HashMap::new());
        } else if source.chars().next().unwrap() == ')' && parse_scopes.len() > 0 {
            source = &source[1..];
            match parse_scopes.pop().unwrap() {
                ParseScope::Over { old_ops } => {
                    ops.push(Op::ExitScope);
                    constants.pop();
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_var_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::Integer,
                            "All elements left on the over offset stack must be integers"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_var_values.clone());
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
                    constants.pop();
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_var_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::String,
                            "All elements left on the var name stack must be strings"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_var_values.clone());
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
                    constants.pop();
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_var_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::String,
                            "All elements left on the get name stack must be strings"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_var_values.clone());
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
                    constants.pop();
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_var_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::Type,
                            "All elements left on the proc_type parameter type stack must be types"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_var_values.clone());
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
                    constants.push(HashMap::new());
                }
                ParseScope::ProcTypeReturnTypes {
                    parameter_types,
                    old_ops,
                } => {
                    ops.push(Op::ExitScope);
                    constants.pop();
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_var_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::Type,
                            "All elements left on the proc_type return type stack must be types"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_var_values.clone());
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
                    constants.pop();
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_var_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::Type,
                            "All elements left on the proc parameter type stack must be types"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_var_values.clone());
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
                    constants.push(HashMap::new());
                }
                ParseScope::ProcReturnTypes {
                    parameter_types,
                    old_ops,
                } => {
                    ops.push(Op::ExitScope);
                    constants.pop();
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_var_types.clone());
                    for typ in type_stack {
                        assert_eq!(
                            typ,
                            Type::Type,
                            "All elements left on the proc return type stack must be types"
                        );
                    }
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_var_values.clone());
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
                    constants.push(HashMap::new());
                }
                ParseScope::ProcBody { .. } => panic!("Cannot use ')' to close a proc body"),
                ParseScope::IfCondition => panic!("Cannot close if body before it is opened"),
                ParseScope::IfThen { .. } => {
                    panic!("Cannot use ')' to close the then scope of an if")
                }
                ParseScope::IfElse { .. } => {
                    panic!("Cannot use ')' to close the else scope of an if")
                }
                ParseScope::WhileCondition { .. } => {
                    panic!("Cannot close if body before it is opened")
                }
                ParseScope::WhileBody { .. } => {
                    panic!("Cannot use ')' to close a while body")
                }
                ParseScope::Const { old_ops } => {
                    ops.push(Op::ExitScope);
                    constants.pop();
                    let mut type_stack = vec![];
                    type_check(&ops, &mut type_stack, builtin_var_types.clone());
                    assert!(
                        type_stack.len() >= 1,
                        "Expected at least 1 element on the type stack"
                    );
                    assert_eq!(
                        type_stack[0],
                        Type::String,
                        "Expected the first element on the stack to be the const name but got '{}'",
                        type_stack[0]
                    );
                    let mut values = vec![];
                    execute(&ops, &mut values, builtin_var_values.clone());
                    let name = match &values[0] {
                        Value::String(value) => {
                            assert!(
                                IDENTIFIER.is_match(&value),
                                "Expected a valid identifier but got {value:?}"
                            );
                            value
                        }
                        _ => unreachable!(),
                    };
                    let values = &values[1..];
                    if let Some(_) = constants
                        .last_mut()
                        .unwrap()
                        .insert(name.clone(), values.iter().cloned().collect())
                    {
                        panic!("Redeclaration of constant '{name}'");
                    }
                    ops = old_ops;
                }
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
                    constants.pop();
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
                ParseScope::IfThen { old_ops } => {
                    ops.push(Op::ExitScope);
                    constants.pop();
                    if let Some(m) = ELSE.find(source) {
                        source = &source[m.as_str().len()..];
                        parse_scopes.push(ParseScope::IfElse {
                            then_ops: ops,
                            old_ops,
                        });
                        ops = vec![Op::EnterScope];
                        constants.push(HashMap::new());
                    } else {
                        let then_ops = ops;
                        ops = old_ops;
                        ops.push(Op::If {
                            then: then_ops,
                            r#else: vec![],
                        });
                    }
                }
                ParseScope::IfCondition => panic!("Cannot close if body before it is opened"),
                ParseScope::IfElse { then_ops, old_ops } => {
                    ops.push(Op::ExitScope);
                    constants.pop();
                    let else_ops = ops;
                    ops = old_ops;
                    ops.push(Op::If {
                        then: then_ops,
                        r#else: else_ops,
                    });
                }
                ParseScope::WhileCondition { .. } => {
                    panic!("Cannot close while body before it is opened")
                }
                ParseScope::WhileBody {
                    condition_ops,
                    old_ops,
                } => {
                    ops.push(Op::ExitScope);
                    constants.pop();
                    let body_ops = ops;
                    ops = old_ops;
                    ops.push(Op::While {
                        condition: condition_ops,
                        body: body_ops,
                    });
                }
                ParseScope::Const { .. } => {
                    panic!("Cannot use '}}' to close a const");
                }
            }
        } else {
            panic!("Unexpected character {:?}", source.chars().next().unwrap());
        }
    }
    ops.push(Op::ExitScope);
    constants.pop();
    assert_eq!(constants.len(), 0);
    type_check(&ops, &mut vec![], builtin_var_types);
    ops
}

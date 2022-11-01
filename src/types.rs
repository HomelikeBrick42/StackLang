use std::collections::HashMap;

use crate::Op;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Null,
    Type,
    String,
    Boolean,
    Character,
    Integer,
    Procedure {
        arguments: Vec<Type>,
        return_values: Vec<Type>,
    },
    Reference(Box<Type>),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Null => write!(f, "null_type"),
            Type::Type => write!(f, "type"),
            Type::String => write!(f, "string"),
            Type::Boolean => write!(f, "bool"),
            Type::Character => write!(f, "char"),
            Type::Integer => write!(f, "int"),
            Type::Procedure {
                arguments,
                return_values,
            } => {
                write!(f, "proc_type(")?;
                for (i, argument) in arguments.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{argument}")?;
                }
                write!(f, ") -> (")?;
                for (i, return_value) in return_values.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{return_value}")?;
                }
                write!(f, ")")
            }
            Type::Reference(referered_type) => write!(f, "{referered_type} ref"),
        }
    }
}

pub fn type_check<'a>(
    ops: impl IntoIterator<Item = &'a Op>,
    stack: &mut Vec<Type>,
    locals: HashMap<String, Type>,
) {
    let mut locals = vec![locals];
    for op in ops {
        match op {
            Op::DumpCurrentTypeStackInTypeChecking => {
                println!("Current types on the stack:");
                for typ in stack.iter().rev() {
                    println!("{typ}");
                }
                panic!("Dumped all the types on the stack");
            }
            Op::Push(value) => {
                stack.push(value.get_type());
            }
            Op::Dup => {
                assert!(stack.len() >= 1, "Expected at least 1 element to duplicate");
                let value = stack.pop().unwrap();
                stack.push(value.clone());
                stack.push(value);
            }
            Op::Drop => {
                assert!(
                    stack.len() >= 1,
                    "Expected at least 1 element to drop from the stack"
                );
                stack.pop();
            }
            Op::Over(depths) => {
                for depth in depths {
                    assert!(
                        stack.len() > *depth,
                        "The stack only has {} elements but tried to get an element {} elements deep",
                        stack.len(),
                        depth + 1
                    );
                    let value = stack.remove(stack.len() - depth - 1);
                    stack.push(value);
                }
            }
            Op::MakeProcedure { typ, ops } => {
                let mut current_locals = HashMap::new();
                for (name, local) in locals.iter().rev().flatten() {
                    if !current_locals.contains_key(name) {
                        current_locals.insert(name.clone(), local.clone());
                    }
                }
                let (arguments, return_values) = match typ {
                    Type::Procedure {
                        arguments,
                        return_values,
                    } => (arguments, return_values),
                    _ => panic!("Expected procedure type but got type '{typ}'"),
                };
                let mut func_stack = arguments.clone();
                type_check(ops.iter(), &mut func_stack, current_locals);
                assert_eq!(&func_stack, return_values); // TODO: error message
                stack.push(typ.clone());
            }
            Op::Call => {
                let procedure_type = stack
                    .pop()
                    .expect("Expected a procedure on the stack but got nothing");
                let (arguments, return_values) = match &procedure_type {
                    Type::Procedure {
                        arguments,
                        return_values,
                    } => (arguments, return_values),
                    _ => panic!("Expected a procedure to call but got type '{procedure_type}'"),
                };
                for (i, typ) in arguments.iter().enumerate().rev() {
                    let actual_typ = stack.pop().expect(&format!(
                        "Expected argument {i} on the stack but got nothing"
                    ));
                    assert_eq!(
                        typ, &actual_typ,
                        "Expected argument {i} to be type '{typ}' but got '{actual_typ}'"
                    );
                }
                stack.append(&mut return_values.clone());
            }
            Op::Add => {
                let b = stack
                    .pop()
                    .expect("Expected first operand to add on the stack but got nothing");
                let a = stack
                    .pop()
                    .expect("Expected second operand to add on the stack but got nothing");
                stack.push(match (&a, &b) {
                    (Type::Integer, Type::Integer) => Type::Integer,
                    _ => panic!("Cannot add types '{a}' and '{b}'"),
                });
            }
            Op::Subtract => {
                let b = stack
                    .pop()
                    .expect("Expected first operand to subtract on the stack but got nothing");
                let a = stack
                    .pop()
                    .expect("Expected second operand to subtract on the stack but got nothing");
                stack.push(match (&a, &b) {
                    (Type::Integer, Type::Integer) => Type::Integer,
                    _ => panic!("Cannot subtract types '{a}' and '{b}'"),
                });
            }
            Op::Multiply => {
                let b = stack
                    .pop()
                    .expect("Expected first operand to multiply on the stack but got nothing");
                let a = stack
                    .pop()
                    .expect("Expected second operand to multiply on the stack but got nothing");
                stack.push(match (&a, &b) {
                    (Type::Integer, Type::Integer) => Type::Integer,
                    _ => panic!("Cannot multiply types '{a}' and '{b}'"),
                });
            }
            Op::DivMod => {
                let b = stack
                    .pop()
                    .expect("Expected first operand to divmod on the stack but got nothing");
                let a = stack
                    .pop()
                    .expect("Expected second operand to divmod on the stack but got nothing");
                match (&a, &b) {
                    (Type::Integer, Type::Integer) => {
                        stack.push(Type::Integer);
                        stack.push(Type::Integer);
                    }
                    _ => panic!("Cannot use divmod on types '{a}' and '{b}'"),
                };
            }
            Op::EnterScope => {
                locals.push(HashMap::new());
            }
            Op::ExitScope => {
                locals.pop().unwrap(); // not an explicit instruction in the language so this *should* never break :)
            }
            Op::NewLocals(names) => {
                for name in names {
                    let value = stack
                        .pop()
                        .expect("Expected value to create new local variable with but got nothing");
                    assert_eq!(
                        locals.last_mut().unwrap().insert(name.clone(), value),
                        None,
                        "Redeclaration of local variable '{name}'"
                    );
                }
            }
            Op::GetLocals(names) => {
                for name in names {
                    let local = locals
                        .iter()
                        .rev()
                        .flatten()
                        .find_map(|(local_name, value)| {
                            if local_name == name {
                                Some(value)
                            } else {
                                None
                            }
                        });
                    if let Some(local) = local {
                        stack.push(Type::Reference(Box::new(local.clone())));
                    } else {
                        panic!("Unable to find name '{name}'");
                    }
                }
            }
            Op::Load => {
                let reference_type = stack
                    .pop()
                    .expect("Expected a reference to load from but got nothing");
                let referenced_type = match &reference_type {
                    Type::Reference(referenced) => referenced,
                    _ => panic!(
                        "Expected a reference type to load from but got type {reference_type}"
                    ),
                };
                stack.push((**referenced_type).clone());
            }
            Op::Store => {
                let reference_type = stack
                    .pop()
                    .expect("Expected a reference to store into but got nothing");
                let referenced_type = match &reference_type {
                    Type::Reference(referenced) => referenced,
                    _ => panic!(
                        "Expected a reference type to store into but got type '{reference_type}'"
                    ),
                };
                let typ = stack
                    .pop()
                    .expect("Expected a value to store but got nothing");
                assert_eq!(
                    &**referenced_type,
                    &typ,
                    "Expected the referenced type '{referenced_type}' to be the same as the value type '{typ}'"
                );
            }
            Op::TypeOf => {
                stack
                    .pop()
                    .expect("Expected a value to get the type of but got nothing");
                stack.push(Type::Type);
            }
            Op::GreaterThan => {
                let b = stack
                    .pop()
                    .expect("Expected first operand to greater than on the stack but got nothing");
                let a = stack
                    .pop()
                    .expect("Expected second operand to greater than on the stack but got nothing");
                stack.push(match (a, b) {
                    (Type::Integer, Type::Integer) => Type::Boolean,
                    (a, b) => panic!("Cannot compare greater than on types '{a}' and '{b}'"),
                });
            }
            Op::LessThan => {
                let b = stack
                    .pop()
                    .expect("Expected first operand to less than on the stack but got nothing");
                let a = stack
                    .pop()
                    .expect("Expected second operand to less than on the stack but got nothing");
                stack.push(match (a, b) {
                    (Type::Integer, Type::Integer) => Type::Boolean,
                    (a, b) => panic!("Cannot compare less than on types '{a}' and '{b}'"),
                });
            }
            Op::Equal => {
                let b = stack
                    .pop()
                    .expect("Expected first operand to equal on the stack but got nothing");
                let a = stack
                    .pop()
                    .expect("Expected second operand to equal on the stack but got nothing");
                assert_eq!(a, b, "Cannot compare types '{a}' and '{b}'");
                stack.push(Type::Boolean);
            }
            Op::Not => {
                let typ = stack
                    .pop()
                    .expect("Expected a boolean to not but got nothing");
                assert_eq!(typ, Type::Boolean, "Expected a boolean but got '{typ}'");
                stack.push(Type::Boolean);
            }
            Op::MakeReferenceType => {
                let typ = stack
                    .pop()
                    .expect("Expected a type to make a reference type from but got nothing");
                assert_eq!(typ, Type::Type, "Expected a type but got '{typ}'");
                stack.push(Type::Type);
            }
            Op::If { then, r#else } => {
                let condition = stack
                    .pop()
                    .expect("Expected if condition on the stack but got nothing");
                assert_eq!(
                    condition,
                    Type::Boolean,
                    "Expected a boolean for if condition but got '{condition}'"
                );
                let mut current_locals = HashMap::new();
                for (name, local) in locals.iter().rev().flatten() {
                    if !current_locals.contains_key(name) {
                        current_locals.insert(name.clone(), local.clone());
                    }
                }
                let mut then_stack = stack.clone();
                type_check(then, &mut then_stack, current_locals.clone());
                type_check(r#else, stack, current_locals);
                assert_eq!(
                    &then_stack, stack,
                    "Both paths through an if must result in the same types on the stack"
                );
            }
            Op::While { condition, body } => {
                let old_stack = stack.clone();
                let mut current_locals = HashMap::new();
                for (name, local) in locals.iter().rev().flatten() {
                    if !current_locals.contains_key(name) {
                        current_locals.insert(name.clone(), local.clone());
                    }
                }
                type_check(condition, stack, current_locals.clone());
                let condition = stack
                    .pop()
                    .expect("Expected while condition on the stack but got nothing");
                assert_eq!(
                    condition,
                    Type::Boolean,
                    "Expected a boolean for while condition but got '{condition}'"
                );
                assert_eq!(
                    &old_stack, stack,
                    "The number of elements after the while condition must be the same as before the while with an extra boolean on top"
                );
                type_check(body, stack, current_locals);
                assert_eq!(
                    &old_stack, stack,
                    "The number of elements after the while body must be the same as before the while"
                );
            }
            Op::Concat => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(match (a, b) {
                    (Type::String, Type::String) => Type::String,
                    (a, b) => panic!("Cannot concat types '{a}' and '{b}'"),
                });
            }
        }
    }
    assert_eq!(locals.len(), 1);
}

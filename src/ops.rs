use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::{Type, Value};

pub enum Op {
    DumpCurrentTypeStackInTypeChecking,
    Push(Value),
    Dup,
    Drop,
    Over(Vec<usize>), // 0 is the current top of the stack
    MakeProcedure { typ: Type, ops: Rc<Vec<Op>> },
    Call,
    Add,
    Subtract,
    Multiply,
    DivMod,
    EnterScope,
    ExitScope,
    NewLocals(Vec<String>),
    GetLocals(Vec<String>),
    Load,
    Store,
    TypeOf,
    GreaterThan,
    LessThan,
    Equal,
    Not,
    MakeReferenceType,
    If { then: Vec<Op>, r#else: Vec<Op> },
    While { condition: Vec<Op>, body: Vec<Op> },
}

pub fn execute<'a>(
    ops: impl IntoIterator<Item = &'a Op>,
    stack: &mut Vec<Value>,
    locals: HashMap<String, Rc<Cell<Value>>>,
) {
    let mut locals = vec![locals];
    for op in ops {
        match op {
            Op::DumpCurrentTypeStackInTypeChecking => {
                unreachable!("This instruction should never make it into a final program");
            }
            Op::Push(value) => {
                stack.push(value.clone());
            }
            Op::Dup => {
                let a = stack.pop().unwrap();
                stack.push(a.clone());
                stack.push(a);
            }
            Op::Drop => {
                stack.pop();
            }
            Op::Over(depths) => {
                for depth in depths {
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
                stack.push(Value::Function {
                    typ: typ.clone(),
                    ops: ops.clone(),
                    locals: current_locals,
                });
            }
            Op::Call => match stack.pop().unwrap() {
                Value::Function { ops, locals, .. } => {
                    execute(ops.iter(), stack, locals);
                }
                Value::BuiltinFunction(_, function) => {
                    function(stack);
                }
                _ => todo!(),
            },
            Op::Add => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                match (a, b) {
                    (Value::Integer(a), Value::Integer(b)) => stack.push(Value::Integer(a + b)),
                    (_, _) => todo!(),
                }
            }
            Op::Subtract => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                match (a, b) {
                    (Value::Integer(a), Value::Integer(b)) => stack.push(Value::Integer(a - b)),
                    (_, _) => todo!(),
                }
            }
            Op::Multiply => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                match (a, b) {
                    (Value::Integer(a), Value::Integer(b)) => stack.push(Value::Integer(a * b)),
                    (_, _) => todo!(),
                }
            }
            Op::DivMod => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                match (a, b) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        stack.push(Value::Integer(a / b));
                        stack.push(Value::Integer(a % b));
                    }
                    (_, _) => todo!(),
                }
            }
            Op::EnterScope => {
                locals.push(HashMap::new());
            }
            Op::ExitScope => {
                locals.pop();
            }
            Op::NewLocals(names) => {
                for name in names {
                    let value = stack
                        .pop()
                        .expect("Expected value to create new local variable with but got nothing");
                    assert!(
                        locals
                            .last_mut()
                            .unwrap()
                            .insert(name.clone(), Rc::new(Cell::new(value)))
                            .is_none(),
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
                        stack.push(Value::Reference(local.clone()));
                    } else {
                        todo!()
                    }
                }
            }
            Op::Load => {
                let reference = match stack.pop().unwrap() {
                    Value::Reference(reference) => reference,
                    _ => todo!(),
                };
                let value = reference.replace(Value::Null);
                let clone = value.clone();
                reference.set(value);
                stack.push(clone);
            }
            Op::Store => {
                let reference = match stack.pop().unwrap() {
                    Value::Reference(pointer) => pointer,
                    _ => todo!(),
                };
                let value = stack.pop().unwrap();
                reference.set(value);
            }
            Op::TypeOf => {
                let value = stack.pop().unwrap();
                stack.push(Value::Type(value.get_type()));
            }
            Op::GreaterThan => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(Value::Boolean(match (a, b) {
                    (Value::Integer(a), Value::Integer(b)) => a > b,
                    (_, _) => todo!(),
                }));
            }
            Op::LessThan => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(Value::Boolean(match (a, b) {
                    (Value::Integer(a), Value::Integer(b)) => a < b,
                    (_, _) => todo!(),
                }));
            }
            Op::Equal => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(Value::Boolean(a == b));
            }
            Op::Not => {
                let value = match stack.pop().unwrap() {
                    Value::Boolean(value) => value,
                    _ => todo!(),
                };
                stack.push(Value::Boolean(!value));
            }
            Op::MakeReferenceType => {
                let typ = match stack.pop().unwrap() {
                    Value::Type(typ) => typ,
                    _ => todo!(),
                };
                stack.push(Value::Type(Type::Reference(Box::new(typ))));
            }
            Op::If { then, r#else } => {
                let condition = stack.pop().unwrap();
                match condition {
                    Value::Boolean(condition) => {
                        let mut current_locals = HashMap::new();
                        for (name, local) in locals.iter().rev().flatten() {
                            if !current_locals.contains_key(name) {
                                current_locals.insert(name.clone(), local.clone());
                            }
                        }
                        if condition {
                            execute(then, stack, current_locals);
                        } else {
                            execute(r#else, stack, current_locals);
                        }
                    }
                    _ => todo!(),
                }
            }
            Op::While { condition, body } => loop {
                let mut current_locals = HashMap::new();
                for (name, local) in locals.iter().rev().flatten() {
                    if !current_locals.contains_key(name) {
                        current_locals.insert(name.clone(), local.clone());
                    }
                }
                execute(condition, stack, current_locals.clone());
                let condition = stack.pop().unwrap();
                match condition {
                    Value::Boolean(condition) => {
                        if !condition {
                            break;
                        }
                        execute(body, stack, current_locals);
                    }
                    _ => todo!(),
                }
            },
        }
    }
}

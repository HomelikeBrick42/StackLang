use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::{Type, Value};

pub enum Op {
    DumpCurrentTypeStackInTypeChecking,
    Push(Value),
    Dup,
    Drop,
    Over(usize), // 0 is the current top of the stack
    MakeFunction { typ: Type, ops: Rc<Vec<Op>> },
    Call,
    Return,
    Add,
    Subtract,
    Multiply,
    DivMod,
    EnterScope,
    ExitScope,
    CreateLocal(String),
    GetLocal(String),
    Load,
    Store,
    TypeOf,
    GreaterThan,
    LessThan,
    Equal,
    Not,
    MakeReferenceType,
}

pub fn execute(ops: &[Op], stack: &mut Vec<Value>, locals: HashMap<String, Rc<Cell<Value>>>) {
    let mut locals = vec![locals];
    let mut ip = 0;
    loop {
        match &ops[ip] {
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
            Op::Over(depth) => {
                let value = stack.remove(stack.len() - depth - 1);
                stack.push(value);
            }
            Op::MakeFunction { typ, ops } => {
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
                    execute(&ops, stack, locals);
                }
                Value::BuiltinFunction(_, function) => {
                    function(stack);
                }
                _ => todo!(),
            },
            Op::Return => break,
            Op::Add => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                match (a, b) {
                    (Value::S64(a), Value::S64(b)) => stack.push(Value::S64(a + b)),
                    (Value::S32(a), Value::S32(b)) => stack.push(Value::S32(a + b)),
                    (Value::S16(a), Value::S16(b)) => stack.push(Value::S16(a + b)),
                    (Value::S8(a), Value::S8(b)) => stack.push(Value::S8(a + b)),
                    (Value::U64(a), Value::U64(b)) => stack.push(Value::U64(a + b)),
                    (Value::U32(a), Value::U32(b)) => stack.push(Value::U32(a + b)),
                    (Value::U16(a), Value::U16(b)) => stack.push(Value::U16(a + b)),
                    (Value::U8(a), Value::U8(b)) => stack.push(Value::U8(a + b)),
                    (_, _) => todo!(),
                }
            }
            Op::Subtract => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                match (a, b) {
                    (Value::S64(a), Value::S64(b)) => stack.push(Value::S64(a - b)),
                    (Value::S32(a), Value::S32(b)) => stack.push(Value::S32(a - b)),
                    (Value::S16(a), Value::S16(b)) => stack.push(Value::S16(a - b)),
                    (Value::S8(a), Value::S8(b)) => stack.push(Value::S8(a - b)),
                    (Value::U64(a), Value::U64(b)) => stack.push(Value::U64(a - b)),
                    (Value::U32(a), Value::U32(b)) => stack.push(Value::U32(a - b)),
                    (Value::U16(a), Value::U16(b)) => stack.push(Value::U16(a - b)),
                    (Value::U8(a), Value::U8(b)) => stack.push(Value::U8(a - b)),
                    (_, _) => todo!(),
                }
            }
            Op::Multiply => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                match (a, b) {
                    (Value::S64(a), Value::S64(b)) => stack.push(Value::S64(a * b)),
                    (Value::S32(a), Value::S32(b)) => stack.push(Value::S32(a * b)),
                    (Value::S16(a), Value::S16(b)) => stack.push(Value::S16(a * b)),
                    (Value::S8(a), Value::S8(b)) => stack.push(Value::S8(a * b)),
                    (Value::U64(a), Value::U64(b)) => stack.push(Value::U64(a * b)),
                    (Value::U32(a), Value::U32(b)) => stack.push(Value::U32(a * b)),
                    (Value::U16(a), Value::U16(b)) => stack.push(Value::U16(a * b)),
                    (Value::U8(a), Value::U8(b)) => stack.push(Value::U8(a * b)),
                    (_, _) => todo!(),
                }
            }
            Op::DivMod => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                match (a, b) {
                    (Value::S64(a), Value::S64(b)) => {
                        stack.push(Value::S64(a / b));
                        stack.push(Value::S64(a % b));
                    }
                    (Value::S32(a), Value::S32(b)) => {
                        stack.push(Value::S32(a / b));
                        stack.push(Value::S32(a % b))
                    }
                    (Value::S16(a), Value::S16(b)) => {
                        stack.push(Value::S16(a / b));
                        stack.push(Value::S16(a % b))
                    }
                    (Value::S8(a), Value::S8(b)) => {
                        stack.push(Value::S8(a / b));
                        stack.push(Value::S8(a % b))
                    }
                    (Value::U64(a), Value::U64(b)) => {
                        stack.push(Value::U64(a / b));
                        stack.push(Value::U64(a % b))
                    }
                    (Value::U32(a), Value::U32(b)) => {
                        stack.push(Value::U32(a / b));
                        stack.push(Value::U32(a % b))
                    }
                    (Value::U16(a), Value::U16(b)) => {
                        stack.push(Value::U16(a / b));
                        stack.push(Value::U16(a % b))
                    }
                    (Value::U8(a), Value::U8(b)) => {
                        stack.push(Value::U8(a / b));
                        stack.push(Value::U8(a % b))
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
            Op::CreateLocal(name) => {
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
            Op::GetLocal(name) => {
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
                match (a, b) {
                    (_, _) => todo!(),
                }
            }
            Op::LessThan => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                match (a, b) {
                    (_, _) => todo!(),
                }
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
        }
        ip += 1;
    }
}

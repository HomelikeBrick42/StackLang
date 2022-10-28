use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::{Op, Type};

#[derive(Clone)]
pub enum Value {
    Null,
    Type(Type),
    String(String),
    Boolean(bool),
    Character(char),
    S64(i64),
    S32(i32),
    S16(i16),
    S8(i8),
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
    Function {
        typ: Type,
        ops: Rc<Vec<Op>>,
        locals: HashMap<String, Rc<Cell<Value>>>,
    },
    BuiltinFunction(Type, Rc<dyn Fn(&mut Vec<Value>)>),
    Reference(Rc<Cell<Value>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Type(a), Value::Type(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Character(a), Value::Character(b)) => a == b,
            (Value::S64(a), Value::S64(b)) => a == b,
            (Value::S32(a), Value::S32(b)) => a == b,
            (Value::S16(a), Value::S16(b)) => a == b,
            (Value::S8(a), Value::S8(b)) => a == b,
            (Value::U64(a), Value::U64(b)) => a == b,
            (Value::U32(a), Value::U32(b)) => a == b,
            (Value::U16(a), Value::U16(b)) => a == b,
            (Value::U8(a), Value::U8(b)) => a == b,
            (
                Value::Function {
                    typ: a_typ,
                    ops: a_ops,
                    ..
                },
                Value::Function {
                    typ: b_typ,
                    ops: b_ops,
                    ..
                },
            ) => a_typ == b_typ && a_ops.as_ptr() == b_ops.as_ptr(),
            (Value::BuiltinFunction(_, _), Value::BuiltinFunction(_, _)) => false, // TODO: find a way to compare builtin functions
            (Value::Reference(a), Value::Reference(b)) => a.as_ptr() == b.as_ptr(),
            _ => false,
        }
    }
}

impl Value {
    pub fn get_type(&self) -> Type {
        match self {
            Value::Null => Type::Null,
            Value::Type(_) => Type::Type,
            Value::String(_) => Type::String,
            Value::Boolean(_) => Type::Boolean,
            Value::Character(_) => Type::Character,
            Value::S64(_) => Type::S64,
            Value::S32(_) => Type::S32,
            Value::S16(_) => Type::S16,
            Value::S8(_) => Type::S8,
            Value::U64(_) => Type::U64,
            Value::U32(_) => Type::U32,
            Value::U16(_) => Type::U16,
            Value::U8(_) => Type::U8,
            Value::Function { typ, .. } => typ.clone(),
            Value::BuiltinFunction(typ, _) => typ.clone(),
            Value::Reference(reference) => {
                let value = reference.replace(Value::Null);
                let clone = value.clone();
                reference.set(value);
                Type::Reference(Box::new(clone.get_type()))
            }
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Type(typ) => write!(f, "{typ}"),
            Value::String(value) => write!(f, "{value:?}"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::Character(value) => write!(f, "{value:?}"),
            Value::S64(value) => write!(f, "{value}"),
            Value::S32(value) => write!(f, "{value}"),
            Value::S16(value) => write!(f, "{value}"),
            Value::S8(value) => write!(f, "{value}"),
            Value::U64(value) => write!(f, "{value}"),
            Value::U32(value) => write!(f, "{value}"),
            Value::U16(value) => write!(f, "{value}"),
            Value::U8(value) => write!(f, "{value}"),
            Value::Function { typ, .. } => write!(f, "{typ}"),
            Value::BuiltinFunction(typ, _) => write!(f, "{typ}"),
            Value::Reference(reference) => {
                let value = reference.replace(Value::Null);
                let clone = value.clone();
                reference.set(value);
                write!(f, "{clone}")
            }
        }
    }
}

use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::{Op, Type};

#[derive(Clone)]
pub enum Value {
    Null,
    Type(Type),
    String(String),
    Label(String),
    Boolean(bool),
    Character(char),
    Integer(i64),
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
            (Value::Label(a), Value::Label(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Character(a), Value::Character(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
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
            Value::Label(_) => Type::Label,
            Value::Boolean(_) => Type::Boolean,
            Value::Character(_) => Type::Character,
            Value::Integer(_) => Type::Integer,
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
            Value::Label(value) => write!(f, ":{value}"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::Character(value) => write!(f, "{value:?}"),
            Value::Integer(value) => write!(f, "{value}"),
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

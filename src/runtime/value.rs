use rust_decimal::Decimal;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::parser::{Stmt, Type, Expr};
use super::builtin::{BuiltinStruct, BuiltinFunction, BuiltinMethod};

#[derive(Clone)]
pub enum Value {
    Unit,
    Number(Decimal),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Function {
        name: String,
        params: Vec<(String, String)>, // (name, type)
        body: Box<Stmt>,
        is_async: bool,
    },
    Closure {
        params: Vec<(String, Option<Type>)>,
        return_type: Option<Type>,
        body: Box<Expr>,
        captured_env: HashMap<String, Rc<RefCell<Value>>>, // Captured variables from enclosing scope
    },
    Struct {
        name: String,
        fields: HashMap<String, Value>,
    },
    Builtin(BuiltinStruct),
    BuiltinFn(BuiltinFunction),
    BoundMethod {
        object: Box<Value>,
        method_name: String,
        method: BuiltinMethod,
    },
    UserMethod {
        object: Box<Value>,
        method_name: String,
        params: Vec<(String, Type)>,
        return_type: Option<Type>,
        body: Box<Stmt>,
        is_async: bool,
    },
    Promise(Rc<RefCell<PromiseData>>),
    EnumVariant {
        enum_name: String,
        variant_name: String,
        values: Vec<Value>, // Tuple values for tuple variants, empty for unit variants
    },
    EnumConstructor {
        enum_name: String,
        variant_name: String,
        arity: usize, // Number of arguments expected
    },
    Module {
        name: String,
        exports: HashMap<String, Value>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PromiseData {
    pub state: PromiseState,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PromiseState {
    Pending(Value), // A closure to execute
    Resolved(Value),
    Rejected(Value),
}

// Manual Debug implementation since BuiltinFunction doesn't implement Debug
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => write!(f, "Unit"),
            Value::Number(n) => write!(f, "Number({:?})", n),
            Value::String(s) => write!(f, "String({:?})", s),
            Value::Boolean(b) => write!(f, "Boolean({:?})", b),
            Value::Array(arr) => write!(f, "Array({:?})", arr),
            Value::Function { name, params, is_async, .. } => {
                write!(f, "Function {{ name: {:?}, params: {:?}, is_async: {:?} }}", name, params, is_async)
            }
            Value::Closure { params, .. } => {
                write!(f, "Closure {{ params: {:?} }}", params)
            }
            Value::Struct { name, fields } => {
                write!(f, "Struct {{ name: {:?}, fields: {:?} }}", name, fields)
            }
            Value::Builtin(builtin) => write!(f, "Builtin({:?})", builtin),
            Value::BuiltinFn(_) => write!(f, "BuiltinFn(<native function>)"),
            Value::BoundMethod { object, method_name, .. } => {
                write!(f, "BoundMethod {{ object: {:?}, method: {:?} }}", object, method_name)
            }
            Value::UserMethod { object, method_name, is_async, .. } => {
                write!(f, "UserMethod {{ object: {:?}, method: {:?}, is_async: {:?} }}", object, method_name, is_async)
            }
            Value::Promise(data) => write!(f, "Promise({:?})", data.borrow()),
            Value::EnumVariant { enum_name, variant_name, values } => {
                write!(f, "EnumVariant {{ {}::{}, values: {:?} }}", enum_name, variant_name, values)
            }
            Value::EnumConstructor { enum_name, variant_name, arity } => {
                write!(f, "EnumConstructor {{ {}::{}, arity: {} }}", enum_name, variant_name, arity)
            }
            Value::Module { name, .. } => {
                write!(f, "Module {{ {} }}", name)
            }
        }
    }
}

// Manual PartialEq implementation
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Unit, Value::Unit) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Function { name: n1, params: p1, is_async: a1, .. }, Value::Function { name: n2, params: p2, is_async: a2, .. }) => {
                n1 == n2 && p1 == p2 && a1 == a2
            }
            (Value::Closure { params: p1, captured_env: c1, .. }, Value::Closure { params: p2, captured_env: c2, .. }) => {
                // Closures are compared by their parameter signatures and captured environment identity
                if p1 != p2 || c1.len() != c2.len() {
                    return false;
                }
                for (k, v1) in c1 {
                    if let Some(v2) = c2.get(k) {
                        if !Rc::ptr_eq(v1, v2) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            (Value::Struct { name: n1, fields: f1 }, Value::Struct { name: n2, fields: f2 }) => {
                n1 == n2 && f1 == f2
            }
            (Value::Builtin(a), Value::Builtin(b)) => a.name == b.name,
            (Value::BoundMethod { object: o1, method_name: m1, .. }, Value::BoundMethod { object: o2, method_name: m2, .. }) => {
                o1 == o2 && m1 == m2
            }
            (Value::UserMethod { object: o1, method_name: m1, .. }, Value::UserMethod { object: o2, method_name: m2, .. }) => {
                o1 == o2 && m1 == m2
            }
            (Value::Promise(a), Value::Promise(b)) => Rc::ptr_eq(a, b),
            (Value::EnumVariant { enum_name: e1, variant_name: v1, values: vals1 }, 
             Value::EnumVariant { enum_name: e2, variant_name: v2, values: vals2 }) => {
                e1 == e2 && v1 == v2 && vals1 == vals2
            }
            (Value::EnumConstructor { enum_name: e1, variant_name: v1, arity: a1 },
             Value::EnumConstructor { enum_name: e2, variant_name: v2, arity: a2 }) => {
                e1 == e2 && v1 == v2 && a1 == a2
            }
            (Value::Module { name: n1, .. }, Value::Module { name: n2, .. }) => n1 == n2,
            _ => false,
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Unit => false,
            Value::Number(n) => *n != Decimal::ZERO,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Function { .. } => true,
            Value::Closure { .. } => true,
            Value::Struct { .. } => true,
            Value::Builtin(_) => true,
            Value::BuiltinFn(_) => true,
            Value::BoundMethod { .. } => true,
            Value::UserMethod { .. } => true,
            Value::Promise(_) => true,
            Value::EnumVariant { .. } => true,
            Value::EnumConstructor { .. } => true,
            Value::Module { .. } => true,
        }
    }
}

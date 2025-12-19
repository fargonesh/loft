use super::value::Value;
use super::{RuntimeError, RuntimeResult};
use std::collections::HashMap;

/// Represents a native Rust function that can be called from loft
pub type BuiltinFunction = fn(&[Value]) -> RuntimeResult<Value>;

/// Represents a builtin method attached to a struct
pub type BuiltinMethod = fn(&Value, &[Value]) -> RuntimeResult<Value>;

/// A builtin struct that can be instantiated and used in loft
#[derive(Debug, Clone)]
pub struct BuiltinStruct {
    pub name: String,
    pub fields: HashMap<String, Value>,
    pub methods: HashMap<String, BuiltinMethod>,
}

impl BuiltinStruct {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, name: impl Into<String>, value: Value) {
        self.fields.insert(name.into(), value);
    }

    pub fn add_method(&mut self, name: impl Into<String>, method: BuiltinMethod) {
        self.methods.insert(name.into(), method);
    }

    pub fn call_method(&self, method_name: &str, args: &[Value]) -> RuntimeResult<Value> {
        if let Some(method) = self.methods.get(method_name) {
            method(&Value::Builtin(self.clone()), args)
        } else {
            Err(RuntimeError::new(format!(
                "Method '{}' not found on builtin struct '{}'",
                method_name, self.name
            )))
        }
    }
}

/// Macro to easily define builtin functions
#[macro_export]
macro_rules! builtin_fn {
    ($name:ident, $fn:expr) => {
        pub fn $name(args: &[Value]) -> RuntimeResult<Value> {
            $fn(args)
        }
    };
}

/// Macro to easily define builtin methods
#[macro_export]
macro_rules! builtin_method {
    ($name:ident, $fn:expr) => {
        pub fn $name(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
            $fn(this, args)
        }
    };
}

use super::value::Value;
use super::{RuntimeError, RuntimeResult, Interpreter};
use std::collections::HashMap;

/// Represents a native Rust function that can be called from loft
pub type BuiltinFunction = fn(&mut Interpreter, &[Value]) -> RuntimeResult<Value>;

/// Represents a builtin method attached to a struct
pub type BuiltinMethod = fn(&mut Interpreter, &Value, &[Value]) -> RuntimeResult<Value>;

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

    pub fn call_method(&self, interpreter: &mut Interpreter, method_name: &str, args: &[Value]) -> RuntimeResult<Value> {
        if let Some(method) = self.methods.get(method_name) {
            method(interpreter, &Value::Builtin(self.clone()), args)
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

/// Macro to extract and validate arguments in builtin functions
#[macro_export]
macro_rules! loft_arg {
    ($args:expr, $idx:expr, Number, $name:expr) => {
        match $args.get($idx) {
            Some($crate::runtime::value::Value::Number(n)) => *n,
            Some(_) => return Err($crate::runtime::RuntimeError::new(format!(
                "{} argument {} must be a number", $name, $idx + 1
            ))),
            None => return Err($crate::runtime::RuntimeError::new(format!(
                "{} requires a number argument at position {}", $name, $idx + 1
            ))),
        }
    };
    ($args:expr, $idx:expr, String, $name:expr) => {
        match $args.get($idx) {
            Some($crate::runtime::value::Value::String(s)) => s,
            Some(_) => return Err($crate::runtime::RuntimeError::new(format!(
                "{} argument {} must be a string", $name, $idx + 1
            ))),
            None => return Err($crate::runtime::RuntimeError::new(format!(
                "{} requires a string argument at position {}", $name, $idx + 1
            ))),
        }
    };
    ($args:expr, $idx:expr, Boolean, $name:expr) => {
        match $args.get($idx) {
            Some($crate::runtime::value::Value::Boolean(b)) => *b,
            Some(_) => return Err($crate::runtime::RuntimeError::new(format!(
                "{} argument {} must be a boolean", $name, $idx + 1
            ))),
            None => return Err($crate::runtime::RuntimeError::new(format!(
                "{} requires a boolean argument at position {}", $name, $idx + 1
            ))),
        }
    };
    ($args:expr, $idx:expr, Array, $name:expr) => {
        match $args.get($idx) {
            Some($crate::runtime::value::Value::Array(a)) => a,
            Some(_) => return Err($crate::runtime::RuntimeError::new(format!(
                "{} argument {} must be an array", $name, $idx + 1
            ))),
            None => return Err($crate::runtime::RuntimeError::new(format!(
                "{} requires an array argument at position {}", $name, $idx + 1
            ))),
        }
    };
}

/// Macro to extract and validate 'this' in builtin methods
#[macro_export]
macro_rules! loft_this {
    ($this:expr, String, $name:expr) => {
        match $this {
            $crate::runtime::value::Value::String(s) => s,
            _ => return Err($crate::runtime::RuntimeError::new(format!(
                "{} can only be called on strings", $name
            ))),
        }
    };
    ($this:expr, Array, $name:expr) => {
        match $this {
            $crate::runtime::value::Value::Array(a) => a,
            _ => return Err($crate::runtime::RuntimeError::new(format!(
                "{} can only be called on arrays", $name
            ))),
        }
    };
}

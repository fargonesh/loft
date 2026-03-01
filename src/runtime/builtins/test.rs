use crate::runtime::builtin::{BuiltinMethod, BuiltinStruct};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use loft_builtin_macros::loft_builtin;

/// Assert that a condition is true
#[loft_builtin(test.assert)]
pub fn test_assert(#[required] _this: &Value, #[types(bool*)] args: &[Value]) -> RuntimeResult<Value> {
    let condition = match &args[0] {
        Value::Boolean(b) => *b,
        _ => unreachable!(),
    };

    if !condition {
        let message = if args.len() > 1 {
            match &args[1] {
                Value::String(s) => s.clone(),
                _ => "Assertion failed".to_string(),
            }
        } else {
            "Assertion failed".to_string()
        };
        return Err(RuntimeError::new(message));
    }

    Ok(Value::Unit)
}

/// Assert that two values are equal
#[loft_builtin(test.assert_eq)]
pub fn test_assert_eq(#[required] _this: &Value, #[required] args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new("test.assert_eq() requires two arguments"));
    }

    let left = &args[0];
    let right = &args[1];

    if left != right {
        let message = if args.len() > 2 {
            match &args[2] {
                Value::String(s) => s.clone(),
                _ => format!("Assertion failed: {:?} != {:?}", left, right),
            }
        } else {
            format!("Assertion failed: {:?} != {:?}", left, right)
        };
        return Err(RuntimeError::new(message));
    }

    Ok(Value::Unit)
}

pub fn create_test_builtin() -> BuiltinStruct {
    let mut methods = std::collections::HashMap::new();

    methods.insert(
        "assert".to_string(),
        test_assert as BuiltinMethod,
    );
    methods.insert(
        "assert_eq".to_string(),
        test_assert_eq as BuiltinMethod,
    );

    BuiltinStruct {
        name: "test".to_string(),
        fields: std::collections::HashMap::new(),
        methods,
    }
}

use crate::runtime::builtin_registry::BuiltinRegistration;

inventory::submit! {
    BuiltinRegistration {
        name: "test",
        factory: create_test_builtin,
        feature: None,
    }
}

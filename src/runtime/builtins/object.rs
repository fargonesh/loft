use crate::runtime::builtin::{BuiltinMethod, BuiltinStruct};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use loft_builtin_macros::{loft_builtin, required, types};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Get all keys from an object
#[loft_builtin(object.keys)]
fn object_keys(#[required] _this: &Value, #[types(object)] args: &[Value]) -> RuntimeResult<Value> {
    match &args[0] {
        Value::Struct { fields, .. } => {
            let keys: Vec<Value> = fields.keys().map(|k| Value::String(k.clone())).collect();
            Ok(Value::Array(keys))
        }
        _ => unreachable!(),
    }
}

/// Get all values from an object
#[loft_builtin(object.values)]
fn object_values(#[required] _this: &Value, #[types(object)] args: &[Value]) -> RuntimeResult<Value> {
    match &args[0] {
        Value::Struct { fields, .. } => {
            let values: Vec<Value> = fields.values().cloned().collect();
            Ok(Value::Array(values))
        }
        _ => unreachable!(),
    }
}

/// Get all entries from an object as [key, value] pairs
#[loft_builtin(object.entries)]
fn object_entries(#[required] _this: &Value, #[types(object)] args: &[Value]) -> RuntimeResult<Value> {
    match &args[0] {
        Value::Struct { fields, .. } => {
            let entries: Vec<Value> = fields
                .iter()
                .map(|(k, v)| Value::Array(vec![Value::String(k.clone()), v.clone()]))
                .collect();
            Ok(Value::Array(entries))
        }
        _ => unreachable!(),
    }
}

/// Check if object has a property
#[loft_builtin(object.has)]
fn object_has(#[required] _this: &Value, #[types(object, string)] args: &[Value]) -> RuntimeResult<Value> {
    match (&args[0], &args[1]) {
        (Value::Struct { fields, .. }, Value::String(key)) => {
            Ok(Value::Boolean(fields.contains_key(key)))
        }
        _ => unreachable!(),
    }
}

/// Assign properties from source objects to target object
#[loft_builtin(object.assign)]
#[required]
#[types(type*)]
fn object_assign(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let result_fields;

    // Start with first object
    if let Value::Struct { fields, .. } = &args[0] {
        result_fields = fields.clone();
    } else {
        return Err(RuntimeError::new("object.assign() target must be an object"));
    }

    // Merge in additional objects
    let mut result_fields = result_fields;
    for arg in &args[1..] {
        if let Value::Struct { fields, .. } = arg {
            for (key, value) in fields {
                result_fields.insert(key.clone(), value.clone());
            }
        } else {
            return Err(RuntimeError::new("object.assign() sources must be objects"));
        }
    }

    Ok(Value::Struct {
        name: "Object".to_string(),
        fields: result_fields,
    })
}

/// Create an object from entries [[key, value], ...]
#[loft_builtin(object.from_entries)]
#[required]
#[types(array)]
fn object_from_entries(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    match &args[0] {
        Value::Array(entries) => {
            let mut fields = HashMap::new();

            for entry in entries {
                match entry {
                    Value::Array(pair) if pair.len() >= 2 => {
                        if let Value::String(key) = &pair[0] {
                            fields.insert(key.clone(), pair[1].clone());
                        } else {
                            return Err(RuntimeError::new("Entry key must be a string"));
                        }
                    }
                    _ => return Err(RuntimeError::new("Each entry must be a [key, value] array")),
                }
            }

            Ok(Value::Struct {
                name: "Object".to_string(),
                fields,
            })
        }
        _ => unreachable!(),
    }
}

/// Get the number of properties in an object
#[loft_builtin(object.size)]
#[required]
#[types(object)]
fn object_size(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    match &args[0] {
        Value::Struct { fields, .. } => Ok(Value::Number(Decimal::from(fields.len()))),
        _ => unreachable!(),
    }
}

pub fn create_object_builtin() -> BuiltinStruct {
    let mut object = BuiltinStruct::new("object");

    object.add_method("keys", object_keys as BuiltinMethod);
    object.add_method("values", object_values as BuiltinMethod);
    object.add_method("entries", object_entries as BuiltinMethod);
    object.add_method("has", object_has as BuiltinMethod);
    object.add_method("assign", object_assign as BuiltinMethod);
    object.add_method("from_entries", object_from_entries as BuiltinMethod);
    object.add_method("size", object_size as BuiltinMethod);

    object
}

// Register the builtin automatically
crate::submit_builtin!("object", create_object_builtin);

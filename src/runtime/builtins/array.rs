use crate::runtime::builtin::{BuiltinMethod, BuiltinStruct};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use loft_builtin_macros::loft_builtin;
use rust_decimal::Decimal;

/// Get the length of an array
#[loft_builtin(array.length)]
// TODO: Elide with #[required] and #[types(array)] for 'this'
fn array_length(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => Ok(Value::Number(Decimal::from(arr.len()))),
        _ => Err(RuntimeError::new("length() can only be called on arrays")),
    }
}

/// Push a value to the end of an array
/// Returns a new array with the value added
#[loft_builtin(array.push)]
// TODO: Elide with #[required] and #[types(array)] for 'this'
fn array_push(this: &Value, #[required] args: &[Value]) -> RuntimeResult<Value> {
    // Note: 'this' type check is manual
    match this {
        Value::Array(arr) => {
            let mut new_arr = arr.clone();
            new_arr.push(args[0].clone());
            Ok(Value::Array(new_arr))
        }
        _ => Err(RuntimeError::new("push() can only be called on arrays")),
    }
}

/// Pop a value from the end of an array
/// Returns the popped value, or Unit if array is empty
/// Note: This doesn't modify the original array
#[loft_builtin(array.pop)]
// TODO: Elide with #[required] and #[types(array)] for 'this'
fn array_pop(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Value::Unit)
            } else {
                Ok(arr.last().unwrap().clone())
            }
        }
        _ => Err(RuntimeError::new("pop() can only be called on arrays")),
    }
}

/// Remove last element and return new array
#[loft_builtin(array.remove_last)]
// TODO: Elide with #[required] and #[types(array)] for 'this'
fn array_remove_last(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Value::Array(arr.clone()))
            } else {
                let mut new_arr = arr.clone();
                new_arr.pop();
                Ok(Value::Array(new_arr))
            }
        }
        _ => Err(RuntimeError::new(
            "remove_last() can only be called on arrays",
        )),
    }
}

/// Get a value at a specific index
#[loft_builtin(array.get)]
// TODO: Elide with #[required] and #[types(array)] for 'this'
fn array_get(this: &Value, #[types(number)] args: &[Value]) -> RuntimeResult<Value> {
    match (this, &args[0]) {
        (Value::Array(arr), Value::Number(idx)) => {
            let idx_usize = idx
                .to_string()
                .parse::<usize>()
                .map_err(|_| RuntimeError::new("Array index must be a non-negative integer"))?;

            Ok(arr.get(idx_usize).cloned().unwrap_or(Value::Unit))
        }
        (Value::Array(_), _) => unreachable!(),
        _ => Err(RuntimeError::new("get() can only be called on arrays")),
    }
}

/// Set a value at a specific index (returns new array)
#[loft_builtin(array.set)]
// TODO: Elide with #[required] and #[types(array)] for 'this'
fn array_set(this: &Value, #[types(number, _)] args: &[Value]) -> RuntimeResult<Value> {
    match (this, &args[0], &args[1]) {
        (Value::Array(arr), Value::Number(idx), value) => {
            let idx_usize = idx
                .to_string()
                .parse::<usize>()
                .map_err(|_| RuntimeError::new("Array index must be a non-negative integer"))?;

            if idx_usize >= arr.len() {
                return Err(RuntimeError::new(format!(
                    "Array index {} out of bounds",
                    idx_usize
                )));
            }

            let mut new_arr = arr.clone();
            new_arr[idx_usize] = value.clone();
            Ok(Value::Array(new_arr))
        }
        (Value::Array(_), _, _) => Err(RuntimeError::new("Array index must be a number")),
        _ => Err(RuntimeError::new("set() can only be called on arrays")),
    }
}

/// Check if array is empty
#[loft_builtin(array.is_empty)]
// TODO: Elide with #[required] and #[types(array)] for 'this'
fn array_is_empty(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => Ok(Value::Boolean(arr.is_empty())),
        _ => Err(RuntimeError::new("is_empty() can only be called on arrays")),
    }
}

/// Create a slice of the array
#[loft_builtin(array.slice)]
// TODO: Elide with #[required] and #[types(array)] for 'this'
fn array_slice(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            let start = if args.is_empty() {
                0
            } else {
                match &args[0] {
                    Value::Number(n) => n.to_string().parse::<usize>().map_err(|_| {
                        RuntimeError::new("Start index must be a non-negative integer")
                    })?,
                    _ => return Err(RuntimeError::new("Start index must be a number")),
                }
            };

            let end = if args.len() < 2 {
                arr.len()
            } else {
                match &args[1] {
                    Value::Number(n) => n.to_string().parse::<usize>().map_err(|_| {
                        RuntimeError::new("End index must be a non-negative integer")
                    })?,
                    _ => return Err(RuntimeError::new("End index must be a number")),
                }
            };

            if start > arr.len() || end > arr.len() || start > end {
                return Err(RuntimeError::new("Invalid slice indices"));
            }

            let sliced = arr[start..end].to_vec();
            Ok(Value::Array(sliced))
        }
        _ => Err(RuntimeError::new("slice() can only be called on arrays")),
    }
}

/// Create the Array builtin struct
pub fn create_array_builtin() -> BuiltinStruct {
    let mut array = BuiltinStruct::new("array");

    // Basic array methods
    array.add_method("length", array_length as BuiltinMethod);
    array.add_method("len", array_length as BuiltinMethod); // Alias
    array.add_method("push", array_push as BuiltinMethod);
    array.add_method("pop", array_pop as BuiltinMethod);
    array.add_method("remove_last", array_remove_last as BuiltinMethod);
    array.add_method("get", array_get as BuiltinMethod);
    array.add_method("set", array_set as BuiltinMethod);
    array.add_method("is_empty", array_is_empty as BuiltinMethod);
    array.add_method("slice", array_slice as BuiltinMethod);

    // Enhanced collection methods
    use crate::runtime::builtins::collections::array as collection_array;
    collection_array::register_collection_methods(&mut array);

    array
}

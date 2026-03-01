use crate::runtime::builtin::{BuiltinMethod, BuiltinStruct};
use crate::runtime::traits::ToString;
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use loft_builtin_macros::loft_builtin;
use rust_decimal::Decimal;

/// Map function - applies a function to each element (placeholder for now)
/// Note: Full implementation would require function execution
#[loft_builtin(array.map)]
fn array_map(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(_) => {
            // Placeholder - full implementation would execute the function on each element
            Err(RuntimeError::new(
                "map() requires function execution support (coming soon)",
            ))
        }
        _ => Err(RuntimeError::new("map() can only be called on arrays")),
    }
}

/// Filter function - filters elements (placeholder for now)
#[loft_builtin(array.filter)]
fn array_filter(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(_) => {
            // Placeholder - full implementation would execute the predicate on each element
            Err(RuntimeError::new(
                "filter() requires function execution support (coming soon)",
            ))
        }
        _ => Err(RuntimeError::new("filter() can only be called on arrays")),
    }
}

/// Reduce function - reduces array to single value (placeholder for now)
#[loft_builtin(array.reduce)]
fn array_reduce(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(_) => {
            // Placeholder - full implementation would execute the reducer function
            Err(RuntimeError::new(
                "reduce() requires function execution support (coming soon)",
            ))
        }
        _ => Err(RuntimeError::new("reduce() can only be called on arrays")),
    }
}

/// Find first element matching predicate (placeholder for now)
#[loft_builtin(array.find)]
fn array_find(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(_) => {
            // Placeholder - full implementation would execute the predicate
            Err(RuntimeError::new(
                "find() requires function execution support (coming soon)",
            ))
        }
        _ => Err(RuntimeError::new("find() can only be called on arrays")),
    }
}

/// Zip two arrays together
#[loft_builtin(array.zip)]
fn array_zip(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new("zip() requires two array arguments"));
    }

    match (&args[0], &args[1]) {
        (Value::Array(arr1), Value::Array(arr2)) => {
            let min_len = arr1.len().min(arr2.len());
            let zipped: Vec<Value> = (0..min_len)
                .map(|i| Value::Array(vec![arr1[i].clone(), arr2[i].clone()]))
                .collect();
            Ok(Value::Array(zipped))
        }
        _ => Err(RuntimeError::new("zip() arguments must be arrays")),
    }
}

/// Chain/concatenate multiple arrays
#[loft_builtin(array.chain)]
fn array_chain(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let mut result = Vec::new();

    for arg in args {
        match arg {
            Value::Array(arr) => result.extend(arr.clone()),
            _ => return Err(RuntimeError::new("chain() arguments must be arrays")),
        }
    }

    Ok(Value::Array(result))
}

/// Flatten an array of arrays by one level
#[loft_builtin(array.flatten)]
fn array_flatten(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            let mut result = Vec::new();

            for item in arr {
                match item {
                    Value::Array(inner) => result.extend(inner.clone()),
                    other => result.push(other.clone()),
                }
            }

            Ok(Value::Array(result))
        }
        _ => Err(RuntimeError::new("flatten() can only be called on arrays")),
    }
}

/// Reverse an array
#[loft_builtin(array.reverse)]
fn array_reverse(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            let mut reversed = arr.clone();
            reversed.reverse();
            Ok(Value::Array(reversed))
        }
        _ => Err(RuntimeError::new("reverse() can only be called on arrays")),
    }
}

/// Sort an array (numbers only for now)
#[loft_builtin(array.sort)]
fn array_sort(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            // Check if all elements are numbers
            let mut numbers: Vec<Decimal> = Vec::new();
            for item in arr {
                match item {
                    Value::Number(n) => numbers.push(*n),
                    _ => {
                        return Err(RuntimeError::new(
                            "sort() currently only supports arrays of numbers",
                        ))
                    }
                }
            }

            numbers.sort();
            let sorted: Vec<Value> = numbers.into_iter().map(Value::Number).collect();
            Ok(Value::Array(sorted))
        }
        _ => Err(RuntimeError::new("sort() can only be called on arrays")),
    }
}

/// Check if array includes a value
#[loft_builtin(array.includes)]
fn array_includes(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("includes() requires a value argument"));
    }

    match this {
        Value::Array(arr) => {
            let search_value = &args[0];
            Ok(Value::Boolean(arr.contains(search_value)))
        }
        _ => Err(RuntimeError::new("includes() can only be called on arrays")),
    }
}

/// Find the index of a value in an array
#[loft_builtin(array.index_of)]
fn array_index_of(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("index_of() requires a value argument"));
    }

    match this {
        Value::Array(arr) => {
            let search_value = &args[0];
            match arr.iter().position(|v| v == search_value) {
                Some(index) => Ok(Value::Number(Decimal::from(index))),
                None => Ok(Value::Number(Decimal::from(-1))),
            }
        }
        _ => Err(RuntimeError::new("index_of() can only be called on arrays")),
    }
}

/// Get first element of array
#[loft_builtin(array.first)]
fn array_first(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Value::Unit)
            } else {
                Ok(arr[0].clone())
            }
        }
        _ => Err(RuntimeError::new("first() can only be called on arrays")),
    }
}

/// Get last element of array
#[loft_builtin(array.last)]
fn array_last(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Value::Unit)
            } else {
                Ok(arr[arr.len() - 1].clone())
            }
        }
        _ => Err(RuntimeError::new("last() can only be called on arrays")),
    }
}

/// Take first n elements
#[loft_builtin(array.take)]
fn array_take(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("take() requires a count argument"));
    }

    match (this, &args[0]) {
        (Value::Array(arr), Value::Number(n)) => {
            let count = n
                .to_string()
                .parse::<usize>()
                .map_err(|_| RuntimeError::new("Count must be a non-negative integer"))?;
            let taken: Vec<Value> = arr.iter().take(count).cloned().collect();
            Ok(Value::Array(taken))
        }
        (Value::Array(_), _) => Err(RuntimeError::new("take() argument must be a number")),
        _ => Err(RuntimeError::new("take() can only be called on arrays")),
    }
}

/// Skip first n elements
#[loft_builtin(array.skip)]
fn array_skip(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("skip() requires a count argument"));
    }

    match (this, &args[0]) {
        (Value::Array(arr), Value::Number(n)) => {
            let count = n
                .to_string()
                .parse::<usize>()
                .map_err(|_| RuntimeError::new("Count must be a non-negative integer"))?;
            let skipped: Vec<Value> = arr.iter().skip(count).cloned().collect();
            Ok(Value::Array(skipped))
        }
        (Value::Array(_), _) => Err(RuntimeError::new("skip() argument must be a number")),
        _ => Err(RuntimeError::new("skip() can only be called on arrays")),
    }
}

/// Get unique elements (dedup)
#[loft_builtin(array.unique)]
fn array_unique(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            let mut unique = Vec::new();
            for item in arr {
                if !unique.contains(item) {
                    unique.push(item.clone());
                }
            }
            Ok(Value::Array(unique))
        }
        _ => Err(RuntimeError::new("unique() can only be called on arrays")),
    }
}

/// Sum all numbers in array
#[loft_builtin(array.sum)]
fn array_sum(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            let mut sum = Decimal::ZERO;
            for item in arr {
                match item {
                    Value::Number(n) => sum += n,
                    _ => {
                        return Err(RuntimeError::new(
                            "sum() can only be used on arrays of numbers",
                        ))
                    }
                }
            }
            Ok(Value::Number(sum))
        }
        _ => Err(RuntimeError::new("sum() can only be called on arrays")),
    }
}

/// Get average of all numbers in array
#[loft_builtin(array.average)]
fn array_average(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Ok(Value::Number(Decimal::ZERO));
            }

            let mut sum = Decimal::ZERO;
            for item in arr {
                match item {
                    Value::Number(n) => sum += n,
                    _ => {
                        return Err(RuntimeError::new(
                            "average() can only be used on arrays of numbers",
                        ))
                    }
                }
            }

            let count = Decimal::from(arr.len());
            Ok(Value::Number(sum / count))
        }
        _ => Err(RuntimeError::new("average() can only be called on arrays")),
    }
}

/// Joins an array together by a delimiter
#[loft_builtin(array.join)]
fn array_join(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::Array(arr) => {
            let delimiter = if let Some(arg) = args.first() {
                match arg {
                    Value::String(s) => s.as_str(),
                    _ => return Err(RuntimeError::new("join() delimiter must be a string")),
                }
            } else {
                ","
            };

            let mut result = String::new();
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    result.push_str(delimiter);
                }
                result.push_str(&item.to_string());
            }
            Ok(Value::String(result))
        }
        _ => Err(RuntimeError::new("join() can only be called on arrays")),
    }
}

pub fn register_collection_methods(builtin: &mut BuiltinStruct) {
    // Note: map, filter, reduce, find require function execution
    builtin.add_method("map", array_map as BuiltinMethod);
    builtin.add_method("filter", array_filter as BuiltinMethod);
    builtin.add_method("reduce", array_reduce as BuiltinMethod);
    builtin.add_method("find", array_find as BuiltinMethod);

    // Functional methods that work now
    builtin.add_method("zip", array_zip as BuiltinMethod);
    builtin.add_method("chain", array_chain as BuiltinMethod);
    builtin.add_method("flatten", array_flatten as BuiltinMethod);
    builtin.add_method("reverse", array_reverse as BuiltinMethod);
    builtin.add_method("sort", array_sort as BuiltinMethod);
    builtin.add_method("includes", array_includes as BuiltinMethod);
    builtin.add_method("index_of", array_index_of as BuiltinMethod);
    builtin.add_method("first", array_first as BuiltinMethod);
    builtin.add_method("last", array_last as BuiltinMethod);
    builtin.add_method("take", array_take as BuiltinMethod);
    builtin.add_method("skip", array_skip as BuiltinMethod);
    builtin.add_method("unique", array_unique as BuiltinMethod);
    builtin.add_method("sum", array_sum as BuiltinMethod);
    builtin.add_method("average", array_average as BuiltinMethod);
    builtin.add_method("join", array_join as BuiltinMethod);
}

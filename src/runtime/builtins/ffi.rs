use crate::runtime::builtin::BuiltinStruct;
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use libloading::{Library, Symbol};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

lazy_static::lazy_static! {
    static ref LOADED_LIBRARIES: Arc<Mutex<HashMap<String, Arc<Library>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Helper to acquire mutex lock with proper error handling
fn lock_libraries() -> RuntimeResult<MutexGuard<'static, HashMap<String, Arc<Library>>>> {
    LOADED_LIBRARIES
        .lock()
        .map_err(|e| RuntimeError::new(format!("Failed to lock library registry: {}", e)))
}

/// Helper to convert Decimal to f64
fn decimal_to_f64(d: &Decimal) -> RuntimeResult<f64> {
    d.to_string()
        .parse::<f64>()
        .map_err(|_| RuntimeError::new("Failed to convert number to f64"))
}

/// Helper to convert Decimal to i32
fn decimal_to_i32(d: &Decimal) -> RuntimeResult<i32> {
    d.to_string()
        .parse::<i32>()
        .map_err(|_| RuntimeError::new("Failed to convert number to i32"))
}

/// Helper to convert Decimal to i64
fn decimal_to_i64(d: &Decimal) -> RuntimeResult<i64> {
    d.to_string()
        .parse::<i64>()
        .map_err(|_| RuntimeError::new("Failed to convert number to i64"))
}

/// Helper to convert Decimal to f32
fn decimal_to_f32(d: &Decimal) -> RuntimeResult<f32> {
    d.to_string()
        .parse::<f32>()
        .map_err(|_| RuntimeError::new("Failed to convert number to f32"))
}

/// Helper to convert f64 result to Value
/// Returns ZERO for NaN and infinite values (FFI functions shouldn't return these normally)
fn f64_to_value(result: f64) -> RuntimeResult<Value> {
    if result.is_nan() {
        return Err(RuntimeError::new("FFI function returned NaN"));
    }
    if result.is_infinite() {
        return Err(RuntimeError::new("FFI function returned infinite value"));
    }
    Ok(Value::Number(
        Decimal::from_f64_retain(result).unwrap_or(Decimal::ZERO),
    ))
}

/// Helper to convert f32 result to Value
/// Returns ZERO for NaN and infinite values (FFI functions shouldn't return these normally)
fn f32_to_value(result: f32) -> RuntimeResult<Value> {
    if result.is_nan() {
        return Err(RuntimeError::new("FFI function returned NaN"));
    }
    if result.is_infinite() {
        return Err(RuntimeError::new("FFI function returned infinite value"));
    }
    Ok(Value::Number(
        Decimal::from_f32_retain(result).unwrap_or(Decimal::ZERO),
    ))
}

/// Load a shared library
///
/// # Example
/// ```loft
/// let lib = ffi.load("libm.so.6");
/// ```
pub fn ffi_load(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("ffi.load() requires a library path"));
    }

    let path = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(RuntimeError::new("Library path must be a string")),
    };

    // Try to get from cache first, or load and cache it
    {
        let mut libs = lock_libraries()?;
        if !libs.contains_key(&path) {
            // Load the library
            let lib = unsafe {
                Library::new(&path).map_err(|e| {
                    RuntimeError::new(format!("Failed to load library '{}': {}", path, e))
                })?
            };
            libs.insert(path.clone(), Arc::new(lib));
        }
    }

    // Create the FFI library struct
    let mut ffi_struct = BuiltinStruct::new("FfiLibrary");
    ffi_struct.add_field("path", Value::String(path.clone()));
    ffi_struct.add_method("symbol", ffi_symbol);
    ffi_struct.add_method("call", ffi_lib_call);

    Ok(Value::Builtin(ffi_struct))
}

/// Convenience method: call a function directly on the library
/// Usage: lib.call("symbol_name", "signature", args...)
fn ffi_lib_call(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new(
            "lib.call() requires at least symbol name and signature",
        ));
    }

    let symbol_name = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(RuntimeError::new("Symbol name must be a string")),
    };

    // Get the symbol first
    let symbol_result = ffi_symbol(this, &[Value::String(symbol_name)])?;

    // Then call it with the remaining arguments
    ffi_call(&symbol_result, &args[1..])
}

/// Get a symbol (function) from a loaded library
fn ffi_symbol(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("symbol() requires a symbol name"));
    }

    // Extract the library path from the struct
    let path = match this {
        Value::Builtin(builtin) => match builtin.fields.get("path") {
            Some(Value::String(s)) => s.clone(),
            _ => {
                return Err(RuntimeError::new(
                    "Internal error: FfiLibrary missing path field",
                ))
            }
        },
        _ => {
            return Err(RuntimeError::new(
                "symbol() must be called on an FfiLibrary",
            ))
        }
    };

    let symbol_name = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(RuntimeError::new("Symbol name must be a string")),
    };

    // Get the library from the global registry
    let libs = lock_libraries()?;
    let lib = libs
        .get(&path)
        .ok_or_else(|| RuntimeError::new(format!("Library '{}' not found in registry", path)))?;

    // Verify the symbol exists
    let _symbol_check: Result<Symbol<unsafe extern "C" fn()>, _> =
        unsafe { lib.get(symbol_name.as_bytes()) };

    if _symbol_check.is_err() {
        return Err(RuntimeError::new(format!(
            "Symbol '{}' not found in library",
            symbol_name
        )));
    }

    // Create a callable FFI function struct
    let mut ffi_func = BuiltinStruct::new("FfiFunction");
    ffi_func.add_field("name", Value::String(symbol_name.clone()));
    ffi_func.add_field("library_path", Value::String(path));
    ffi_func.add_method("call", ffi_call);

    Ok(Value::Builtin(ffi_func))
}

/// Call a foreign function with type conversion
fn ffi_call(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    // Extract the symbol name and library path from the struct
    let (symbol_name, library_path) = match this {
        Value::Builtin(builtin) => {
            let name = match builtin.fields.get("name") {
                Some(Value::String(s)) => s.clone(),
                _ => {
                    return Err(RuntimeError::new(
                        "Internal error: FfiFunction missing name field",
                    ))
                }
            };
            let path = match builtin.fields.get("library_path") {
                Some(Value::String(s)) => s.clone(),
                _ => {
                    return Err(RuntimeError::new(
                        "Internal error: FfiFunction missing library_path field",
                    ))
                }
            };
            (name, path)
        }
        _ => return Err(RuntimeError::new("call() must be called on an FfiFunction")),
    };

    // Get the library from the global registry
    let libs = lock_libraries()?;
    let lib = libs.get(&library_path).ok_or_else(|| {
        RuntimeError::new(format!("Library '{}' not found in registry", library_path))
    })?;

    // For simplicity, we'll support common function signatures
    // Users can specify the signature as the first argument

    if args.is_empty() {
        return Err(RuntimeError::new(
            "call() requires at least a signature string (e.g., 'f64(f64)' for cos)",
        ));
    }

    let signature = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(RuntimeError::new(
                "First argument must be a signature string",
            ))
        }
    };

    let fn_args = &args[1..];

    // Parse signature to determine how to call the function
    // Format: "return_type(arg_type1, arg_type2, ...)"
    // Supported types: i32, i64, f32, f64, void

    let parts: Vec<&str> = signature.split('(').collect();
    if parts.len() != 2 {
        return Err(RuntimeError::new(
            "Invalid signature format. Expected: 'return_type(arg_types)'",
        ));
    }

    let return_type = parts[0].trim();
    let arg_types_str = parts[1].trim_end_matches(')');
    let arg_types: Vec<&str> = if arg_types_str.is_empty() {
        vec![]
    } else {
        arg_types_str.split(',').map(|s| s.trim()).collect()
    };

    if arg_types.len() != fn_args.len() {
        return Err(RuntimeError::new(format!(
            "Argument count mismatch: signature expects {} args, but {} provided",
            arg_types.len(),
            fn_args.len()
        )));
    }

    // Call based on signature
    call_by_signature(lib, &symbol_name, return_type, &arg_types, fn_args)
}

/// Call function based on parsed signature
fn call_by_signature(
    lib: &Arc<Library>,
    symbol_name: &str,
    return_type: &str,
    arg_types: &[&str],
    args: &[Value],
) -> RuntimeResult<Value> {
    // This is a simplified implementation supporting common cases
    // A full implementation would need a more sophisticated type system

    match (return_type, arg_types) {
        // f64() functions
        ("f64", []) if args.is_empty() => {
            let func: Symbol<unsafe extern "C" fn() -> f64> = unsafe {
                lib.get(symbol_name.as_bytes())
                    .map_err(|e| RuntimeError::new(format!("Failed to get symbol: {}", e)))?
            };
            let result = unsafe { func() };
            f64_to_value(result)
        }

        // f64(f64) functions like sqrt, sin, cos, etc.
        ("f64", ["f64"]) if args.len() == 1 => {
            let arg = match &args[0] {
                Value::Number(n) => decimal_to_f64(n)?,
                _ => return Err(RuntimeError::new("Expected number argument")),
            };

            let func: Symbol<unsafe extern "C" fn(f64) -> f64> = unsafe {
                lib.get(symbol_name.as_bytes())
                    .map_err(|e| RuntimeError::new(format!("Failed to get symbol: {}", e)))?
            };
            let result = unsafe { func(arg) };
            f64_to_value(result)
        }

        // f64(f64, f64) functions like pow
        ("f64", ["f64", "f64"]) if args.len() == 2 => {
            let arg1 = match &args[0] {
                Value::Number(n) => decimal_to_f64(n)?,
                _ => return Err(RuntimeError::new("Expected number for first argument")),
            };
            let arg2 = match &args[1] {
                Value::Number(n) => decimal_to_f64(n)?,
                _ => return Err(RuntimeError::new("Expected number for second argument")),
            };

            let func: Symbol<unsafe extern "C" fn(f64, f64) -> f64> = unsafe {
                lib.get(symbol_name.as_bytes())
                    .map_err(|e| RuntimeError::new(format!("Failed to get symbol: {}", e)))?
            };
            let result = unsafe { func(arg1, arg2) };
            f64_to_value(result)
        }

        // i32(i32) functions
        ("i32", ["i32"]) if args.len() == 1 => {
            let arg = match &args[0] {
                Value::Number(n) => decimal_to_i32(n)?,
                _ => return Err(RuntimeError::new("Expected number argument")),
            };

            let func: Symbol<unsafe extern "C" fn(i32) -> i32> = unsafe {
                lib.get(symbol_name.as_bytes())
                    .map_err(|e| RuntimeError::new(format!("Failed to get symbol: {}", e)))?
            };
            let result = unsafe { func(arg) };
            Ok(Value::Number(Decimal::from(result)))
        }

        // i32(i32, i32) functions
        ("i32", ["i32", "i32"]) if args.len() == 2 => {
            let arg1 = match &args[0] {
                Value::Number(n) => decimal_to_i32(n)?,
                _ => return Err(RuntimeError::new("Expected number for first argument")),
            };
            let arg2 = match &args[1] {
                Value::Number(n) => decimal_to_i32(n)?,
                _ => return Err(RuntimeError::new("Expected number for second argument")),
            };

            let func: Symbol<unsafe extern "C" fn(i32, i32) -> i32> = unsafe {
                lib.get(symbol_name.as_bytes())
                    .map_err(|e| RuntimeError::new(format!("Failed to get symbol: {}", e)))?
            };
            let result = unsafe { func(arg1, arg2) };
            Ok(Value::Number(Decimal::from(result)))
        }

        // i64(i64) functions
        ("i64", ["i64"]) if args.len() == 1 => {
            let arg = match &args[0] {
                Value::Number(n) => decimal_to_i64(n)?,
                _ => return Err(RuntimeError::new("Expected number argument")),
            };

            let func: Symbol<unsafe extern "C" fn(i64) -> i64> = unsafe {
                lib.get(symbol_name.as_bytes())
                    .map_err(|e| RuntimeError::new(format!("Failed to get symbol: {}", e)))?
            };
            let result = unsafe { func(arg) };
            Ok(Value::Number(Decimal::from(result)))
        }

        // f32(f32) functions
        ("f32", ["f32"]) if args.len() == 1 => {
            let arg = match &args[0] {
                Value::Number(n) => decimal_to_f32(n)?,
                _ => return Err(RuntimeError::new("Expected number argument")),
            };

            let func: Symbol<unsafe extern "C" fn(f32) -> f32> = unsafe {
                lib.get(symbol_name.as_bytes())
                    .map_err(|e| RuntimeError::new(format!("Failed to get symbol: {}", e)))?
            };
            let result = unsafe { func(arg) };
            f32_to_value(result)
        }

        // void() functions
        ("void", []) if args.is_empty() => {
            let func: Symbol<unsafe extern "C" fn()> = unsafe {
                lib.get(symbol_name.as_bytes())
                    .map_err(|e| RuntimeError::new(format!("Failed to get symbol: {}", e)))?
            };
            unsafe { func() };
            Ok(Value::Unit)
        }

        // void(i32) functions
        ("void", ["i32"]) if args.len() == 1 => {
            let arg = match &args[0] {
                Value::Number(n) => decimal_to_i32(n)?,
                _ => return Err(RuntimeError::new("Expected number argument")),
            };

            let func: Symbol<unsafe extern "C" fn(i32)> = unsafe {
                lib.get(symbol_name.as_bytes())
                    .map_err(|e| RuntimeError::new(format!("Failed to get symbol: {}", e)))?
            };
            unsafe { func(arg) };
            Ok(Value::Unit)
        }

        _ => Err(RuntimeError::new(format!(
            "Unsupported function signature: {}({})",
            return_type,
            arg_types.join(", ")
        ))),
    }
}

/// Create the FFI builtin struct
pub fn create_ffi_builtin() -> BuiltinStruct {
    let mut ffi = BuiltinStruct::new("ffi");
    ffi.add_method("load", ffi_load);
    ffi
}

// Register the builtin automatically
crate::submit_builtin!("ffi", create_ffi_builtin, "ffi");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_load_invalid_path() {
        let result = ffi_load(&Value::Unit, &[Value::String("nonexistent.so".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_ffi_load_requires_path() {
        let result = ffi_load(&Value::Unit, &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("requires a library path"));
    }
}

use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult, Interpreter};
use crate::{loft_arg, loft_this};
use rust_decimal::Decimal;
use loft_builtin_macros::loft_builtin;

/// Split a string by a delimiter
/// @param delimiter: str
/// @return Array<str>
#[loft_builtin(string.split)]
fn string_split(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let s = loft_this!(this, String, "split()");
    let delim = loft_arg!(args, 0, String, "split()");
    
    let parts: Vec<Value> = s.split(delim.as_str())
        .map(|part| Value::String(part.to_string()))
        .collect();
    Ok(Value::Array(parts))
}

/// Join an array of strings with a delimiter
/// @param array: Array<any>
/// @param delimiter: str
/// @return str
#[loft_builtin(string.join)]
fn string_join(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let arr = loft_arg!(args, 0, Array, "join()");
    let delim = loft_arg!(args, 1, String, "join()");
    
    let strings: Result<Vec<String>, RuntimeError> = arr.iter()
        .map(|v| match v {
            Value::String(s) => Ok(s.clone()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Boolean(b) => Ok(b.to_string()),
            _ => Err(RuntimeError::new("join() array must contain strings, numbers, or booleans")),
        })
        .collect();
    
    let strings = strings?;
    Ok(Value::String(strings.join(delim)))
}

/// Trim whitespace from both ends of a string
/// @return str
#[loft_builtin(string.trim)]
fn string_trim(_interpreter: &mut Interpreter, this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    let s = loft_this!(this, String, "trim()");
    Ok(Value::String(s.trim().to_string()))
}

/// Trim whitespace from the start of a string
/// @return str
#[loft_builtin(string.trim_start)]
fn string_trim_start(_interpreter: &mut Interpreter, this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    let s = loft_this!(this, String, "trim_start()");
    Ok(Value::String(s.trim_start().to_string()))
}

/// Trim whitespace from the end of a string
/// @return str
#[loft_builtin(string.trim_end)]
fn string_trim_end(_interpreter: &mut Interpreter, this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    let s = loft_this!(this, String, "trim_end()");
    Ok(Value::String(s.trim_end().to_string()))
}

/// Replace all occurrences of a substring with another
/// @param pattern: str
/// @param replacement: str
/// @return str
#[loft_builtin(string.replace)]
fn string_replace(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let s = loft_this!(this, String, "replace()");
    let pattern = loft_arg!(args, 0, String, "replace()");
    let replacement = loft_arg!(args, 1, String, "replace()");
    
    Ok(Value::String(s.replace(pattern.as_str(), replacement.as_str())))
}

/// Convert string to uppercase
/// @return str
#[loft_builtin(string.to_upper)]
fn string_to_upper(_interpreter: &mut Interpreter, this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    let s = loft_this!(this, String, "to_upper()");
    Ok(Value::String(s.to_uppercase()))
}

/// Convert string to lowercase
#[loft_builtin(string.to_lower)]
fn string_to_lower(_interpreter: &mut Interpreter, this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        _ => Err(RuntimeError::new("to_lower() can only be called on strings")),
    }
}

/// Check if string starts with a prefix
#[loft_builtin(string.starts_with)]
fn string_starts_with(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("starts_with() requires a prefix argument"));
    }
    
    match (this, &args[0]) {
        (Value::String(s), Value::String(prefix)) => {
            Ok(Value::Boolean(s.starts_with(prefix.as_str())))
        }
        (Value::String(_), _) => Err(RuntimeError::new("starts_with() argument must be a string")),
        _ => Err(RuntimeError::new("starts_with() can only be called on strings")),
    }
}

/// Check if string ends with a suffix
#[loft_builtin(string.ends_with)]
fn string_ends_with(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("ends_with() requires a suffix argument"));
    }
    
    match (this, &args[0]) {
        (Value::String(s), Value::String(suffix)) => {
            Ok(Value::Boolean(s.ends_with(suffix.as_str())))
        }
        (Value::String(_), _) => Err(RuntimeError::new("ends_with() argument must be a string")),
        _ => Err(RuntimeError::new("ends_with() can only be called on strings")),
    }
}

/// Check if string contains a substring
#[loft_builtin(string.contains)]
fn string_contains(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("contains() requires a substring argument"));
    }
    
    match (this, &args[0]) {
        (Value::String(s), Value::String(substring)) => {
            Ok(Value::Boolean(s.contains(substring.as_str())))
        }
        (Value::String(_), _) => Err(RuntimeError::new("contains() argument must be a string")),
        _ => Err(RuntimeError::new("contains() can only be called on strings")),
    }
}

/// Get the length of a string
#[loft_builtin(string.length)]
fn string_length(_interpreter: &mut Interpreter, this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    match this {
        Value::String(s) => Ok(Value::Number(Decimal::from(s.len()))),
        _ => Err(RuntimeError::new("length() can only be called on strings")),
    }
}

/// Get a substring
#[loft_builtin(string.substring)]
fn string_substring(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("substring() requires a start index"));
    }
    
    match this {
        Value::String(s) => {
            let start = match &args[0] {
                Value::Number(n) => n.to_string().parse::<usize>()
                    .map_err(|_| RuntimeError::new("Start index must be a non-negative integer"))?,
                _ => return Err(RuntimeError::new("Start index must be a number")),
            };
            
            let end = if args.len() > 1 {
                match &args[1] {
                    Value::Number(n) => n.to_string().parse::<usize>()
                        .map_err(|_| RuntimeError::new("End index must be a non-negative integer"))?,
                    _ => return Err(RuntimeError::new("End index must be a number")),
                }
            } else {
                s.len()
            };
            
            if start > s.len() || end > s.len() || start > end {
                return Err(RuntimeError::new("Invalid substring indices"));
            }
            
            Ok(Value::String(s[start..end].to_string()))
        }
        _ => Err(RuntimeError::new("substring() can only be called on strings")),
    }
}

/// Repeat a string n times
#[loft_builtin(string.repeat)]
fn string_repeat(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("repeat() requires a count argument"));
    }
    
    match (this, &args[0]) {
        (Value::String(s), Value::Number(n)) => {
            let count = n.to_string().parse::<usize>()
                .map_err(|_| RuntimeError::new("Count must be a non-negative integer"))?;
            Ok(Value::String(s.repeat(count)))
        }
        (Value::String(_), _) => Err(RuntimeError::new("repeat() argument must be a number")),
        _ => Err(RuntimeError::new("repeat() can only be called on strings")),
    }
}

/// Pad string to a certain length with spaces on the left
#[loft_builtin(string.pad_start)]
fn string_pad_start(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("pad_start() requires a length argument"));
    }
    
    match (this, &args[0]) {
        (Value::String(s), Value::Number(n)) => {
            let target_len = n.to_string().parse::<usize>()
                .map_err(|_| RuntimeError::new("Length must be a non-negative integer"))?;
            
            let pad_char = if args.len() > 1 {
                match &args[1] {
                    Value::String(c) if c.len() == 1 => c.chars().next().unwrap(),
                    Value::String(_) => return Err(RuntimeError::new("Pad character must be a single character")),
                    _ => ' ',
                }
            } else {
                ' '
            };
            
            if s.len() >= target_len {
                Ok(Value::String(s.clone()))
            } else {
                let padding = pad_char.to_string().repeat(target_len - s.len());
                Ok(Value::String(format!("{}{}", padding, s)))
            }
        }
        (Value::String(_), _) => Err(RuntimeError::new("pad_start() length must be a number")),
        _ => Err(RuntimeError::new("pad_start() can only be called on strings")),
    }
}

/// Pad string to a certain length with spaces on the right
#[loft_builtin(string.pad_end)]
fn string_pad_end(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("pad_end() requires a length argument"));
    }
    
    match (this, &args[0]) {
        (Value::String(s), Value::Number(n)) => {
            let target_len = n.to_string().parse::<usize>()
                .map_err(|_| RuntimeError::new("Length must be a non-negative integer"))?;
            
            let pad_char = if args.len() > 1 {
                match &args[1] {
                    Value::String(c) if c.len() == 1 => c.chars().next().unwrap(),
                    Value::String(_) => return Err(RuntimeError::new("Pad character must be a single character")),
                    _ => ' ',
                }
            } else {
                ' '
            };
            
            if s.len() >= target_len {
                Ok(Value::String(s.clone()))
            } else {
                let padding = pad_char.to_string().repeat(target_len - s.len());
                Ok(Value::String(format!("{}{}", s, padding)))
            }
        }
        (Value::String(_), _) => Err(RuntimeError::new("pad_end() length must be a number")),
        _ => Err(RuntimeError::new("pad_end() can only be called on strings")),
    }
}

pub fn create_string_builtin() -> BuiltinStruct {
    let mut string = BuiltinStruct::new("string");
    
    // These are methods that can be called on string values
    // They will be available via the method dispatch system
    string.add_method("split", string_split as BuiltinMethod);
    string.add_method("join", string_join as BuiltinMethod);
    string.add_method("trim", string_trim as BuiltinMethod);
    string.add_method("trim_start", string_trim_start as BuiltinMethod);
    string.add_method("trim_end", string_trim_end as BuiltinMethod);
    string.add_method("replace", string_replace as BuiltinMethod);
    string.add_method("to_upper", string_to_upper as BuiltinMethod);
    string.add_method("to_lower", string_to_lower as BuiltinMethod);
    string.add_method("starts_with", string_starts_with as BuiltinMethod);
    string.add_method("ends_with", string_ends_with as BuiltinMethod);
    string.add_method("contains", string_contains as BuiltinMethod);
    string.add_method("length", string_length as BuiltinMethod);
    string.add_method("len", string_length as BuiltinMethod); // Alias
    string.add_method("substring", string_substring as BuiltinMethod);
    string.add_method("repeat", string_repeat as BuiltinMethod);
    string.add_method("pad_start", string_pad_start as BuiltinMethod);
    string.add_method("pad_end", string_pad_end as BuiltinMethod);
    
    string
}

// Register the builtin automatically
crate::submit_builtin!("string", create_string_builtin);

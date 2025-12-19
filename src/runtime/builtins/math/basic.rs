use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use rust_decimal::Decimal;
use loft_builtin_macros::loft_builtin;

/// Round a number to the nearest integer
#[loft_builtin(math.round)]
fn math_round(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.round() requires a number argument"));
    }
    
    match &args[0] {
        Value::Number(n) => {
            let rounded = n.round();
            Ok(Value::Number(rounded))
        }
        _ => Err(RuntimeError::new("math.round() argument must be a number")),
    }
}

/// Floor a number (round down to nearest integer)
#[loft_builtin(math.floor)]
fn math_floor(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.floor() requires a number argument"));
    }
    
    match &args[0] {
        Value::Number(n) => {
            let floored = n.floor();
            Ok(Value::Number(floored))
        }
        _ => Err(RuntimeError::new("math.floor() argument must be a number")),
    }
}

/// Ceiling a number (round up to nearest integer)
#[loft_builtin(math.ceil)]
fn math_ceil(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.ceil() requires a number argument"));
    }
    
    match &args[0] {
        Value::Number(n) => {
            let ceiled = n.ceil();
            Ok(Value::Number(ceiled))
        }
        _ => Err(RuntimeError::new("math.ceil() argument must be a number")),
    }
}

/// Absolute value of a number
#[loft_builtin(math.abs)]
fn math_abs(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.abs() requires a number argument"));
    }
    
    match &args[0] {
        Value::Number(n) => {
            let abs = n.abs();
            Ok(Value::Number(abs))
        }
        _ => Err(RuntimeError::new("math.abs() argument must be a number")),
    }
}

/// Sign of a number (-1, 0, or 1)
#[loft_builtin(math.sign)]
fn math_sign(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.sign() requires a number argument"));
    }
    
    match &args[0] {
        Value::Number(n) => {
            if n.is_zero() {
                Ok(Value::Number(Decimal::ZERO))
            } else if n.is_sign_negative() {
                Ok(Value::Number(Decimal::from(-1)))
            } else {
                Ok(Value::Number(Decimal::ONE))
            }
        }
        _ => Err(RuntimeError::new("math.sign() argument must be a number")),
    }
}

/// Minimum of two numbers
#[loft_builtin(math.min)]
fn math_min(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new("math.min() requires two number arguments"));
    }
    
    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(if a < b { *a } else { *b }))
        }
        _ => Err(RuntimeError::new("math.min() arguments must be numbers")),
    }
}

/// Maximum of two numbers
#[loft_builtin(math.max)]
fn math_max(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new("math.max() requires two number arguments"));
    }
    
    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(if a > b { *a } else { *b }))
        }
        _ => Err(RuntimeError::new("math.max() arguments must be numbers")),
    }
}

/// Clamp a number between min and max
#[loft_builtin(math.clamp)]
fn math_clamp(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 3 {
        return Err(RuntimeError::new("math.clamp() requires value, min, and max arguments"));
    }
    
    match (&args[0], &args[1], &args[2]) {
        (Value::Number(value), Value::Number(min), Value::Number(max)) => {
            let result = if value < min {
                *min
            } else if value > max {
                *max
            } else {
                *value
            };
            Ok(Value::Number(result))
        }
        _ => Err(RuntimeError::new("math.clamp() arguments must be numbers")),
    }
}

pub fn register_basic_methods(math: &mut BuiltinStruct) {
    math.add_method("round", math_round as BuiltinMethod);
    math.add_method("floor", math_floor as BuiltinMethod);
    math.add_method("ceil", math_ceil as BuiltinMethod);
    math.add_method("abs", math_abs as BuiltinMethod);
    math.add_method("sign", math_sign as BuiltinMethod);
    math.add_method("min", math_min as BuiltinMethod);
    math.add_method("max", math_max as BuiltinMethod);
    math.add_method("clamp", math_clamp as BuiltinMethod);
}

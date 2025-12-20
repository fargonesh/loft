use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use loft_builtin_macros::loft_builtin;

/// Sine function
#[loft_builtin(math.sin)]
fn math_sin(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.sin() requires a number argument"));
    }
    
    match &args[0] {
        Value::Number(n) => {
            let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
            let result = n_f64.sin();
            
            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result invalid"))
        }
        _ => Err(RuntimeError::new("math.sin() argument must be a number")),
    }
}

/// Cosine function
#[loft_builtin(math.cos)]
fn math_cos(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.cos() requires a number argument"));
    }
    
    match &args[0] {
        Value::Number(n) => {
            let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
            let result = n_f64.cos();
            
            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result invalid"))
        }
        _ => Err(RuntimeError::new("math.cos() argument must be a number")),
    }
}

/// Tangent function
#[loft_builtin(math.tan)]
fn math_tan(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.tan() requires a number argument"));
    }
    
    match &args[0] {
        Value::Number(n) => {
            let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
            let result = n_f64.tan();
            
            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result invalid"))
        }
        _ => Err(RuntimeError::new("math.tan() argument must be a number")),
    }
}

/// Arcsine function
#[loft_builtin(math.asin)]
fn math_asin(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.asin() requires a number argument"));
    }
    
    match &args[0] {
        Value::Number(n) => {
            let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
            if !(-1.0..=1.0).contains(&n_f64) {
                return Err(RuntimeError::new("math.asin() argument must be between -1 and 1"));
            }
            let result = n_f64.asin();
            
            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result invalid"))
        }
        _ => Err(RuntimeError::new("math.asin() argument must be a number")),
    }
}

/// Arccosine function
#[loft_builtin(math.acos)]
fn math_acos(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.acos() requires a number argument"));
    }
    
    match &args[0] {
        Value::Number(n) => {
            let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
            if !(-1.0..=1.0).contains(&n_f64) {
                return Err(RuntimeError::new("math.acos() argument must be between -1 and 1"));
            }
            let result = n_f64.acos();
            
            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result invalid"))
        }
        _ => Err(RuntimeError::new("math.acos() argument must be a number")),
    }
}

/// Arctangent function
#[loft_builtin(math.atan)]
fn math_atan(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.atan() requires a number argument"));
    }
    
    match &args[0] {
        Value::Number(n) => {
            let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
            let result = n_f64.atan();
            
            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result invalid"))
        }
        _ => Err(RuntimeError::new("math.atan() argument must be a number")),
    }
}

/// Two-argument arctangent function
#[loft_builtin(math.atan2)]
fn math_atan2(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new("math.atan2() requires y and x arguments"));
    }
    
    match (&args[0], &args[1]) {
        (Value::Number(y), Value::Number(x)) => {
            let y_f64 = y.to_f64().ok_or_else(|| RuntimeError::new("Invalid y value"))?;
            let x_f64 = x.to_f64().ok_or_else(|| RuntimeError::new("Invalid x value"))?;
            let result = y_f64.atan2(x_f64);
            
            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result invalid"))
        }
        _ => Err(RuntimeError::new("math.atan2() arguments must be numbers")),
    }
}

pub fn register_trigonometry_methods(math: &mut BuiltinStruct) {
    math.add_method("sin", math_sin as BuiltinMethod);
    math.add_method("cos", math_cos as BuiltinMethod);
    math.add_method("tan", math_tan as BuiltinMethod);
    math.add_method("asin", math_asin as BuiltinMethod);
    math.add_method("acos", math_acos as BuiltinMethod);
    math.add_method("atan", math_atan as BuiltinMethod);
    math.add_method("atan2", math_atan2 as BuiltinMethod);
}

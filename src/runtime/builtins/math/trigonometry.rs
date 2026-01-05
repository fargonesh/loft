use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult, Interpreter};
use crate::loft_arg;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use loft_builtin_macros::loft_builtin;

/// Sine function
/// @param x: num
/// @return num
#[loft_builtin(math.sin)]
fn math_sin(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.sin()");
    
    let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
    let result = n_f64.sin();
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result invalid"))
}

/// Cosine function
/// @param x: num
/// @return num
#[loft_builtin(math.cos)]
fn math_cos(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.cos()");
    
    let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
    let result = n_f64.cos();
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result invalid"))
}

/// Tangent function
/// @param x: num
/// @return num
#[loft_builtin(math.tan)]
fn math_tan(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.tan()");
    
    let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
    let result = n_f64.tan();
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result invalid"))
}

/// Arcsine function
/// @param x: num
/// @return num
#[loft_builtin(math.asin)]
fn math_asin(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.asin()");
    
    let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
    if !(-1.0..=1.0).contains(&n_f64) {
        return Err(RuntimeError::new("math.asin() argument must be between -1 and 1"));
    }
    let result = n_f64.asin();
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result invalid"))
}

/// Arccosine function
/// @param x: num
/// @return num
#[loft_builtin(math.acos)]
fn math_acos(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.acos()");
    
    let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
    if !(-1.0..=1.0).contains(&n_f64) {
        return Err(RuntimeError::new("math.acos() argument must be between -1 and 1"));
    }
    let result = n_f64.acos();
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result invalid"))
}

/// Arctangent function
/// @param x: num
/// @return num
#[loft_builtin(math.atan)]
fn math_atan(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.atan()");
    
    let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
    let result = n_f64.atan();
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result invalid"))
}

/// Two-argument arctangent function
/// @param y: num
/// @param x: num
/// @return num
#[loft_builtin(math.atan2)]
fn math_atan2(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let y = loft_arg!(args, 0, Number, "math.atan2()");
    let x = loft_arg!(args, 1, Number, "math.atan2()");
    
    let y_f64 = y.to_f64().ok_or_else(|| RuntimeError::new("Invalid y value"))?;
    let x_f64 = x.to_f64().ok_or_else(|| RuntimeError::new("Invalid x value"))?;
    let result = y_f64.atan2(x_f64);
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result invalid"))
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

use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult, Interpreter};
use crate::loft_arg;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use loft_builtin_macros::loft_builtin;

/// Power: base^exponent
/// @param base: num
/// @param exponent: num
/// @return num
#[loft_builtin(math.pow)]
fn math_pow(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let base = loft_arg!(args, 0, Number, "math.pow()");
    let exp = loft_arg!(args, 1, Number, "math.pow()");
    
    let base_f64 = base.to_f64().ok_or_else(|| RuntimeError::new("Invalid base number"))?;
    let exp_f64 = exp.to_f64().ok_or_else(|| RuntimeError::new("Invalid exponent number"))?;
    let result = base_f64.powf(exp_f64);
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result too large or invalid"))
}

/// Square root
/// @param n: num
/// @return num
#[loft_builtin(math.sqrt)]
fn math_sqrt(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.sqrt()");
    
    let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
    if n_f64 < 0.0 {
        return Err(RuntimeError::new("Cannot take square root of negative number"));
    }
    let result = n_f64.sqrt();
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result invalid"))
}

/// Exponential function (e^x)
/// @param x: num
/// @return num
#[loft_builtin(math.exp)]
fn math_exp(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.exp()");
    
    let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
    let result = n_f64.exp();
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result too large or invalid"))
}

/// Natural logarithm (ln)
/// @param x: num
/// @return num
#[loft_builtin(math.ln)]
fn math_ln(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.ln()");
    
    let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
    if n_f64 <= 0.0 {
        return Err(RuntimeError::new("Cannot take logarithm of non-positive number"));
    }
    let result = n_f64.ln();
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result invalid"))
}

/// Base-10 logarithm
/// @param x: num
/// @return num
#[loft_builtin(math.log10)]
fn math_log10(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.log10()");
    
    let n_f64 = n.to_f64().ok_or_else(|| RuntimeError::new("Invalid number"))?;
    if n_f64 <= 0.0 {
        return Err(RuntimeError::new("Cannot take logarithm of non-positive number"));
    }
    let result = n_f64.log10();
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result invalid"))
}

/// Logarithm with custom base
/// @param value: num
/// @param base: num
/// @return num
#[loft_builtin(math.log)]
fn math_log(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let value = loft_arg!(args, 0, Number, "math.log()");
    let base = loft_arg!(args, 1, Number, "math.log()");
    
    let value_f64 = value.to_f64().ok_or_else(|| RuntimeError::new("Invalid value"))?;
    let base_f64 = base.to_f64().ok_or_else(|| RuntimeError::new("Invalid base"))?;
    
    if value_f64 <= 0.0 {
        return Err(RuntimeError::new("Cannot take logarithm of non-positive number"));
    }
    if base_f64 <= 0.0 || base_f64 == 1.0 {
        return Err(RuntimeError::new("Invalid logarithm base"));
    }
    
    let result = value_f64.log(base_f64);
    
    Decimal::from_f64_retain(result)
        .map(Value::Number)
        .ok_or_else(|| RuntimeError::new("Result invalid"))
}

pub fn register_exponential_methods(math: &mut BuiltinStruct) {
    math.add_method("pow", math_pow as BuiltinMethod);
    math.add_method("sqrt", math_sqrt as BuiltinMethod);
    math.add_method("exp", math_exp as BuiltinMethod);
    math.add_method("ln", math_ln as BuiltinMethod);
    math.add_method("log10", math_log10 as BuiltinMethod);
    math.add_method("log", math_log as BuiltinMethod);
}

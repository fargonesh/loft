use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeResult, Interpreter};
use crate::loft_arg;
use rust_decimal::Decimal;
use loft_builtin_macros::loft_builtin;

/// Round a number to the nearest integer
/// @param n: num
/// @return num
#[loft_builtin(math.round)]
fn math_round(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.round()");
    Ok(Value::Number(n.round()))
}

/// Floor a number (round down to nearest integer)
/// @param n: num
/// @return num
#[loft_builtin(math.floor)]
fn math_floor(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.floor()");
    Ok(Value::Number(n.floor()))
}

/// Ceiling a number (round up to nearest integer)
/// @param n: num
/// @return num
#[loft_builtin(math.ceil)]
fn math_ceil(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.ceil()");
    Ok(Value::Number(n.ceil()))
}

/// Absolute value of a number
/// @param n: num
/// @return num
#[loft_builtin(math.abs)]
fn math_abs(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.abs()");
    Ok(Value::Number(n.abs()))
}

/// Sign of a number (-1, 0, or 1)
/// @param n: num
/// @return num
#[loft_builtin(math.sign)]
fn math_sign(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let n = loft_arg!(args, 0, Number, "math.sign()");
    
    if n.is_zero() {
        Ok(Value::Number(Decimal::ZERO))
    } else if n.is_sign_negative() {
        Ok(Value::Number(Decimal::from(-1)))
    } else {
        Ok(Value::Number(Decimal::ONE))
    }
}

/// Minimum of two numbers
/// @param a: num
/// @param b: num
/// @return num
#[loft_builtin(math.min)]
fn math_min(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let a = loft_arg!(args, 0, Number, "math.min()");
    let b = loft_arg!(args, 1, Number, "math.min()");
    
    Ok(Value::Number(a.min(b)))
}

/// Maximum of two numbers
/// @param a: num
/// @param b: num
/// @return num
#[loft_builtin(math.max)]
fn math_max(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let a = loft_arg!(args, 0, Number, "math.max()");
    let b = loft_arg!(args, 1, Number, "math.max()");
    
    Ok(Value::Number(a.max(b)))
}

/// Clamp a number between min and max
/// @param value: num
/// @param min: num
/// @param max: num
/// @return num
#[loft_builtin(math.clamp)]
fn math_clamp(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let value = loft_arg!(args, 0, Number, "math.clamp()");
    let min = loft_arg!(args, 1, Number, "math.clamp()");
    let max = loft_arg!(args, 2, Number, "math.clamp()");
    
    Ok(Value::Number(value.min(max).max(min)))
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

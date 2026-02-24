use crate::runtime::builtin::{BuiltinMethod, BuiltinStruct};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use loft_builtin_macros::loft_builtin;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// Power: base^exponent
#[loft_builtin(math.pow)]
fn math_pow(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new(
            "math.pow() requires base and exponent arguments",
        ));
    }

    match (&args[0], &args[1]) {
        (Value::Number(base), Value::Number(exp)) => {
            let base_f64 = base
                .to_f64()
                .ok_or_else(|| RuntimeError::new("Invalid base number"))?;
            let exp_f64 = exp
                .to_f64()
                .ok_or_else(|| RuntimeError::new("Invalid exponent number"))?;
            let result = base_f64.powf(exp_f64);

            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result too large or invalid"))
        }
        _ => Err(RuntimeError::new("math.pow() arguments must be numbers")),
    }
}

/// Square root
#[loft_builtin(math.sqrt)]
fn math_sqrt(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.sqrt() requires a number argument"));
    }

    match &args[0] {
        Value::Number(n) => {
            let n_f64 = n
                .to_f64()
                .ok_or_else(|| RuntimeError::new("Invalid number"))?;
            if n_f64 < 0.0 {
                return Err(RuntimeError::new(
                    "Cannot take square root of negative number",
                ));
            }
            let result = n_f64.sqrt();

            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result invalid"))
        }
        _ => Err(RuntimeError::new("math.sqrt() argument must be a number")),
    }
}

/// Exponential function (e^x)
#[loft_builtin(math.exp)]
fn math_exp(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.exp() requires a number argument"));
    }

    match &args[0] {
        Value::Number(n) => {
            let n_f64 = n
                .to_f64()
                .ok_or_else(|| RuntimeError::new("Invalid number"))?;
            let result = n_f64.exp();

            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result too large or invalid"))
        }
        _ => Err(RuntimeError::new("math.exp() argument must be a number")),
    }
}

/// Natural logarithm (ln)
#[loft_builtin(math.ln)]
fn math_ln(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.ln() requires a number argument"));
    }

    match &args[0] {
        Value::Number(n) => {
            let n_f64 = n
                .to_f64()
                .ok_or_else(|| RuntimeError::new("Invalid number"))?;
            if n_f64 <= 0.0 {
                return Err(RuntimeError::new(
                    "Cannot take logarithm of non-positive number",
                ));
            }
            let result = n_f64.ln();

            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result invalid"))
        }
        _ => Err(RuntimeError::new("math.ln() argument must be a number")),
    }
}

/// Base-10 logarithm
#[loft_builtin(math.log10)]
fn math_log10(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("math.log10() requires a number argument"));
    }

    match &args[0] {
        Value::Number(n) => {
            let n_f64 = n
                .to_f64()
                .ok_or_else(|| RuntimeError::new("Invalid number"))?;
            if n_f64 <= 0.0 {
                return Err(RuntimeError::new(
                    "Cannot take logarithm of non-positive number",
                ));
            }
            let result = n_f64.log10();

            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result invalid"))
        }
        _ => Err(RuntimeError::new("math.log10() argument must be a number")),
    }
}

/// Logarithm with custom base
#[loft_builtin(math.log)]
fn math_log(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new(
            "math.log() requires value and base arguments",
        ));
    }

    match (&args[0], &args[1]) {
        (Value::Number(value), Value::Number(base)) => {
            let value_f64 = value
                .to_f64()
                .ok_or_else(|| RuntimeError::new("Invalid value"))?;
            let base_f64 = base
                .to_f64()
                .ok_or_else(|| RuntimeError::new("Invalid base"))?;

            if value_f64 <= 0.0 {
                return Err(RuntimeError::new(
                    "Cannot take logarithm of non-positive number",
                ));
            }
            if base_f64 <= 0.0 || base_f64 == 1.0 {
                return Err(RuntimeError::new("Invalid logarithm base"));
            }

            let result = value_f64.log(base_f64);

            Decimal::from_f64_retain(result)
                .map(Value::Number)
                .ok_or_else(|| RuntimeError::new("Result invalid"))
        }
        _ => Err(RuntimeError::new("math.log() arguments must be numbers")),
    }
}

pub fn register_exponential_methods(math: &mut BuiltinStruct) {
    math.add_method("pow", math_pow as BuiltinMethod);
    math.add_method("sqrt", math_sqrt as BuiltinMethod);
    math.add_method("exp", math_exp as BuiltinMethod);
    math.add_method("ln", math_ln as BuiltinMethod);
    math.add_method("log10", math_log10 as BuiltinMethod);
    math.add_method("log", math_log as BuiltinMethod);
}

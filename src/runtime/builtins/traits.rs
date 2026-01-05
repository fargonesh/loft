use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult, Interpreter};
use loft_builtin_macros::loft_builtin;

/// These are builtin methods that can be called on any value
/// They map to the trait implementations
/// add method - adds two values (wraps Add trait)
#[loft_builtin(value.add)]
pub fn builtin_add(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("add() requires exactly one argument"));
    }
    
    use crate::runtime::traits::Add;
    this.add(&args[0])
}

/// sub method - subtracts two values (wraps Sub trait)
#[loft_builtin(value.sub)]
pub fn builtin_sub(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("sub() requires exactly one argument"));
    }
    
    use crate::runtime::traits::Sub;
    this.sub(&args[0])
}

/// mul method - multiplies two values (wraps Mul trait)
#[loft_builtin(value.mul)]
pub fn builtin_mul(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("mul() requires exactly one argument"));
    }
    
    use crate::runtime::traits::Mul;
    this.mul(&args[0])
}

/// div method - divides two values (wraps Div trait)
#[loft_builtin(value.div)]
pub fn builtin_div(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("div() requires exactly one argument"));
    }
    
    use crate::runtime::traits::Div;
    this.div(&args[0])
}


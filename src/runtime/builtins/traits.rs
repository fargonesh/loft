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

/// gt method - greater than comparison (wraps Ord trait)
#[loft_builtin(value.gt)]
pub fn builtin_gt(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("gt() requires exactly one argument"));
    }

    use crate::runtime::traits::Ord;
    Ord::gt(this, &args[0])
}

/// ge method - greater than or equal comparison (wraps Ord trait)
#[loft_builtin(value.ge)]
pub fn builtin_ge(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("ge() requires exactly one argument"));
    }

    use crate::runtime::traits::Ord;
    Ord::ge(this, &args[0])
}

/// lt method - less than comparison (wraps Ord trait)
#[loft_builtin(value.lt)]
pub fn builtin_lt(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("lt() requires exactly one argument"));
    }

    use crate::runtime::traits::Ord;
    Ord::lt(this, &args[0])
}

/// le method - less than or equal comparison (wraps Ord trait)
#[loft_builtin(value.le)]
pub fn builtin_le(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("le() requires exactly one argument"));
    }

    use crate::runtime::traits::Ord;
    Ord::le(this, &args[0])
}

/// eq method - equality comparison (wraps Ord trait)
#[loft_builtin(value.eq)]
pub fn builtin_eq(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("eq() requires exactly one argument"));
    }

    use crate::runtime::traits::Ord;
    Ord::eq(this, &args[0])
}

/// ne method - inequality comparison (wraps Ord trait)
#[loft_builtin(value.ne)]
pub fn builtin_ne(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("ne() requires exactly one argument"));
    }

    use crate::runtime::traits::Ord;
    Ord::ne(this, &args[0])
}

/// index method - performs indexing (wraps Index trait)
#[loft_builtin(value.index)]
pub fn builtin_index(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            "index() requires exactly one argument (the index)",
        ));
    }

    use crate::runtime::traits::Index;
    this.index(&args[0])
}

/// bit_and method - bitwise AND (wraps BitAnd trait)
#[loft_builtin(value.bit_and)]
pub fn builtin_bit_and(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("bit_and() requires exactly one argument"));
    }

    use crate::runtime::traits::BitAnd;
    this.bit_and(&args[0])
}

/// bit_or method - bitwise OR (wraps BitOr trait)
#[loft_builtin(value.bit_or)]
pub fn builtin_bit_or(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("bit_or() requires exactly one argument"));
    }

    use crate::runtime::traits::BitOr;
    this.bit_or(&args[0])
}

/// bit_xor method - bitwise XOR (wraps BitXor trait)
#[loft_builtin(value.bit_xor)]
pub fn builtin_bit_xor(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("bit_xor() requires exactly one argument"));
    }

    use crate::runtime::traits::BitXor;
    this.bit_xor(&args[0])
}

/// shl method - left shift (wraps Shl trait)
#[loft_builtin(value.shl)]
pub fn builtin_shl(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("shl() requires exactly one argument"));
    }

    use crate::runtime::traits::Shl;
    this.shl(&args[0])
}

/// shr method - right shift (wraps Shr trait)
#[loft_builtin(value.shr)]
pub fn builtin_shr(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::new("shr() requires exactly one argument"));
    }

    use crate::runtime::traits::Shr;
    this.shr(&args[0])
}

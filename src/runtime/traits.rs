use super::value::Value;
use super::{RuntimeError, RuntimeResult};
use rust_decimal::Decimal;

/// Trait for addition operation.
/// Allows values to be added together using the `+` operator.
/// Implemented for numbers (arithmetic addition) and strings (concatenation).
pub trait Add {
    fn add(&self, other: &Value) -> RuntimeResult<Value>;
}

/// Trait for subtraction operation.
/// Allows values to be subtracted using the `-` operator.
/// Currently implemented for numbers only.
pub trait Sub {
    fn sub(&self, other: &Value) -> RuntimeResult<Value>;
}

/// Trait for multiplication operation.
/// Allows values to be multiplied using the `*` operator.
/// Currently implemented for numbers only.
pub trait Mul {
    fn mul(&self, other: &Value) -> RuntimeResult<Value>;
}

/// Trait for division operation.
/// Allows values to be divided using the `/` operator.
/// Currently implemented for numbers only. Division by zero returns an error.
pub trait Div {
    fn div(&self, other: &Value) -> RuntimeResult<Value>;
}

/// Trait for bitwise AND operation.
pub trait BitAnd {
    fn bit_and(&self, other: &Value) -> RuntimeResult<Value>;
}

/// Trait for bitwise OR operation.
pub trait BitOr {
    fn bit_or(&self, other: &Value) -> RuntimeResult<Value>;
}

/// Trait for bitwise XOR operation.
pub trait BitXor {
    fn bit_xor(&self, other: &Value) -> RuntimeResult<Value>;
}

/// Trait for bitwise left shift operation.
pub trait Shl {
    fn shl(&self, other: &Value) -> RuntimeResult<Value>;
}

/// Trait for bitwise right shift operation.
pub trait Shr {
    fn shr(&self, other: &Value) -> RuntimeResult<Value>;
}

/// Trait for indexing operation.
/// Allows values to be indexed using bracket notation `[index]`.
///
/// # Implementations
/// - Arrays: indexed by number, returns element at position
/// - Strings: indexed by number, returns character at position
/// - Objects (Structs): indexed by string key, returns field value
pub trait Index {
    fn index(&self, index: &Value) -> RuntimeResult<Value>;
}

/// Trait for comparison operations.
/// Allows values to be compared using `>`, `>=`, `<`, `<=`, `==`, `!=`.
pub trait Ord {
    fn gt(&self, other: &Value) -> RuntimeResult<Value>;
    fn ge(&self, other: &Value) -> RuntimeResult<Value>;
    fn lt(&self, other: &Value) -> RuntimeResult<Value>;
    fn le(&self, other: &Value) -> RuntimeResult<Value>;
    fn eq(&self, other: &Value) -> RuntimeResult<Value>;
    fn ne(&self, other: &Value) -> RuntimeResult<Value>;
}

/// Trait for string conversion - provides consistent string representation for all Value types.
pub trait ToString {
    fn to_string(&self) -> String;
}

// Implementations for Value
impl Add for Value {
    fn add(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(*l + *r)),
            (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
            // Allow string + any by coercing the right-hand side to its string representation
            (Value::String(l), _) => Ok(Value::String(format!("{}{}", l, other.to_string()))),
            _ => Err(RuntimeError::new(format!(
                "Cannot add {:?} and {:?}",
                self, other
            ))),
        }
    }
}

impl Sub for Value {
    fn sub(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(*l - *r)),
            _ => Err(RuntimeError::new(format!(
                "Cannot subtract {:?} and {:?}",
                self, other
            ))),
        }
    }
}

impl Mul for Value {
    fn mul(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(*l * *r)),
            _ => Err(RuntimeError::new(format!(
                "Cannot multiply {:?} and {:?}",
                self, other
            ))),
        }
    }
}

impl Div for Value {
    fn div(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => {
                if *r == Decimal::ZERO {
                    Err(RuntimeError::new("Division by zero"))
                } else {
                    Ok(Value::Number(*l / *r))
                }
            }
            _ => Err(RuntimeError::new(format!(
                "Cannot divide {:?} by {:?}",
                self, other
            ))),
        }
    }
}

impl BitAnd for Value {
    fn bit_and(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => {
                let l_i64 = l
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .parse::<i64>()
                    .map_err(|_| RuntimeError::new("Bitwise AND requires integer operands"))?;
                let r_i64 = r
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .parse::<i64>()
                    .map_err(|_| RuntimeError::new("Bitwise AND requires integer operands"))?;
                Ok(Value::Number(Decimal::from(l_i64 & r_i64)))
            }
            _ => Err(RuntimeError::new(format!(
                "Cannot perform bitwise AND on {:?} and {:?}",
                self, other
            ))),
        }
    }
}

impl BitOr for Value {
    fn bit_or(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => {
                let l_i64 = l
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .parse::<i64>()
                    .map_err(|_| RuntimeError::new("Bitwise OR requires integer operands"))?;
                let r_i64 = r
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .parse::<i64>()
                    .map_err(|_| RuntimeError::new("Bitwise OR requires integer operands"))?;
                Ok(Value::Number(Decimal::from(l_i64 | r_i64)))
            }
            _ => Err(RuntimeError::new(format!(
                "Cannot perform bitwise OR on {:?} and {:?}",
                self, other
            ))),
        }
    }
}

impl BitXor for Value {
    fn bit_xor(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => {
                let l_i64 = l
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .parse::<i64>()
                    .map_err(|_| RuntimeError::new("Bitwise XOR requires integer operands"))?;
                let r_i64 = r
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .parse::<i64>()
                    .map_err(|_| RuntimeError::new("Bitwise XOR requires integer operands"))?;
                Ok(Value::Number(Decimal::from(l_i64 ^ r_i64)))
            }
            _ => Err(RuntimeError::new(format!(
                "Cannot perform bitwise XOR on {:?} and {:?}",
                self, other
            ))),
        }
    }
}

impl Shl for Value {
    fn shl(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => {
                let l_i64 = l
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .parse::<i64>()
                    .map_err(|_| RuntimeError::new("Left shift requires integer operands"))?;
                let r_u32 = r
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .parse::<u32>()
                    .map_err(|_| {
                        RuntimeError::new("Right operand of shift must be a non-negative integer")
                    })?;
                Ok(Value::Number(Decimal::from(l_i64 << r_u32)))
            }
            _ => Err(RuntimeError::new(format!(
                "Cannot perform left shift on {:?} and {:?}",
                self, other
            ))),
        }
    }
}

impl Shr for Value {
    fn shr(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => {
                let l_i64 = l
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .parse::<i64>()
                    .map_err(|_| RuntimeError::new("Right shift requires integer operands"))?;
                let r_u32 = r
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .parse::<u32>()
                    .map_err(|_| {
                        RuntimeError::new("Right operand of shift must be a non-negative integer")
                    })?;
                Ok(Value::Number(Decimal::from(l_i64 >> r_u32)))
            }
            _ => Err(RuntimeError::new(format!(
                "Cannot perform right shift on {:?} and {:?}",
                self, other
            ))),
        }
    }
}

impl Index for Value {
    fn index(&self, index: &Value) -> RuntimeResult<Value> {
        match (self, index) {
            (Value::Array(arr), Value::Number(idx)) => {
                let idx_usize = idx
                    .to_string()
                    .parse::<usize>()
                    .map_err(|_| RuntimeError::new("Array index must be a non-negative integer"))?;

                arr.get(idx_usize).cloned().ok_or_else(|| {
                    RuntimeError::new(format!("Array index {} out of bounds", idx_usize))
                })
            }
            (Value::String(s), Value::Number(idx)) => {
                let idx_usize = idx.to_string().parse::<usize>().map_err(|_| {
                    RuntimeError::new("String index must be a non-negative integer")
                })?;

                s.chars()
                    .nth(idx_usize)
                    .map(|c| Value::String(c.to_string()))
                    .ok_or_else(|| {
                        RuntimeError::new(format!("String index {} out of bounds", idx_usize))
                    })
            }
            (Value::Struct { fields, .. }, Value::String(key)) => {
                fields.get(key).cloned().ok_or_else(|| {
                    RuntimeError::new(format!("Object does not have property '{}'", key))
                })
            }
            (Value::Array(_), _) => Err(RuntimeError::new("Array index must be a number")),
            (Value::String(_), _) => Err(RuntimeError::new("String index must be a number")),
            (Value::Struct { .. }, _) => Err(RuntimeError::new("Object index must be a string")),
            _ => Err(RuntimeError::new(format!(
                "Cannot index value of type {:?}",
                self
            ))),
        }
    }
}

impl Ord for Value {
    fn gt(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),
            _ => Err(RuntimeError::new(format!(
                "Cannot compare {:?} > {:?}",
                self, other
            ))),
        }
    }

    fn ge(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l >= r)),
            _ => Err(RuntimeError::new(format!(
                "Cannot compare {:?} >= {:?}",
                self, other
            ))),
        }
    }

    fn lt(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
            _ => Err(RuntimeError::new(format!(
                "Cannot compare {:?} < {:?}",
                self, other
            ))),
        }
    }

    fn le(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l <= r)),
            _ => Err(RuntimeError::new(format!(
                "Cannot compare {:?} <= {:?}",
                self, other
            ))),
        }
    }

    fn eq(&self, other: &Value) -> RuntimeResult<Value> {
        Ok(Value::Boolean(self == other))
    }

    fn ne(&self, other: &Value) -> RuntimeResult<Value> {
        Ok(Value::Boolean(self != other))
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Unit => "()".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Function { name, .. } => format!("<function {}>", name),
            Value::Struct { name, .. } => format!("<struct {}>", name),
            Value::Builtin(b) => format!("<builtin {}>", b.name),
            Value::BuiltinFn(_) => "<builtin function>".to_string(),
            Value::BoundMethod { method_name, .. } => format!("<bound method {}>", method_name),
            Value::UserMethod { method_name, .. } => format!("<method {}>", method_name),
            Value::Closure { params, .. } => format!("<closure with {} params>", params.len()),
            Value::Promise(value) => format!("<promise {}>", value.to_string()),
            Value::EnumVariant {
                enum_name,
                variant_name,
                values,
            } => {
                // Option and Result are rendered without the enum prefix for idiomatic output
                if enum_name == "Option" || enum_name == "Result" {
                    if values.is_empty() {
                        variant_name.clone()
                    } else {
                        let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                        format!("{}({})", variant_name, vals.join(", "))
                    }
                } else if values.is_empty() {
                    format!("{}.{}", enum_name, variant_name)
                } else {
                    let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                    format!("{}.{}({})", enum_name, variant_name, vals.join(", "))
                }
            }
            Value::EnumConstructor {
                enum_name,
                variant_name,
                ..
            } => {
                format!("{}.{}", enum_name, variant_name)
            }
            Value::Module { name, .. } => {
                format!("<module {}>", name)
            }
        }
    }
}

/// Helper function to call the appropriate trait method based on binary operator.
/// Routes operator calls to the correct trait implementation for arithmetic and comparison operations.
///
/// # Arguments
/// * `op` - The operator string ("+", "-", "*", "/", ">", ">=", "<", "<=", "==", "!=")
/// * `left` - The left operand value
/// * `right` - The right operand value
///
/// # Returns
/// The result of the operation or an error if the operation is not supported for the given types.
pub fn call_binop_trait(op: &str, left: &Value, right: &Value) -> RuntimeResult<Value> {
    match op {
        "+" => left.add(right),
        "-" => left.sub(right),
        "*" => left.mul(right),
        "/" => left.div(right),
        "&" => left.bit_and(right),
        "|" => left.bit_or(right),
        "^" => left.bit_xor(right),
        "<<" => left.shl(right),
        ">>" => left.shr(right),
        ">" => Ord::gt(left, right),
        ">=" => Ord::ge(left, right),
        "<" => Ord::lt(left, right),
        "<=" => Ord::le(left, right),
        "==" => Ord::eq(left, right),
        "!=" => Ord::ne(left, right),
        _ => Err(RuntimeError::new(format!("Unknown operator: {}", op))),
    }
}

/// Helper function to call the Index trait for bracket notation access.
///
/// # Arguments
/// * `value` - The value to index (array, string, or object)
/// * `index` - The index to access (number for arrays/strings, string for objects)
///
/// # Returns
/// The value at the given index or an error if indexing is not supported.
pub fn call_index_trait(value: &Value, index: &Value) -> RuntimeResult<Value> {
    value.index(index)
}

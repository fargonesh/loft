use super::value::{Value, PromiseState};
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

/// Trait for string conversion - provides consistent string representation for all Value types.
/// This replaces Debug formatting in most user-facing output like term.print and array.join.
pub trait ToString {
    fn to_string(&self) -> String;
}

// Implementations for Value
impl Add for Value {
    fn add(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(*l + *r)),
            (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
            (Value::String(l), _) => Ok(Value::String(format!("{}{}", l, other.to_string()))),
            (_, Value::String(r)) => Ok(Value::String(format!("{}{}", self.to_string(), r))),
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

impl Index for Value {
    fn index(&self, index: &Value) -> RuntimeResult<Value> {
        match (self, index) {
            (Value::Array(arr), Value::Number(idx)) => {
                let idx_usize = idx.to_string().parse::<usize>()
                    .map_err(|_| RuntimeError::new("Array index must be a non-negative integer"))?;
                
                arr.get(idx_usize)
                    .cloned()
                    .ok_or_else(|| RuntimeError::new(format!("Array index {} out of bounds", idx_usize)))
            }
            (Value::String(s), Value::Number(idx)) => {
                let idx_usize = idx.to_string().parse::<usize>()
                    .map_err(|_| RuntimeError::new("String index must be a non-negative integer"))?;
                
                s.chars().nth(idx_usize)
                    .map(|c| Value::String(c.to_string()))
                    .ok_or_else(|| RuntimeError::new(format!("String index {} out of bounds", idx_usize)))
            }
            (Value::Struct { fields, .. }, Value::String(key)) => {
                fields.get(key)
                    .cloned()
                    .ok_or_else(|| RuntimeError::new(format!("Object does not have property '{}'", key)))
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
            },
            Value::Function { name, .. } => format!("<function {}>", name),
            Value::Struct { name, .. } => format!("<struct {}>", name),
            Value::Builtin(b) => format!("<builtin {}>", b.name),
            Value::BuiltinFn(_) => "<builtin function>".to_string(),
            Value::BoundMethod { method_name, .. } => format!("<bound method {}>", method_name),
            Value::UserMethod { method_name, .. } => format!("<method {}>", method_name),
            Value::Closure { params, .. } => format!("<closure with {} params>", params.len()),
            Value::Promise(promise) => {
                let data = promise.borrow();
                match &data.state {
                    PromiseState::Pending(_) => "<promise pending>".to_string(),
                    PromiseState::Resolved(v) => format!("<promise resolved: {}>", v.to_string()),
                    PromiseState::Rejected(v) => format!("<promise rejected: {}>", v.to_string()),
                }
            },
            Value::EnumVariant { enum_name, variant_name, values } => {
                if values.is_empty() {
                    format!("{}.{}", enum_name, variant_name)
                } else {
                    let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                    format!("{}.{}({})", enum_name, variant_name, vals.join(", "))
                }
            }
            Value::EnumConstructor { enum_name, variant_name, .. } => {
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
        "%" => match (left, right) {
            (Value::Number(l), Value::Number(r)) => {
                if *r == Decimal::ZERO {
                    Err(RuntimeError::new("Division by zero (modulo)"))
                } else {
                    Ok(Value::Number(*l % *r))
                }
            }
            _ => Err(RuntimeError::new(format!(
                "Cannot calculate modulo of {:?} and {:?}",
                left, right
            ))),
        },
        // Comparison operators don't use traits, handle them separately
        ">" => match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),
            _ => Err(RuntimeError::new(format!(
                "Cannot compare {:?} > {:?}",
                left, right
            ))),
        },
        ">=" => match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l >= r)),
            _ => Err(RuntimeError::new(format!(
                "Cannot compare {:?} >= {:?}",
                left, right
            ))),
        },
        "<" => match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
            _ => Err(RuntimeError::new(format!(
                "Cannot compare {:?} < {:?}",
                left, right
            ))),
        },
        "<=" => match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l <= r)),
            _ => Err(RuntimeError::new(format!(
                "Cannot compare {:?} <= {:?}",
                left, right
            ))),
        },
        "==" => match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l == r)),
            (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l == r)),
            (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l == r)),
            _ => Ok(Value::Boolean(false)),
        },
        "!=" => match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l != r)),
            (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l != r)),
            (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l != r)),
            _ => Ok(Value::Boolean(true)),
        },
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

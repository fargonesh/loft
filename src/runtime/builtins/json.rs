use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult, Interpreter};
use serde_json;
use std::collections::HashMap;
use rust_decimal::Decimal;
use loft_builtin_macros::loft_builtin;

/// Parse a JSON string into a loft value
#[loft_builtin(json.parse)]
fn json_parse(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("json.parse() requires a string argument"));
    }
    
    let json_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(RuntimeError::new("json.parse() argument must be a string")),
    };
    
    let json_value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| RuntimeError::new(format!("Failed to parse JSON: {}", e)))?;
    
    json_to_loft_value(json_value)
}

#[loft_builtin(json.stringify)]
fn json_stringify(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("json.stringify() requires a value argument"));
    }
    
    let json_value = loft_value_to_json(&args[0])?;
    let json_str = serde_json::to_string(&json_value)
        .map_err(|e| RuntimeError::new(format!("Failed to stringify JSON: {}", e)))?;
    
    Ok(Value::String(json_str))
}

/// Convert a loft value to a pretty-printed JSON string
#[loft_builtin(json.stringify_pretty)]
fn json_stringify_pretty(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("json.stringify_pretty() requires a value argument"));
    }
    
    let json_value = loft_value_to_json(&args[0])?;
    let json_str = serde_json::to_string_pretty(&json_value)
        .map_err(|e| RuntimeError::new(format!("Failed to stringify JSON: {}", e)))?;
    
    Ok(Value::String(json_str))
}

pub(crate) fn json_to_loft_value(json: serde_json::Value) -> RuntimeResult<Value> {
    match json {
        serde_json::Value::Null => Ok(Value::Unit),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Number(Decimal::from(i)))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(Decimal::try_from(f)
                    .map_err(|e| RuntimeError::new(format!("Invalid number: {}", e)))?))
            } else {
                Err(RuntimeError::new("Invalid JSON number"))
            }
        },
        serde_json::Value::String(s) => Ok(Value::String(s)),
        serde_json::Value::Array(arr) => {
            let mut values = Vec::new();
            for item in arr {
                values.push(json_to_loft_value(item)?);
            }
            Ok(Value::Array(values))
        },
        serde_json::Value::Object(obj) => {
            let mut fields = HashMap::new();
            for (key, value) in obj {
                fields.insert(key, json_to_loft_value(value)?);
            }
            Ok(Value::Struct {
                name: "Object".to_string(),
                fields,
            })
        },
    }
}

fn loft_value_to_json(value: &Value) -> RuntimeResult<serde_json::Value> {
    use rust_decimal::prelude::ToPrimitive;
    
    match value {
        Value::Unit => Ok(serde_json::Value::Null),
        Value::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Number(n) => {
            if let Some(f) = n.to_f64() {
                Ok(serde_json::json!(f))
            } else {
                Err(RuntimeError::new("Failed to convert number to JSON"))
            }
        },
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Array(arr) => {
            let mut json_arr = Vec::new();
            for item in arr {
                json_arr.push(loft_value_to_json(item)?);
            }
            Ok(serde_json::Value::Array(json_arr))
        },
        Value::Struct { fields, .. } => {
            let mut json_obj = serde_json::Map::new();
            for (key, value) in fields {
                json_obj.insert(key.clone(), loft_value_to_json(value)?);
            }
            Ok(serde_json::Value::Object(json_obj))
        },
        _ => Err(RuntimeError::new(format!("Cannot convert {:?} to JSON", value))),
    }
}

pub fn create_json_builtin() -> BuiltinStruct {
    let mut json = BuiltinStruct::new("json");
    
    json.add_method("parse", json_parse as BuiltinMethod);
    json.add_method("stringify", json_stringify as BuiltinMethod);
    json.add_method("stringify_pretty", json_stringify_pretty as BuiltinMethod);
    
    json
}

// Register the builtin automatically
crate::submit_builtin!("json", create_json_builtin);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_json_parse_simple() {
        let mut interpreter = Interpreter::new();
        let json_str = r#"{"name": "Alice", "age": 30}"#;
        let result = json_parse(&mut interpreter, &Value::Unit, &[Value::String(json_str.to_string())]);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_json_stringify_object() {
        let mut interpreter = Interpreter::new();
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::String("Bob".to_string()));
        fields.insert("age".to_string(), Value::Number(Decimal::from(25)));
        
        let obj = Value::Struct {
            name: "Object".to_string(),
            fields,
        };
        
        let result = json_stringify(&mut interpreter, &Value::Unit, &[obj]);
        assert!(result.is_ok());
        
        let json_str = match result.unwrap() {
            Value::String(s) => s,
            _ => panic!("Expected string"),
        };
        
        assert!(json_str.contains("name"));
        assert!(json_str.contains("Bob"));
    }
    
    #[test]
    fn test_json_parse_array() {
        let mut interpreter = Interpreter::new();
        let json_str = "[1, 2, 3]";
        let result = json_parse(&mut interpreter, &Value::Unit, &[Value::String(json_str.to_string())]);
        assert!(result.is_ok());
        
        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected array"),
        }
    }
}

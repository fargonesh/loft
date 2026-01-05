use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult, Interpreter};
use rust_decimal::Decimal;
use loft_builtin_macros::loft_builtin;
use base64::{Engine as _, engine::general_purpose};

/// Encode a string to base64
#[loft_builtin(encoding.base64_encode)]
fn base64_encode(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("encoding.base64_encode() requires a string argument"));
    }
    
    let input = match &args[0] {
        Value::String(s) => s.as_bytes(),
        _ => return Err(RuntimeError::new("encoding.base64_encode() argument must be a string")),
    };
    
    let encoded = general_purpose::STANDARD.encode(input);
    Ok(Value::String(encoded))
}

/// Decode a base64 string
#[loft_builtin(encoding.base64_decode)]
fn base64_decode(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("encoding.base64_decode() requires a string argument"));
    }
    
    let input = match &args[0] {
        Value::String(s) => s,
        _ => return Err(RuntimeError::new("encoding.base64_decode() argument must be a string")),
    };
    
    let decoded = general_purpose::STANDARD.decode(input)
        .map_err(|e| RuntimeError::new(format!("Failed to decode base64: {}", e)))?;
    
    let decoded_str = String::from_utf8(decoded)
        .map_err(|e| RuntimeError::new(format!("Decoded data is not valid UTF-8: {}", e)))?;
    
    Ok(Value::String(decoded_str))
}

/// URL encode a string
#[loft_builtin(encoding.url_encode)]
fn url_encode(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("encoding.url_encode() requires a string argument"));
    }
    
    let input = match &args[0] {
        Value::String(s) => s,
        _ => return Err(RuntimeError::new("encoding.url_encode() argument must be a string")),
    };
    
    let encoded = urlencoding::encode(input);
    Ok(Value::String(encoded.to_string()))
}

/// URL decode a string
#[loft_builtin(encoding.url_decode)]
fn url_decode(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("encoding.url_decode() requires a string argument"));
    }
    
    let input = match &args[0] {
        Value::String(s) => s,
        _ => return Err(RuntimeError::new("encoding.url_decode() argument must be a string")),
    };
    
    let decoded = urlencoding::decode(input)
        .map_err(|e| RuntimeError::new(format!("Failed to decode URL: {}", e)))?;
    
    Ok(Value::String(decoded.to_string()))
}

/// Convert string to bytes array
#[loft_builtin(encoding.to_bytes)]
fn to_bytes(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("encoding.to_bytes() requires a string argument"));
    }
    
    let input = match &args[0] {
        Value::String(s) => s,
        _ => return Err(RuntimeError::new("encoding.to_bytes() argument must be a string")),
    };
    
    let bytes: Vec<Value> = input.as_bytes()
        .iter()
        .map(|&b| Value::Number(Decimal::from(b)))
        .collect();
    
    Ok(Value::Array(bytes))
}

/// Convert bytes array to string
#[loft_builtin(encoding.from_bytes)]
fn from_bytes(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("encoding.from_bytes() requires an array argument"));
    }
    
    let bytes_array = match &args[0] {
        Value::Array(arr) => arr,
        _ => return Err(RuntimeError::new("encoding.from_bytes() argument must be an array")),
    };
    
    let mut bytes = Vec::new();
    for item in bytes_array {
        match item {
            Value::Number(n) => {
                let byte = n.to_string().parse::<u8>()
                    .map_err(|_| RuntimeError::new("Byte value must be between 0 and 255"))?;
                bytes.push(byte);
            },
            _ => return Err(RuntimeError::new("Array must contain only numbers")),
        }
    }
    
    let string = String::from_utf8(bytes)
        .map_err(|e| RuntimeError::new(format!("Invalid UTF-8 bytes: {}", e)))?;
    
    Ok(Value::String(string))
}

pub fn create_encoding_builtin() -> BuiltinStruct {
    let mut encoding = BuiltinStruct::new("encoding");
    
    encoding.add_method("base64_encode", base64_encode as BuiltinMethod);
    encoding.add_method("base64_decode", base64_decode as BuiltinMethod);
    encoding.add_method("url_encode", url_encode as BuiltinMethod);
    encoding.add_method("url_decode", url_decode as BuiltinMethod);
    encoding.add_method("to_bytes", to_bytes as BuiltinMethod);
    encoding.add_method("from_bytes", from_bytes as BuiltinMethod);
    
    encoding
}

// Register the builtin automatically
crate::submit_builtin!("encoding", create_encoding_builtin);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_base64_encode_decode() {
        let mut interpreter = Interpreter::new();
        let input = "Hello World";
        let encoded = base64_encode(&mut interpreter, &Value::Unit, &[Value::String(input.to_string())]).unwrap();
        
        let encoded_str = match encoded {
            Value::String(s) => s,
            _ => panic!("Expected string"),
        };
        
        let decoded = base64_decode(&mut interpreter, &Value::Unit, &[Value::String(encoded_str)]).unwrap();
        
        match decoded {
            Value::String(s) => assert_eq!(s, input),
            _ => panic!("Expected string"),
        }
    }
    
    #[test]
    fn test_url_encode_decode() {
        let mut interpreter = Interpreter::new();
        let input = "hello world!";
        let encoded = url_encode(&mut interpreter, &Value::Unit, &[Value::String(input.to_string())]).unwrap();
        
        let encoded_str = match encoded {
            Value::String(s) => s,
            _ => panic!("Expected string"),
        };
        
        assert!(encoded_str.contains("%20"));
        
        let decoded = url_decode(&mut interpreter, &Value::Unit, &[Value::String(encoded_str)]).unwrap();
        
        match decoded {
            Value::String(s) => assert_eq!(s, input),
            _ => panic!("Expected string"),
        }
    }
    
    #[test]
    fn test_to_bytes_from_bytes() {
        let mut interpreter = Interpreter::new();
        let input = "Hi";
        let bytes = to_bytes(&mut interpreter, &Value::Unit, &[Value::String(input.to_string())]).unwrap();
        
        let bytes_array = match bytes {
            Value::Array(arr) => arr,
            _ => panic!("Expected array"),
        };
        
        assert_eq!(bytes_array.len(), 2);
        
        let decoded = from_bytes(&mut interpreter, &Value::Unit, &[Value::Array(bytes_array)]).unwrap();
        
        match decoded {
            Value::String(s) => assert_eq!(s, input),
            _ => panic!("Expected string"),
        }
    }
}

use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult, Interpreter};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use loft_builtin_macros::loft_builtin;
use std::time::{SystemTime, UNIX_EPOCH};

/// Simple pseudo-random number generator state
static mut RNG_STATE: u64 = 0;

fn init_rng() {
    unsafe {
        if RNG_STATE == 0 {
            RNG_STATE = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
        }
    }
}

fn next_random() -> u64 {
    unsafe {
        // Simple xorshift algorithm
        let mut x = RNG_STATE;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        RNG_STATE = x;
        x
    }
}

/// Generate a random number between 0 and 1
#[loft_builtin(random.random)]
fn random_random(_interpreter: &mut Interpreter, _this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    init_rng();
    let r = next_random();
    let normalized = (r as f64) / (u64::MAX as f64);
    Ok(Value::Number(Decimal::try_from(normalized)
        .map_err(|e| RuntimeError::new(format!("Failed to create random number: {}", e)))?))
}

/// Generate a random integer in range [min, max)
#[loft_builtin(random.range)]
fn random_range(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new("random.range() requires min and max arguments"));
    }
    
    let min = match &args[0] {
        Value::Number(n) => n.to_i64().ok_or_else(|| RuntimeError::new("min must be an integer"))?,
        _ => return Err(RuntimeError::new("random.range() min must be a number")),
    };
    
    let max = match &args[1] {
        Value::Number(n) => n.to_i64().ok_or_else(|| RuntimeError::new("max must be an integer"))?,
        _ => return Err(RuntimeError::new("random.range() max must be a number")),
    };
    
    if min >= max {
        return Err(RuntimeError::new("min must be less than max"));
    }
    
    init_rng();
    let r = next_random();
    let range = (max - min) as u64;
    let value = min + ((r % range) as i64);
    
    Ok(Value::Number(Decimal::from(value)))
}

/// Pick a random element from an array
#[loft_builtin(random.choice)]
fn random_choice(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("random.choice() requires an array argument"));
    }
    
    match &args[0] {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Err(RuntimeError::new("Cannot pick from empty array"));
            }
            
            init_rng();
            let r = next_random();
            let index = (r % arr.len() as u64) as usize;
            Ok(arr[index].clone())
        },
        _ => Err(RuntimeError::new("random.choice() argument must be an array")),
    }
}

/// Shuffle an array randomly
#[loft_builtin(random.shuffle)]
fn random_shuffle(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("random.shuffle() requires an array argument"));
    }
    
    match &args[0] {
        Value::Array(arr) => {
            let mut result = arr.clone();
            let len = result.len();
            
            init_rng();
            
            // Fisher-Yates shuffle
            for i in (1..len).rev() {
                let j = (next_random() % ((i + 1) as u64)) as usize;
                result.swap(i, j);
            }
            
            Ok(Value::Array(result))
        },
        _ => Err(RuntimeError::new("random.shuffle() argument must be an array")),
    }
}

/// Set the random seed
#[loft_builtin(random.seed)]
fn random_seed(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("random.seed() requires a number argument"));
    }
    
    let seed = match &args[0] {
        Value::Number(n) => n.to_u64().ok_or_else(|| RuntimeError::new("Seed must be a positive integer"))?,
        _ => return Err(RuntimeError::new("random.seed() argument must be a number")),
    };
    
    unsafe {
        RNG_STATE = if seed == 0 { 1 } else { seed };
    }
    
    Ok(Value::Unit)
}

pub fn create_random_builtin() -> BuiltinStruct {
    let mut random = BuiltinStruct::new("random");
    
    random.add_method("random", random_random as BuiltinMethod);
    random.add_method("range", random_range as BuiltinMethod);
    random.add_method("choice", random_choice as BuiltinMethod);
    random.add_method("shuffle", random_shuffle as BuiltinMethod);
    random.add_method("seed", random_seed as BuiltinMethod);
    
    random
}

// Register the builtin automatically
crate::submit_builtin!("random", create_random_builtin);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_random_generates_number() {
        let mut interpreter = Interpreter::new();
        let result = random_random(&mut interpreter, &Value::Unit, &[]);
        assert!(result.is_ok());
        
        match result.unwrap() {
            Value::Number(_) => {},
            _ => panic!("Expected number"),
        }
    }
    
    #[test]
    fn test_random_range() {
        let mut interpreter = Interpreter::new();
        let result = random_range(&mut interpreter, &Value::Unit, &[
            Value::Number(Decimal::from(1)),
            Value::Number(Decimal::from(10)),
        ]);
        assert!(result.is_ok());
        
        match result.unwrap() {
            Value::Number(n) => {
                let val = n.to_i64().unwrap();
                assert!((1..10).contains(&val));
            },
            _ => panic!("Expected number"),
        }
    }
    
    #[test]
    fn test_random_choice() {
        let mut interpreter = Interpreter::new();
        let arr = vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ];
        
        let result = random_choice(&mut interpreter, &Value::Unit, &[Value::Array(arr.clone())]);
        assert!(result.is_ok());
        
        let chosen = result.unwrap();
        assert!(arr.contains(&chosen));
    }
    
    #[test]
    fn test_random_shuffle() {
        let mut interpreter = Interpreter::new();
        let arr = vec![
            Value::Number(Decimal::from(1)),
            Value::Number(Decimal::from(2)),
            Value::Number(Decimal::from(3)),
        ];
        
        let result = random_shuffle(&mut interpreter, &Value::Unit, &[Value::Array(arr.clone())]);
        assert!(result.is_ok());
        
        match result.unwrap() {
            Value::Array(shuffled) => {
                assert_eq!(shuffled.len(), arr.len());
            },
            _ => panic!("Expected array"),
        }
    }
    
    #[test]
    fn test_random_seed_reproducibility() {
        let mut interpreter = Interpreter::new();
        random_seed(&mut interpreter, &Value::Unit, &[Value::Number(Decimal::from(42))]).unwrap();
        let r1 = random_random(&mut interpreter, &Value::Unit, &[]).unwrap();
        
        random_seed(&mut interpreter, &Value::Unit, &[Value::Number(Decimal::from(42))]).unwrap();
        let r2 = random_random(&mut interpreter, &Value::Unit, &[]).unwrap();
        
        match (r1, r2) {
            (Value::Number(n1), Value::Number(n2)) => {
                assert_eq!(n1, n2);
            },
            _ => panic!("Expected numbers"),
        }
    }
}

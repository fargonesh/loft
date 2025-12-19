use crate::runtime::builtin::BuiltinStruct;
use crate::runtime::value::Value;
use rust_decimal::Decimal;

pub mod basic;
pub mod exponential;
pub mod trigonometry;

/// Create the Math builtin struct
pub fn create_math_builtin() -> BuiltinStruct {
    let mut math = BuiltinStruct::new("math");
    
    // Add constants as fields
    math.add_field("PI", Value::Number(
        Decimal::from_f64_retain(std::f64::consts::PI).unwrap()
    ));
    math.add_field("E", Value::Number(
        Decimal::from_f64_retain(std::f64::consts::E).unwrap()
    ));
    math.add_field("TAU", Value::Number(
        Decimal::from_f64_retain(std::f64::consts::TAU).unwrap()
    ));
    
    // Register methods from submodules
    basic::register_basic_methods(&mut math);
    exponential::register_exponential_methods(&mut math);
    trigonometry::register_trigonometry_methods(&mut math);
    
    math
}

// Register the builtin automatically
crate::submit_builtin!("math", create_math_builtin);

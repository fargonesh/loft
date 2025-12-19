pub mod array;
pub mod term;
pub mod time;
pub mod traits;
pub mod math;
pub mod string;
pub mod collections;
pub mod io;
pub mod web;
pub mod json;
pub mod console;
pub mod object;
pub mod encoding;
pub mod random;
pub mod ffi;

use crate::runtime::value::Value;
use crate::runtime::builtin_registry::BuiltinRegistration;

/// Initialize all builtins and return them as a vector of (name, value) pairs
pub fn init_builtins() -> Vec<(String, Value)> {
    let mut builtins = Vec::new();
    
    // Collect automatically registered builtins from inventory
    for registration in inventory::iter::<BuiltinRegistration> {
        let builtin_struct = (registration.factory)();
        builtins.push((registration.name.to_string(), Value::Builtin(builtin_struct)));
    }
    
    // Note: Array methods are available directly on array values
    // via method calls (e.g., arr.push(value), arr.length(), etc.)
    
    // Note: The Add, Sub, Mul, Div trait methods are available directly on values
    // via method calls (e.g., value.add(other), value.sub(other), etc.)
    // They don't need to be registered as separate builtins since they're
    // implemented as traits on the Value type itself.
    
    builtins
}

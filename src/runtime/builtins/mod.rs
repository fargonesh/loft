pub mod array;
pub mod collections;
pub mod encoding;
#[cfg(not(target_arch = "wasm32"))]
pub mod ffi;
#[cfg(not(target_arch = "wasm32"))]
pub mod io;
pub mod json;
pub mod math;
pub mod object;
pub mod random;
pub mod string;
pub mod term;
pub mod test;
pub mod time;
pub mod traits;
#[cfg(not(target_arch = "wasm32"))]
pub mod web;

use crate::runtime::builtin_registry::BuiltinRegistration;
use crate::runtime::value::Value;

/// Initialize all builtins and return them as a vector of (name, value) pairs
/// If enabled_features is None, all builtins are loaded.
/// Otherwise, only builtins with no feature or an enabled feature are loaded.
pub fn init_builtins(enabled_features: Option<&[String]>) -> Vec<(String, Value)> {
    let mut builtins = Vec::new();

    // Collect automatically registered builtins from inventory
    for registration in inventory::iter::<BuiltinRegistration> {
        let should_load = match registration.feature {
            None => true,
            Some(feature) => {
                if let Some(enabled) = enabled_features {
                    enabled.iter().any(|f| f == feature)
                } else {
                    true // If no feature list provided, load all
                }
            }
        };

        if should_load {
            let builtin_struct = (registration.factory)();
            builtins.push((
                registration.name.to_string(),
                Value::Builtin(builtin_struct),
            ));
        }
    }

    // Note: Array methods are available directly on array values
    // via method calls (e.g., arr.push(value), arr.length(), etc.)

    // Note: The Add, Sub, Mul, Div trait methods are available directly on values
    // via method calls (e.g., value.add(other), value.sub(other), etc.)
    // They don't need to be registered as separate builtins since they're
    // implemented as traits on the Value type itself.

    builtins
}

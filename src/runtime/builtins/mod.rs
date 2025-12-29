pub mod array;
pub mod collections;
pub mod encoding;
#[cfg(not(target_arch = "wasm32"))]
pub mod io;
pub mod json;
pub mod math;
pub mod object;
pub mod random;
pub mod string;
pub mod term;
pub mod time;
pub mod traits;
#[cfg(not(target_arch = "wasm32"))]
pub mod web;

#[cfg(all(feature = "ffi", not(target_arch = "wasm32")))]
pub mod ffi;

use crate::runtime::builtin_registry::BuiltinRegistration;
use crate::runtime::value::Value;

/// Initialize all builtins and return them as a vector of (name, value) pairs
pub fn init_builtins() -> Vec<(String, Value)> {
    let mut builtins = Vec::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Collect automatically registered builtins from inventory
        for registration in inventory::iter::<BuiltinRegistration> {
            let builtin_struct = (registration.factory)();
            builtins.push((
                registration.name.to_string(),
                Value::Builtin(builtin_struct),
            ));
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Manually register builtins for WASM since inventory doesn't work well
        builtins.push(("json".to_string(), Value::Builtin(json::create_json_builtin())));
        builtins.push(("random".to_string(), Value::Builtin(random::create_random_builtin())));
        builtins.push(("term".to_string(), Value::Builtin(term::create_term_builtin())));
        builtins.push(("time".to_string(), Value::Builtin(time::create_time_builtin())));
        builtins.push(("encoding".to_string(), Value::Builtin(encoding::create_encoding_builtin())));
        builtins.push(("string".to_string(), Value::Builtin(string::create_string_builtin())));
        builtins.push(("object".to_string(), Value::Builtin(object::create_object_builtin())));
        builtins.push(("math".to_string(), Value::Builtin(math::create_math_builtin())));

        // Global print/println for convenience
        builtins.push(("print".to_string(), Value::BuiltinFn(|args| term::term_print(&Value::Unit, args))));
        builtins.push(("println".to_string(), Value::BuiltinFn(|args| term::term_println(&Value::Unit, args))));
    }

    // Note: Array methods are available directly on array values
    // via method calls (e.g., arr.push(value), arr.length(), etc.)

    // Note: The Add, Sub, Mul, Div trait methods are available directly on values
    // via method calls (e.g., value.add(other), value.sub(other), etc.)
    // They don't need to be registered as separate builtins since they're
    // implemented as traits on the Value type itself.

    builtins
}

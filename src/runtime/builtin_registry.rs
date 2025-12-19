use crate::runtime::builtin::BuiltinStruct;

/// A factory function that creates a builtin
pub type BuiltinFactory = fn() -> BuiltinStruct;

/// A registered builtin with its name and factory function
pub struct BuiltinRegistration {
    pub name: &'static str,
    pub factory: BuiltinFactory,
}

impl BuiltinRegistration {
    pub const fn new(name: &'static str, factory: BuiltinFactory) -> Self {
        Self { name, factory }
    }
}

// This allows builtins to be collected at compile time
inventory::collect!(BuiltinRegistration);

/// Submit a builtin registration to the inventory
#[macro_export]
macro_rules! submit_builtin {
    ($name:expr, $factory:expr) => {
        inventory::submit! {
            $crate::runtime::builtin_registry::BuiltinRegistration::new($name, $factory)
        }
    };
}

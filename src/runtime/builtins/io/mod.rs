use crate::runtime::builtin::BuiltinStruct;

pub mod fs;

pub fn create_io_builtin() -> BuiltinStruct {
    // For now, we'll just provide the fs builtin
    // In the future, we can add network operations, channels, etc.
    fs::create_fs_builtin()
}

// Register the builtin automatically
crate::submit_builtin!("fs", create_io_builtin);

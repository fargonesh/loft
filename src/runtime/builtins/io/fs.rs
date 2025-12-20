use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use crate::runtime::permission_context::{check_read_permission, check_write_permission};
use std::fs;
use std::path::Path;
use loft_builtin_macros::loft_builtin;

/// Read entire file contents as a string
#[loft_builtin(fs.read)]
fn fs_read_file(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("fs.read() requires a file path argument"));
    }
    
    match &args[0] {
        Value::String(path) => {
            // Check read permission
            check_read_permission(path, Some("fs.read()"))
                .map_err(RuntimeError::new)?;
            
            fs::read_to_string(path)
                .map(Value::String)
                .map_err(|e| RuntimeError::new(format!("Failed to read file: {}", e)))
        }
        _ => Err(RuntimeError::new("fs.read() argument must be a string")),
    }
}

/// Write string contents to a file
#[loft_builtin(fs.write)]
fn fs_write_file(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new("fs.write() requires path and content arguments"));
    }
    
    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(content)) => {
            // Check write permission
            check_write_permission(path, Some("fs.write()"))
                .map_err(RuntimeError::new)?;
            
            fs::write(path, content)
                .map(|_| Value::Unit)
                .map_err(|e| RuntimeError::new(format!("Failed to write file: {}", e)))
        }
        (Value::String(_), _) => Err(RuntimeError::new("fs.write() content must be a string")),
        _ => Err(RuntimeError::new("fs.write() path must be a string")),
    }
}

/// Append string contents to a file
#[loft_builtin(fs.append)]
fn fs_append_file(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new("fs.append() requires path and content arguments"));
    }
    
    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(content)) => {
            // Check write permission
            check_write_permission(path, Some("fs.append()"))
                .map_err(RuntimeError::new)?;
            
            use std::fs::OpenOptions;
            use std::io::Write;
            
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .and_then(|mut file| file.write_all(content.as_bytes()))
                .map(|_| Value::Unit)
                .map_err(|e| RuntimeError::new(format!("Failed to append to file: {}", e)))
        }
        (Value::String(_), _) => Err(RuntimeError::new("fs.append() content must be a string")),
        _ => Err(RuntimeError::new("fs.append() path must be a string")),
    }
}

/// Check if a file exists
#[loft_builtin(fs.exists)]
fn fs_exists(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("fs.exists() requires a path argument"));
    }
    
    match &args[0] {
        Value::String(path) => {
            // Check read permission
            check_read_permission(path, Some("fs.exists()"))
                .map_err(RuntimeError::new)?;
            
            Ok(Value::Boolean(Path::new(path).exists()))
        }
        _ => Err(RuntimeError::new("fs.exists() argument must be a string")),
    }
}

/// Check if path is a file
#[loft_builtin(fs.is_file)]
fn fs_is_file(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("fs.is_file() requires a path argument"));
    }
    
    match &args[0] {
        Value::String(path) => {
            // Check read permission
            check_read_permission(path, Some("fs.is_file()"))
                .map_err(RuntimeError::new)?;
            
            Ok(Value::Boolean(Path::new(path).is_file()))
        }
        _ => Err(RuntimeError::new("fs.is_file() argument must be a string")),
    }
}

/// Check if path is a directory
#[loft_builtin(fs.is_dir)]
fn fs_is_dir(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("fs.is_dir() requires a path argument"));
    }
    
    match &args[0] {
        Value::String(path) => {
            // Check read permission
            check_read_permission(path, Some("fs.is_dir()"))
                .map_err(RuntimeError::new)?;
            
            Ok(Value::Boolean(Path::new(path).is_dir()))
        }
        _ => Err(RuntimeError::new("fs.is_dir() argument must be a string")),
    }
}

/// Create a directory
#[loft_builtin(fs.create_dir)]
fn fs_create_dir(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("fs.create_dir() requires a path argument"));
    }
    
    match &args[0] {
        Value::String(path) => {
            // Check write permission
            check_write_permission(path, Some("fs.create_dir()"))
                .map_err(RuntimeError::new)?;
            
            fs::create_dir_all(path)
                .map(|_| Value::Unit)
                .map_err(|e| RuntimeError::new(format!("Failed to create directory: {}", e)))
        }
        _ => Err(RuntimeError::new("fs.create_dir() argument must be a string")),
    }
}

/// Remove a file
#[loft_builtin(fs.remove_file)]
fn fs_remove_file(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("fs.remove_file() requires a path argument"));
    }
    
    match &args[0] {
        Value::String(path) => {
            // Check write permission
            check_write_permission(path, Some("fs.remove_file()"))
                .map_err(RuntimeError::new)?;
            
            fs::remove_file(path)
                .map(|_| Value::Unit)
                .map_err(|e| RuntimeError::new(format!("Failed to remove file: {}", e)))
        }
        _ => Err(RuntimeError::new("fs.remove_file() argument must be a string")),
    }
}

/// Remove a directory
#[loft_builtin(fs.remove_dir)]
fn fs_remove_dir(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("fs.remove_dir() requires a path argument"));
    }
    
    match &args[0] {
        Value::String(path) => {
            // Check write permission
            check_write_permission(path, Some("fs.remove_dir()"))
                .map_err(RuntimeError::new)?;
            
            fs::remove_dir_all(path)
                .map(|_| Value::Unit)
                .map_err(|e| RuntimeError::new(format!("Failed to remove directory: {}", e)))
        }
        _ => Err(RuntimeError::new("fs.remove_dir() argument must be a string")),
    }
}

/// List directory contents
#[loft_builtin(fs.list_dir)]
fn fs_list_dir(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("fs.list_dir() requires a path argument"));
    }
    
    match &args[0] {
        Value::String(path) => {
            // Check read permission
            check_read_permission(path, Some("fs.list_dir()"))
                .map_err(RuntimeError::new)?;
            
            fs::read_dir(path)
                .map_err(|e| RuntimeError::new(format!("Failed to read directory: {}", e)))
                .and_then(|entries| {
                    let mut files = Vec::new();
                    for entry in entries {
                        match entry {
                            Ok(entry) => {
                                if let Some(file_name) = entry.file_name().to_str() {
                                    files.push(Value::String(file_name.to_string()));
                                }
                            }
                            Err(e) => return Err(RuntimeError::new(format!("Failed to read entry: {}", e))),
                        }
                    }
                    Ok(Value::Array(files))
                })
        }
        _ => Err(RuntimeError::new("fs.list_dir() argument must be a string")),
    }
}

/// Copy a file
#[loft_builtin(fs.copy)]
fn fs_copy(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new("fs.copy() requires source and destination arguments"));
    }
    
    match (&args[0], &args[1]) {
        (Value::String(src), Value::String(dst)) => {
            // Check read permission for source
            check_read_permission(src, Some("fs.copy()"))
                .map_err(RuntimeError::new)?;
            // Check write permission for destination
            check_write_permission(dst, Some("fs.copy()"))
                .map_err(RuntimeError::new)?;
            
            fs::copy(src, dst)
                .map(|_| Value::Unit)
                .map_err(|e| RuntimeError::new(format!("Failed to copy file: {}", e)))
        }
        _ => Err(RuntimeError::new("fs.copy() arguments must be strings")),
    }
}

/// Rename/move a file
#[loft_builtin(fs.rename)]
fn fs_rename(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new("fs.rename() requires source and destination arguments"));
    }
    
    match (&args[0], &args[1]) {
        (Value::String(src), Value::String(dst)) => {
            // Check write permission for both source and destination
            check_write_permission(src, Some("fs.rename()"))
                .map_err(RuntimeError::new)?;
            check_write_permission(dst, Some("fs.rename()"))
                .map_err(RuntimeError::new)?;
            
            fs::rename(src, dst)
                .map(|_| Value::Unit)
                .map_err(|e| RuntimeError::new(format!("Failed to rename file: {}", e)))
        }
        _ => Err(RuntimeError::new("fs.rename() arguments must be strings")),
    }
}

/// Get file metadata (size, modified time, etc.)
#[loft_builtin(fs.metadata)]
fn fs_metadata(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("fs.metadata() requires a path argument"));
    }
    
    match &args[0] {
        Value::String(path) => {
            // Check read permission
            check_read_permission(path, Some("fs.metadata()"))
                .map_err(RuntimeError::new)?;
            
            fs::metadata(path)
                .map_err(|e| RuntimeError::new(format!("Failed to get metadata: {}", e)))
                .map(|metadata| {
                    use rust_decimal::Decimal;
                    
                    // Return an array with [size, is_file, is_dir]
                    Value::Array(vec![
                        Value::Number(Decimal::from(metadata.len())),
                        Value::Boolean(metadata.is_file()),
                        Value::Boolean(metadata.is_dir()),
                    ])
                })
        }
        _ => Err(RuntimeError::new("fs.metadata() argument must be a string")),
    }
}

pub fn create_fs_builtin() -> BuiltinStruct {
    let mut fs = BuiltinStruct::new("fs");
    
    fs.add_method("read", fs_read_file as BuiltinMethod);
    fs.add_method("write", fs_write_file as BuiltinMethod);
    fs.add_method("append", fs_append_file as BuiltinMethod);
    fs.add_method("exists", fs_exists as BuiltinMethod);
    fs.add_method("is_file", fs_is_file as BuiltinMethod);
    fs.add_method("is_dir", fs_is_dir as BuiltinMethod);
    fs.add_method("create_dir", fs_create_dir as BuiltinMethod);
    fs.add_method("remove_file", fs_remove_file as BuiltinMethod);
    fs.add_method("remove_dir", fs_remove_dir as BuiltinMethod);
    fs.add_method("list_dir", fs_list_dir as BuiltinMethod);
    fs.add_method("copy", fs_copy as BuiltinMethod);
    fs.add_method("rename", fs_rename as BuiltinMethod);
    fs.add_method("metadata", fs_metadata as BuiltinMethod);
    
    fs
}

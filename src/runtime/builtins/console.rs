use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::RuntimeResult;
use loft_builtin_macros::loft_builtin;

/// Helper function to format values for console-style output
fn format_value(value: &Value) -> String {
    match value {
        Value::Unit => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", items.join(", "))
        },
        Value::Struct { name, fields } => {
            if name == "Object" {
                let items: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            } else {
                format!("{}(...)", name)
            }
        },
        Value::Function { .. } => "[Function]".to_string(),
        Value::Closure { .. } => "[Closure]".to_string(),
        Value::Builtin(_) => "[Builtin]".to_string(),
        Value::Promise(_) => "[Promise]".to_string(),
        Value::BuiltinFn(_) => "[BuiltinFn]".to_string(),
        Value::BoundMethod { .. } => "[BoundMethod]".to_string(),
        Value::UserMethod { .. } => "[Method]".to_string(),
        Value::EnumVariant { enum_name, variant_name, values } => {
            if values.is_empty() {
                format!("{}.{}", enum_name, variant_name)
            } else {
                let vals: Vec<String> = values.iter().map(format_value).collect();
                format!("{}.{}({})", enum_name, variant_name, vals.join(", "))
            }
        }
        Value::EnumConstructor { enum_name, variant_name, .. } => {
            format!("{}.{}", enum_name, variant_name)
        }
        Value::Module { name, .. } => {
            format!("<module {}>", name)
        }
    }
}

/// Log values to console (alias for term.log)
#[loft_builtin(console.log)]
fn console_log(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", format_value(arg));
    }
    println!();
    Ok(Value::Unit)
}

/// Log an error message to console
#[loft_builtin(console.error)]
fn console_error(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    eprint!("[ERROR] ");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            eprint!(" ");
        }
        eprint!("{}", format_value(arg));
    }
    eprintln!();
    Ok(Value::Unit)
}

/// Log a warning message to console
#[loft_builtin(console.warn)]
fn console_warn(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    eprint!("[WARN] ");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            eprint!(" ");
        }
        eprint!("{}", format_value(arg));
    }
    eprintln!();
    Ok(Value::Unit)
}

/// Log an info message to console
#[loft_builtin(console.info)]
fn console_info(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    print!("[INFO] ");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", format_value(arg));
    }
    println!();
    Ok(Value::Unit)
}

/// Log a debug message to console
#[loft_builtin(console.debug)]
fn console_debug(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    print!("[DEBUG] ");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", format_value(arg));
    }
    println!();
    Ok(Value::Unit)
}

/// Create console builtin (now an alias for term with console methods)
/// Note: This is maintained for backward compatibility. 
/// New code should prefer using term.log(), term.error(), etc.
pub fn create_console_builtin() -> BuiltinStruct {
    let mut console = BuiltinStruct::new("console");
    
    console.add_method("log", console_log as BuiltinMethod);
    console.add_method("error", console_error as BuiltinMethod);
    console.add_method("warn", console_warn as BuiltinMethod);
    console.add_method("info", console_info as BuiltinMethod);
    console.add_method("debug", console_debug as BuiltinMethod);
    
    console
}

// Register the builtin automatically
crate::submit_builtin!("console", create_console_builtin);

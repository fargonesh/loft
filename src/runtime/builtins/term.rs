use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use crate::runtime::traits::ToString;
use crate::runtime::permission_context::check_run_permission;
use loft_builtin_macros::loft_builtin;

/// Print text to the terminal
#[loft_builtin(term.print)]
fn term_print(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg.to_string());
    }
    Ok(Value::Unit)
}

/// Print text to the terminal with a newline
#[loft_builtin(term.println)]
fn term_println(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    term_print(_this, args)?;
    println!();
    Ok(Value::Unit)
}

/// Clear the terminal screen
#[loft_builtin(term.clear)]
fn term_clear(_this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    print!("\x1B[2J\x1B[1;1H");
    Ok(Value::Unit)
}

/// Read a line from the terminal
#[loft_builtin(term.read_line)]
fn term_read_line(_this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    use std::io::{self, BufRead};
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();
    handle
        .read_line(&mut line)
        .map_err(|e| RuntimeError::new(format!("Failed to read line: {}", e)))?;
    // Remove trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(Value::String(line))
}

/// Get the terminal size (width, height)
#[loft_builtin(term.size)]
fn term_size(_this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    use std::process::Command;
    
    // Check permission to run tput command
    check_run_permission("tput", Some("term.size()"))
        .map_err(RuntimeError::new)?;
    
    // Try to get terminal size using tput
    let width_output = Command::new("tput")
        .arg("cols")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<i32>().ok());
    
    let height_output = Command::new("tput")
        .arg("lines")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<i32>().ok());
    
    match (width_output, height_output) {
        (Some(width), Some(height)) => {
            use rust_decimal::Decimal;
            Ok(Value::Array(vec![
                Value::Number(Decimal::from(width)),
                Value::Number(Decimal::from(height)),
            ]))
        }
        _ => {
            // Default fallback size
            use rust_decimal::Decimal;
            Ok(Value::Array(vec![
                Value::Number(Decimal::from(80)),
                Value::Number(Decimal::from(24)),
            ]))
        }
    }
}

/// Set text color (basic ANSI colors)
#[loft_builtin(term.color)]
fn term_color(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("term.color() requires a color argument"));
    }
    
    if let Value::String(color) = &args[0] {
        let ansi_code = match color.to_lowercase().as_str() {
            "black" => "30",
            "red" => "31",
            "green" => "32",
            "yellow" => "33",
            "blue" => "34",
            "magenta" | "purple" => "35",
            "cyan" => "36",
            "white" => "37",
            "reset" => "0",
            _ => return Err(RuntimeError::new(format!("Unknown color: {}", color))),
        };
        print!("\x1B[{}m", ansi_code);
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::new("term.color() argument must be a string"))
    }
}

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

/// Log values to console (alias for term.println)
#[loft_builtin(term.log)]
fn term_log(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
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
#[loft_builtin(term.error)]
fn term_error(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
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
#[loft_builtin(term.warn)]
fn term_warn(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
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
#[loft_builtin(term.info)]
fn term_info(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
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
#[loft_builtin(term.debug)]
fn term_debug(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
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

/// Create the Term builtin struct
pub fn create_term_builtin() -> BuiltinStruct {
    let mut term = BuiltinStruct::new("term");
    
    term.add_method("print", term_print as BuiltinMethod);
    term.add_method("println", term_println as BuiltinMethod);
    term.add_method("clear", term_clear as BuiltinMethod);
    term.add_method("read_line", term_read_line as BuiltinMethod);
    term.add_method("size", term_size as BuiltinMethod);
    term.add_method("color", term_color as BuiltinMethod);
    
    // Console-style methods (merged from console module)
    term.add_method("log", term_log as BuiltinMethod);
    term.add_method("error", term_error as BuiltinMethod);
    term.add_method("warn", term_warn as BuiltinMethod);
    term.add_method("info", term_info as BuiltinMethod);
    term.add_method("debug", term_debug as BuiltinMethod);
    
    term
}

// Register the builtin automatically
crate::submit_builtin!("term", create_term_builtin);

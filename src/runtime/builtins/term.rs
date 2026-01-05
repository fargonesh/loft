use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult, Interpreter};
use crate::runtime::traits::ToString;
use crate::runtime::permission_context::check_run_permission;
use loft_builtin_macros::loft_builtin;

/// Print text to the terminal
#[loft_builtin(term.print)]
fn term_print(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
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
fn term_println(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    term_print(_interpreter, _this, args)?;
    println!();
    Ok(Value::Unit)
}

/// Alias for println (for console migration)
#[loft_builtin(term.log)]
fn term_log(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    term_println(_interpreter, this, args)
}

/// Alias for println with [ERROR] prefix
#[loft_builtin(term.error)]
fn term_error(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    print!("\x1B[31m[ERROR]\x1B[0m ");
    term_println(_interpreter, this, args)
}

/// Alias for println with [WARN] prefix
#[loft_builtin(term.warn)]
fn term_warn(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    print!("\x1B[33m[WARN]\x1B[0m ");
    term_println(_interpreter, this, args)
}

/// Alias for println with [INFO] prefix
#[loft_builtin(term.info)]
fn term_info(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    print!("\x1B[34m[INFO]\x1B[0m ");
    term_println(_interpreter, this, args)
}

/// Alias for println with [DEBUG] prefix
#[loft_builtin(term.debug)]
fn term_debug(_interpreter: &mut Interpreter, this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    print!("\x1B[36m[DEBUG]\x1B[0m ");
    term_println(_interpreter, this, args)
}

/// Clear the terminal screen
#[loft_builtin(term.clear)]
fn term_clear(_interpreter: &mut Interpreter, _this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    print!("\x1B[2J\x1B[1;1H");
    Ok(Value::Unit)
}

/// Read a line from the terminal
#[loft_builtin(term.read_line)]
fn term_read_line(_interpreter: &mut Interpreter, _this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
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
fn term_size(_interpreter: &mut Interpreter, _this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
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
fn term_color(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
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

/// Create the Term builtin struct
pub fn create_term_builtin() -> BuiltinStruct {
    let mut term = BuiltinStruct::new("term");
    
    term.add_method("print", term_print as BuiltinMethod);
    term.add_method("println", term_println as BuiltinMethod);
    term.add_method("clear", term_clear as BuiltinMethod);
    term.add_method("read_line", term_read_line as BuiltinMethod);
    term.add_method("size", term_size as BuiltinMethod);
    term.add_method("color", term_color as BuiltinMethod);
    
    // Logging methods
    term.add_method("log", term_log as BuiltinMethod);
    term.add_method("error", term_error as BuiltinMethod);
    term.add_method("warn", term_warn as BuiltinMethod);
    term.add_method("info", term_info as BuiltinMethod);
    term.add_method("debug", term_debug as BuiltinMethod);
    
    term
}

// Register the builtin automatically
crate::submit_builtin!("term", create_term_builtin);

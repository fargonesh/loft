use wasm_bindgen::prelude::*;
use loft::parser::{InputStream, Parser};
// Use `loft::runtime::value::Value` fully qualified to avoid ambiguity if any, 
// and import ToString trait to use `to_string()` method.
use loft::runtime::value::Value;
use loft::runtime::Interpreter;
use loft::runtime::traits::ToString; 
use loft::formatter::TokenFormatter; // Import TokenFormatter
use std::cell::RefCell;

thread_local! {
    static OUTPUT_BUFFER: RefCell<String> = RefCell::new(String::new());
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

fn custom_print(_this: &Value, args: &[Value]) -> loft::runtime::RuntimeResult<Value> {
    OUTPUT_BUFFER.with(|b| {
        let mut buf = b.borrow_mut();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                buf.push(' ');
            }
            buf.push_str(&arg.to_string());
        }
    });
    Ok(Value::Unit)
}

fn custom_println(this: &Value, args: &[Value]) -> loft::runtime::RuntimeResult<Value> {
    custom_print(this, args)?;
    OUTPUT_BUFFER.with(|b| b.borrow_mut().push('\n'));
    Ok(Value::Unit)
}

#[wasm_bindgen]
pub fn run_code(source: &str) -> String {
    // Reset buffer
    OUTPUT_BUFFER.with(|b| b.borrow_mut().clear());

    let source = source.to_string();
    let input = InputStream::new("playground", &source);
    let mut parser = Parser::new(input);

    match parser.parse() {
        Ok(stmts) => {
            let mut interpreter = Interpreter::new();

            // Override term.print and term.println
            if let Some(Value::Builtin(mut term)) = interpreter.env.get("term").cloned() {
                term.methods.insert("print".to_string(), custom_print);
                term.methods.insert("println".to_string(), custom_println);
                let _ = interpreter.env.update("term", Value::Builtin(term));
            }

            match interpreter.eval_program(stmts) {
                Ok(_) => OUTPUT_BUFFER.with(|b| b.borrow().clone()),
                Err(e) => {
                    let output = OUTPUT_BUFFER.with(|b| b.borrow().clone());
                    format!("{}\nRuntime Error: {}", output, e)
                }
            }
        }
        Err(e) => format!("Parse Error: {}", e),
    }
}

#[wasm_bindgen]
pub fn format_code(source: &str) -> Result<String, String> {
    let formatter = TokenFormatter::new();
    formatter.format(source)
}

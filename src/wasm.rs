use wasm_bindgen::prelude::*;
use crate::parser::{InputStream, Parser};
use crate::runtime::{Interpreter, value::Value};
use crate::formatter::TokenFormatter;
use miette::GraphicalReportHandler;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct LoftResult {
    output: String,
    error: Option<String>,
}

#[wasm_bindgen]
impl LoftResult {
    #[wasm_bindgen(getter)]
    pub fn output(&self) -> String {
        self.output.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
}

#[wasm_bindgen]
pub fn run_loft(code: &str) -> LoftResult {
    let code_string = code.to_string();
    let mut interpreter = Interpreter::with_source("playground", &code_string);
    let stream = InputStream::new("playground", &code_string);
    let mut parser = Parser::new(stream);

    crate::runtime::output::clear_stdout();

    match parser.parse() {
        Ok(stmts) => {
            match interpreter.eval_program(stmts) {
                Ok(result) => {
                    let mut output = crate::runtime::output::get_stdout();
                    if result != Value::Unit {
                        if !output.is_empty() && !output.ends_with('\n') {
                            output.push('\n');
                        }
                        output.push_str(&format!("{:?}", result));
                    };
                    LoftResult {
                        output,
                        error: None,
                    }
                }
                Err(e) => {
                    let mut out = String::new();
                    let _ = GraphicalReportHandler::default().render_report(&mut out, &e);
                    LoftResult {
                        output: crate::runtime::output::get_stdout(),
                        error: Some(out),
                    }
                }
            }
        }
        Err(e) => {
            let mut out = String::new();
            let _ = GraphicalReportHandler::default().render_report(&mut out, &e);
            LoftResult {
                output: crate::runtime::output::get_stdout(),
                error: Some(out),
            }
        }
    }
}

#[wasm_bindgen]
pub fn format_loft(code: &str) -> String {
    let formatter = TokenFormatter::new();
    match formatter.format(code) {
        Ok(formatted) => formatted,
        Err(_) => code.to_string(),
    }
}

#[derive(serde::Serialize)]
struct StdlibMetadata {
    builtins: Vec<BuiltinMetadata>,
    globals: Vec<String>,
}

#[derive(serde::Serialize)]
struct BuiltinMetadata {
    name: String,
    methods: Vec<String>,
    fields: Vec<String>,
}

#[wasm_bindgen]
pub fn get_stdlib_metadata() -> String {
    let builtins_list = crate::runtime::builtins::init_builtins();
    let mut metadata = StdlibMetadata {
        builtins: Vec::new(),
        globals: Vec::new(),
    };

    for (name, value) in builtins_list {
        match value {
            Value::Builtin(b) => {
                metadata.builtins.push(BuiltinMetadata {
                    name: b.name.clone(),
                    methods: b.methods.keys().cloned().collect(),
                    fields: b.fields.keys().cloned().collect(),
                });
            }
            Value::BuiltinFn(_) => {
                metadata.globals.push(name);
            }
            _ => {
                metadata.globals.push(name);
            }
        }
    }

    serde_json::to_string(&metadata).unwrap_or_else(|_| "{}".to_string())
}

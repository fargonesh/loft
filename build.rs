use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::{Attribute, Item, ItemFn, Meta, Lit};

type BuiltinInfo = HashMap<String, HashMap<String, serde_json::Value>>;

fn extract_doc_comments(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &meta.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            return Some(lit_str.value());
                        }
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn extract_loft_builtin_attr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("loft_builtin") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens = meta_list.tokens.to_string();
                return Some(tokens);
            }
        }
    }
    None
}

fn parse_builtin_path(path: &str) -> (String, String) {
    // Parse paths like "web.request", "math.sin", etc.
    // Remove any whitespace first
    let path = path.trim();
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() >= 2 {
        (parts[0].trim().to_string(), parts[1].trim().to_string())
    } else {
        ("unknown".to_string(), path.to_string())
    }
}

fn extract_function_signature(_func: &ItemFn) -> (Vec<String>, String) {
    // This is a simplified extraction - in practice you'd want more robust parsing
    // For now, we'll return basic info
    let params = vec!["...args".to_string()]; // Placeholder
    let return_type = "any".to_string(); // Placeholder
    (params, return_type)
}

fn scan_rust_file(path: &Path, builtins: &mut BuiltinInfo) -> std::io::Result<()> {
    let content = fs::read_to_string(path)?;
    let syntax = match syn::parse_file(&content) {
        Ok(s) => s,
        Err(_) => return Ok(()), // Skip files that can't be parsed
    };

    for item in syntax.items {
        if let Item::Fn(func) = item {
            if let Some(path_str) = extract_loft_builtin_attr(&func.attrs) {
                let doc = extract_doc_comments(&func.attrs);
                let (module_name, method_name) = parse_builtin_path(&path_str);
                let (_params, _return_type) = extract_function_signature(&func);

                // Add to builtins structure
                let module = builtins
                    .entry(module_name.clone())
                    .or_insert_with(HashMap::new);

                let mut method_info = HashMap::new();
                method_info.insert("documentation".to_string(), serde_json::Value::String(doc));
                
                module.insert(
                    method_name,
                    serde_json::json!(method_info),
                );
            }
        }
    }

    Ok(())
}

fn scan_directory(dir: &Path, builtins: &mut BuiltinInfo) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                scan_directory(&path, builtins)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let _ = scan_rust_file(&path, builtins);
            }
        }
    }
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=src/runtime/builtins/");
    println!("cargo:rerun-if-changed=build.rs");

    let mut builtins: BuiltinInfo = HashMap::new();

    // Scan the builtins directory
    let builtins_path = Path::new("src/runtime/builtins");
    if let Err(e) = scan_directory(builtins_path, &mut builtins) {
        eprintln!("Warning: Failed to scan builtins directory: {}", e);
    }

    // Generate JSON
    let json = serde_json::to_string_pretty(&builtins).unwrap_or_else(|_| "{}".to_string());

    // Write to output file
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("builtins_generated.json");
    
    if let Err(e) = fs::write(&dest_path, json) {
        eprintln!("Warning: Failed to write builtins.json: {}", e);
    } else {
        println!("Generated builtins documentation at {:?}", dest_path);
    }
}

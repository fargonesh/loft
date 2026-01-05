use std::collections::HashMap;
use std::fs;
use std::path::Path;
use regex::Regex;
use crate::docgen::stdlib::{StdlibTypes, BuiltinDef, MethodDef, TypeDef};

pub struct StdlibScanner;

impl StdlibScanner {
    pub fn scan(root_dir: &Path) -> StdlibTypes {
        let mut stdlib = StdlibTypes {
            builtins: HashMap::new(),
            string_methods: HashMap::new(),
            array_methods: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
        };

        // Initialize basic types
        stdlib.types.insert("Array".to_string(), TypeDef {
            kind: "primitive".to_string(),
            documentation: "Array primitive type".to_string(),
            fields: HashMap::new(),
            methods: HashMap::new(),
        });

        Self::scan_dir(root_dir, &mut stdlib);
        stdlib
    }

    fn scan_dir(dir: &Path, stdlib: &mut StdlibTypes) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::scan_dir(&path, stdlib);
                } else if path.extension().map_or(false, |ext| ext == "rs") {
                    Self::scan_file(&path, stdlib);
                }
            }
        }
    }

    fn scan_file(path: &Path, stdlib: &mut StdlibTypes) {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        // Regex to find doc comments and loft_builtin attribute
        // Matches:
        // /// Doc comment line 1
        // /// Doc comment line 2
        // #[loft_builtin(name)]
        // fn func_name(this_arg, ...)
        
        let re = Regex::new(r"(?m)((?:^\s*///.*\n)+)\s*#\[loft_builtin\(([\w\.]+)\)\]\s*fn\s+\w+\s*\(([^,)]+)").unwrap();
        
        for cap in re.captures_iter(&content) {
            let docs = &cap[1];
            let name = &cap[2];
            let first_arg = &cap[3]; // "this: &Value" or "_this: &Value"

            let mut documentation = docs.lines()
                .map(|l| l.trim().trim_start_matches("///").trim())
                .collect::<Vec<_>>()
                .join("\n");

            // Parse @param and @return tags
            let mut params = Vec::new();
            let mut return_type = "any".to_string();

            let param_re = Regex::new(r"@param\s+(\w+):\s*([\w<>\[\]]+)").unwrap();
            let return_re = Regex::new(r"@return\s+([\w<>\[\]]+)").unwrap();

            for p_cap in param_re.captures_iter(&documentation) {
                params.push(format!("{}: {}", &p_cap[1], &p_cap[2]));
            }

            if let Some(r_cap) = return_re.captures(&documentation) {
                return_type = r_cap[1].to_string();
            }

            // Clean up documentation by removing tags
            documentation = documentation.lines()
                .filter(|l| !l.trim().starts_with("@param") && !l.trim().starts_with("@return"))
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();

            let is_static = first_arg.trim().starts_with("_");
            
            // Parse name "array.zip" -> "array", "zip"
            let parts: Vec<&str> = name.split('.').collect();
            if parts.len() != 2 {
                continue;
            }
            
            let namespace = parts[0];
            let method_name = parts[1];

            let method_def = MethodDef {
                params: if params.is_empty() { vec!["...args".to_string()] } else { params },
                return_type,
                documentation,
            };

            if namespace == "array" {
                if is_static {
                    // Add to builtins["array"].methods
                    stdlib.builtins.entry("array".to_string())
                        .or_insert_with(|| BuiltinDef {
                            kind: "module".to_string(),
                            documentation: "Array module".to_string(),
                            methods: HashMap::new(),
                            constants: HashMap::new(),
                        })
                        .methods.insert(method_name.to_string(), method_def);
                } else {
                    // Add to array_methods
                    stdlib.array_methods.insert(method_name.to_string(), method_def);
                }
            } else if namespace == "string" || namespace == "str" {
                 stdlib.string_methods.insert(method_name.to_string(), method_def);
            } else {
                // Assume other namespaces are builtins
                stdlib.builtins.entry(namespace.to_string())
                    .or_insert_with(|| BuiltinDef {
                        kind: "module".to_string(),
                        documentation: format!("{} module", namespace),
                        methods: HashMap::new(),
                        constants: HashMap::new(),
                    })
                    .methods.insert(method_name.to_string(), method_def);
            }
        }
    }
}

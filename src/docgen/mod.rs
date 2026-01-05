pub mod stdlib;
pub mod scanner;

use crate::parser::{InputStream, Parser, Stmt, Type};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use regex;

#[derive(Debug, Clone)]
pub struct DocItem {
    pub name: String,
    pub kind: DocItemKind,
    pub documentation: Option<String>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DocItemKind {
    Function {
        params: Vec<(String, String)>,
        return_type: String,
        is_async: bool,
        is_exported: bool,
    },
    Struct {
        fields: Vec<(String, String)>,
        implemented_traits: Vec<String>,
    },
    Trait {
        methods: Vec<String>,
        implementors: Vec<String>,
    },
    Constant {
        const_type: String,
    },
    Variable {
        var_type: String,
    },
}

pub struct DocGenerator {
    items: Vec<DocItem>,
    source_files: HashMap<PathBuf, String>,
    impl_relations: Vec<(String, String)>,
}

impl Default for DocGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DocGenerator {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            source_files: HashMap::new(),
            impl_relations: Vec::new(),
        }
    }

    /// Parse a loft file and extract documentation items
    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

        self.source_files.insert(path.to_path_buf(), content.clone());

        let input_stream = InputStream::new(path.to_str().unwrap_or("unknown"), &content);
        let mut parser = Parser::new(input_stream);

        let stmts = parser.parse()
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e.message))?;

        self.extract_items(&stmts, &content);
        Ok(())
    }

    /// Extract documentation items from parsed statements
    fn extract_items(&mut self, stmts: &[Stmt], source: &str) {
        let doc_comments = Self::extract_doc_comments(source);

        for stmt in stmts {
            match stmt {
                Stmt::FunctionDecl { 
                    name, 
                    params, 
                    return_type, 
                    is_async,
                    is_exported,
                    .. 
                } => {
                    let params_vec: Vec<(String, String)> = params.iter()
                        .map(|(n, t)| (n.clone(), Self::type_to_string(t)))
                        .collect();

                    let signature = format!(
                        "{}{}fn {}({}) -> {}",
                        if *is_exported { "teach " } else { "" },
                        if *is_async { "async " } else { "" },
                        name,
                        params_vec.iter()
                            .map(|(n, t)| format!("{}: {}", n, t))
                            .collect::<Vec<_>>()
                            .join(", "),
                        Self::opt_type_to_string(return_type)
                    );

                    self.items.push(DocItem {
                        name: name.clone(),
                        kind: DocItemKind::Function {
                            params: params_vec,
                            return_type: Self::opt_type_to_string(return_type),
                            is_async: *is_async,
                            is_exported: *is_exported,
                        },
                        documentation: doc_comments.get(name).cloned(),
                        signature: Some(signature),
                    });
                }
                Stmt::StructDecl { name, fields } => {
                    let fields_vec: Vec<(String, String)> = fields.iter()
                        .map(|(n, t)| (n.clone(), Self::type_to_string(t)))
                        .collect();

                    let signature = format!(
                        "def {} {{\n{}\n}}",
                        name,
                        fields_vec.iter()
                            .map(|(n, t)| format!("    {}: {}", n, t))
                            .collect::<Vec<_>>()
                            .join(",\n")
                    );

                    self.items.push(DocItem {
                        name: name.clone(),
                        kind: DocItemKind::Struct {
                            fields: fields_vec,
                            implemented_traits: Vec::new(),
                        },
                        documentation: doc_comments.get(name).cloned(),
                        signature: Some(signature),
                    });
                }
                Stmt::TraitDecl { name, methods } => {
                    let method_names: Vec<String> = methods.iter()
                        .map(|m| match m {
                            crate::parser::TraitMethod::Signature { name, .. } => name.clone(),
                            crate::parser::TraitMethod::Default { name, .. } => name.clone(),
                        })
                        .collect();

                    self.items.push(DocItem {
                        name: name.clone(),
                        kind: DocItemKind::Trait {
                            methods: method_names,
                            implementors: Vec::new(),
                        },
                        documentation: doc_comments.get(name).cloned(),
                        signature: Some(format!("trait {}", name)),
                    });
                }
                Stmt::ConstDecl { name, const_type, .. } => {
                    let type_str = const_type.as_ref()
                        .map(Self::type_to_string)
                        .unwrap_or_else(|| "unknown".to_string());

                    self.items.push(DocItem {
                        name: name.clone(),
                        kind: DocItemKind::Constant {
                            const_type: type_str.clone(),
                        },
                        documentation: doc_comments.get(name).cloned(),
                        signature: Some(format!("const {}: {}", name, type_str)),
                    });
                }
                Stmt::VarDecl { name, var_type, .. } => {
                    let type_str = var_type.as_ref()
                        .map(Self::type_to_string)
                        .unwrap_or_else(|| "unknown".to_string());

                    self.items.push(DocItem {
                        name: name.clone(),
                        kind: DocItemKind::Variable {
                            var_type: type_str.clone(),
                        },
                        documentation: doc_comments.get(name).cloned(),
                        signature: Some(format!("let {}: {}", name, type_str)),
                    });
                }
                Stmt::ImplBlock { type_name, trait_name, methods } => {
                    if let Some(trait_name) = trait_name {
                        self.impl_relations.push((trait_name.clone(), type_name.clone()));
                    }
                    // Recursively process methods in impl blocks
                    self.extract_items(methods, source);
                }
                _ => {}
            }
        }
    }

    /// Extract doc comments from source code
    fn extract_doc_comments(source: &str) -> HashMap<String, String> {
        let lines: Vec<&str> = source.lines().collect();
        let mut doc_map: HashMap<String, String> = HashMap::new();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            // Check for /// style doc comment
            if line.starts_with("///") {
                let mut doc_lines = vec![];
                while i < lines.len() && lines[i].trim().starts_with("///") {
                    let doc_text = lines[i].trim().strip_prefix("///").unwrap().trim();
                    if !doc_text.is_empty() {
                        doc_lines.push(doc_text);
                    }
                    i += 1;
                }

                // Find next declaration
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                if i < lines.len() {
                    if let Some(name) = Self::extract_name_from_declaration(lines[i]) {
                        doc_map.insert(name, doc_lines.join("\n"));
                    }
                }
            } else if line.starts_with("/**") {
                // Block doc comment
                let mut doc_lines = vec![];
                
                if line.contains("*/") {
                    // Single-line block comment
                    let doc_text = line.strip_prefix("/**")
                        .and_then(|s| s.strip_suffix("*/"))
                        .map(|s| s.trim())
                        .unwrap_or("");
                    doc_lines.push(doc_text);
                    i += 1;
                } else {
                    // Multi-line block comment
                    let first_line = line.strip_prefix("/**").unwrap().trim();
                    if !first_line.is_empty() && !first_line.starts_with("*") {
                        doc_lines.push(first_line);
                    }
                    i += 1;

                    while i < lines.len() {
                        let comment_line = lines[i].trim();
                        if comment_line.ends_with("*/") {
                            let text = comment_line.strip_suffix("*/").unwrap().trim();
                            if text.starts_with("*") {
                                let text = text.strip_prefix("*").unwrap().trim();
                                if !text.is_empty() {
                                    doc_lines.push(text);
                                }
                            } else if !text.is_empty() {
                                doc_lines.push(text);
                            }
                            i += 1;
                            break;
                        } else {
                            if comment_line.starts_with("*") {
                                let text = comment_line.strip_prefix("*").unwrap().trim();
                                if !text.is_empty() {
                                    doc_lines.push(text);
                                }
                            }
                            i += 1;
                        }
                    }
                }

                // Find next declaration
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                if i < lines.len() {
                    if let Some(name) = Self::extract_name_from_declaration(lines[i]) {
                        doc_map.insert(name, doc_lines.join("\n"));
                    }
                }
            }

            i += 1;
        }

        doc_map
    }

    /// Extract the name from a declaration line
    fn extract_name_from_declaration(line: &str) -> Option<String> {
        let line = line.trim();

        let patterns = [
            ("teach fn ", true),
            ("async fn ", true),
            ("fn ", true),
            ("teach async fn ", true),
            ("let mut ", true),
            ("let ", true),
            ("const ", true),
            ("def ", true),
            ("trait ", true),
            ("struct ", true),
        ];

        for (pattern, extract_next) in patterns {
            if line.starts_with(pattern)
                && extract_next {
                    let rest = line.strip_prefix(pattern).unwrap();
                    // Extract identifier (alphanumeric + underscore)
                    let name: String = rest.chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !name.is_empty() {
                        return Some(name);
                    }
                }
        }

        None
    }

    /// Convert a Type to a string representation
    fn type_to_string(ty: &Type) -> String {
        match ty {
            Type::Named(name) => name.clone(),
            Type::Generic { base, type_args } => {
                format!(
                    "{}<{}>",
                    base,
                    type_args.iter()
                        .map(Self::type_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Type::Function { params, return_type } => {
                format!(
                    "({}) -> {}",
                    params.iter()
                        .map(Self::type_to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                    Self::type_to_string(return_type)
                )
            }
        }
    }
    
    fn opt_type_to_string(ty: &Option<Type>) -> String {
        match ty {
            Some(t) => Self::type_to_string(t),
            None => "void".to_string(),
        }
    }

    /// Generate HTML documentation
    pub fn generate_html(&mut self, output_dir: &Path, package_name: &str) -> Result<(), String> {
        // Post-process implementations
        let relations = self.impl_relations.clone();
        for (trait_name, type_name) in relations {
            // Update trait
            if let Some(item) = self.items.iter_mut().find(|i| i.name == trait_name) {
                if let DocItemKind::Trait { implementors, .. } = &mut item.kind {
                    if !implementors.contains(&type_name) {
                        implementors.push(type_name.clone());
                    }
                }
            }
            // Update struct
            if let Some(item) = self.items.iter_mut().find(|i| i.name == type_name) {
                if let DocItemKind::Struct { implemented_traits, .. } = &mut item.kind {
                    if !implemented_traits.contains(&trait_name) {
                        implemented_traits.push(trait_name.clone());
                    }
                }
            }
        }

        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Generate index.html
        let index_html = self.generate_index_html(package_name);
        let index_path = output_dir.join("index.html");
        fs::write(&index_path, index_html)
            .map_err(|e| format!("Failed to write index.html: {}", e))?;

        // Generate CSS
        let css = self.generate_css();
        let css_path = output_dir.join("style.css");
        fs::write(&css_path, css)
            .map_err(|e| format!("Failed to write style.css: {}", e))?;

        Ok(())
    }

    fn generate_index_html(&self, package_name: &str) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("    <title>{} - loft Documentation</title>\n", package_name));
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str(&format!("    <h1>{} Documentation</h1>\n", package_name));
        html.push_str("    <div class=\"container\">\n");

        // Group items by kind
        let functions: Vec<_> = self.items.iter()
            .filter(|item| matches!(item.kind, DocItemKind::Function { .. }))
            .collect();
        let structs: Vec<_> = self.items.iter()
            .filter(|item| matches!(item.kind, DocItemKind::Struct { .. }))
            .collect();
        let traits: Vec<_> = self.items.iter()
            .filter(|item| matches!(item.kind, DocItemKind::Trait { .. }))
            .collect();
        let constants: Vec<_> = self.items.iter()
            .filter(|item| matches!(item.kind, DocItemKind::Constant { .. }))
            .collect();

        // Functions
        if !functions.is_empty() {
            html.push_str("        <h2>Functions</h2>\n");
            for item in functions {
                html.push_str(&self.generate_item_html(item));
            }
        }

        // Structs
        if !structs.is_empty() {
            html.push_str("        <h2>Structs</h2>\n");
            for item in structs {
                html.push_str(&self.generate_item_html(item));
            }
        }

        // Traits
        if !traits.is_empty() {
            html.push_str("        <h2>Traits</h2>\n");
            for item in traits {
                html.push_str(&self.generate_item_html(item));
            }
        }

        // Constants
        if !constants.is_empty() {
            html.push_str("        <h2>Constants</h2>\n");
            for item in constants {
                html.push_str(&self.generate_item_html(item));
            }
        }

        html.push_str("    </div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");
        html
    }

    fn generate_item_html(&self, item: &DocItem) -> String {
        let mut html = String::new();
        html.push_str("        <div class=\"doc-item\">\n");
        html.push_str(&format!("            <h3 id=\"{}\">{}</h3>\n", item.name, item.name));

        // Generate signature with links
        let signature_html = match &item.kind {
            DocItemKind::Function { params, return_type, is_async, is_exported } => {
                let params_html = params.iter().map(|(name, ty)| {
                    format!("{}: {}", name, self.type_to_html_string(ty))
                }).collect::<Vec<_>>().join(", ");
                
                format!("{}{}fn {}({}) -> {}",
                    if *is_exported { "teach " } else { "" },
                    if *is_async { "async " } else { "" },
                    item.name,
                    params_html,
                    self.type_to_html_string(return_type)
                )
            }
            _ => {
                if let Some(sig) = &item.signature {
                    Self::escape_html(sig)
                } else {
                    String::new()
                }
            }
        };

        if !signature_html.is_empty() {
            html.push_str("            <pre class=\"signature\"><code>");
            html.push_str(&signature_html);
            html.push_str("</code></pre>\n");
        }

        if let Some(doc) = &item.documentation {
            html.push_str("            <div class=\"description\">\n");
            for line in doc.lines() {
                html.push_str("                <p>");
                html.push_str(&Self::escape_html(line));
                html.push_str("</p>\n");
            }
            html.push_str("            </div>\n");
        }

        // Add details based on kind
        match &item.kind {
            DocItemKind::Function { params, return_type, is_exported, .. } => {
                if !params.is_empty() {
                    html.push_str("            <h4>Parameters</h4>\n");
                    html.push_str("            <ul class=\"parameters\">\n");
                    for (name, ty) in params {
                        html.push_str(&format!(
                            "                <li><code>{}</code>: <code>{}</code></li>\n",
                            Self::escape_html(name),
                            self.type_to_html_string(ty)
                        ));
                    }
                    html.push_str("            </ul>\n");
                }
                html.push_str(&format!(
                    "            <p><strong>Returns:</strong> <code>{}</code></p>\n",
                    self.type_to_html_string(return_type)
                ));
                if *is_exported {
                    html.push_str("            <p class=\"exported\">✓ Exported (teach)</p>\n");
                }
            }
            DocItemKind::Struct { fields, implemented_traits } => {
                if !implemented_traits.is_empty() {
                    html.push_str("            <h4>Implemented Traits</h4>\n");
                    html.push_str("            <ul class=\"traits\">\n");
                    for trait_name in implemented_traits {
                        html.push_str(&format!(
                            "                <li><a href=\"#{}\">{}</a></li>\n",
                            trait_name,
                            Self::escape_html(trait_name)
                        ));
                    }
                    html.push_str("            </ul>\n");
                }

                if !fields.is_empty() {
                    html.push_str("            <h4>Fields</h4>\n");
                    html.push_str("            <ul class=\"fields\">\n");
                    for (name, ty) in fields {
                        html.push_str(&format!(
                            "                <li><code>{}</code>: <code>{}</code></li>\n",
                            Self::escape_html(name),
                            self.type_to_html_string(ty)
                        ));
                    }
                    html.push_str("            </ul>\n");
                }
            }
            DocItemKind::Trait { methods, implementors } => {
                if !implementors.is_empty() {
                    html.push_str("            <h4>Implementors</h4>\n");
                    html.push_str("            <ul class=\"implementors\">\n");
                    for type_name in implementors {
                        html.push_str(&format!(
                            "                <li><a href=\"#{}\">{}</a></li>\n",
                            type_name,
                            Self::escape_html(type_name)
                        ));
                    }
                    html.push_str("            </ul>\n");
                }

                if !methods.is_empty() {
                    html.push_str("            <h4>Methods</h4>\n");
                    html.push_str("            <ul class=\"methods\">\n");
                    for method in methods {
                        html.push_str(&format!(
                            "                <li><code>{}</code></li>\n",
                            Self::escape_html(method)
                        ));
                    }
                    html.push_str("            </ul>\n");
                }
            }
            _ => {}
        }

        html.push_str("        </div>\n");
        html
    }

    fn type_to_html_string(&self, type_str: &str) -> String {
        // Basic heuristic: if the type string contains a known item name, link it.
        // This is a simplification. Ideally we'd parse the type string.
        let mut html = Self::escape_html(type_str);
        let mut replacements = Vec::new();
        
        // Sort items by name length descending to avoid partial replacements
        let mut item_names: Vec<String> = self.items.iter().map(|i| i.name.clone()).collect();
        item_names.sort_by_key(|b| std::cmp::Reverse(b.len()));

        for (i, name) in item_names.iter().enumerate() {
            // Only replace whole words
            let pattern = format!(r"\b{}\b", regex::escape(name));
            if let Ok(re) = regex::Regex::new(&pattern) {
                if re.is_match(&html) {
                    let placeholder = format!("__ITEM_{}__", i);
                    html = re.replace_all(&html, placeholder.as_str()).to_string();
                    replacements.push((placeholder, format!("<a href=\"#{}\">{}</a>", name, name)));
                }
            }
        }

        // Apply replacements
        for (placeholder, replacement) in replacements {
            html = html.replace(&placeholder, &replacement);
        }
        html
    }

    fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    fn generate_css(&self) -> String {
        r#"body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    line-height: 1.6;
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
    background-color: #f5f5f5;
    color: #333;
}

h1 {
    color: #2c3e50;
    border-bottom: 3px solid #3498db;
    padding-bottom: 10px;
    margin-bottom: 30px;
}

h2 {
    color: #34495e;
    margin-top: 40px;
    margin-bottom: 20px;
    border-bottom: 2px solid #ecf0f1;
    padding-bottom: 8px;
}

h3 {
    color: #2980b9;
    margin-top: 0;
}

h4 {
    color: #7f8c8d;
    margin-top: 15px;
    margin-bottom: 10px;
}

.container {
    background-color: white;
    padding: 30px;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.doc-item {
    margin-bottom: 40px;
    padding: 20px;
    border: 1px solid #ecf0f1;
    border-radius: 6px;
    background-color: #fafafa;
}

.signature {
    background-color: #2c3e50;
    color: #ecf0f1;
    padding: 15px;
    border-radius: 5px;
    overflow-x: auto;
    margin: 15px 0;
}

.signature code {
    font-family: "Courier New", Courier, monospace;
    font-size: 14px;
}

.description {
    margin: 15px 0;
    color: #555;
}

.description p {
    margin: 8px 0;
}

.parameters, .fields, .methods {
    list-style-type: none;
    padding-left: 0;
}

.parameters li, .fields li, .methods li {
    padding: 8px;
    margin: 5px 0;
    background-color: #fff;
    border-left: 3px solid #3498db;
    padding-left: 15px;
}

code {
    background-color: #ecf0f1;
    padding: 2px 6px;
    border-radius: 3px;
    font-family: "Courier New", Courier, monospace;
    font-size: 13px;
}

.exported {
    color: #27ae60;
    font-weight: bold;
    margin-top: 10px;
}

a {
    color: #3498db;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_doc_comments() {
        let source = r#"
/// This is a documented function
/// It adds two numbers
fn add(a: num, b: num) -> num {
    return a + b;
}

/** This is a variable with documentation */
let x = 42;
"#;

        let doc_comments = DocGenerator::extract_doc_comments(source);
        assert!(doc_comments.contains_key("add"));
        assert_eq!(doc_comments.get("add").unwrap(), "This is a documented function\nIt adds two numbers");
        assert!(doc_comments.contains_key("x"));
    }

    #[test]
    fn test_parse_simple_function() {
        let source = r#"
/// A simple addition function
fn add(a: num, b: num) -> num {
    return a + b;
}
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(source.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let mut doc_gen = DocGenerator::new();
        let result = doc_gen.parse_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(doc_gen.items.len(), 1);
        assert_eq!(doc_gen.items[0].name, "add");
        assert!(doc_gen.items[0].documentation.is_some());
    }

    #[test]
    fn test_extract_name_from_declaration() {
        assert_eq!(DocGenerator::extract_name_from_declaration("fn test() {}"), Some("test".to_string()));
        assert_eq!(DocGenerator::extract_name_from_declaration("teach fn exported() {}"), Some("exported".to_string()));
        assert_eq!(DocGenerator::extract_name_from_declaration("let x = 42;"), Some("x".to_string()));
        assert_eq!(DocGenerator::extract_name_from_declaration("const PI = 3.14;"), Some("PI".to_string()));
        assert_eq!(DocGenerator::extract_name_from_declaration("def Point {"), Some("Point".to_string()));
    }
}

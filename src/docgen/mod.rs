pub mod stdlib;
pub mod terminal;

use crate::parser::{InputStream, Parser, Stmt, Type};
use regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub items: Vec<DocItem>,
    pub source_files: HashMap<PathBuf, String>,
    pub impl_relations: Vec<(String, String)>,
}

impl DocGenerator {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            source_files: HashMap::new(),
            impl_relations: Vec::new(),
        }
    }

    /// Parse a Twang file and extract documentation items
    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

        self.source_files
            .insert(path.to_path_buf(), content.clone());

        let input_stream = InputStream::new(path.to_str().unwrap_or("unknown"), &content);
        let mut parser = Parser::new(input_stream);

        let stmts = parser
            .parse()
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
                    let params_vec: Vec<(String, String)> = params
                        .iter()
                        .map(|(n, t)| (n.clone(), Self::type_to_string(t)))
                        .collect();

                    let signature = format!(
                        "{}{}fn {}({}) -> {}",
                        if *is_exported { "teach " } else { "" },
                        if *is_async { "async " } else { "" },
                        name,
                        params_vec
                            .iter()
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
                    let fields_vec: Vec<(String, String)> = fields
                        .iter()
                        .map(|(n, t)| (n.clone(), Self::type_to_string(t)))
                        .collect();

                    let signature = format!(
                        "def {} {{\n{}\n}}",
                        name,
                        fields_vec
                            .iter()
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
                    let method_names: Vec<String> = methods
                        .iter()
                        .filter_map(|m| match m {
                            crate::parser::TraitMethod::Signature { name, .. } => {
                                Some(name.clone())
                            }
                            crate::parser::TraitMethod::Default { name, .. } => Some(name.clone()),
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
                Stmt::ConstDecl {
                    name, const_type, ..
                } => {
                    let type_str = const_type
                        .as_ref()
                        .map(|t| Self::type_to_string(t))
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
                    let type_str = var_type
                        .as_ref()
                        .map(|t| Self::type_to_string(t))
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
                Stmt::ImplBlock {
                    type_name,
                    trait_name,
                    methods,
                } => {
                    if let Some(trait_name) = trait_name {
                        self.impl_relations
                            .push((trait_name.clone(), type_name.clone()));
                    }
                    // Recursively process methods in impl blocks
                    self.extract_items(methods, source);
                }
                Stmt::AttrStmt { stmt, .. } => {
                    self.extract_items(std::slice::from_ref(stmt), source);
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
                    let doc_text = line
                        .strip_prefix("/**")
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
            if line.starts_with(pattern) {
                if extract_next {
                    let rest = line.strip_prefix(pattern).unwrap();
                    // Extract identifier (alphanumeric + underscore)
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !name.is_empty() {
                        return Some(name);
                    }
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
                    type_args
                        .iter()
                        .map(|t| Self::type_to_string(t))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Type::Function {
                params,
                return_type,
            } => {
                format!(
                    "({}) -> {}",
                    params
                        .iter()
                        .map(|t| Self::type_to_string(t))
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
                if let DocItemKind::Struct {
                    implemented_traits, ..
                } = &mut item.kind
                {
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
        fs::write(&css_path, css).map_err(|e| format!("Failed to write style.css: {}", e))?;

        Ok(())
    }

    fn generate_index_html(&self, package_name: &str) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str(&format!(
            "    <title>{} - loft Documentation</title>\n",
            package_name
        ));
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("</head>\n<body>\n");

        // Sidebar
        html.push_str("    <div class=\"sidebar\">\n");
        html.push_str(&format!(
            "        <h2><a href=\"index.html\">{}</a></h2>\n",
            package_name
        ));

        // Group items by kind
        let functions: Vec<_> = self
            .items
            .iter()
            .filter(|item| matches!(item.kind, DocItemKind::Function { .. }))
            .collect();
        let structs: Vec<_> = self
            .items
            .iter()
            .filter(|item| matches!(item.kind, DocItemKind::Struct { .. }))
            .collect();
        let traits: Vec<_> = self
            .items
            .iter()
            .filter(|item| matches!(item.kind, DocItemKind::Trait { .. }))
            .collect();
        let constants: Vec<_> = self
            .items
            .iter()
            .filter(|item| matches!(item.kind, DocItemKind::Constant { .. }))
            .collect();

        if !functions.is_empty() {
            html.push_str("        <div class=\"nav-section\">\n");
            html.push_str("            <h3>Functions</h3>\n");
            html.push_str("            <ul>\n");
            for item in &functions {
                html.push_str(&format!(
                    "                <li><a href=\"#{}\">{}</a></li>\n",
                    item.name, item.name
                ));
            }
            html.push_str("            </ul>\n");
            html.push_str("        </div>\n");
        }

        if !structs.is_empty() {
            html.push_str("        <div class=\"nav-section\">\n");
            html.push_str("            <h3>Structs</h3>\n");
            html.push_str("            <ul>\n");
            for item in &structs {
                html.push_str(&format!(
                    "                <li><a href=\"#{}\">{}</a></li>\n",
                    item.name, item.name
                ));
            }
            html.push_str("            </ul>\n");
            html.push_str("        </div>\n");
        }

        if !traits.is_empty() {
            html.push_str("        <div class=\"nav-section\">\n");
            html.push_str("            <h3>Traits</h3>\n");
            html.push_str("            <ul>\n");
            for item in &traits {
                html.push_str(&format!(
                    "                <li><a href=\"#{}\">{}</a></li>\n",
                    item.name, item.name
                ));
            }
            html.push_str("            </ul>\n");
            html.push_str("        </div>\n");
        }

        html.push_str("    </div>\n"); // end sidebar

        html.push_str("    <div class=\"content\">\n");
        html.push_str(&format!(
            "        <div class=\"breadcrumb\"><a href=\"index.html\">packages</a> / <span>{}</span></div>\n",
            package_name
        ));
        html.push_str(&format!("        <h1>{} Documentation</h1>\n", package_name));

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
        html.push_str("        <div class=\"method-item\">\n");

        let is_exported = match &item.kind {
            DocItemKind::Function { is_exported, .. } => *is_exported,
            _ => false,
        };

        html.push_str(&format!(
            "            <h3 id=\"{}\">{}{}</h3>\n",
            item.name,
            item.name,
            if is_exported { " (pub)" } else { "" }
        ));

        // Generate signature with links
        let signature_html = match &item.kind {
            DocItemKind::Function {
                params,
                return_type,
                is_async,
                ..
            } => {
                let params_html = params
                    .iter()
                    .map(|(name, ty)| format!("{}: {}", name, self.type_to_html_string(ty)))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!(
                    "{}fn {}({}) -> {}",
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
            for line in doc.lines() {
                html.push_str("            <p>");
                html.push_str(&Self::escape_html(line));
                html.push_str("</p>\n");
            }
        }

        // Add details based on kind
        match &item.kind {
            DocItemKind::Function {
                return_type, ..
            } => {
                html.push_str(&format!(
                    "            <p><strong>Returns:</strong> <code>{}</code></p>\n",
                    self.type_to_html_string(return_type)
                ));
            }
            DocItemKind::Struct {
                fields,
                implemented_traits,
            } => {
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
            DocItemKind::Trait {
                methods,
                implementors,
            } => {
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

        // Primitives from stdlib
        let primitives = [
            ("str", "/d/std/string.html"),
            ("num", "/d/std/num.html"),
            ("bool", "/d/std/bool.html"),
            ("void", "/d/std/void.html"),
            ("Array", "/d/std/array.html"),
        ];

        for (i, (prim, link)) in primitives.iter().enumerate() {
            let pattern = format!(r"\b{}\b", regex::escape(prim));
            if let Ok(re) = regex::Regex::new(&pattern) {
                if re.is_match(&html) {
                    let placeholder = format!("__PRIM_{}__", i);
                    html = re.replace_all(&html, placeholder.as_str()).to_string();
                    replacements.push((placeholder, format!("<a href=\"{}\">{}</a>", link, prim)));
                }
            }
        }

        // Sort items by name length descending to avoid partial replacements
        let mut item_names: Vec<String> = self.items.iter().map(|i| i.name.clone()).collect();
        item_names.sort_by(|a, b| b.len().cmp(&a.len()));

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
        r#":root {
    --color-bio-cream: #fdfcf0;
    --color-bio-black: #1a1a1a;
    --color-bio-green: #64992f;
    --color-bio-green-light: #4a7c43;
    --color-bio-offwhite: #f5f5f5;
    --color-bio-gold: #d4a017;
    --color-border: #e5e7eb;
}

.docs-root * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

.docs-root {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    line-height: 1.6;
    background-color: var(--color-bio-cream);
    color: var(--color-bio-black);
    display: flex;
    min-height: 100vh;
}

.sidebar {
    width: 260px;
    background-color: #fff;
    border-right: 1px solid var(--color-border);
    padding: 20px;
    position: sticky;
    top: 0;
    height: 100vh;
    overflow-y: auto;
}

.sidebar h2 {
    font-size: 22px;
    margin-bottom: 20px;
    color: var(--color-bio-black);
}

.sidebar h2 a {
    color: var(--color-bio-black);
    text-decoration: none;
}

.sidebar h2 a:hover {
    color: var(--color-bio-green);
}

.nav-section {
    margin-bottom: 24px;
}

.nav-section h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #6b7280;
    margin-bottom: 8px;
}

.nav-section ul {
    list-style: none;
}

.nav-section li {
    margin-bottom: 4px;
}

.nav-section a {
    color: var(--color-bio-black);
    text-decoration: none;
    font-size: 14px;
    display: block;
    padding: 4px 0;
}

.nav-section a:hover {
    color: var(--color-bio-green);
}

.content {
    flex: 1;
    padding: 40px 60px;
    max-width: 1000px;
}

.breadcrumb {
    font-size: 14px;
    color: #6b7280;
    margin-bottom: 16px;
}

.breadcrumb a {
    color: #6b7280;
    text-decoration: none;
}

.breadcrumb a:hover {
    color: var(--color-bio-green);
}

h1 {
    font-size: 36px;
    margin-bottom: 16px;
    color: var(--color-bio-black);
}

h2 {
    font-size: 24px;
    margin-top: 40px;
    margin-bottom: 20px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--color-border);
    color: var(--color-bio-black);
}

h3 {
    font-size: 18px;
    margin-bottom: 12px;
    color: var(--color-bio-black);
}

.description {
    font-size: 18px;
    color: #4b5563;
    margin-bottom: 32px;
}

.item-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
    margin-top: 16px;
}

.item-card {
    background-color: #fff;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 16px;
    transition: box-shadow 0.2s, border-color 0.2s;
}

.item-card:hover {
    border-color: var(--color-bio-green);
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

.item-card a {
    color: var(--color-bio-green);
    text-decoration: none;
}

.item-card a:hover {
    text-decoration: underline;
}

.method-item {
    background-color: #fff;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 20px;
    margin-bottom: 16px;
}

.signature {
    background-color: var(--color-bio-offwhite);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 12px 16px;
    margin: 12px 0;
    overflow-x: auto;
}

.signature code {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    font-size: 14px;
    color: var(--color-bio-black);
}

.example {
    background-color: var(--color-bio-offwhite);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 12px 16px;
    margin: 12px 0;
    overflow-x: auto;
}

.example code {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    font-size: 13px;
    color: var(--color-bio-black);
    white-space: pre;
}

code {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    font-size: 13px;
    background-color: var(--color-bio-offwhite);
    padding: 2px 6px;
    border-radius: 3px;
    color: var(--color-bio-green-light);
}

p {
    margin-bottom: 12px;
}

strong {
    font-weight: 600;
}

.exported {
    color: var(--color-bio-green);
    font-weight: 600;
    font-size: 12px;
    text-transform: uppercase;
    margin-top: 10px;
}

@media (max-width: 768px) {
    .sidebar {
        display: none;
    }
    
    .content {
        margin-left: 0;
        padding: 20px;
    }
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
        assert_eq!(
            doc_comments.get("add").unwrap(),
            "This is a documented function\nIt adds two numbers"
        );
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
        assert_eq!(
            DocGenerator::extract_name_from_declaration("fn test() {}"),
            Some("test".to_string())
        );
        assert_eq!(
            DocGenerator::extract_name_from_declaration("teach fn exported() {}"),
            Some("exported".to_string())
        );
        assert_eq!(
            DocGenerator::extract_name_from_declaration("let x = 42;"),
            Some("x".to_string())
        );
        assert_eq!(
            DocGenerator::extract_name_from_declaration("const PI = 3.14;"),
            Some("PI".to_string())
        );
        assert_eq!(
            DocGenerator::extract_name_from_declaration("def Point {"),
            Some("Point".to_string())
        );
    }
}

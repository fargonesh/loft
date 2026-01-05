use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdlibTypes {
    pub builtins: HashMap<String, BuiltinDef>,
    pub string_methods: HashMap<String, MethodDef>,
    pub array_methods: HashMap<String, MethodDef>,
    pub types: HashMap<String, TypeDef>,
    pub traits: HashMap<String, TraitDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinDef {
    pub kind: String,
    pub documentation: String,
    #[serde(default)]
    pub methods: HashMap<String, MethodDef>,
    #[serde(default)]
    pub constants: HashMap<String, ConstantDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDef {
    pub params: Vec<String>,
    pub return_type: String,
    pub documentation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDef {
    #[serde(rename = "type")]
    pub const_type: String,
    pub documentation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub kind: String,
    pub documentation: String,
    #[serde(default)]
    pub fields: HashMap<String, FieldDef>,
    #[serde(default)]
    pub methods: HashMap<String, MethodDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    #[serde(rename = "type")]
    pub field_type: String,
    pub documentation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitDef {
    pub documentation: String,
    pub methods: HashMap<String, MethodDef>,
}

pub struct StdlibDocGenerator {
    stdlib: StdlibTypes,
}

impl StdlibDocGenerator {
    pub fn new(stdlib_json: &str) -> Result<Self, String> {
        let stdlib = serde_json::from_str::<StdlibTypes>(stdlib_json)
            .map_err(|e| format!("Failed to parse stdlib_types.json: {}", e))?;
        Ok(Self { stdlib })
    }

    pub fn from_source(root_dir: &Path) -> Result<Self, String> {
        let stdlib = crate::docgen::scanner::StdlibScanner::scan(root_dir);
        Ok(Self { stdlib })
    }

    pub fn merge_source(&mut self, root_dir: &Path) {
        let scanned = crate::docgen::scanner::StdlibScanner::scan(root_dir);
        
        // Merge builtins
        for (name, builtin) in scanned.builtins {
            if let Some(existing) = self.stdlib.builtins.get_mut(&name) {
                existing.methods.extend(builtin.methods);
                existing.constants.extend(builtin.constants);
            } else {
                self.stdlib.builtins.insert(name, builtin);
            }
        }

        // Merge methods
        self.stdlib.string_methods.extend(scanned.string_methods);
        self.stdlib.array_methods.extend(scanned.array_methods);
        
        // Merge types and traits
        self.stdlib.types.extend(scanned.types);
        self.stdlib.traits.extend(scanned.traits);
    }

    pub fn generate_html(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Generate main index page
        let index_html = self.generate_index_html();
        fs::write(output_dir.join("index.html"), index_html)
            .map_err(|e| format!("Failed to write index.html: {}", e))?;

        // Generate individual pages for each builtin
        let mut processed_array = false;
        for (name, builtin) in &self.stdlib.builtins {
            if name == "array" {
                let array_html = self.generate_merged_array_html(builtin, &self.stdlib.array_methods);
                fs::write(output_dir.join("array.html"), array_html)
                    .map_err(|e| format!("Failed to write array.html: {}", e))?;
                processed_array = true;
                continue;
            }
            let builtin_html = self.generate_builtin_html(name, builtin);
            fs::write(output_dir.join(format!("{}.html", name)), builtin_html)
                .map_err(|e| format!("Failed to write {}.html: {}", name, e))?;
        }

        // Generate pages for string and array methods
        let string_html = self.generate_methods_html("str", &self.stdlib.string_methods, "String primitive type with utility methods for text manipulation.");
        fs::write(output_dir.join("string.html"), string_html)
            .map_err(|e| format!("Failed to write string.html: {}", e))?;

        if !processed_array {
            let array_html = self.generate_methods_html("Array", &self.stdlib.array_methods, "Array primitive type with utility methods for collection manipulation.");
            fs::write(output_dir.join("array.html"), array_html)
                .map_err(|e| format!("Failed to write array.html: {}", e))?;
        }

        // Generate pages for other primitives (empty methods for now)
        let empty_methods = HashMap::new();
        let num_html = self.generate_methods_html("num", &empty_methods, "Numeric primitive type.");
        fs::write(output_dir.join("num.html"), num_html)
            .map_err(|e| format!("Failed to write num.html: {}", e))?;

        let bool_html = self.generate_methods_html("bool", &empty_methods, "Boolean primitive type.");
        fs::write(output_dir.join("bool.html"), bool_html)
            .map_err(|e| format!("Failed to write bool.html: {}", e))?;

        let void_html = self.generate_methods_html("void", &empty_methods, "Void primitive type.");
        fs::write(output_dir.join("void.html"), void_html)
            .map_err(|e| format!("Failed to write void.html: {}", e))?;

        // Generate pages for types
        for (name, type_def) in &self.stdlib.types {
            let type_html = self.generate_type_html(name, type_def);
            fs::write(output_dir.join(format!("type-{}.html", name)), type_html)
                .map_err(|e| format!("Failed to write type-{}.html: {}", name, e))?;
        }

        // Generate pages for traits
        for (name, trait_def) in &self.stdlib.traits {
            let trait_html = self.generate_trait_html(name, trait_def);
            fs::write(output_dir.join(format!("trait-{}.html", name)), trait_html)
                .map_err(|e| format!("Failed to write trait-{}.html: {}", name, e))?;
        }

        // Generate CSS
        let css = self.generate_css();
        fs::write(output_dir.join("style.css"), css)
            .map_err(|e| format!("Failed to write style.css: {}", e))?;

        // Generate Search Index
        let search_index = self.generate_search_index();
        fs::write(output_dir.join("search_index.js"), search_index)
            .map_err(|e| format!("Failed to write search_index.js: {}", e))?;

        // Generate Search Script
        let search_js = self.generate_search_js();
        fs::write(output_dir.join("search.js"), search_js)
            .map_err(|e| format!("Failed to write search.js: {}", e))?;

        Ok(())
    }

    fn generate_search_index(&self) -> String {
        
        // Primitives
        let mut items = vec![
            "{name: \"str\", type: \"primitive\", url: \"string.html\", doc: \"String primitive type\"}".to_string(),
            "{name: \"Array\", type: \"primitive\", url: \"array.html\", doc: \"Array primitive type\"}".to_string(),
            "{name: \"num\", type: \"primitive\", url: \"num.html\", doc: \"Numeric primitive type\"}".to_string(),
            "{name: \"bool\", type: \"primitive\", url: \"bool.html\", doc: \"Boolean primitive type\"}".to_string(),
            "{name: \"void\", type: \"primitive\", url: \"void.html\", doc: \"Void primitive type\"}".to_string(),
        ];

        // Builtins
        for (name, builtin) in &self.stdlib.builtins {
            if name == "array" || name == "string" { continue; }
            items.push(format!("{{name: \"{}\", type: \"builtin\", url: \"{}.html\", doc: {:?}}}", 
                name, name, builtin.documentation));
            
            for (method_name, method) in &builtin.methods {
                items.push(format!("{{name: \"{}.{}\", type: \"method\", url: \"{}.html#{}\", doc: {:?}}}", 
                    name, method_name, name, method_name, method.documentation));
            }
        }

        // Types
        for (name, type_def) in &self.stdlib.types {
            items.push(format!("{{name: \"{}\", type: \"type\", url: \"type-{}.html\", doc: {:?}}}", 
                name, name, type_def.documentation));
            
            for (method_name, method) in &type_def.methods {
                items.push(format!("{{name: \"{}.{}\", type: \"method\", url: \"type-{}.html#{}\", doc: {:?}}}", 
                    name, method_name, name, method_name, method.documentation));
            }
        }

        // Traits
        for (name, trait_def) in &self.stdlib.traits {
            items.push(format!("{{name: \"{}\", type: \"trait\", url: \"trait-{}.html\", doc: {:?}}}", 
                name, name, trait_def.documentation));
            
            for (method_name, method) in &trait_def.methods {
                items.push(format!("{{name: \"{}.{}\", type: \"method\", url: \"trait-{}.html#{}\", doc: {:?}}}", 
                    name, method_name, name, method_name, method.documentation));
            }
        }

        // String methods
        for (method_name, method) in &self.stdlib.string_methods {
            items.push(format!("{{name: \"str.{}\", type: \"method\", url: \"string.html#{}\", doc: {:?}}}", 
                method_name, method_name, method.documentation));
        }

        // Array methods
        for (method_name, method) in &self.stdlib.array_methods {
            items.push(format!("{{name: \"Array.{}\", type: \"method\", url: \"array.html#{}\", doc: {:?}}}", 
                method_name, method_name, method.documentation));
        }

        format!("const SEARCH_INDEX = [{}];", items.join(","))
    }

    fn generate_search_js(&self) -> String {
        r#"
document.addEventListener('DOMContentLoaded', () => {
    const searchInput = document.getElementById('doc-search');
    const sidebar = document.querySelector('.sidebar');
    
    if (!searchInput || !sidebar) return;

    // Create results container
    const resultsContainer = document.createElement('div');
    resultsContainer.id = 'search-results';
    resultsContainer.style.display = 'none';
    sidebar.insertBefore(resultsContainer, sidebar.children[2]); // Insert after title and search input

    searchInput.addEventListener('input', (e) => {
        const query = e.target.value.toLowerCase();
        
        if (query.length < 2) {
            resultsContainer.style.display = 'none';
            document.querySelectorAll('.nav-section').forEach(el => el.style.display = 'block');
            return;
        }

        // Hide normal nav
        document.querySelectorAll('.nav-section').forEach(el => el.style.display = 'none');
        resultsContainer.style.display = 'block';
        resultsContainer.innerHTML = '';

        const results = SEARCH_INDEX.filter(item => 
            item.name.toLowerCase().includes(query) || 
            (item.doc && item.doc.toLowerCase().includes(query))
        ).slice(0, 20);

        if (results.length === 0) {
            resultsContainer.innerHTML = '<div class="no-results">No results found</div>';
            return;
        }

        const ul = document.createElement('ul');
        results.forEach(item => {
            const li = document.createElement('li');
            const a = document.createElement('a');
            a.href = item.url;
            a.innerHTML = `<span class="result-name">${item.name}</span> <span class="result-type">${item.type}</span>`;
            li.appendChild(a);
            ul.appendChild(li);
        });
        resultsContainer.appendChild(ul);
    });
});
"#.to_string()
    }

    fn generate_index_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("    <title>loft Standard Library Documentation</title>\n");
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("    <script src=\"search_index.js\"></script>\n");
        html.push_str("    <script src=\"search.js\"></script>\n");
        html.push_str("</head>\n<body>\n");
        html.push_str("    <div class=\"sidebar\">\n");
        html.push_str("        <h2>loft stdlib</h2>\n");
        html.push_str("        <div class=\"search-container\">\n");
        html.push_str("            <input type=\"text\" id=\"doc-search\" placeholder=\"Search docs...\">\n");
        html.push_str("        </div>\n");
        html.push_str("        <div class=\"nav-section\">\n");
        html.push_str("            <h3>Primitives</h3>\n");
        html.push_str("            <ul>\n");
        html.push_str("                <li><a href=\"string.html\">str</a></li>\n");
        html.push_str("                <li><a href=\"array.html\">Array</a></li>\n");
        html.push_str("                <li><a href=\"num.html\">num</a></li>\n");
        html.push_str("                <li><a href=\"bool.html\">bool</a></li>\n");
        html.push_str("                <li><a href=\"void.html\">void</a></li>\n");
        html.push_str("            </ul>\n");
        html.push_str("        </div>\n");

        html.push_str("        <div class=\"nav-section\">\n");
        html.push_str("            <h3>Builtins</h3>\n");
        html.push_str("            <ul>\n");
        for name in self.stdlib.builtins.keys() {
            if name == "array" || name == "string" { continue; }
            html.push_str(&format!("                <li><a href=\"{}.html\">{}</a></li>\n", name, name));
        }
        html.push_str("            </ul>\n");
        html.push_str("        </div>\n");

        if !self.stdlib.types.is_empty() {
            html.push_str("        <div class=\"nav-section\">\n");
            html.push_str("            <h3>Types</h3>\n");
            html.push_str("            <ul>\n");
            for name in self.stdlib.types.keys() {
                html.push_str(&format!("                <li><a href=\"type-{}.html\">{}</a></li>\n", name, name));
            }
            html.push_str("            </ul>\n");
            html.push_str("        </div>\n");
        }

        if !self.stdlib.traits.is_empty() {
            html.push_str("        <div class=\"nav-section\">\n");
            html.push_str("            <h3>Traits</h3>\n");
            html.push_str("            <ul>\n");
            for name in self.stdlib.traits.keys() {
                html.push_str(&format!("                <li><a href=\"trait-{}.html\">{}</a></li>\n", name, name));
            }
            html.push_str("            </ul>\n");
            html.push_str("        </div>\n");
        }

        html.push_str("    </div>\n");
        html.push_str("    <div class=\"content\">\n");
        html.push_str("        <h1>loft Standard Library</h1>\n");
        html.push_str("        <p class=\"intro\">Welcome to the loft standard library documentation. The standard library provides essential functionality for building loft applications.</p>\n");
        
        html.push_str("        <h2>Primitives</h2>\n");
        html.push_str("        <p>Core primitive types with built-in methods:</p>\n");
        html.push_str("        <div class=\"item-grid\">\n");
        html.push_str("            <div class=\"item-card\"><a href=\"string.html\"><strong>str</strong></a><br>Text manipulation and utilities</div>\n");
        html.push_str("            <div class=\"item-card\"><a href=\"array.html\"><strong>Array</strong></a><br>Collection operations and transformations</div>\n");
        html.push_str("            <div class=\"item-card\"><a href=\"num.html\"><strong>num</strong></a><br>Numeric type (decimal)</div>\n");
        html.push_str("            <div class=\"item-card\"><a href=\"bool.html\"><strong>bool</strong></a><br>Boolean type (true/false)</div>\n");
        html.push_str("            <div class=\"item-card\"><a href=\"void.html\"><strong>void</strong></a><br>Unit type representing no value</div>\n");
        html.push_str("        </div>\n");

        html.push_str("        <h2>Builtin Modules</h2>\n");
        html.push_str("        <p>Standard builtin modules for common operations:</p>\n");
        html.push_str("        <div class=\"item-grid\">\n");
        for (name, builtin) in &self.stdlib.builtins {
            if name == "array" || name == "string" { continue; }
            html.push_str(&format!(
                "            <div class=\"item-card\"><a href=\"{}.html\"><strong>{}</strong></a><br>{}</div>\n",
                name, name, Self::escape_html(&builtin.documentation)
            ));
        }
        html.push_str("        </div>\n");

        html.push_str("    </div>\n");
        html.push_str("</body>\n</html>\n");
        html
    }

    fn generate_merged_array_html(&self, builtin: &BuiltinDef, instance_methods: &HashMap<String, MethodDef>) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("    <title>Array - loft stdlib</title>\n");
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("    <script src=\"search_index.js\"></script>\n");
        html.push_str("    <script src=\"search.js\"></script>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str(&self.generate_sidebar());
        
        html.push_str("    <div class=\"content\">\n");
        html.push_str("        <div class=\"breadcrumb\"><a href=\"index.html\">stdlib</a> / <span>Array</span></div>\n");
        html.push_str("        <h1>Array</h1>\n");
        html.push_str(&format!("        <p class=\"description\">{}</p>\n", Self::escape_html(&builtin.documentation)));

        // Static Methods (from builtin)
        if !builtin.methods.is_empty() {
            html.push_str("        <h2>Static Methods</h2>\n");
            for (method_name, method) in &builtin.methods {
                html.push_str("        <div class=\"method-item\">\n");
                html.push_str(&format!("            <h3 id=\"static-{}\">{}</h3>\n", method_name, method_name));
                html.push_str(&format!("            <pre class=\"signature\"><code>Array.{}({}) -> {}</code></pre>\n", 
                    method_name,
                    self.format_params(&method.params),
                    self.link_type(&method.return_type)
                ));
                html.push_str(&format!("            <p><strong>Returns:</strong> <code>{}</code></p>\n", 
                    self.link_type(&method.return_type)));
                html.push_str(&format!("            <p>{}</p>\n", Self::escape_html(&method.documentation)));
                html.push_str("        </div>\n");
            }
        }

        // Instance Methods (from array_methods)
        if !instance_methods.is_empty() {
            html.push_str("        <h2>Instance Methods</h2>\n");
            for (method_name, method) in instance_methods {
                html.push_str("        <div class=\"method-item\">\n");
                html.push_str(&format!("            <h3 id=\"instance-{}\">{}</h3>\n", method_name, method_name));
                html.push_str(&format!("            <pre class=\"signature\"><code>array.{}({}) -> {}</code></pre>\n", 
                    method_name,
                    self.format_params(&method.params),
                    self.link_type(&method.return_type)
                ));
                html.push_str(&format!("            <p><strong>Returns:</strong> <code>{}</code></p>\n", 
                    self.link_type(&method.return_type)));
                html.push_str(&format!("            <p>{}</p>\n", Self::escape_html(&method.documentation)));
                html.push_str("        </div>\n");
            }
        }

        html.push_str("    </div>\n");
        html.push_str("</body>\n</html>\n");
        html
    }

    fn generate_builtin_html(&self, name: &str, builtin: &BuiltinDef) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("    <title>{} - loft stdlib</title>\n", name));
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("    <script src=\"search_index.js\"></script>\n");
        html.push_str("    <script src=\"search.js\"></script>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str(&self.generate_sidebar());
        
        html.push_str("    <div class=\"content\">\n");
        html.push_str("        <div class=\"breadcrumb\"><a href=\"index.html\">stdlib</a> / <span>");
        html.push_str(name);
        html.push_str("</span></div>\n");
        html.push_str(&format!("        <h1>{}</h1>\n", name));
        html.push_str(&format!("        <p class=\"description\">{}</p>\n", Self::escape_html(&builtin.documentation)));

        // Constants
        if !builtin.constants.is_empty() {
            html.push_str("        <h2>Constants</h2>\n");
            for (const_name, constant) in &builtin.constants {
                html.push_str("        <div class=\"method-item\">\n");
                html.push_str(&format!("            <h3 id=\"{}\">{}</h3>\n", const_name, const_name));
                html.push_str(&format!("            <pre class=\"signature\"><code>{}.{}: {}</code></pre>\n", 
                    name, const_name, self.link_type(&constant.const_type)));
                html.push_str(&format!("            <p>{}</p>\n", Self::escape_html(&constant.documentation)));
                html.push_str("        </div>\n");
            }
        }

        // Methods
        if !builtin.methods.is_empty() {
            html.push_str("        <h2>Methods</h2>\n");
            for (method_name, method) in &builtin.methods {
                html.push_str("        <div class=\"method-item\">\n");
                html.push_str(&format!("            <h3 id=\"{}\">{}</h3>\n", method_name, method_name));
                html.push_str(&format!("            <pre class=\"signature\"><code>{}.{}({}) -> {}</code></pre>\n", 
                    name, 
                    method_name,
                    self.format_params(&method.params),
                    self.link_type(&method.return_type)
                ));
                html.push_str(&format!("            <p><strong>Returns:</strong> <code>{}</code></p>\n", 
                    self.link_type(&method.return_type)));
                html.push_str(&format!("            <p>{}</p>\n", Self::escape_html(&method.documentation)));
                html.push_str("        </div>\n");
            }
        }

        html.push_str("    </div>\n");
        html.push_str("</body>\n</html>\n");
        html
    }

    fn generate_methods_html(&self, title: &str, methods: &HashMap<String, MethodDef>, description: &str) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("    <title>{} - loft stdlib</title>\n", title));
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("    <script src=\"search_index.js\"></script>\n");
        html.push_str("    <script src=\"search.js\"></script>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str(&self.generate_sidebar());
        
        html.push_str("    <div class=\"content\">\n");
        html.push_str("        <div class=\"breadcrumb\"><a href=\"index.html\">stdlib</a> / <span>");
        html.push_str(title);
        html.push_str("</span></div>\n");
        html.push_str(&format!("        <h1>{}</h1>\n", title));
        html.push_str(&format!("        <p class=\"description\">{}</p>\n", Self::escape_html(description)));

        html.push_str("        <h2>Methods</h2>\n");
        for (method_name, method) in methods {
            html.push_str("        <div class=\"method-item\">\n");
            html.push_str(&format!("            <h3 id=\"{}\">{}</h3>\n", method_name, method_name));
            html.push_str(&format!("            <pre class=\"signature\"><code>value.{}({}) -> {}</code></pre>\n", 
                method_name,
                self.format_params(&method.params),
                self.link_type(&method.return_type)
            ));
            html.push_str(&format!("            <p><strong>Returns:</strong> <code>{}</code></p>\n", 
                self.link_type(&method.return_type)));
            html.push_str(&format!("            <p>{}</p>\n", Self::escape_html(&method.documentation)));
            
            // Add usage examples for common methods
            if let Some(example) = self.get_usage_example(title, method_name) {
                html.push_str("            <h4>Example</h4>\n");
                html.push_str(&format!("            <pre class=\"example\"><code>{}</code></pre>\n", Self::escape_html(example)));
            }
            
            html.push_str("        </div>\n");
        }

        html.push_str("    </div>\n");
        html.push_str("</body>\n</html>\n");
        html
    }

    fn generate_type_html(&self, name: &str, type_def: &TypeDef) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("    <title>{} - loft stdlib</title>\n", name));
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("    <script src=\"search_index.js\"></script>\n");
        html.push_str("    <script src=\"search.js\"></script>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str(&self.generate_sidebar());
        
        html.push_str("    <div class=\"content\">\n");
        html.push_str("        <div class=\"breadcrumb\"><a href=\"index.html\">stdlib</a> / <span>");
        html.push_str(name);
        html.push_str("</span></div>\n");
        html.push_str(&format!("        <h1>Type: {}</h1>\n", name));
        html.push_str(&format!("        <p class=\"description\">{}</p>\n", Self::escape_html(&type_def.documentation)));

        if !type_def.fields.is_empty() {
            html.push_str("        <h2>Fields</h2>\n");
            for (field_name, field) in &type_def.fields {
                html.push_str("        <div class=\"method-item\">\n");
                html.push_str(&format!("            <h3 id=\"{}\">{}</h3>\n", field_name, field_name));
                html.push_str(&format!("            <pre class=\"signature\"><code>{}: {}</code></pre>\n", 
                    field_name, self.link_type(&field.field_type)));
                html.push_str(&format!("            <p>{}</p>\n", Self::escape_html(&field.documentation)));
                html.push_str("        </div>\n");
            }
        }

        if !type_def.methods.is_empty() {
            html.push_str("        <h2>Methods</h2>\n");
            for (method_name, method) in &type_def.methods {
                html.push_str("        <div class=\"method-item\">\n");
                html.push_str(&format!("            <h3 id=\"{}\">{}</h3>\n", method_name, method_name));
                html.push_str(&format!("            <pre class=\"signature\"><code>{}.{}({}) -> {}</code></pre>\n", 
                    name, method_name, self.format_params(&method.params), self.link_type(&method.return_type)));
                html.push_str(&format!("            <p><strong>Returns:</strong> <code>{}</code></p>\n", 
                    self.link_type(&method.return_type)));
                html.push_str(&format!("            <p>{}</p>\n", Self::escape_html(&method.documentation)));
                html.push_str("        </div>\n");
            }
        }

        html.push_str("    </div>\n");
        html.push_str("</body>\n</html>\n");
        html
    }

    fn generate_trait_html(&self, name: &str, trait_def: &TraitDef) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("    <title>{} - loft stdlib</title>\n", name));
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("    <script src=\"search_index.js\"></script>\n");
        html.push_str("    <script src=\"search.js\"></script>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str(&self.generate_sidebar());
        
        html.push_str("    <div class=\"content\">\n");
        html.push_str("        <div class=\"breadcrumb\"><a href=\"index.html\">stdlib</a> / <span>");
        html.push_str(name);
        html.push_str("</span></div>\n");
        html.push_str(&format!("        <h1>Trait: {}</h1>\n", name));
        html.push_str(&format!("        <p class=\"description\">{}</p>\n", Self::escape_html(&trait_def.documentation)));

        html.push_str("        <h2>Required Methods</h2>\n");
        for (method_name, method) in &trait_def.methods {
            html.push_str("        <div class=\"method-item\">\n");
            html.push_str(&format!("            <h3 id=\"{}\">{}</h3>\n", method_name, method_name));
            html.push_str(&format!("            <pre class=\"signature\"><code>fn {}({}) -> {}</code></pre>\n", 
                method_name, self.format_params(&method.params), self.link_type(&method.return_type)));
            html.push_str(&format!("            <p><strong>Returns:</strong> <code>{}</code></p>\n", 
                self.link_type(&method.return_type)));
            html.push_str(&format!("            <p>{}</p>\n", Self::escape_html(&method.documentation)));
            html.push_str("        </div>\n");
        }

        html.push_str("    </div>\n");
        html.push_str("</body>\n</html>\n");
        html
    }

    fn format_params(&self, params: &[String]) -> String {
        params.iter().map(|p| {
            if let Some((name, type_part)) = p.split_once(':') {
                format!("{}: {}", name, self.link_type(type_part.trim()))
            } else {
                self.link_type(p)
            }
        }).collect::<Vec<_>>().join(", ")
    }

    fn generate_sidebar(&self) -> String {
        let mut html = String::new();
        html.push_str("    <div class=\"sidebar\">\n");
        html.push_str("        <h2><a href=\"index.html\">loft stdlib</a></h2>\n");
        html.push_str("        <div class=\"search-container\">\n");
        html.push_str("            <input type=\"text\" id=\"doc-search\" placeholder=\"Search docs...\">\n");
        html.push_str("        </div>\n");
        
        html.push_str("        <div class=\"nav-section\">\n");
        html.push_str("            <h3>Primitives</h3>\n");
        html.push_str("            <ul>\n");
        html.push_str("                <li><a href=\"string.html\">str</a></li>\n");
        html.push_str("                <li><a href=\"array.html\">Array</a></li>\n");
        html.push_str("                <li><a href=\"num.html\">num</a></li>\n");
        html.push_str("                <li><a href=\"bool.html\">bool</a></li>\n");
        html.push_str("                <li><a href=\"void.html\">void</a></li>\n");
        html.push_str("            </ul>\n");
        html.push_str("        </div>\n");

        html.push_str("        <div class=\"nav-section\">\n");
        html.push_str("            <h3>Builtins</h3>\n");
        html.push_str("            <ul>\n");
        for name in self.stdlib.builtins.keys() {
            if name == "array" || name == "string" { continue; }
            html.push_str(&format!("                <li><a href=\"{}.html\">{}</a></li>\n", name, name));
        }
        html.push_str("            </ul>\n");
        html.push_str("        </div>\n");

        if !self.stdlib.types.is_empty() {
            html.push_str("        <div class=\"nav-section\">\n");
            html.push_str("            <h3>Types</h3>\n");
            html.push_str("            <ul>\n");
            for name in self.stdlib.types.keys() {
                html.push_str(&format!("                <li><a href=\"type-{}.html\">{}</a></li>\n", name, name));
            }
            html.push_str("            </ul>\n");
            html.push_str("        </div>\n");
        }

        if !self.stdlib.traits.is_empty() {
            html.push_str("        <div class=\"nav-section\">\n");
            html.push_str("            <h3>Traits</h3>\n");
            html.push_str("            <ul>\n");
            for name in self.stdlib.traits.keys() {
                html.push_str(&format!("                <li><a href=\"trait-{}.html\">{}</a></li>\n", name, name));
            }
            html.push_str("            </ul>\n");
            html.push_str("        </div>\n");
        }

        html.push_str("    </div>\n");
        html
    }

    fn get_usage_example(&self, title: &str, method_name: &str) -> Option<&'static str> {
        match (title, method_name) {
            ("str", "split") => Some("let words = \"hello world\".split(\" \");\nterm.println(words); // [\"hello\", \"world\"]"),
            ("str", "to_upper") => Some("let loud = \"hello\".to_upper();\nterm.println(loud); // \"HELLO\""),
            ("str", "to_lower") => Some("let quiet = \"HELLO\".to_lower();\nterm.println(quiet); // \"hello\""),
            ("str", "trim") => Some("let clean = \"  hello  \".trim();\nterm.println(clean); // \"hello\""),
            ("str", "replace") => Some("let fixed = \"hello world\".replace(\"world\", \"loft\");\nterm.println(fixed); // \"hello loft\""),
            ("str", "length") => Some("let len = \"hello\".length();\nterm.println(len); // 5"),
            ("Array", "push") => Some("let arr = [1, 2, 3];\nlet new_arr = arr.push(4);\nterm.println(new_arr); // [1, 2, 3, 4]"),
            ("Array", "map") => Some("let numbers = [1, 2, 3];\nlet doubled = numbers.map(x => x * 2);\nterm.println(doubled); // [2, 4, 6]"),
            ("Array", "filter") => Some("let numbers = [1, 2, 3, 4, 5];\nlet evens = numbers.filter(x => x % 2 == 0);\nterm.println(evens); // [2, 4]"),
            ("Array", "length") => Some("let len = [1, 2, 3].length();\nterm.println(len); // 3"),
            ("Array", "first") => Some("let first = [1, 2, 3].first();\nterm.println(first); // 1"),
            ("Array", "last") => Some("let last = [1, 2, 3].last();\nterm.println(last); // 3"),
            _ => None,
        }
    }

    fn link_type(&self, type_str: &str) -> String {
        let mut html = Self::escape_html(type_str);
        let mut replacements = Vec::new();
        
        // Primitives
        let primitives = [
            ("str", "string.html"), 
            ("num", "num.html"), 
            ("bool", "bool.html"), 
            ("void", "void.html"), 
            ("Array", "array.html")
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

        // Builtins
        for (i, name) in self.stdlib.builtins.keys().enumerate() {
             let pattern = format!(r"\b{}\b", regex::escape(name));
             if let Ok(re) = regex::Regex::new(&pattern) {
                 if re.is_match(&html) {
                     let placeholder = format!("__BUILTIN_{}__", i);
                     html = re.replace_all(&html, placeholder.as_str()).to_string();
                     replacements.push((placeholder, format!("<a href=\"{}.html\">{}</a>", name, name)));
                 }
             }
        }

        // Types
        for (i, name) in self.stdlib.types.keys().enumerate() {
             let pattern = format!(r"\b{}\b", regex::escape(name));
             if let Ok(re) = regex::Regex::new(&pattern) {
                 if re.is_match(&html) {
                     let placeholder = format!("__TYPE_{}__", i);
                     html = re.replace_all(&html, placeholder.as_str()).to_string();
                     replacements.push((placeholder, format!("<a href=\"type-{}.html\">{}</a>", name, name)));
                 }
             }
        }

        // Traits
        for (i, name) in self.stdlib.traits.keys().enumerate() {
             let pattern = format!(r"\b{}\b", regex::escape(name));
             if let Ok(re) = regex::Regex::new(&pattern) {
                 if re.is_match(&html) {
                     let placeholder = format!("__TRAIT_{}__", i);
                     html = re.replace_all(&html, placeholder.as_str()).to_string();
                     replacements.push((placeholder, format!("<a href=\"trait-{}.html\">{}</a>", name, name)));
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
        r#"
:root {
    --color-bio-cream: #fdfcf0;
    --color-bio-black: #1a1a1a;
    --color-bio-green: #64992f;
    --color-bio-green-light: #4a7c43;
    --color-bio-offwhite: #f5f5f5;
    --color-bio-gold: #d4a017;
    --color-border: #e5e7eb;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
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
    position: fixed;
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
    margin-left: 260px;
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

/* Search Styles */
.search-container {
    margin-bottom: 20px;
    position: relative;
}

#search-input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-size: 14px;
    outline: none;
}

#search-input:focus {
    border-color: var(--color-bio-green);
    box-shadow: 0 0 0 2px rgba(100, 153, 47, 0.2);
}

#search-results {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: white;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    box-shadow: 0 4px 6px rgba(0,0,0,0.1);
    max-height: 300px;
    overflow-y: auto;
    z-index: 1000;
    display: none;
}

.search-result-item {
    padding: 8px 12px;
    cursor: pointer;
    border-bottom: 1px solid #f0f0f0;
}

.search-result-item:last-child {
    border-bottom: none;
}

.search-result-item:hover, .search-result-item.selected {
    background-color: #f9fafb;
}

.search-result-item a {
    text-decoration: none;
    color: var(--color-bio-black);
    display: block;
}

.search-result-item .type {
    font-size: 11px;
    color: #6b7280;
    text-transform: uppercase;
    margin-right: 8px;
    display: inline-block;
    width: 50px;
}

.search-result-item .name {
    font-weight: 500;
    color: var(--color-bio-green);
}

.search-result-item .desc {
    font-size: 12px;
    color: #6b7280;
    margin-top: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
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

    #[test]
    fn test_link_type() {
        let json = r#"{
            "builtins": {},
            "types": {},
            "traits": {},
            "string_methods": {},
            "array_methods": {}
        }"#;
        let generator = StdlibDocGenerator::new(json).unwrap();
        
        // Test primitive linking
        let linked = generator.link_type("str");
        assert_eq!(linked, "<a href=\"string.html\">str</a>");

        // Test generic linking
        let linked = generator.link_type("Array<str>");
        assert_eq!(linked, "<a href=\"array.html\">Array</a>&lt;<a href=\"string.html\">str</a>&gt;");
    }
}

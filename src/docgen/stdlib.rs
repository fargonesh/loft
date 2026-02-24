use regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

    pub fn generate_html(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Generate main index page
        let index_html = self.generate_index_html();
        fs::write(output_dir.join("index.html"), index_html)
            .map_err(|e| format!("Failed to write index.html: {}", e))?;

        // Generate individual pages for each builtin
        for (name, builtin) in &self.stdlib.builtins {
            let builtin_html = self.generate_builtin_html(name, builtin);
            fs::write(output_dir.join(format!("{}.html", name)), builtin_html)
                .map_err(|e| format!("Failed to write {}.html: {}", name, e))?;
        }

        // Generate pages for string and array methods
        let string_html = self.generate_methods_html(
            "str",
            &self.stdlib.string_methods,
            "String primitive type with utility methods for text manipulation.",
        );
        fs::write(output_dir.join("string.html"), string_html)
            .map_err(|e| format!("Failed to write string.html: {}", e))?;

        let array_html = self.generate_methods_html(
            "Array",
            &self.stdlib.array_methods,
            "Array primitive type with utility methods for collection manipulation.",
        );
        fs::write(output_dir.join("array.html"), array_html)
            .map_err(|e| format!("Failed to write array.html: {}", e))?;

        // Generate pages for other primitives (empty methods for now)
        let empty_methods = HashMap::new();
        let num_html = self.generate_methods_html("num", &empty_methods, "Numeric primitive type.");
        fs::write(output_dir.join("num.html"), num_html)
            .map_err(|e| format!("Failed to write num.html: {}", e))?;

        let bool_html =
            self.generate_methods_html("bool", &empty_methods, "Boolean primitive type.");
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

        Ok(())
    }

    fn generate_index_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str("    <title>loft Standard Library Documentation</title>\n");
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("</head>\n<body>\n");
        html.push_str("    <div class=\"sidebar\">\n");
        html.push_str("        <h2>loft stdlib</h2>\n");
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
            html.push_str(&format!(
                "                <li><a href=\"{}.html\">{}</a></li>\n",
                name, name
            ));
        }
        html.push_str("            </ul>\n");
        html.push_str("        </div>\n");

        if !self.stdlib.types.is_empty() {
            html.push_str("        <div class=\"nav-section\">\n");
            html.push_str("            <h3>Types</h3>\n");
            html.push_str("            <ul>\n");
            for name in self.stdlib.types.keys() {
                html.push_str(&format!(
                    "                <li><a href=\"type-{}.html\">{}</a></li>\n",
                    name, name
                ));
            }
            html.push_str("            </ul>\n");
            html.push_str("        </div>\n");
        }

        if !self.stdlib.traits.is_empty() {
            html.push_str("        <div class=\"nav-section\">\n");
            html.push_str("            <h3>Traits</h3>\n");
            html.push_str("            <ul>\n");
            for name in self.stdlib.traits.keys() {
                html.push_str(&format!(
                    "                <li><a href=\"trait-{}.html\">{}</a></li>\n",
                    name, name
                ));
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

    fn generate_builtin_html(&self, name: &str, builtin: &BuiltinDef) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str(&format!("    <title>{} - loft stdlib</title>\n", name));
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("</head>\n<body>\n");

        html.push_str(&self.generate_sidebar());

        html.push_str("    <div class=\"content\">\n");
        html.push_str(
            "        <div class=\"breadcrumb\"><a href=\"index.html\">stdlib</a> / <span>",
        );
        html.push_str(name);
        html.push_str("</span></div>\n");
        html.push_str(&format!("        <h1>{}</h1>\n", name));
        html.push_str(&format!(
            "        <p class=\"description\">{}</p>\n",
            Self::escape_html(&builtin.documentation)
        ));

        // Constants
        if !builtin.constants.is_empty() {
            html.push_str("        <h2>Constants</h2>\n");
            for (const_name, constant) in &builtin.constants {
                html.push_str("        <div class=\"method-item\">\n");
                html.push_str(&format!(
                    "            <h3 id=\"{}\">{}</h3>\n",
                    const_name, const_name
                ));
                html.push_str(&format!(
                    "            <pre class=\"signature\"><code>{}.{}: {}</code></pre>\n",
                    name,
                    const_name,
                    self.link_type(&constant.const_type)
                ));
                html.push_str(&format!(
                    "            <p>{}</p>\n",
                    Self::escape_html(&constant.documentation)
                ));
                html.push_str("        </div>\n");
            }
        }

        // Methods
        if !builtin.methods.is_empty() {
            html.push_str("        <h2>Methods</h2>\n");
            for (method_name, method) in &builtin.methods {
                html.push_str("        <div class=\"method-item\">\n");
                html.push_str(&format!(
                    "            <h3 id=\"{}\">{}</h3>\n",
                    method_name, method_name
                ));
                html.push_str(&format!(
                    "            <pre class=\"signature\"><code>{}.{}({}) -> {}</code></pre>\n",
                    name,
                    method_name,
                    self.format_params(&method.params),
                    self.link_type(&method.return_type)
                ));
                html.push_str(&format!(
                    "            <p><strong>Returns:</strong> <code>{}</code></p>\n",
                    self.link_type(&method.return_type)
                ));
                html.push_str(&format!(
                    "            <p>{}</p>\n",
                    Self::escape_html(&method.documentation)
                ));
                html.push_str("        </div>\n");
            }
        }

        html.push_str("    </div>\n");
        html.push_str("</body>\n</html>\n");
        html
    }

    fn generate_methods_html(
        &self,
        title: &str,
        methods: &HashMap<String, MethodDef>,
        description: &str,
    ) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str(&format!("    <title>{} - loft stdlib</title>\n", title));
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("</head>\n<body>\n");

        html.push_str(&self.generate_sidebar());

        html.push_str("    <div class=\"content\">\n");
        html.push_str(
            "        <div class=\"breadcrumb\"><a href=\"index.html\">stdlib</a> / <span>",
        );
        html.push_str(title);
        html.push_str("</span></div>\n");
        html.push_str(&format!("        <h1>{}</h1>\n", title));
        html.push_str(&format!(
            "        <p class=\"description\">{}</p>\n",
            Self::escape_html(description)
        ));

        html.push_str("        <h2>Methods</h2>\n");
        for (method_name, method) in methods {
            html.push_str("        <div class=\"method-item\">\n");
            html.push_str(&format!(
                "            <h3 id=\"{}\">{}</h3>\n",
                method_name, method_name
            ));
            html.push_str(&format!(
                "            <pre class=\"signature\"><code>value.{}({}) -> {}</code></pre>\n",
                method_name,
                self.format_params(&method.params),
                self.link_type(&method.return_type)
            ));
            html.push_str(&format!(
                "            <p><strong>Returns:</strong> <code>{}</code></p>\n",
                self.link_type(&method.return_type)
            ));
            html.push_str(&format!(
                "            <p>{}</p>\n",
                Self::escape_html(&method.documentation)
            ));

            // Add usage examples for common methods
            if let Some(example) = self.get_usage_example(title, method_name) {
                html.push_str("            <h4>Example</h4>\n");
                html.push_str(&format!(
                    "            <pre class=\"example\"><code>{}</code></pre>\n",
                    Self::escape_html(example)
                ));
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
        html.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str(&format!("    <title>{} - loft stdlib</title>\n", name));
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("</head>\n<body>\n");

        html.push_str(&self.generate_sidebar());

        html.push_str("    <div class=\"content\">\n");
        html.push_str(
            "        <div class=\"breadcrumb\"><a href=\"index.html\">stdlib</a> / <span>",
        );
        html.push_str(name);
        html.push_str("</span></div>\n");
        html.push_str(&format!("        <h1>Type: {}</h1>\n", name));
        html.push_str(&format!(
            "        <p class=\"description\">{}</p>\n",
            Self::escape_html(&type_def.documentation)
        ));

        if !type_def.fields.is_empty() {
            html.push_str("        <h2>Fields</h2>\n");
            for (field_name, field) in &type_def.fields {
                html.push_str("        <div class=\"method-item\">\n");
                html.push_str(&format!(
                    "            <h3 id=\"{}\">{}</h3>\n",
                    field_name, field_name
                ));
                html.push_str(&format!(
                    "            <pre class=\"signature\"><code>{}: {}</code></pre>\n",
                    field_name,
                    self.link_type(&field.field_type)
                ));
                html.push_str(&format!(
                    "            <p>{}</p>\n",
                    Self::escape_html(&field.documentation)
                ));
                html.push_str("        </div>\n");
            }
        }

        if !type_def.methods.is_empty() {
            html.push_str("        <h2>Methods</h2>\n");
            for (method_name, method) in &type_def.methods {
                html.push_str("        <div class=\"method-item\">\n");
                html.push_str(&format!(
                    "            <h3 id=\"{}\">{}</h3>\n",
                    method_name, method_name
                ));
                html.push_str(&format!(
                    "            <pre class=\"signature\"><code>{}.{}({}) -> {}</code></pre>\n",
                    name,
                    method_name,
                    self.format_params(&method.params),
                    self.link_type(&method.return_type)
                ));
                html.push_str(&format!(
                    "            <p><strong>Returns:</strong> <code>{}</code></p>\n",
                    self.link_type(&method.return_type)
                ));
                html.push_str(&format!(
                    "            <p>{}</p>\n",
                    Self::escape_html(&method.documentation)
                ));
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
        html.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str(&format!("    <title>{} - loft stdlib</title>\n", name));
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("</head>\n<body>\n");

        html.push_str(&self.generate_sidebar());

        html.push_str("    <div class=\"content\">\n");
        html.push_str(
            "        <div class=\"breadcrumb\"><a href=\"index.html\">stdlib</a> / <span>",
        );
        html.push_str(name);
        html.push_str("</span></div>\n");
        html.push_str(&format!("        <h1>Trait: {}</h1>\n", name));
        html.push_str(&format!(
            "        <p class=\"description\">{}</p>\n",
            Self::escape_html(&trait_def.documentation)
        ));

        html.push_str("        <h2>Required Methods</h2>\n");
        for (method_name, method) in &trait_def.methods {
            html.push_str("        <div class=\"method-item\">\n");
            html.push_str(&format!(
                "            <h3 id=\"{}\">{}</h3>\n",
                method_name, method_name
            ));
            html.push_str(&format!(
                "            <pre class=\"signature\"><code>fn {}({}) -> {}</code></pre>\n",
                method_name,
                self.format_params(&method.params),
                self.link_type(&method.return_type)
            ));
            html.push_str(&format!(
                "            <p><strong>Returns:</strong> <code>{}</code></p>\n",
                self.link_type(&method.return_type)
            ));
            html.push_str(&format!(
                "            <p>{}</p>\n",
                Self::escape_html(&method.documentation)
            ));
            html.push_str("        </div>\n");
        }

        html.push_str("    </div>\n");
        html.push_str("</body>\n</html>\n");
        html
    }

    fn format_params(&self, params: &[String]) -> String {
        params
            .iter()
            .map(|p| {
                if let Some((name, type_part)) = p.split_once(':') {
                    format!("{}: {}", name, self.link_type(type_part.trim()))
                } else {
                    self.link_type(p)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_sidebar(&self) -> String {
        let mut html = String::new();
        html.push_str("    <div class=\"sidebar\">\n");
        html.push_str("        <h2><a href=\"index.html\">loft stdlib</a></h2>\n");

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
            html.push_str(&format!(
                "                <li><a href=\"{}.html\">{}</a></li>\n",
                name, name
            ));
        }
        html.push_str("            </ul>\n");
        html.push_str("        </div>\n");

        if !self.stdlib.types.is_empty() {
            html.push_str("        <div class=\"nav-section\">\n");
            html.push_str("            <h3>Types</h3>\n");
            html.push_str("            <ul>\n");
            for name in self.stdlib.types.keys() {
                html.push_str(&format!(
                    "                <li><a href=\"type-{}.html\">{}</a></li>\n",
                    name, name
                ));
            }
            html.push_str("            </ul>\n");
            html.push_str("        </div>\n");
        }

        if !self.stdlib.traits.is_empty() {
            html.push_str("        <div class=\"nav-section\">\n");
            html.push_str("            <h3>Traits</h3>\n");
            html.push_str("            <ul>\n");
            for name in self.stdlib.traits.keys() {
                html.push_str(&format!(
                    "                <li><a href=\"trait-{}.html\">{}</a></li>\n",
                    name, name
                ));
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
            ("Array", "array.html"),
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
                    replacements.push((
                        placeholder,
                        format!("<a href=\"{}.html\">{}</a>", name, name),
                    ));
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
                    replacements.push((
                        placeholder,
                        format!("<a href=\"type-{}.html\">{}</a>", name, name),
                    ));
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
                    replacements.push((
                        placeholder,
                        format!("<a href=\"trait-{}.html\">{}</a>", name, name),
                    ));
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
        r#"* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    line-height: 1.6;
    background-color: #fafafa;
    color: #333;
    display: flex;
    min-height: 100vh;
}

.sidebar {
    width: 260px;
    background-color: #fff;
    border-right: 1px solid #e1e4e8;
    padding: 20px;
    position: fixed;
    height: 100vh;
    overflow-y: auto;
}

.sidebar h2 {
    font-size: 22px;
    margin-bottom: 20px;
    color: #24292e;
}

.sidebar h2 a {
    color: #24292e;
    text-decoration: none;
}

.sidebar h2 a:hover {
    color: #0366d6;
}

.nav-section {
    margin-bottom: 24px;
}

.nav-section h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #6a737d;
    margin-bottom: 8px;
    font-weight: 600;
}

.nav-section ul {
    list-style: none;
}

.nav-section li {
    margin-bottom: 4px;
}

.nav-section a {
    color: #586069;
    text-decoration: none;
    font-size: 14px;
    display: block;
    padding: 4px 8px;
    border-radius: 4px;
    transition: background-color 0.2s, color 0.2s;
}

.nav-section a:hover {
    background-color: #f6f8fa;
    color: #0366d6;
}

.content {
    margin-left: 260px;
    flex: 1;
    padding: 40px 60px;
    max-width: 1200px;
}

.breadcrumb {
    font-size: 14px;
    color: #586069;
    margin-bottom: 16px;
}

.breadcrumb a {
    color: #0366d6;
    text-decoration: none;
}

.breadcrumb a:hover {
    text-decoration: underline;
}

h1 {
    font-size: 36px;
    font-weight: 600;
    margin-bottom: 16px;
    color: #24292e;
    border-bottom: 1px solid #e1e4e8;
    padding-bottom: 16px;
}

h2 {
    font-size: 24px;
    font-weight: 600;
    margin-top: 32px;
    margin-bottom: 16px;
    color: #24292e;
}

h3 {
    font-size: 18px;
    font-weight: 600;
    margin-bottom: 8px;
    color: #24292e;
}

h4 {
    font-size: 14px;
    font-weight: 600;
    margin-top: 16px;
    margin-bottom: 8px;
    color: #586069;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.intro {
    font-size: 16px;
    color: #586069;
    margin-bottom: 32px;
}

.description {
    font-size: 16px;
    color: #586069;
    margin-bottom: 24px;
}

.item-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
    margin-top: 16px;
}

.item-card {
    background-color: #fff;
    border: 1px solid #e1e4e8;
    border-radius: 6px;
    padding: 16px;
    transition: box-shadow 0.2s, border-color 0.2s;
}

.item-card:hover {
    border-color: #0366d6;
    box-shadow: 0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.24);
}

.item-card a {
    color: #0366d6;
    text-decoration: none;
}

.item-card a:hover {
    text-decoration: underline;
}

.method-item {
    background-color: #fff;
    border: 1px solid #e1e4e8;
    border-radius: 6px;
    padding: 20px;
    margin-bottom: 16px;
}

.signature {
    background-color: #f6f8fa;
    border: 1px solid #e1e4e8;
    border-radius: 6px;
    padding: 12px 16px;
    margin: 12px 0;
    overflow-x: auto;
}

.signature code {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    font-size: 14px;
    color: #24292e;
}

.example {
    background-color: #f6f8fa;
    border: 1px solid #e1e4e8;
    border-radius: 6px;
    padding: 12px 16px;
    margin: 12px 0;
    overflow-x: auto;
}

.example code {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    font-size: 13px;
    color: #24292e;
    white-space: pre;
}

code {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    font-size: 13px;
    background-color: #f6f8fa;
    padding: 2px 6px;
    border-radius: 3px;
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
        assert_eq!(
            linked,
            "<a href=\"array.html\">Array</a>&lt;<a href=\"string.html\">str</a>&gt;"
        );
    }
}

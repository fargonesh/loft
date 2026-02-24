use crate::docgen::stdlib::{MethodDef, StdlibTypes};
use crate::docgen::{DocItem, DocItemKind};
use owo_colors::OwoColorize;
use std::collections::HashMap;

pub fn display_stdlib_doc(topic: &str, stdlib: &StdlibTypes) -> bool {
    let mut found = false;

    // Check builtins
    if let Some(builtin) = stdlib.builtins.get(topic) {
        println!("{}", "NAME".bright_green().bold());
        println!(
            "    {} - {}\n",
            topic.bright_white().bold(),
            "Built-in module".bright_black()
        );

        println!("{}", "DESCRIPTION".bright_green().bold());
        for line in builtin.documentation.lines() {
            println!("    {}", line);
        }
        println!();

        if !builtin.constants.is_empty() {
            println!("{}", "CONSTANTS".bright_green().bold());
            let mut keys: Vec<_> = builtin.constants.keys().collect();
            keys.sort();
            for name in keys {
                let constant = &builtin.constants[name];
                println!(
                    "    {} : {}",
                    name.bright_yellow(),
                    constant.const_type.bright_blue()
                );
                println!("        {}\n", constant.documentation);
            }
        }

        if !builtin.methods.is_empty() {
            println!("{}", "METHODS".bright_green().bold());
            let mut keys: Vec<_> = builtin.methods.keys().collect();
            keys.sort();
            for name in keys {
                let method = &builtin.methods[name];
                println!(
                    "    {}({}) -> {}",
                    name.bright_yellow(),
                    method.params.join(", ").bright_blue(),
                    method.return_type.bright_magenta()
                );
                println!("        {}\n", method.documentation);
            }
        }
        found = true;
    }

    // Check special topics like string, array
    if topic == "string" || topic == "str" {
        if found {
            println!("{}\n", "--- Primitive Methods ---".dimmed());
        }
        display_methods_doc("string", &stdlib.string_methods, "String primitive type");
        found = true;
    } else if topic == "array" {
        if found {
            println!("{}\n", "--- Primitive Methods ---".dimmed());
        }
        display_methods_doc("array", &stdlib.array_methods, "Array primitive type");
        found = true;
    }

    if found {
        return true;
    }

    // Check types
    if let Some(ty) = stdlib.types.get(topic) {
        println!("{}", "NAME".bright_green().bold());
        println!(
            "    {} - {}\n",
            topic.bright_white().bold(),
            format!("Type ({})", ty.kind).bright_black()
        );

        println!("{}", "DESCRIPTION".bright_green().bold());
        println!("    {}\n", ty.documentation);

        if !ty.fields.is_empty() {
            println!("{}", "FIELDS".bright_green().bold());
            let mut keys: Vec<_> = ty.fields.keys().collect();
            keys.sort();
            for name in keys {
                let field = &ty.fields[name];
                println!(
                    "    {} : {}",
                    name.bright_yellow(),
                    field.field_type.bright_blue()
                );
                println!("        {}\n", field.documentation);
            }
        }

        if !ty.methods.is_empty() {
            println!("{}", "METHODS".bright_green().bold());
            let mut keys: Vec<_> = ty.methods.keys().collect();
            keys.sort();
            for name in keys {
                let method = &ty.methods[name];
                println!(
                    "    {}({}) -> {}",
                    name.bright_yellow(),
                    method.params.join(", ").bright_blue(),
                    method.return_type.bright_magenta()
                );
                println!("        {}\n", method.documentation);
            }
        }
        return true;
    }

    // Check traits
    if let Some(tr) = stdlib.traits.get(topic) {
        println!("{}", "NAME".bright_green().bold());
        println!(
            "    {} - {}\n",
            topic.bright_white().bold(),
            "Trait".bright_black()
        );

        println!("{}", "DESCRIPTION".bright_green().bold());
        println!("    {}\n", tr.documentation);

        if !tr.methods.is_empty() {
            println!("{}", "METHODS".bright_green().bold());
            let mut keys: Vec<_> = tr.methods.keys().collect();
            keys.sort();
            for name in keys {
                let method = &tr.methods[name];
                println!(
                    "    {}({}) -> {}",
                    name.bright_yellow(),
                    method.params.join(", ").bright_blue(),
                    method.return_type.bright_magenta()
                );
                println!("        {}\n", method.documentation);
            }
        }
        return true;
    }

    false
}

fn display_methods_doc(name: &str, methods: &HashMap<String, MethodDef>, description: &str) {
    println!("{}", "NAME".bright_green().bold());
    println!(
        "    {} - {}\n",
        name.bright_white().bold(),
        description.bright_black()
    );

    println!("{}", "METHODS".bright_green().bold());
    let mut keys: Vec<_> = methods.keys().collect();
    keys.sort();
    for name in keys {
        let method = &methods[name];
        println!(
            "    {}({}) -> {}",
            name.bright_yellow(),
            method.params.join(", ").bright_blue(),
            method.return_type.bright_magenta()
        );
        println!("        {}\n", method.documentation);
    }
}

pub fn display_doc_item(item: &DocItem) {
    println!("{}", "NAME".bright_green().bold());
    println!(
        "    {} - {}\n",
        item.name.bright_white().bold(),
        match &item.kind {
            DocItemKind::Function { .. } => "Function",
            DocItemKind::Struct { .. } => "Struct",
            DocItemKind::Trait { .. } => "Trait",
            DocItemKind::Constant { .. } => "Constant",
            DocItemKind::Variable { .. } => "Variable",
        }
        .bright_black()
    );

    if let Some(sig) = &item.signature {
        println!("{}", "SYNOPSIS".bright_green().bold());
        println!("    {}\n", sig.bright_cyan());
    }

    println!("{}", "DESCRIPTION".bright_green().bold());
    if let Some(doc) = &item.documentation {
        // Simple word wrap or just indentation
        for line in doc.lines() {
            println!("    {}", line);
        }
        println!();
    } else {
        println!("    No documentation available.\n");
    }

    match &item.kind {
        DocItemKind::Struct { fields, .. } if !fields.is_empty() => {
            println!("{}", "FIELDS".bright_green().bold());
            for (name, ty) in fields {
                println!("    {} : {}\n", name.bright_yellow(), ty.bright_blue());
            }
        }
        DocItemKind::Trait { methods, .. } if !methods.is_empty() => {
            println!("{}", "METHODS".bright_green().bold());
            for method in methods {
                println!("    {}\n", method.bright_yellow());
            }
        }
        _ => {}
    }
}

pub fn list_topics(stdlib: &StdlibTypes) {
    println!("{}", "STANDARD LIBRARY".bright_green().bold());

    // Primitives
    println!("\n  {}", "PRIMITIVES".bright_white().bold());
    let mut primitives = vec!["string".to_string(), "array".to_string(), "num".to_string(), "bool".to_string(), "void".to_string()];
    primitives.sort();
    for chunk in primitives.chunks(5) {
        println!("    {}", chunk.join(", ").to_lowercase());
    }

    // Built-in Modules
    println!("\n  {}", "MODULES".bright_white().bold());
    let mut modules: Vec<String> = stdlib.builtins.keys().cloned().collect();
    modules.sort();
    for chunk in modules.chunks(5) {
        println!("    {}", chunk.join(", ").to_lowercase());
    }

    // Traits
    if !stdlib.traits.is_empty() {
        println!("\n  {}", "TRAITS".bright_white().bold());
        let mut traits: Vec<String> = stdlib.traits.keys().cloned().collect();
        traits.sort();
        for chunk in traits.chunks(5) {
            println!("    {}", chunk.join(", ").to_lowercase());
        }
    }

    // Types
    if !stdlib.types.is_empty() {
        println!("\n  {}", "TYPES".bright_white().bold());
        let mut types: Vec<String> = stdlib.types.keys().cloned().collect();
        types.sort();
        for chunk in types.chunks(5) {
            println!("    {}", chunk.join(", ").to_lowercase());
        }
    }

    println!();
}

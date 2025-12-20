use crate::parser::{Expr, Stmt, Type, TemplatePart, TraitMethod};

mod token_formatter;
pub use token_formatter::TokenFormatter;

/// Formatter for loft source code.
/// 
/// This formatter converts parsed AST back to formatted source code.
/// 
/// # Limitations
/// 
/// - Comments are not preserved as they are not part of the AST
/// - Some whitespace details may differ from the original source
/// 
/// For comment preservation, use `TokenFormatter` instead.
pub struct Formatter {
    indent_size: usize,
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter {
    pub fn new() -> Self {
        Self { indent_size: 4 }
    }

    pub fn format_program(&self, stmts: &[Stmt]) -> String {
        stmts.iter()
            .map(|stmt| self.format_stmt(stmt, 0))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn indent(&self, level: usize) -> String {
        " ".repeat(level * self.indent_size)
    }

    fn format_stmt(&self, stmt: &Stmt, level: usize) -> String {
        let indent = self.indent(level);
        match stmt {
            Stmt::ImportDecl { path } => {
                format!("{}learn \"{}\";", indent, path.join("/"))
            }
            Stmt::VarDecl { name, var_type, mutable, value } => {
                let mut_kw = if *mutable { "mut " } else { "" };
                let type_annotation = var_type.as_ref()
                    .map(|t| format!(": {}", self.format_type(t)))
                    .unwrap_or_default();
                let val = value.as_ref()
                    .map(|v| format!(" = {}", self.format_expr(v)))
                    .unwrap_or_default();
                format!("{}let {}{}{}{};", indent, mut_kw, name, type_annotation, val)
            }
            Stmt::ConstDecl { name, const_type, value } => {
                let type_annotation = const_type.as_ref()
                    .map(|t| format!(": {}", self.format_type(t)))
                    .unwrap_or_default();
                format!("{}const {}{} = {};", indent, name, type_annotation, self.format_expr(value))
            }
            Stmt::FunctionDecl { name, type_params, params, return_type, body, is_async, is_exported } => {
                let export = if *is_exported { "teach " } else { "" };
                let async_kw = if *is_async { "async " } else { "" };
                let type_params_str = if !type_params.is_empty() {
                    format!("<{}>", type_params.join(", "))
                } else {
                    String::new()
                };
                let params_str = params.iter()
                    .map(|(n, t)| format!("{}: {}", n, self.format_type(t)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let return_str = match return_type {
                    Some(t) => format!(" -> {}", self.format_type(t)),
                    None => String::new(),
                };
                let body_str = self.format_stmt(body, level);
                
                format!("{}{}{}fn {}{}({}){} {}", 
                    indent, export, async_kw, name, type_params_str, params_str, return_str, body_str.trim_start())
            }
            Stmt::StructDecl { name, fields } => {
                if fields.is_empty() {
                    format!("{}def {} {{}}", indent, name)
                } else {
                    let fields_str = fields.iter()
                        .map(|(n, t)| format!("{}{}: {},", self.indent(level + 1), n, self.format_type(t)))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("{}def {} {{\n{}\n{}}}", indent, name, fields_str, indent)
                }
            }
            Stmt::ImplBlock { type_name, trait_name, methods } => {
                let trait_part = trait_name.as_ref()
                    .map(|t| format!(" {} for", t))
                    .unwrap_or_default();
                let methods_str = methods.iter()
                    .map(|m| self.format_stmt(m, level + 1))
                    .collect::<Vec<_>>()
                    .join("\n\n");
                format!("{}impl{} {} {{\n{}\n{}}}", indent, trait_part, type_name, methods_str, indent)
            }
            Stmt::TraitDecl { name, methods } => {
                let methods_str = methods.iter()
                    .map(|m| self.format_trait_method(m, level + 1))
                    .collect::<Vec<_>>()
                    .join("\n\n");
                format!("{}trait {} {{\n{}\n{}}}", indent, name, methods_str, indent)
            }
            Stmt::EnumDecl { name, variants } => {
                let variants_str = variants.iter()
                    .map(|(n, types)| {
                        if let Some(ts) = types {
                            let types_str = ts.iter()
                                .map(|t| self.format_type(t))
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("{}{}({}),", self.indent(level + 1), n, types_str)
                        } else {
                            format!("{}{},", self.indent(level + 1), n)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{}enum {} {{\n{}\n{}}}", indent, name, variants_str, indent)
            }
            Stmt::Assign { name, value } => {
                format!("{}{} = {};", indent, name, self.format_expr(value))
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let then_str = self.format_stmt(then_branch, level);
                let else_str = else_branch.as_ref()
                    .map(|e| format!(" else {}", self.format_stmt(e, level).trim_start()))
                    .unwrap_or_default();
                format!("{}if ({}) {}{}", indent, self.format_expr(condition), then_str.trim_start(), else_str)
            }
            Stmt::While { condition, body } => {
                let body_str = self.format_stmt(body, level);
                format!("{}while ({}) {}", indent, self.format_expr(condition), body_str.trim_start())
            }
            Stmt::For { var, iterable, body } => {
                let body_str = self.format_stmt(body, level);
                format!("{}for {} in {} {}", indent, var, self.format_expr(iterable), body_str.trim_start())
            }
            Stmt::Match { expr, arms } => {
                let arms_str = arms.iter()
                    .map(|(pattern, stmt)| {
                        let pattern_str = self.format_expr(pattern);
                        let stmt_str = self.format_stmt(stmt, level + 2);
                        format!("{}{} => {},", self.indent(level + 1), pattern_str, stmt_str.trim())
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{}match {} {{\n{}\n{}}}", indent, self.format_expr(expr), arms_str, indent)
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    format!("{}return {};", indent, self.format_expr(e))
                } else {
                    format!("{}return;", indent)
                }
            }
            Stmt::Break => format!("{}break;", indent),
            Stmt::Continue => format!("{}continue;", indent),
            Stmt::Expr(expr) => {
                format!("{}{};", indent, self.format_expr(expr))
            }
            Stmt::Block(stmts) => {
                if stmts.is_empty() {
                    format!("{}{{}}", indent)
                } else {
                    let stmts_str = stmts.iter()
                        .map(|s| self.format_stmt(s, level + 1))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("{}{{\n{}\n{}}}", indent, stmts_str, indent)
                }
            }
        }
    }

    fn format_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Number(n) => n.to_string(),
            Expr::Ident(s) => s.clone(),
            Expr::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            Expr::Boolean(b) => b.to_string(),
            Expr::BinOp { op, left, right } => {
                format!("{} {} {}", self.format_expr(left), op, self.format_expr(right))
            }
            Expr::UnaryOp { op, expr } => {
                format!("{}{}", op, self.format_expr(expr))
            }
            Expr::Call { func, args } => {
                let args_str = args.iter()
                    .map(|a| self.format_expr(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", self.format_expr(func), args_str)
            }
            Expr::FieldAccess { object, field } => {
                format!("{}.{}", self.format_expr(object), field)
            }
            Expr::ArrayLiteral(items) => {
                let items_str = items.iter()
                    .map(|i| self.format_expr(i))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", items_str)
            }
            Expr::StructLiteral { name, fields } => {
                let fields_str = fields.iter()
                    .map(|(n, e)| format!("{}: {}", n, self.format_expr(e)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", name, fields_str)
            }
            Expr::Index { array, index } => {
                format!("{}[{}]", self.format_expr(array), self.format_expr(index))
            }
            Expr::Lambda { params, return_type, body } => {
                let params_str = if params.len() == 1 && params[0].1.is_none() {
                    params[0].0.clone()
                } else {
                    let params_vec = params.iter()
                        .map(|(n, t)| {
                            if let Some(ty) = t {
                                format!("{}: {}", n, self.format_type(ty))
                            } else {
                                n.clone()
                            }
                        })
                        .collect::<Vec<_>>();
                    format!("({})", params_vec.join(", "))
                };
                let return_str = return_type.as_ref()
                    .map(|t| format!(" -> {}", self.format_type(t)))
                    .unwrap_or_default();
                format!("{}{} => {}", params_str, return_str, self.format_expr(body))
            }
            Expr::Block(stmts) => {
                if stmts.is_empty() {
                    "{}".to_string()
                } else {
                    let stmts_str = stmts.iter()
                        .map(|s| self.format_stmt(s, 1))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("{{\n{}\n}}", stmts_str)
                }
            }
            Expr::Await(expr) => format!("await {}", self.format_expr(expr)),
            Expr::Async(expr) => format!("async {}", self.format_expr(expr)),
            Expr::Lazy(expr) => format!("lazy {}", self.format_expr(expr)),
            Expr::TemplateLiteral { parts } => {
                let parts_str = parts.iter()
                    .map(|p| match p {
                        TemplatePart::Text(t) => t.clone(),
                        TemplatePart::Expression(e) => format!("${{{}}}", self.format_expr(e)),
                    })
                    .collect::<Vec<_>>()
                    .join("");
                format!("`{}`", parts_str)
            }
            Expr::Match { expr, arms } => {
                let arms_str = arms.iter()
                    .map(|(pattern, body)| {
                        format!("    {} => {},", self.format_expr(pattern), self.format_expr(body))
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("match {} {{\n{}\n}}", self.format_expr(expr), arms_str)
            }
            Expr::Try(expr) => format!("{}?", self.format_expr(expr)),
        }
    }

    fn format_type(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => name.clone(),
            Type::Generic { base, type_args } => {
                let args_str = type_args.iter()
                    .map(|t| self.format_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", base, args_str)
            }
            Type::Function { params, return_type } => {
                let params_str = params.iter()
                    .map(|t| self.format_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}) -> {}", params_str, self.format_type(return_type))
            }
        }
    }

    fn format_trait_method(&self, method: &TraitMethod, level: usize) -> String {
        let indent = self.indent(level);
        match method {
            TraitMethod::Signature { name, params, return_type } => {
                let params_str = params.iter()
                    .map(|(n, t)| format!("{}: {}", n, self.format_type(t)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}fn {}({}) -> {};", indent, name, params_str, self.format_type(return_type))
            }
            TraitMethod::Default { name, params, return_type, body } => {
                let params_str = params.iter()
                    .map(|(n, t)| format!("{}: {}", n, self.format_type(t)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let body_str = self.format_stmt(body, level);
                format!("{}fn {}({}) -> {} {}", indent, name, params_str, self.format_type(return_type), body_str.trim_start())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Parser, InputStream};

    #[test]
    fn test_format_simple_var() {
        let input = "let x=42;".to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();
        
        let formatter = Formatter::new();
        let formatted = formatter.format_program(&stmts);
        
        assert_eq!(formatted, "let x = 42;");
    }

    #[test]
    fn test_format_function() {
        let input = "fn add(a:num,b:num)->num{return a+b;}".to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();
        
        let formatter = Formatter::new();
        let formatted = formatter.format_program(&stmts);
        
        assert!(formatted.contains("fn add"));
        assert!(formatted.contains("a: num"));
        assert!(formatted.contains("b: num"));
    }
}

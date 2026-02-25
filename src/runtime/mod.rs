pub mod builtin;
pub mod builtin_registry;
pub mod builtins;
pub mod permission_context;
pub mod permissions;
pub mod traits;
pub mod value;

use crate::parser::{Expr, InputStream, Parser, Stmt, TraitMethod, Type};
use builtins::init_builtins;
use miette::{Diagnostic, LabeledSpan, NamedSource};
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use traits::{call_binop_trait, call_index_trait, ToString};
use value::Value;

fn init_stdlib_traits() -> HashMap<String, Vec<TraitMethod>> {
    let mut traits = HashMap::new();

    // Printable
    traits.insert(
        "Printable".to_string(),
        vec![TraitMethod::Signature {
            name: "print".to_string(),
            params: vec![("self".to_string(), Type::Named("Self".to_string()))],
            return_type: Type::Named("str".to_string()),
        }],
    );

    // Mul
    traits.insert(
        "Mul".to_string(),
        vec![TraitMethod::Signature {
            name: "mul".to_string(),
            params: vec![
                ("self".to_string(), Type::Named("Self".to_string())),
                ("other".to_string(), Type::Named("any".to_string())),
            ],
            return_type: Type::Named("any".to_string()),
        }],
    );

    // Div
    traits.insert(
        "Div".to_string(),
        vec![TraitMethod::Signature {
            name: "div".to_string(),
            params: vec![
                ("self".to_string(), Type::Named("Self".to_string())),
                ("other".to_string(), Type::Named("any".to_string())),
            ],
            return_type: Type::Named("any".to_string()),
        }],
    );

    // Add
    traits.insert(
        "Add".to_string(),
        vec![TraitMethod::Signature {
            name: "add".to_string(),
            params: vec![
                ("self".to_string(), Type::Named("Self".to_string())),
                ("other".to_string(), Type::Named("any".to_string())),
            ],
            return_type: Type::Named("any".to_string()),
        }],
    );

    // Sub
    traits.insert(
        "Sub".to_string(),
        vec![TraitMethod::Signature {
            name: "sub".to_string(),
            params: vec![
                ("self".to_string(), Type::Named("Self".to_string())),
                ("other".to_string(), Type::Named("any".to_string())),
            ],
            return_type: Type::Named("any".to_string()),
        }],
    );

    // Ord
    traits.insert(
        "Ord".to_string(),
        vec![
            TraitMethod::Signature {
                name: "gt".to_string(),
                params: vec![
                    ("self".to_string(), Type::Named("Self".to_string())),
                    ("other".to_string(), Type::Named("any".to_string())),
                ],
                return_type: Type::Named("bool".to_string()),
            },
            TraitMethod::Signature {
                name: "ge".to_string(),
                params: vec![
                    ("self".to_string(), Type::Named("Self".to_string())),
                    ("other".to_string(), Type::Named("any".to_string())),
                ],
                return_type: Type::Named("bool".to_string()),
            },
            TraitMethod::Signature {
                name: "lt".to_string(),
                params: vec![
                    ("self".to_string(), Type::Named("Self".to_string())),
                    ("other".to_string(), Type::Named("any".to_string())),
                ],
                return_type: Type::Named("bool".to_string()),
            },
            TraitMethod::Signature {
                name: "le".to_string(),
                params: vec![
                    ("self".to_string(), Type::Named("Self".to_string())),
                    ("other".to_string(), Type::Named("any".to_string())),
                ],
                return_type: Type::Named("bool".to_string()),
            },
            TraitMethod::Signature {
                name: "eq".to_string(),
                params: vec![
                    ("self".to_string(), Type::Named("Self".to_string())),
                    ("other".to_string(), Type::Named("any".to_string())),
                ],
                return_type: Type::Named("bool".to_string()),
            },
            TraitMethod::Signature {
                name: "ne".to_string(),
                params: vec![
                    ("self".to_string(), Type::Named("Self".to_string())),
                    ("other".to_string(), Type::Named("any".to_string())),
                ],
                return_type: Type::Named("bool".to_string()),
            },
        ],
    );

    // Index
    traits.insert(
        "Index".to_string(),
        vec![TraitMethod::Signature {
            name: "index".to_string(),
            params: vec![
                ("self".to_string(), Type::Named("Self".to_string())),
                ("index".to_string(), Type::Named("any".to_string())),
            ],
            return_type: Type::Named("any".to_string()),
        }],
    );

    // BitAnd
    traits.insert(
        "BitAnd".to_string(),
        vec![TraitMethod::Signature {
            name: "bit_and".to_string(),
            params: vec![
                ("self".to_string(), Type::Named("Self".to_string())),
                ("other".to_string(), Type::Named("any".to_string())),
            ],
            return_type: Type::Named("any".to_string()),
        }],
    );

    // BitOr
    traits.insert(
        "BitOr".to_string(),
        vec![TraitMethod::Signature {
            name: "bit_or".to_string(),
            params: vec![
                ("self".to_string(), Type::Named("Self".to_string())),
                ("other".to_string(), Type::Named("any".to_string())),
            ],
            return_type: Type::Named("any".to_string()),
        }],
    );

    // BitXor
    traits.insert(
        "BitXor".to_string(),
        vec![TraitMethod::Signature {
            name: "bit_xor".to_string(),
            params: vec![
                ("self".to_string(), Type::Named("Self".to_string())),
                ("other".to_string(), Type::Named("any".to_string())),
            ],
            return_type: Type::Named("any".to_string()),
        }],
    );

    // Shl
    traits.insert(
        "Shl".to_string(),
        vec![TraitMethod::Signature {
            name: "shl".to_string(),
            params: vec![
                ("self".to_string(), Type::Named("Self".to_string())),
                ("other".to_string(), Type::Named("any".to_string())),
            ],
            return_type: Type::Named("any".to_string()),
        }],
    );

    // Shr
    traits.insert(
        "Shr".to_string(),
        vec![TraitMethod::Signature {
            name: "shr".to_string(),
            params: vec![
                ("self".to_string(), Type::Named("Self".to_string())),
                ("other".to_string(), Type::Named("any".to_string())),
            ],
            return_type: Type::Named("any".to_string()),
        }],
    );

    traits
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub path: Option<String>,
    pub source: Option<NamedSource<String>>,
    pub position: Option<usize>,
    pub len: Option<usize>,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
            source: None,
            position: None,
            len: None,
        }
    }

    pub fn with_source(
        message: impl Into<String>,
        path: String,
        source_code: String,
        position: usize,
        len: usize,
    ) -> Self {
        Self {
            message: message.into(),
            path: Some(path.clone()),
            source: Some(NamedSource::new(path, source_code)),
            position: Some(position),
            len: Some(len),
        }
    }

    /// Create an error with source context but no specific position
    /// This will show the source code but not highlight a specific location
    pub fn with_source_context(
        message: impl Into<String>,
        path: String,
        source_code: String,
    ) -> Self {
        Self {
            message: message.into(),
            path: Some(path.clone()),
            source: Some(NamedSource::new(path, source_code)),
            position: None,
            len: None,
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

impl Diagnostic for RuntimeError {
    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.source.as_ref().map(|s| s as &dyn miette::SourceCode)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        if let (Some(position), Some(len)) = (self.position, self.len) {
            Some(Box::new(
                vec![LabeledSpan::new(Some(self.message.clone()), position, len)].into_iter(),
            ))
        } else if self.source.is_some() {
            // If we have source but no position, show a label at the beginning
            // This ensures the source code is displayed by miette
            Some(Box::new(
                vec![LabeledSpan::new(Some(self.message.clone()), 0, 1)].into_iter(),
            ))
        } else {
            None
        }
    }
}

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    pub fn update(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        // Allow re-assignment of any variable (not just mutable ones)
        // Also allow shadowing by creating a new variable in the current scope if not found
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        // If variable not found, create it in the current scope (shadowing)
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), value);
        }
        Ok(())
    }

    /// Capture all variables from the current environment
    /// This is used when creating closures to capture their environment
    pub fn capture_all(&self) -> HashMap<String, Value> {
        let mut captured = HashMap::new();
        for scope in &self.scopes {
            for (name, value) in scope {
                captured.insert(name.clone(), value.clone());
            }
        }
        captured
    }
}

pub struct Interpreter {
    env: Environment,
    source_path: Option<String>,
    source_code: Option<String>,
    // Track trait declarations: trait_name -> methods
    traits: HashMap<String, Vec<TraitMethod>>,
    // Track impl blocks: type_name -> method_name -> (params, return_type, body)
    // Format: type_name -> method_name -> (params, return_type, body, trait_name_if_any)
    impl_methods: HashMap<
        String,
        HashMap<
            String,
            (
                Vec<(String, crate::parser::Type)>,
                Option<crate::parser::Type>,
                Box<Stmt>,
                Option<String>,
            ),
        >,
    >,
    // Track enum declarations: enum_name -> variants
    // Format: enum_name -> Vec<(variant_name, Option<Vec<Type>>)>
    enums: HashMap<String, Vec<(String, Option<Vec<Type>>)>>,
    // Module cache: module_path -> exported_values
    module_cache: HashMap<String, HashMap<String, Value>>,
    // Current module's exports
    exports: HashMap<String, Value>,
    // Enabled features for gating
    enabled_features: std::collections::HashSet<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Environment::new();

        // Initialize builtins
        for (name, value) in init_builtins(None) {
            env.set(name, value);
        }

        Self {
            env,
            source_path: None,
            source_code: None,
            traits: init_stdlib_traits(),
            impl_methods: HashMap::new(),
            enums: HashMap::new(),
            module_cache: HashMap::new(),
            exports: HashMap::new(),
            enabled_features: std::collections::HashSet::new(),
        }
    }

    pub fn with_source(source_path: impl Into<String>, source_code: impl Into<String>) -> Self {
        let mut env = Environment::new();

        // Initialize builtins
        for (name, value) in init_builtins(None) {
            env.set(name, value);
        }

        Self {
            env,
            source_path: Some(source_path.into()),
            source_code: Some(source_code.into()),
            traits: init_stdlib_traits(),
            impl_methods: HashMap::new(),
            enums: HashMap::new(),
            module_cache: HashMap::new(),
            exports: HashMap::new(),
            enabled_features: std::collections::HashSet::new(),
        }
    }

    pub fn with_features(mut self, features: Vec<String>) -> Self {
        let feature_set: std::collections::HashSet<String> = features.into_iter().collect();
        self.enabled_features = feature_set.clone();

        // Re-initialize builtins with the new features
        self.env = Environment::new();
        let features_slice: Vec<String> = self.enabled_features.iter().cloned().collect();
        for (name, value) in init_builtins(Some(&features_slice)) {
            self.env.set(name, value);
        }

        self
    }

    fn check_gated(&self, attr: &crate::parser::Attribute) -> bool {
        if attr.name != "gated" {
            return true;
        }

        fn eval_gated(expr: &Expr, enabled: &std::collections::HashSet<String>) -> bool {
            match expr {
                Expr::Ident(name) => enabled.contains(name),
                Expr::Call { func, args } => {
                    if let Expr::Ident(ref name) = **func {
                        match name.as_str() {
                            "all" => args.iter().all(|arg| eval_gated(arg, enabled)),
                            "any" => args.iter().any(|arg| eval_gated(arg, enabled)),
                            "not" => args.get(0).map_or(false, |arg| !eval_gated(arg, enabled)),
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }

        attr.args
            .iter()
            .all(|arg| eval_gated(arg, &self.enabled_features))
    }

    pub fn eval_program(&mut self, stmts: Vec<Stmt>) -> RuntimeResult<Value> {
        let mut last_value = Value::Unit;
        for stmt in stmts {
            last_value = self.eval_stmt(stmt)?;
        }
        Ok(last_value)
    }

    pub fn eval_stmt(&mut self, stmt: Stmt) -> RuntimeResult<Value> {
        match stmt {
            Stmt::ImportDecl { path } => {
                // Handle module imports
                self.load_module(&path)
            }
            Stmt::VarDecl {
                name,
                var_type: _,
                mutable: _,
                value,
            } => {
                let val = if let Some(expr) = value {
                    self.eval_expr(expr)?
                } else {
                    Value::Unit
                };
                self.env.set(name, val);
                Ok(Value::Unit)
            }
            Stmt::ConstDecl {
                name,
                const_type: _,
                value,
            } => {
                let val = self.eval_expr(value)?;
                self.env.set(name, val);
                Ok(Value::Unit)
            }
            Stmt::Assign { name, value } => {
                let val = self.eval_expr(value)?;
                self.env.update(&name, val)?;
                Ok(Value::Unit)
            }
            Stmt::AttrStmt { attr, stmt } => {
                if self.check_gated(&attr) {
                    self.eval_stmt(*stmt)
                } else {
                    Ok(Value::Unit)
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    self.eval_stmt(*then_branch)
                } else if let Some(else_stmt) = else_branch {
                    self.eval_stmt(*else_stmt)
                } else {
                    Ok(Value::Unit)
                }
            }
            Stmt::While { condition, body } => {
                while self.eval_expr(condition.clone())?.is_truthy() {
                    self.eval_stmt(*body.clone())?;
                }
                Ok(Value::Unit)
            }
            Stmt::Return(expr) => {
                let val = if let Some(e) = expr {
                    self.eval_expr(e)?
                } else {
                    Value::Unit
                };
                // In a full implementation, we'd use a special control flow mechanism
                Ok(val)
            }
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::Block(stmts) => {
                self.env.push_scope();
                let mut last_value = Value::Unit;
                for stmt in stmts {
                    last_value = self.eval_stmt(stmt)?;
                }
                self.env.pop_scope();
                Ok(last_value)
            }
            Stmt::FunctionDecl {
                name,
                params,
                body,
                is_async,
                is_exported,
                ..
            } => {
                // Store function in environment
                let func_value = Value::Function {
                    name: name.clone(),
                    params: params
                        .iter()
                        .map(|(n, t)| (n.clone(), format!("{:?}", t)))
                        .collect(),
                    body: body.clone(),
                    is_async,
                };
                self.env.set(name.clone(), func_value.clone());

                // If exported, add to module exports
                if is_exported {
                    self.exports.insert(name, func_value);
                }

                Ok(Value::Unit)
            }
            Stmt::TraitDecl { name, methods } => {
                // Store trait declaration
                self.traits.insert(name, methods);
                Ok(Value::Unit)
            }
            Stmt::ImplBlock {
                type_name,
                trait_name,
                methods,
            } => {
                // If implementing a trait, validate it
                if let Some(ref t_name) = trait_name {
                    if let Some(trait_methods) = self.traits.get(t_name) {
                        // Check for missing required methods and validate signatures
                        for tm in trait_methods {
                            let (t_method_name, t_params, t_return_type) = match tm {
                                TraitMethod::Signature {
                                    name,
                                    params,
                                    return_type,
                                } => (name, params, return_type),
                                TraitMethod::Default {
                                    name,
                                    params,
                                    return_type,
                                    ..
                                } => (name, params, return_type),
                            };

                            // Find implementation
                            let implementation = methods.iter().find(|m| {
                                if let Stmt::FunctionDecl { name, .. } = m {
                                    name == t_method_name
                                } else {
                                    false
                                }
                            });

                            if let Some(Stmt::FunctionDecl {
                                params: impl_params,
                                return_type: impl_return_type,
                                ..
                            }) = implementation
                            {
                                // Validate signature
                                // 1. Check parameter count
                                if impl_params.len() != t_params.len() {
                                    return Err(RuntimeError::new(format!(
                                        "Method '{}' of trait '{}' expects {} parameters, but implementation has {}",
                                        t_method_name, t_name, t_params.len(), impl_params.len()
                                    )));
                                }

                                // 2. Check parameter types
                                for (i, (impl_name, impl_type)) in impl_params.iter().enumerate() {
                                    let (_, t_type) = &t_params[i];
                                    let is_any = matches!(t_type, Type::Named(n) if n == "any");
                                    if !is_any && impl_type != t_type {
                                        return Err(RuntimeError::new(format!(
                                            "Parameter '{}' of method '{}' has incorrect type. Expected {:?}, found {:?}",
                                            impl_name, t_method_name, t_type, impl_type
                                        )));
                                    }
                                }

                                // 3. Check return type
                                let is_any_return =
                                    matches!(t_return_type, Type::Named(n) if n == "any");
                                // Compare Option<Type> with Type
                                let types_match = match impl_return_type {
                                    Some(impl_type) => impl_type == t_return_type,
                                    None => {
                                        matches!(t_return_type, Type::Named(n) if n == "void" || n == "unit")
                                    }
                                };
                                if !is_any_return && !types_match {
                                    return Err(RuntimeError::new(format!(
                                        "Method '{}' of trait '{}' has incorrect return type. Expected {:?}, found {:?}",
                                        t_method_name, t_name, t_return_type, impl_return_type
                                    )));
                                }
                            } else if matches!(tm, TraitMethod::Signature { .. }) {
                                // Missing required method
                                return Err(RuntimeError::new(format!(
                                    "Missing implementation for method '{}' of trait '{}'",
                                    t_method_name, t_name
                                )));
                            }
                        }
                    }
                }

                // Store methods for this type (with optional trait association)
                let type_methods = self
                    .impl_methods
                    .entry(type_name)
                    .or_insert_with(HashMap::new);

                // Process each method in the impl block
                for method_stmt in methods {
                    if let Stmt::FunctionDecl {
                        name: method_name,
                        params,
                        return_type,
                        body,
                        ..
                    } = method_stmt
                    {
                        // Store the method with its signature
                        type_methods
                            .insert(method_name, (params, return_type, body, trait_name.clone()));
                    }
                }

                Ok(Value::Unit)
            }
            Stmt::StructDecl { .. } => {
                // StructDecl is a declaration - it doesn't execute anything
                // It's just a type definition
                Ok(Value::Unit)
            }
            Stmt::EnumDecl { name, variants } => {
                // Store the enum definition
                self.enums.insert(name.clone(), variants.clone());

                // Create a namespace object for the enum that allows accessing variants
                // We'll store constructors for each variant in the environment
                for (variant_name, variant_types) in variants {
                    let full_name = format!("{}.{}", name, variant_name);

                    // For unit variants, store the variant directly
                    // For tuple variants, we would need a constructor function (not yet implemented)
                    if variant_types.is_none() {
                        let variant = Value::EnumVariant {
                            enum_name: name.clone(),
                            variant_name: variant_name.clone(),
                            values: vec![],
                        };
                        self.env.set(full_name, variant);
                    }
                }

                Ok(Value::Unit)
            }
            Stmt::Match { expr, arms } => {
                let value = self.eval_expr(expr)?;

                // Try each pattern arm until one matches
                for (pattern, body) in arms {
                    if let Some(bindings) = self.match_pattern(&pattern, &value)? {
                        // Pattern matched, bind any captured variables and execute body
                        self.env.push_scope();
                        for (name, val) in bindings {
                            self.env.set(name, val);
                        }
                        let result = self.eval_stmt(body)?;
                        self.env.pop_scope();
                        return Ok(result);
                    }
                }

                // No pattern matched
                Err(self.error("Match expression did not match any pattern".to_string()))
            }
            Stmt::For { .. } | Stmt::Break | Stmt::Continue => {
                // For loops not yet fully implemented
                Ok(Value::Unit)
            }
        }
    }

    pub fn eval_expr(&mut self, expr: Expr) -> RuntimeResult<Value> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(n)),
            Expr::String(s) => Ok(Value::String(s)),
            Expr::Boolean(b) => Ok(Value::Boolean(b)),
            Expr::Ident(name) => self
                .env
                .get(&name)
                .cloned()
                .ok_or_else(|| self.error(format!("Variable '{}' not found", name))),
            Expr::BinOp { op, left, right } => {
                let left_val = self.eval_expr(*left)?;
                let right_val = self.eval_expr(*right)?;
                self.eval_binop(&op, left_val, right_val)
            }
            Expr::Call { func, args } => {
                // Always evaluate func first (which could be a FieldAccess or any expr)
                let func_val = self.eval_expr(*func)?;
                let arg_vals: RuntimeResult<Vec<_>> =
                    args.into_iter().map(|arg| self.eval_expr(arg)).collect();
                let arg_vals = arg_vals?;
                match func_val {
                    Value::Function {
                        params,
                        body,
                        is_async,
                        name,
                        ..
                    } => {
                        // Check argument count
                        if params.len() != arg_vals.len() {
                            return Err(self.error(format!(
                                "Function '{}' expects {} arguments, got {}",
                                name,
                                params.len(),
                                arg_vals.len()
                            )));
                        }

                        // Optional: Check argument types if type annotations exist
                        // Note: params are stored as (name, type_string)
                        // For now, we skip type validation since we'd need to parse the type strings

                        // Create new scope for function
                        self.env.push_scope();

                        // Bind parameters
                        for ((param_name, _), arg_val) in params.iter().zip(arg_vals.iter()) {
                            self.env.set(param_name.clone(), arg_val.clone());
                        }

                        // Execute function body
                        let result = self.eval_stmt(*body)?;

                        self.env.pop_scope();

                        // If async, wrap result in a Promise
                        if is_async {
                            Ok(Value::Promise(Box::new(result)))
                        } else {
                            Ok(result)
                        }
                    }
                    Value::BuiltinFn(builtin_fn) => builtin_fn(&arg_vals),
                    Value::BoundMethod { object, method, .. } => {
                        // Call the bound method with the object as 'this'
                        method(&object, &arg_vals)
                    }
                    Value::UserMethod {
                        object,
                        params,
                        body,
                        ..
                    } => {
                        // Check argument count (exclude 'self' parameter which is already bound)
                        // The params includes 'self', but we don't include it in arg_vals
                        if params.len() != arg_vals.len() + 1 {
                            return Err(self.error(format!(
                                "Expected {} arguments (plus self), got {}",
                                params.len() - 1,
                                arg_vals.len()
                            )));
                        }

                        // Create new scope for method
                        self.env.push_scope();

                        // Bind 'self' to the object
                        self.env.set("self".to_string(), (*object).clone());

                        // Bind other parameters
                        for ((param_name, _), arg_val) in params.iter().skip(1).zip(arg_vals.iter())
                        {
                            self.env.set(param_name.clone(), arg_val.clone());
                        }

                        // Execute method body
                        let result = self.eval_stmt(*body)?;

                        self.env.pop_scope();

                        Ok(result)
                    }
                    Value::Closure {
                        params,
                        body,
                        captured_env,
                        ..
                    } => {
                        // Check argument count
                        if params.len() != arg_vals.len() {
                            return Err(self.error(format!(
                                "Expected {} arguments, got {}",
                                params.len(),
                                arg_vals.len()
                            )));
                        }

                        // Create new scope for closure
                        self.env.push_scope();

                        // First, restore the captured environment
                        for (name, value) in captured_env {
                            self.env.set(name, value);
                        }

                        // Then bind parameters (which can shadow captured variables)
                        for ((param_name, _), arg_val) in params.iter().zip(arg_vals.iter()) {
                            self.env.set(param_name.clone(), arg_val.clone());
                        }

                        // Execute closure body (which is an expression, not a statement)
                        let result = self.eval_expr(*body)?;

                        self.env.pop_scope();

                        Ok(result)
                    }

                    Value::Builtin(builtin_struct) => {
                        // This shouldn't happen - builtins aren't directly callable
                        Err(self.error(format!(
                            "Cannot call builtin struct '{}' directly. Use its methods instead.",
                            builtin_struct.name
                        )))
                    }
                    Value::EnumConstructor {
                        enum_name,
                        variant_name,
                        arity,
                    } => {
                        // Check argument count
                        if arg_vals.len() != arity {
                            return Err(self.error(format!(
                                "Enum variant {}.{} expects {} arguments, got {}",
                                enum_name,
                                variant_name,
                                arity,
                                arg_vals.len()
                            )));
                        }

                        // Construct the enum variant
                        Ok(Value::EnumVariant {
                            enum_name,
                            variant_name,
                            values: arg_vals,
                        })
                    }
                    _ => Err(self.error(format!("Cannot call value of type {:?}", func_val))),
                }
            }
            Expr::FieldAccess { object, field } => {
                // Special case: check if this is an enum variant access (e.g., Color.Red)
                if let Expr::Ident(ref type_name) = *object {
                    if let Some(variants) = self.enums.get(type_name) {
                        // Check if the field is a valid variant
                        for (variant_name, variant_types) in variants {
                            if variant_name == &field {
                                // Return the variant or a constructor function
                                if variant_types.is_none() {
                                    // Unit variant - return the value directly
                                    return Ok(Value::EnumVariant {
                                        enum_name: type_name.clone(),
                                        variant_name: field.clone(),
                                        values: vec![],
                                    });
                                } else {
                                    // Tuple variant - return a constructor
                                    let num_types = variant_types.as_ref().unwrap().len();
                                    return Ok(Value::EnumConstructor {
                                        enum_name: type_name.clone(),
                                        variant_name: field.clone(),
                                        arity: num_types,
                                    });
                                }
                            }
                        }
                        return Err(self.error(format!(
                            "Variant '{}' not found on enum '{}'",
                            field, type_name
                        )));
                    }
                }

                // Normal field access
                let obj_val = self.eval_expr(*object)?;
                match obj_val {
                    Value::Builtin(builtin_struct) => {
                        // Check if it's a field
                        if let Some(field_val) = builtin_struct.fields.get(&field) {
                            Ok(field_val.clone())
                        } else if let Some(method) = builtin_struct.methods.get(&field) {
                            // Return a bound method
                            Ok(Value::BoundMethod {
                                object: Box::new(Value::Builtin(builtin_struct.clone())),
                                method_name: field.clone(),
                                method: *method,
                            })
                        } else {
                            Err(self.error(format!(
                                "Field or method '{}' not found on builtin struct '{}'",
                                field, builtin_struct.name
                            )))
                        }
                    }
                    Value::Struct { fields, name } => {
                        // First check for fields
                        if let Some(field_val) = fields.get(&field) {
                            return Ok(field_val.clone());
                        }

                        // Then check for user-defined methods in impl blocks
                        if let Some(methods) = self.impl_methods.get(&name) {
                            if let Some((params, return_type, body, _)) = methods.get(&field) {
                                return Ok(Value::UserMethod {
                                    object: Box::new(Value::Struct {
                                        fields,
                                        name: name.clone(),
                                    }),
                                    method_name: field.clone(),
                                    params: params.clone(),
                                    return_type: return_type.clone(),
                                    body: body.clone(),
                                });
                            }
                        }

                        Err(self.error(format!(
                            "Field or method '{}' not found on struct '{}'",
                            field, name
                        )))
                    }
                    Value::Array(_) => {
                        // Handle array methods
                        use crate::runtime::builtins::array;
                        let array_builtin = array::create_array_builtin();
                        if let Some(method) = array_builtin.methods.get(&field) {
                            Ok(Value::BoundMethod {
                                object: Box::new(obj_val.clone()),
                                method_name: field.clone(),
                                method: *method,
                            })
                        } else {
                            Err(self.error(format!("Method '{}' not found on array", field)))
                        }
                    }
                    Value::String(_) => {
                        // Handle string methods
                        use crate::runtime::builtins::string;
                        let string_builtin = string::create_string_builtin();
                        if let Some(method) = string_builtin.methods.get(&field) {
                            Ok(Value::BoundMethod {
                                object: Box::new(obj_val.clone()),
                                method_name: field.clone(),
                                method: *method,
                            })
                        } else {
                            Err(self.error(format!("Method '{}' not found on string", field)))
                        }
                    }
                    Value::Module { exports, .. } => {
                        // Access module export
                        if let Some(value) = exports.get(&field) {
                            Ok(value.clone())
                        } else {
                            Err(self.error(format!("Export '{}' not found in module", field)))
                        }
                    }
                    _ => Err(self.error(format!(
                        "Cannot access field on value of type {:?}",
                        obj_val
                    ))),
                }
            }
            Expr::Block(stmts) => {
                self.env.push_scope();
                let mut last_value = Value::Unit;
                for stmt in stmts {
                    last_value = self.eval_stmt(stmt)?;
                }
                self.env.pop_scope();
                Ok(last_value)
            }
            Expr::UnaryOp { .. } => Err(self.error("Expression type not yet implemented")),
            Expr::Lambda {
                params,
                return_type,
                body,
            } => {
                // Create a closure by capturing the current environment
                let captured_env = self.env.capture_all();
                Ok(Value::Closure {
                    params,
                    return_type,
                    body,
                    captured_env,
                })
            }
            Expr::Await(expr) => {
                // Evaluate the expression (should be a Promise)
                let value = self.eval_expr(*expr)?;
                match value {
                    Value::Promise(result) => {
                        // Unlike JavaScript, our promises are already resolved
                        // since async functions execute immediately
                        Ok(*result)
                    }
                    _ => Err(self.error(format!("Cannot await non-promise value: {:?}", value))),
                }
            }
            Expr::Async(expr) => {
                // Evaluate the expression eagerly and wrap in a Promise
                // This represents "eager async" - starts execution immediately
                let result = self.eval_expr(*expr)?;
                Ok(Value::Promise(Box::new(result)))
            }
            Expr::Lazy(expr) => {
                // Create a lazy future that doesn't execute until awaited
                // For now, we'll treat it similar to a regular promise
                // In a full implementation, this would create an unevaluated thunk
                let result = self.eval_expr(*expr)?;
                Ok(Value::Promise(Box::new(result)))
            }
            Expr::ArrayLiteral(elements) => {
                let mut array_values = Vec::new();
                for element in elements {
                    array_values.push(self.eval_expr(element)?);
                }
                Ok(Value::Array(array_values))
            }
            Expr::StructLiteral { name, fields } => {
                let mut field_values = HashMap::new();
                for (field_name, field_expr) in fields {
                    let value = self.eval_expr(field_expr)?;
                    field_values.insert(field_name, value);
                }
                Ok(Value::Struct {
                    name,
                    fields: field_values,
                })
            }
            Expr::Index { array, index } => {
                let array_val = self.eval_expr(*array)?;
                let index_val = self.eval_expr(*index)?;

                // Check if this is a struct with a custom index method
                if let Value::Struct { name, .. } = &array_val {
                    if let Some(methods) = self.impl_methods.get(name) {
                        if let Some((params, _return_type, body, _trait_name)) =
                            methods.get("index")
                        {
                            // Found indexing trait implementation
                            if params.len() != 2 {
                                return Err(self.error(format!(
                                     "Indexing trait method 'index' should have 2 parameters (self, index), found {}",
                                     params.len()
                                 )));
                            }

                            self.env.push_scope();

                            // Bind self
                            self.env.set("self".to_string(), array_val.clone());

                            // Bind index (second param)
                            let index_param_name = &params[1].0;
                            self.env.set(index_param_name.clone(), index_val.clone());

                            // Execute body
                            let result = self.eval_stmt(*body.clone())?;

                            self.env.pop_scope();

                            return Ok(result);
                        }
                    }
                }

                call_index_trait(&array_val, &index_val)
            }
            Expr::TemplateLiteral { parts } => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        crate::parser::TemplatePart::Text(text) => {
                            result.push_str(&text);
                        }
                        crate::parser::TemplatePart::Expression(expr) => {
                            let value = self.eval_expr(expr)?;
                            result.push_str(&self.value_to_string(&value)?);
                        }
                    }
                }
                Ok(Value::String(result))
            }
            Expr::Match { expr, arms } => {
                let value = self.eval_expr(*expr)?;

                // Try each pattern arm until one matches
                for (pattern, body_expr) in arms {
                    if let Some(bindings) = self.match_pattern(&pattern, &value)? {
                        // Pattern matched, bind any captured variables and evaluate body expression
                        self.env.push_scope();
                        for (name, val) in bindings {
                            self.env.set(name, val);
                        }
                        let result = self.eval_expr(body_expr)?;
                        self.env.pop_scope();
                        return Ok(result);
                    }
                }

                // No pattern matched
                Err(self.error("Match expression did not match any pattern".to_string()))
            }
            Expr::Try(expr) => {
                // Error propagation operator (?)
                // Evaluate the expression, which should be a Result-like enum
                let value = self.eval_expr(*expr)?;

                // Check if it's an enum variant
                if let Value::EnumVariant {
                    enum_name: _,
                    variant_name,
                    values,
                } = &value
                {
                    // Check if this is an Err/Error variant
                    if variant_name.to_lowercase().contains("err") {
                        // This is an error - propagate it by returning the error variant
                        return Ok(value.clone());
                    }

                    // Check if this is an Ok/Some variant - unwrap it
                    if variant_name.to_lowercase().contains("ok")
                        || variant_name.to_lowercase().contains("some")
                    {
                        // Return the wrapped value (first element of tuple variant)
                        if values.len() == 1 {
                            return Ok(values[0].clone());
                        } else if values.is_empty() {
                            return Ok(Value::Unit);
                        } else {
                            // Multiple values - return as tuple (array for now)
                            return Ok(Value::Array(values.clone()));
                        }
                    }
                }

                // Not a Result/Option type - just return the value
                Ok(value)
            }
        }
    }

    fn value_to_string(&mut self, value: &Value) -> RuntimeResult<String> {
        if let Value::Struct { name, .. } = value {
            // Check for user-defined methods in impl blocks
            if let Some(methods) = self.impl_methods.get(name) {
                if let Some((_params, _return_type, body, _)) = methods.get("to_string") {
                    // Create new scope for method
                    self.env.push_scope();

                    // Bind 'self'
                    self.env.set("self".to_string(), value.clone());

                    // Execute method body
                    let result = self.eval_stmt(*body.clone())?;

                    self.env.pop_scope();

                    if let Value::String(s) = result {
                        return Ok(s);
                    }
                }
            }
        }
        Ok(value.to_string())
    }

    fn eval_binop(&mut self, op: &str, left: Value, right: Value) -> RuntimeResult<Value> {
        // Check for user-defined trait implementations
        if let Value::Struct {
            name: ref type_name,
            ..
        } = left
        {
            let method_name = match op {
                "+" => "add",
                "-" => "sub",
                "*" => "mul",
                "/" => "div",
                "&" => "bit_and",
                "|" => "bit_or",
                "^" => "bit_xor",
                "<<" => "shl",
                ">>" => "shr",
                ">" => "gt",
                ">=" => "ge",
                "<" => "lt",
                "<=" => "le",
                "==" => "eq",
                "!=" => "ne",
                _ => "",
            };

            if !method_name.is_empty() {
                // We need to clone type_name to avoid borrowing issues with self.impl_methods
                let type_name_clone = type_name.clone();
                if let Some(methods) = self.impl_methods.get(&type_name_clone) {
                    if let Some((params, _return_type, body, _trait_name)) =
                        methods.get(method_name)
                    {
                        // Found implementation

                        // Check arg count (should be 2: self, other)
                        if params.len() != 2 {
                            return Err(self.error(format!(
                                 "Trait method '{}' should have 2 parameters (self, other), found {}",
                                 method_name, params.len()
                             )));
                        }

                        self.env.push_scope();

                        // Bind self
                        self.env.set("self".to_string(), left.clone());

                        // Bind other (second param)
                        let other_param_name = &params[1].0;
                        self.env.set(other_param_name.clone(), right.clone());

                        // Execute body
                        let result = self.eval_stmt(*body.clone())?;

                        self.env.pop_scope();

                        return Ok(result);
                    }
                }
            }
        }

        // Use the trait-based system instead of matches
        match call_binop_trait(op, &left, &right) {
            Ok(v) => Ok(v),
            Err(e) => {
                // Provide better error message with type information
                let left_type = self.type_of(&left);
                let right_type = self.type_of(&right);
                let enhanced_msg =
                    format!("{} (left: {}, right: {})", e.message, left_type, right_type);
                Err(self.error(enhanced_msg))
            }
        }
    }

    /// Match a pattern against a value
    /// Returns Some(bindings) if the pattern matches, where bindings are variables to bind
    /// Returns None if the pattern doesn't match
    fn match_pattern(
        &mut self,
        pattern: &Expr,
        value: &Value,
    ) -> RuntimeResult<Option<HashMap<String, Value>>> {
        match pattern {
            // Literal patterns
            Expr::Number(n) => {
                if let Value::Number(v) = value {
                    Ok(if n == v { Some(HashMap::new()) } else { None })
                } else {
                    Ok(None)
                }
            }
            Expr::String(s) => {
                if let Value::String(v) = value {
                    Ok(if s == v { Some(HashMap::new()) } else { None })
                } else {
                    Ok(None)
                }
            }
            Expr::Boolean(b) => {
                if let Value::Boolean(v) = value {
                    Ok(if b == v { Some(HashMap::new()) } else { None })
                } else {
                    Ok(None)
                }
            }

            // Identifier pattern - binds the value to a variable
            Expr::Ident(name) => {
                if name == "_" {
                    // Wildcard pattern matches anything
                    Ok(Some(HashMap::new()))
                } else {
                    // Check if this is an enum variant (qualified name like Color.Red)
                    // If so, try to match it as a pattern
                    if let Some(enum_variant) = self.env.get(name) {
                        if let Value::EnumVariant { .. } = enum_variant {
                            // Match against the enum variant
                            Ok(if enum_variant == value {
                                Some(HashMap::new())
                            } else {
                                None
                            })
                        } else {
                            // Regular identifier binding
                            let mut bindings = HashMap::new();
                            bindings.insert(name.clone(), value.clone());
                            Ok(Some(bindings))
                        }
                    } else {
                        // Regular identifier binding
                        let mut bindings = HashMap::new();
                        bindings.insert(name.clone(), value.clone());
                        Ok(Some(bindings))
                    }
                }
            }

            // Field access pattern (e.g., Color.Red)
            Expr::FieldAccess { object, field } => {
                if let Expr::Ident(enum_name) = &**object {
                    // Check if value is an enum variant
                    if let Value::EnumVariant {
                        enum_name: val_enum,
                        variant_name,
                        values,
                    } = value
                    {
                        if enum_name == val_enum && field == variant_name {
                            // Unit variant matched
                            if values.is_empty() {
                                Ok(Some(HashMap::new()))
                            } else {
                                // Tuple variant needs destructuring
                                Ok(None)
                            }
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }

            // Call pattern for tuple variants (e.g., Color.RGB(r, g, b))
            Expr::Call { func, args } => {
                if let Expr::FieldAccess { object, field } = &**func {
                    if let Expr::Ident(enum_name) = &**object {
                        // Check if value is an enum variant
                        if let Value::EnumVariant {
                            enum_name: val_enum,
                            variant_name,
                            values,
                        } = value
                        {
                            if enum_name == val_enum && field == variant_name {
                                // Match tuple variant with destructuring
                                if args.len() == values.len() {
                                    let mut bindings = HashMap::new();
                                    for (pattern, val) in args.iter().zip(values.iter()) {
                                        if let Some(sub_bindings) =
                                            self.match_pattern(pattern, val)?
                                        {
                                            bindings.extend(sub_bindings);
                                        } else {
                                            return Ok(None);
                                        }
                                    }
                                    Ok(Some(bindings))
                                } else {
                                    Ok(None)
                                }
                            } else {
                                Ok(None)
                            }
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }

            _ => {
                // Unsupported pattern
                Err(self.error(format!("Unsupported pattern type: {:?}", pattern)))
            }
        }
    }

    /// Load and execute a module, returning its exports
    fn load_module(&mut self, path: &[String]) -> RuntimeResult<Value> {
        // Convert path to module identifier
        let module_id = path.join("::");

        // Extract module name from path
        // For "./math_utils" or "math", use base name
        // For "package::module", use last component
        let module_name = if path.len() == 1 {
            // Single path element - extract base name
            let path_str = &path[0];
            PathBuf::from(path_str)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(path_str)
                .to_string()
        } else {
            // Multiple components - use last one
            path.last().unwrap().clone()
        };

        // Check if module is already cached
        if let Some(exports) = self.module_cache.get(&module_id) {
            // Create module namespace object
            let module_value = Value::Module {
                name: module_name.clone(),
                exports: exports.clone(),
            };
            self.env.set(module_name, module_value);
            return Ok(Value::Unit);
        }

        // Resolve module path to file
        let file_path = self.resolve_module_path(path)?;

        // Read module source
        let source = std::fs::read_to_string(&file_path).map_err(|e| {
            RuntimeError::new(format!(
                "Failed to read module '{}': {}",
                file_path.display(),
                e
            ))
        })?;

        // Parse module
        let stream = InputStream::new(file_path.to_str().unwrap(), &source);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().map_err(|e| {
            RuntimeError::new(format!("Failed to parse module '{}': {}", module_id, e))
        })?;

        // Create new interpreter for module with fresh environment
        let mut module_interpreter = Interpreter::with_source(file_path.to_str().unwrap(), source)
            .with_features(self.enabled_features.iter().cloned().collect());

        // Execute module
        module_interpreter.eval_program(stmts)?;

        // Cache the module's exports
        let exports = module_interpreter.exports.clone();
        self.module_cache.insert(module_id.clone(), exports.clone());

        // Create module namespace object and add to environment
        let module_value = Value::Module {
            name: module_name.clone(),
            exports,
        };
        self.env.set(module_name, module_value);

        Ok(Value::Unit)
    }

    /// Resolve a module path to a file system path
    fn resolve_module_path(&self, path: &[String]) -> RuntimeResult<PathBuf> {
        // If path starts with ".", it's a relative import
        if path[0].starts_with('.') {
            // Relative to current file
            if let Some(current_path) = &self.source_path {
                let current_file = PathBuf::from(current_path);
                let current_dir = current_file
                    .parent()
                    .ok_or_else(|| RuntimeError::new("Cannot determine parent directory"))?;

                let relative_path = path[0].trim_start_matches("./");
                let mut module_path = current_dir.join(relative_path);

                // Try .lf extension
                module_path.set_extension("lf");
                if module_path.exists() {
                    return Ok(module_path);
                }

                // Try as directory with mod.lf
                let mut dir_path = current_dir.join(relative_path);
                dir_path.push("mod.lf");
                if dir_path.exists() {
                    return Ok(dir_path);
                }

                return Err(RuntimeError::new(format!(
                    "Module not found: {}",
                    path.join("::")
                )));
            } else {
                return Err(RuntimeError::new(
                    "Cannot resolve relative import without source path",
                ));
            }
        }

        // Otherwise, treat as package import
        // For now, look in current directory or stdlib
        let module_name = &path[0];

        // Check if it's a builtin module (already loaded)
        // For simplicity, just look for file in current directory
        let mut module_path = PathBuf::from(module_name);
        module_path.set_extension("lf");

        if module_path.exists() {
            return Ok(module_path);
        }

        Err(RuntimeError::new(format!(
            "Module not found: {}",
            path.join("::")
        )))
    }

    /// Get the runtime type name of a value
    fn type_of(&self, value: &Value) -> String {
        match value {
            Value::Unit => "unit".to_string(),
            Value::Number(_) => "num".to_string(),
            Value::String(_) => "str".to_string(),
            Value::Boolean(_) => "bool".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Function { .. } => "function".to_string(),
            Value::Closure { .. } => "closure".to_string(),
            Value::Struct { name, .. } => name.clone(),
            Value::Builtin(b) => b.name.clone(),
            Value::BuiltinFn(_) => "builtin_function".to_string(),
            Value::BoundMethod { .. } => "method".to_string(),
            Value::UserMethod { .. } => "method".to_string(),
            Value::Promise(_) => "promise".to_string(),
            Value::EnumVariant { enum_name, .. } => enum_name.clone(),
            Value::EnumConstructor { enum_name, .. } => format!("{}_constructor", enum_name),
            Value::Module { name, .. } => format!("module_{}", name),
        }
    }

    /// Check if a value matches an expected type name
    fn check_type(&self, value: &Value, expected_type: &str) -> bool {
        let actual_type = self.type_of(value);

        // "any" matches anything
        if expected_type == "any" {
            return true;
        }

        // Check for exact match
        if actual_type == expected_type {
            return true;
        }

        // Special cases
        match (expected_type, value) {
            // Array types
            ("array", Value::Array(_)) => true,
            // Function types
            ("function", Value::Function { .. }) => true,
            ("function", Value::Closure { .. }) => true,
            ("function", Value::BuiltinFn(_)) => true,
            _ => false,
        }
    }

    /// Create a runtime error with source context if available
    fn error(&self, message: impl Into<String>) -> RuntimeError {
        if let (Some(path), Some(source)) = (&self.source_path, &self.source_code) {
            RuntimeError::with_source_context(message, path.clone(), source.clone())
        } else {
            RuntimeError::new(message)
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;
    use crate::parser::{InputStream, Parser};

    #[test]
    fn test_eval_simple_expr() {
        let input = "2 + 3 * 4".to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let expr = parser.parse_expression().unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(expr).unwrap();

        assert_eq!(result, Value::Number(Decimal::from(14)));
    }

    #[test]
    fn test_eval_var_decl() {
        let input = "let x = 42;".to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("x"),
            Some(&Value::Number(Decimal::from(42)))
        );
    }

    #[test]
    fn test_trait_based_add() {
        let input = "let x = 5 + 3;".to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("x"),
            Some(&Value::Number(Decimal::from(8)))
        );
    }

    #[test]
    fn test_trait_based_sub() {
        let input = "let x = 10 - 3;".to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("x"),
            Some(&Value::Number(Decimal::from(7)))
        );
    }

    #[test]
    fn test_trait_based_mul() {
        let input = "let x = 4 * 5;".to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("x"),
            Some(&Value::Number(Decimal::from(20)))
        );
    }

    #[test]
    fn test_trait_based_div() {
        let input = "let x = 20 / 4;".to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("x"),
            Some(&Value::Number(Decimal::from(5)))
        );
    }

    #[test]
    fn test_string_concat_with_traits() {
        let input = r#"let x = "hello" + " world";"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("x"),
            Some(&Value::String("hello world".to_string()))
        );
    }

    #[test]
    fn test_invalid_operation_runtime_error() {
        let input = r#"let x = "hello" - "world";"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_program(stmts);

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Cannot subtract"));
    }

    #[test]
    fn test_runtime_error_with_source_context() {
        let input = "let x = 5;\nlet y = unknown;".to_string();
        let stream = InputStream::new("test.lf", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::with_source("test.lf", &input);
        let result = interpreter.eval_program(stmts);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("Variable 'unknown' not found"));

        // Verify that source context is available
        assert!(error.source.is_some());
        assert_eq!(error.path, Some("test.lf".to_string()));
    }

    #[test]
    fn test_runtime_error_diagnostic_trait() {
        use miette::Diagnostic;

        let input = "let x = 5;".to_string();
        let error = RuntimeError::with_source_context("Test error", "test.lf".to_string(), input);

        // Verify Diagnostic trait implementation
        assert!(error.source_code().is_some());
        assert!(error.labels().is_some());
        assert_eq!(error.severity(), Some(miette::Severity::Error));
    }

    #[test]
    fn test_builtin_term_exists() {
        let interpreter = Interpreter::new();
        let term = interpreter.env.get("term");

        assert!(term.is_some());
        if let Some(Value::Builtin(builtin)) = term {
            assert_eq!(builtin.name, "term");
        } else {
            panic!("term should be a Builtin value");
        }
    }

    #[test]
    fn test_function_declaration() {
        let input = r#"
            fn add(a: num, b: num) -> num {
                return a + b;
            }
            let result = add(5, 3);
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from(8)))
        );
    }

    #[test]
    fn test_async_function_returns_promise() {
        let input = r#"
            async fn fetch_data(url: str) -> str {
                return url;
            }
            let promise = fetch_data("test_url");
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        // Check that the promise exists
        if let Some(Value::Promise(value)) = interpreter.env.get("promise") {
            assert_eq!(**value, Value::String("test_url".to_string()));
        } else {
            panic!("Expected promise value");
        }
    }

    #[test]
    fn test_await_expression() {
        let input = r#"
            async fn get_number() -> num {
                return 42;
            }
            let result = await get_number();
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from(42)))
        );
    }

    #[test]
    fn test_async_function_executes_immediately() {
        let input = r#"
            async fn add(a: num, b: num) -> num {
                return a + b;
            }
            let result = await add(10, 20);
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from(30)))
        );
    }

    #[test]
    fn test_regular_function_call() {
        let input = r#"
            fn multiply(x: num, y: num) -> num {
                return x * y;
            }
            let result = multiply(6, 7);
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from(42)))
        );
    }

    #[test]
    fn test_term_println_method_call() {
        let input = r#"term.println("Hello, World!");"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_program(stmts);

        // Should succeed without error
        assert!(result.is_ok());
    }

    #[test]
    fn test_term_print_method_call() {
        let input = r#"term.print("Hello");"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_program(stmts);

        // Should succeed without error
        assert!(result.is_ok());
    }

    #[test]
    fn test_term_color_method_call() {
        let input = r#"term.color("red");"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_program(stmts);

        // Should succeed without error
        assert!(result.is_ok());
    }

    #[test]
    fn test_term_size_method_call() {
        let input = r#"let size = term.size();"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_program(stmts);

        // Should succeed without error
        assert!(result.is_ok());

        // Check that size is an array
        if let Some(Value::Array(arr)) = interpreter.env.get("size") {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("size should be an array");
        }
    }

    #[test]
    fn test_variable_reassignment() {
        let input = "let x = 5; x = 10;".to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("x"),
            Some(&Value::Number(Decimal::from(10)))
        );
    }

    #[test]
    fn test_variable_shadowing() {
        let input = "let x = 5; { let x = 10; } let y = x;".to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        // After the inner block ends, x should be back to 5
        assert_eq!(
            interpreter.env.get("y"),
            Some(&Value::Number(Decimal::from(5)))
        );
    }

    #[test]
    fn test_variable_reassignment_without_mut() {
        // Test that variables can be reassigned even without 'mut' keyword
        let input = "let z = 100; z = 200;".to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("z"),
            Some(&Value::Number(Decimal::from(200)))
        );
    }

    #[test]
    fn test_builtin_time_exists() {
        let interpreter = Interpreter::new();
        let time = interpreter.env.get("time");

        assert!(time.is_some());
        if let Some(Value::Builtin(builtin)) = time {
            assert_eq!(builtin.name, "time");
            assert!(builtin.methods.contains_key("sleep"));
            assert!(builtin.methods.contains_key("now"));
            assert!(builtin.methods.contains_key("format"));
        } else {
            panic!("time should be a Builtin value");
        }
    }

    #[test]
    fn test_time_sleep_returns_promise() {
        let input = r#"let promise = time.sleep(100);"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        // Check that the promise exists and contains Unit
        if let Some(Value::Promise(value)) = interpreter.env.get("promise") {
            assert_eq!(**value, Value::Unit);
        } else {
            panic!("Expected promise value from time.sleep()");
        }
    }

    #[test]
    fn test_time_now_returns_number() {
        let input = r#"let timestamp = time.now();"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        // Check that we got a number (timestamp)
        if let Some(Value::Number(_)) = interpreter.env.get("timestamp") {
            // Success - we got a number as expected
        } else {
            panic!("Expected number value from time.now()");
        }
    }

    // Math builtin tests
    #[test]
    fn test_math_round() {
        let input = r#"let result = math.round(3.7);"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from(4)))
        );
    }

    #[test]
    fn test_math_pow() {
        let input = r#"let result = math.pow(2, 3);"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from(8)))
        );
    }

    #[test]
    fn test_math_sqrt() {
        let input = r#"let result = math.sqrt(16);"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from(4)))
        );
    }

    #[test]
    fn test_math_min_max() {
        let input = r#"
            let min_val = math.min(5, 10);
            let max_val = math.max(5, 10);
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("min_val"),
            Some(&Value::Number(Decimal::from(5)))
        );
        assert_eq!(
            interpreter.env.get("max_val"),
            Some(&Value::Number(Decimal::from(10)))
        );
    }

    #[test]
    fn test_math_constants() {
        let input = r#"let pi = math.PI;"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        // PI should be defined
        assert!(interpreter.env.get("pi").is_some());
    }

    // String method tests
    #[test]
    fn test_string_split() {
        let input = r#"let result = "hello,world".split(",");"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        if let Some(Value::Array(arr)) = interpreter.env.get("result") {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::String("hello".to_string()));
            assert_eq!(arr[1], Value::String("world".to_string()));
        } else {
            panic!("result should be an array");
        }
    }

    #[test]
    fn test_string_trim() {
        let input = r#"let result = "  hello  ".trim();"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::String("hello".to_string()))
        );
    }

    #[test]
    fn test_string_to_upper_lower() {
        let input = r#"
            let upper = "hello".to_upper();
            let lower = "HELLO".to_lower();
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("upper"),
            Some(&Value::String("HELLO".to_string()))
        );
        assert_eq!(
            interpreter.env.get("lower"),
            Some(&Value::String("hello".to_string()))
        );
    }

    #[test]
    fn test_string_replace() {
        let input = r#"let result = "hello world".replace("world", "rust");"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::String("hello rust".to_string()))
        );
    }

    #[test]
    fn test_string_contains() {
        let input = r#"let result = "hello world".contains("world");"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(interpreter.env.get("result"), Some(&Value::Boolean(true)));
    }

    // Array collection method tests
    #[test]
    fn test_array_reverse() {
        let input = r#"let result = [1, 2, 3].reverse();"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        if let Some(Value::Array(arr)) = interpreter.env.get("result") {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Number(Decimal::from(3)));
            assert_eq!(arr[1], Value::Number(Decimal::from(2)));
            assert_eq!(arr[2], Value::Number(Decimal::from(1)));
        } else {
            panic!("result should be an array");
        }
    }

    #[test]
    fn test_array_flatten() {
        let input = r#"let result = [[1, 2], [3, 4]].flatten();"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        if let Some(Value::Array(arr)) = interpreter.env.get("result") {
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[0], Value::Number(Decimal::from(1)));
            assert_eq!(arr[3], Value::Number(Decimal::from(4)));
        } else {
            panic!("result should be an array");
        }
    }

    #[test]
    fn test_array_sum() {
        let input = r#"let result = [1, 2, 3, 4].sum();"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from(10)))
        );
    }

    #[test]
    fn test_array_average() {
        let input = r#"let result = [1, 2, 3, 4].average();"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from_f64_retain(2.5).unwrap()))
        );
    }

    #[test]
    fn test_array_unique() {
        let input = r#"let result = [1, 2, 2, 3, 3, 3].unique();"#.to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        if let Some(Value::Array(arr)) = interpreter.env.get("result") {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Number(Decimal::from(1)));
            assert_eq!(arr[1], Value::Number(Decimal::from(2)));
            assert_eq!(arr[2], Value::Number(Decimal::from(3)));
        } else {
            panic!("result should be an array");
        }
    }

    #[test]
    fn test_array_take_skip() {
        let input = r#"
            let take_result = [1, 2, 3, 4, 5].take(3);
            let skip_result = [1, 2, 3, 4, 5].skip(2);
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        if let Some(Value::Array(arr)) = interpreter.env.get("take_result") {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("take_result should be an array");
        }

        if let Some(Value::Array(arr)) = interpreter.env.get("skip_result") {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Number(Decimal::from(3)));
        } else {
            panic!("skip_result should be an array");
        }
    }

    #[test]
    fn test_to_string_trait() {
        use crate::runtime::traits::ToString;

        // Test Number
        let num = Value::Number(Decimal::from(42));
        assert_eq!(num.to_string(), "42");

        // Test String
        let str_val = Value::String("hello".to_string());
        assert_eq!(str_val.to_string(), "hello");

        // Test Boolean
        let bool_val = Value::Boolean(true);
        assert_eq!(bool_val.to_string(), "true");

        // Test Unit
        let unit_val = Value::Unit;
        assert_eq!(unit_val.to_string(), "()");

        // Test Array
        let arr_val = Value::Array(vec![
            Value::Number(Decimal::from(1)),
            Value::String("hello".to_string()),
            Value::Boolean(false),
        ]);
        assert_eq!(arr_val.to_string(), "[1, hello, false]");
    }

    #[test]
    fn test_array_join() {
        let input = r#"
            let result = [1, "hello", true].join(", ");
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::String("1, hello, true".to_string()))
        );
    }

    // Closure tests
    #[test]
    fn test_simple_closure() {
        let input = r#"
            let square = (x: num) => x * x;
            let result = square(5);
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from(25)))
        );
    }

    #[test]
    fn test_closure_with_captured_variable() {
        let input = r#"
            let multiplier = 3;
            let multiply_by = (x: num) => x * multiplier;
            let result = multiply_by(7);
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from(21)))
        );
    }

    #[test]
    fn test_closure_without_param_types() {
        let input = r#"
            let add_one = x => x + 1;
            let result = add_one(10);
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("result"),
            Some(&Value::Number(Decimal::from(11)))
        );
    }

    #[test]
    fn test_closure_arity_check() {
        let input = r#"
            let add = (a: num, b: num) => a + b;
            let result = add(5);
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_program(stmts);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("Expected 2 arguments, got 1"));
    }

    #[test]
    fn test_object_indexing() {
        let input = r#"
            def Person {
                name: str,
                age: num,
            }
            
            let person = Person { name: "Alice", age: 30 };
            let name_value = person["name"];
            let age_value = person["age"];
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("name_value"),
            Some(&Value::String("Alice".to_string()))
        );
        assert_eq!(
            interpreter.env.get("age_value"),
            Some(&Value::Number(Decimal::from(30)))
        );
    }

    #[test]
    fn test_object_indexing_with_dynamic_object() {
        // Test with a plain object (not a defined struct)
        let input = r#"
            let obj = Object { x: 10, y: 20 };
            let x_val = obj["x"];
            let y_val = obj["y"];
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("x_val"),
            Some(&Value::Number(Decimal::from(10)))
        );
        assert_eq!(
            interpreter.env.get("y_val"),
            Some(&Value::Number(Decimal::from(20)))
        );
    }

    #[test]
    fn test_object_indexing_missing_key() {
        let input = r#"
            let obj = Object { x: 10 };
            let missing = obj["missing_key"];
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_program(stmts);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("does not have property"));
    }

    #[test]
    fn test_array_indexing_with_number() {
        let input = r#"
            let arr = [10, 20, 30];
            let val = arr[1];
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("val"),
            Some(&Value::Number(Decimal::from(20)))
        );
    }

    #[test]
    fn test_string_indexing_with_number() {
        let input = r#"
            let s = "hello";
            let char = s[1];
        "#
        .to_string();
        let stream = InputStream::new("test", &input);
        let mut parser = Parser::new(stream);
        let stmts = parser.parse().unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.eval_program(stmts).unwrap();

        assert_eq!(
            interpreter.env.get("char"),
            Some(&Value::String("e".to_string()))
        );
    }
}

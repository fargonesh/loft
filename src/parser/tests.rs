use super::*;
use crate::parser::input_stream::InputStream;

#[test]
fn test_parse_var_decl() {
    let input = "let x = 42;".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::VarDecl { name, value, .. } => {
            assert_eq!(name, "x");
            assert!(matches!(value, Some(Expr::Number(_))));
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_parse_binary_expr() {
    let input = "2 + 3 * 4".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let expr = parser.parse_expression().unwrap();

    // Should parse as 2 + (3 * 4) due to precedence
    match expr {
        Expr::BinOp { op, left, right } => {
            assert_eq!(op, "+");
            assert!(matches!(*left, Expr::Number(_)));
            assert!(matches!(*right, Expr::BinOp { op, .. } if op == "*"));
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_if_statement() {
    let input = "if (x > 0) { return x; }".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let stmt = parser.parse_statement().unwrap();

    match stmt {
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            assert!(matches!(condition, Expr::BinOp { .. }));
            assert!(matches!(*then_branch, Stmt::Block(_)));
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_import() {
    let input = "learn \"std\";".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::ImportDecl { path } => {
            assert_eq!(path.len(), 1);
            assert_eq!(path[0], "std");
        }
        _ => panic!("Expected import declaration"),
    }
}

#[test]
fn test_parse_import_with_path() {
    let input = "learn \"project::value\";".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::ImportDecl { path } => {
            assert_eq!(path.len(), 2);
            assert_eq!(path[0], "project");
            assert_eq!(path[1], "value");
        }
        _ => panic!("Expected import declaration"),
    }
}

#[test]
fn test_parse_import_with_module_path() {
    let input = "learn \"project::module::value\";".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::ImportDecl { path } => {
            assert_eq!(path.len(), 3);
            assert_eq!(path[0], "project");
            assert_eq!(path[1], "module");
            assert_eq!(path[2], "value");
        }
        _ => panic!("Expected import declaration"),
    }
}

#[test]
fn test_parse_exported_function() {
    let input = "teach fn add(a: num, b: num) -> num { return a + b; }".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::FunctionDecl {
            name,
            is_exported,
            params,
            ..
        } => {
            assert_eq!(name, "add");
            assert_eq!(*is_exported, true);
            assert_eq!(params.len(), 2);
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_parse_struct_with_def() {
    let input = "def Person { name: str, age: num }".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::StructDecl { name, fields } => {
            assert_eq!(name, "Person");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected struct declaration"),
    }
}

#[test]
fn test_parse_lambda_simple() {
    let input = "let f = v => v;".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::VarDecl { name, value, .. } => {
            assert_eq!(name, "f");
            match value {
                Some(Expr::Lambda { params, .. }) => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0].0, "v");
                }
                _ => panic!("Expected lambda expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_parse_lambda_single_param_with_type() {
    let input = "let f = (v: num) => v;".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::VarDecl { name, value, .. } => {
            assert_eq!(name, "f");
            match value {
                Some(Expr::Lambda { params, .. }) => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0].0, "v");
                }
                _ => panic!("Expected lambda expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_parse_lambda_with_types() {
    let input = "let f = (v: num, a: num) => v + a;".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::VarDecl { name, value, .. } => {
            assert_eq!(name, "f");
            match value {
                Some(Expr::Lambda { params, .. }) => {
                    assert_eq!(params.len(), 2);
                    assert_eq!(params[0].0, "v");
                    assert_eq!(params[1].0, "a");
                }
                _ => panic!("Expected lambda expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_parse_trait_with_default_impl() {
    let input = r#"
        trait ToString {
            fn to_string(self) -> str;
            fn to_string_upper(self) -> str {
                return self;
            }
        }
    "#
    .to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::TraitDecl { name, methods } => {
            assert_eq!(name, "ToString");
            assert_eq!(methods.len(), 2);
            // First should be signature
            match &methods[0] {
                crate::parser::TraitMethod::Signature { name, .. } => {
                    assert_eq!(name, "to_string");
                }
                _ => panic!("Expected trait method signature"),
            }
            // Second should be default impl
            match &methods[1] {
                crate::parser::TraitMethod::Default { name, .. } => {
                    assert_eq!(name, "to_string_upper");
                }
                _ => panic!("Expected trait method default impl"),
            }
        }
        _ => panic!("Expected trait declaration"),
    }
}

#[test]
fn test_parse_await_expr() {
    let input = "let result = await future;".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::VarDecl { name, value, .. } => {
            assert_eq!(name, "result");
            match value {
                Some(Expr::Await(expr)) => match &**expr {
                    Expr::Ident(n) => assert_eq!(n, "future"),
                    _ => panic!("Expected identifier in await"),
                },
                _ => panic!("Expected await expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_parse_async_expr() {
    let input = "let promise = async fetch_data();".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::VarDecl { name, value, .. } => {
            assert_eq!(name, "promise");
            match value {
                Some(Expr::Async(expr)) => match &**expr {
                    Expr::Call { func, .. } => match &**func {
                        Expr::Ident(n) => assert_eq!(n, "fetch_data"),
                        _ => panic!("Expected function identifier"),
                    },
                    _ => panic!("Expected function call in async"),
                },
                _ => panic!("Expected async expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_parse_lazy_expr() {
    let input = "let future = lazy expensive_computation();".to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 1);

    match &result[0] {
        Stmt::VarDecl { name, value, .. } => {
            assert_eq!(name, "future");
            match value {
                Some(Expr::Lazy(expr)) => match &**expr {
                    Expr::Call { func, .. } => match &**func {
                        Expr::Ident(n) => assert_eq!(n, "expensive_computation"),
                        _ => panic!("Expected function identifier"),
                    },
                    _ => panic!("Expected function call in lazy"),
                },
                _ => panic!("Expected lazy expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_parse_await_async_and_lazy() {
    let input = r#"
        let eager_result = await async fetch_data();
        let lazy_result = await lazy compute_value();
    "#
    .to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 2);

    // Test await with async
    match &result[0] {
        Stmt::VarDecl { name, value, .. } => {
            assert_eq!(name, "eager_result");
            match value {
                Some(Expr::Await(expr)) => match &**expr {
                    Expr::Async(async_expr) => match &**async_expr {
                        Expr::Call { func, .. } => match &**func {
                            Expr::Ident(n) => assert_eq!(n, "fetch_data"),
                            _ => panic!("Expected function identifier"),
                        },
                        _ => panic!("Expected function call in async"),
                    },
                    _ => panic!("Expected async expression in await"),
                },
                _ => panic!("Expected await expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }

    // Test await with lazy
    match &result[1] {
        Stmt::VarDecl { name, value, .. } => {
            assert_eq!(name, "lazy_result");
            match value {
                Some(Expr::Await(expr)) => match &**expr {
                    Expr::Lazy(lazy_expr) => match &**lazy_expr {
                        Expr::Call { func, .. } => match &**func {
                            Expr::Ident(n) => assert_eq!(n, "compute_value"),
                            _ => panic!("Expected function identifier"),
                        },
                        _ => panic!("Expected function call in lazy"),
                    },
                    _ => panic!("Expected lazy expression in await"),
                },
                _ => panic!("Expected await expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_async_fn_vs_async_expr() {
    let input = r#"
        async fn fetch_data() -> str { return "data"; }
        let promise = async some_function();
    "#
    .to_string();
    let stream = InputStream::new("test", &input);
    let mut parser = Parser::new(stream);

    let result = parser.parse().unwrap();
    assert_eq!(result.len(), 2);

    // First should be async function declaration
    match &result[0] {
        Stmt::FunctionDecl { name, is_async, .. } => {
            assert_eq!(name, "fetch_data");
            assert_eq!(*is_async, true);
        }
        _ => panic!("Expected async function declaration"),
    }

    // Second should be variable declaration with async expression
    match &result[1] {
        Stmt::VarDecl { name, value, .. } => {
            assert_eq!(name, "promise");
            match value {
                Some(Expr::Async(_)) => {
                    // Success - we correctly parsed async expression
                }
                _ => panic!("Expected async expression"),
            }
        }
        _ => panic!("Expected variable declaration with async expression"),
    }
}

#[test]
fn test_fn_keyword_parsing() {
    let source = "fn factorial(n: num) -> num { return n; }".to_string();
    let input = InputStream::new("test", &source);
    let mut parser = Parser::new(input);
    let stmts = parser.parse().unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_parse_call_with_binary_op_arg() {
    let source = "term.println(\"Check: \" + (10 * 20));".to_string();
    let input = InputStream::new("test", &source);
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(stmts) => assert_eq!(stmts.len(), 1),
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_mixed_expression() {
    let source = "term.println(\"Hello\"); term.println(\"Check\" + (10));".to_string();
    let input = InputStream::new("test", &source);
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(stmts) => assert_eq!(stmts.len(), 2),
        Err(e) => panic!("Parse failed: {}", e),
    }
}

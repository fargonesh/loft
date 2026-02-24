pub mod input_stream;
pub mod token_stream;

use input_stream::{Error, Result};
use rust_decimal::Decimal;
use token_stream::{Token, TokenStream};

// Re-export commonly used items
pub use input_stream::InputStream;
pub use token_stream::Token as TokenType;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Number(Decimal),
    Ident(String),
    String(String),
    Boolean(bool),
    BinOp {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: String,
        expr: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    ArrayLiteral(Vec<Expr>),
    StructLiteral {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Lambda {
        params: Vec<(String, Option<Type>)>,
        return_type: Option<Type>,
        body: Box<Expr>,
    },
    Block(Vec<Stmt>),
    Await(Box<Expr>),
    Async(Box<Expr>), // Eager async expression
    Lazy(Box<Expr>),  // Lazy async expression
    TemplateLiteral {
        parts: Vec<TemplatePart>,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<(Expr, Expr)>, // pattern => expression
    },
    Try(Box<Expr>), // Error propagation with ?
}

#[derive(Clone, Debug, PartialEq)]
pub enum TemplatePart {
    Text(String),
    Expression(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Named(String),
    Generic {
        base: String,
        type_args: Vec<Type>,
    },
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    ImportDecl {
        path: Vec<String>, // e.g., ["project", "module", "value"]
    },
    VarDecl {
        name: String,
        var_type: Option<Type>,
        mutable: bool,
        value: Option<Expr>,
    },
    ConstDecl {
        name: String,
        const_type: Option<Type>,
        value: Expr,
    },
    FunctionDecl {
        name: String,
        type_params: Vec<String>,
        params: Vec<(String, Type)>,
        return_type: Option<Type>,
        body: Box<Stmt>,
        is_async: bool,
        is_exported: bool,
    },
    AttrStmt {
        attr: Attribute,
        stmt: Box<Stmt>,
    },
    StructDecl {
        name: String,
        fields: Vec<(String, Type)>,
    },
    ImplBlock {
        type_name: String,
        trait_name: Option<String>,
        methods: Vec<Stmt>,
    },
    TraitDecl {
        name: String,
        methods: Vec<TraitMethod>,
    },
    EnumDecl {
        name: String,
        variants: Vec<(String, Option<Vec<Type>>)>,
    },
    Assign {
        name: String,
        value: Expr,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    For {
        var: String,
        iterable: Expr,
        body: Box<Stmt>,
    },
    Match {
        expr: Expr,
        arms: Vec<(Expr, Stmt)>,
    },
    Return(Option<Expr>),
    Break,
    Continue,
    Expr(Expr),
    Block(Vec<Stmt>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum TraitMethod {
    Signature {
        name: String,
        params: Vec<(String, Type)>,
        return_type: Type,
    },
    Default {
        name: String,
        params: Vec<(String, Type)>,
        return_type: Type,
        body: Box<Stmt>,
    },
}

pub struct Parser<'a> {
    tokens: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: InputStream<'a>) -> Self {
        Self {
            tokens: TokenStream::new(input),
        }
    }

    // Generic parsing utilities
    fn peek(&mut self) -> Result<Option<Token>> {
        // If there's a token in the buffer, return it without consuming
        if !self.tokens.buffer.is_empty() {
            return Ok(Some(self.tokens.buffer[0].clone()));
        }

        // Otherwise, parse the next token and put it in the buffer
        let token_opt = self.tokens.parse_next()?;
        if let Some(ref token) = token_opt {
            self.tokens.buffer.insert(0, token.clone());
        }
        Ok(token_opt)
    }

    fn next(&mut self) -> Result<Option<Token>> {
        match self.tokens.next() {
            Some(Ok(token)) => Ok(Some(token)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    fn expect_punct(&mut self, punct: &str) -> Result<()> {
        let token_opt = self.next()?;
        match token_opt {
            Some(Token::Punct(p)) if p == punct => Ok(()),
            Some(token) => {
                let error_msg = format!("Expected '{}' but got {}", punct, token);
                Err(self.tokens.croak(error_msg, None))
            }
            None => {
                let error_msg = format!("Expected '{}' but got EOF", punct);
                Err(self.tokens.croak(error_msg, None))
            }
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<()> {
        let token_opt = self.next()?;
        match token_opt {
            Some(Token::Keyword(k)) if k == keyword => Ok(()),
            Some(token) => {
                let error_msg = format!("Expected keyword '{}' but got {}", keyword, token);
                Err(self.tokens.croak(error_msg, None))
            }
            None => {
                let error_msg = format!("Expected keyword '{}' but got EOF", keyword);
                Err(self.tokens.croak(error_msg, None))
            }
        }
    }

    fn is_punct(&self, token: &Token, punct: &str) -> bool {
        matches!(token, Token::Punct(p) if p == punct)
    }

    fn is_keyword(&self, token: &Token, keyword: &str) -> bool {
        matches!(token, Token::Keyword(k) if k == keyword)
    }

    fn is_op(&self, token: &Token, op: &str) -> bool {
        matches!(token, Token::Op(o) if o == op)
    }

    // Main parsing functions
    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while let Some(_) = self.peek()? {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    pub fn parse_recoverable(&mut self) -> (Vec<Stmt>, Vec<Error>) {
        let mut statements = Vec::new();
        let mut errors = Vec::new();

        loop {
            match self.peek() {
                Ok(Some(_)) => match self.parse_statement() {
                    Ok(stmt) => statements.push(stmt),
                    Err(err) => {
                        errors.push(err);
                        self.synchronize();
                    }
                },
                Ok(None) => break,
                Err(err) => {
                    errors.push(err);
                    break;
                }
            }
        }

        (statements, errors)
    }

    fn synchronize(&mut self) {
        // Advance until we find a statement boundary
        let _ = self.next(); // Consume the token that caused the error

        while let Ok(Some(token)) = self.peek() {
            if self.is_punct(&token, ";") {
                let _ = self.next();
                return;
            }

            // Check for keywords that start statements
            if let Token::Keyword(k) = &token {
                match k.as_str() {
                    "fn" | "let" | "const" | "if" | "while" | "for" | "return" | "teach"
                    | "learn" | "def" | "impl" | "trait" => return,
                    _ => {}
                }
            }

            let _ = self.next();
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        if let Some(token) = self.peek()? {
            match token {
                Token::Punct(p) if p == "#" => {
                    self.next()?; // consume #
                    self.parse_attribute_statement()
                }
                Token::Keyword(k) if k == "let" => self.parse_var_decl(false),
                Token::Keyword(k) if k == "mut" => {
                    self.next()?; // consume mut
                    self.parse_var_decl(true)
                }
                Token::Keyword(k) if k == "const" => self.parse_const_decl(),
                Token::Keyword(k) if k == "fn" => self.parse_function_decl(false, false),
                Token::Keyword(k) if k == "teach" => {
                    self.next()?; // consume 'teach'
                    self.parse_function_decl(false, true)
                }
                Token::Keyword(k) if k == "async" => {
                    self.next()?; // consume 'async'
                                  // Check if this is 'async fn' or 'async <expr>'
                    if let Some(Token::Keyword(k)) = self.peek()? {
                        if k == "fn" {
                            // This is 'async fn'
                            self.parse_function_decl(true, false)
                        } else {
                            // This is 'async <expr>' - put back the async token and parse as expression
                            self.tokens.push_back(Token::Keyword("async".to_string()));
                            let expr = self.parse_expression()?;
                            self.maybe_consume_semicolon();
                            Ok(Stmt::Expr(expr))
                        }
                    } else {
                        // This is 'async <expr>' - put back the async token and parse as expression
                        self.tokens.push_back(Token::Keyword("async".to_string()));
                        let expr = self.parse_expression()?;
                        self.maybe_consume_semicolon();
                        Ok(Stmt::Expr(expr))
                    }
                }
                Token::Keyword(k) if k == "def" => self.parse_struct_decl(),
                Token::Keyword(k) if k == "enum" => self.parse_enum_decl(),
                Token::Keyword(k) if k == "trait" => self.parse_trait_decl(),
                Token::Keyword(k) if k == "impl" => self.parse_impl_block(),
                Token::Keyword(k) if k == "learn" => self.parse_import_statement(),
                Token::Keyword(k) if k == "if" => self.parse_if_statement(),
                Token::Keyword(k) if k == "while" => self.parse_while_statement(),
                Token::Keyword(k) if k == "for" => self.parse_for_statement(),
                Token::Keyword(k) if k == "match" => self.parse_match_statement(),
                Token::Keyword(k) if k == "return" => self.parse_return_statement(),
                Token::Keyword(k) if k == "break" => {
                    self.next()?;
                    self.maybe_consume_semicolon();
                    Ok(Stmt::Break)
                }
                Token::Keyword(k) if k == "continue" => {
                    self.next()?;
                    self.maybe_consume_semicolon();
                    Ok(Stmt::Continue)
                }
                Token::Punct(p) if p == "{" => self.parse_block_statement(),
                _ => {
                    // Try to detect assignment: identifier = expression
                    // We need to look ahead to distinguish assignment from expression
                    if let Token::Ident(name) = &token {
                        // Peek ahead to see if next token is '='
                        // Save the current token for later if needed
                        let name_clone = name.clone();
                        self.next()?; // consume the identifier

                        if let Some(Token::Op(op)) = self.peek()? {
                            if op == "=" {
                                // This is an assignment statement
                                self.next()?; // consume '='
                                let value = self.parse_expression()?;
                                self.maybe_consume_semicolon();
                                return Ok(Stmt::Assign {
                                    name: name_clone,
                                    value,
                                });
                            }
                        }

                        // Not an assignment, put the identifier back and parse as expression
                        self.tokens.push_back(Token::Ident(name_clone));
                    }

                    // Parse as expression (token will be consumed by parse_expression)
                    let expr = self.parse_expression()?;
                    self.maybe_consume_semicolon();
                    Ok(Stmt::Expr(expr))
                }
            }
        } else {
            Err(self
                .tokens
                .croak("Unexpected end of input".to_string(), None))
        }
    }

    fn parse_attribute_statement(&mut self) -> Result<Stmt> {
        self.expect_punct("[")?;

        let name = match self.next()? {
            Some(Token::Ident(i)) => i,
            Some(token) => {
                return Err(self
                    .tokens
                    .croak(format!("Expected attribute name but got {}", token), None))
            }
            None => {
                return Err(self
                    .tokens
                    .croak("Expected attribute name but got EOF".to_string(), None))
            }
        };

        let mut args = Vec::new();
        if let Some(token) = self.peek()? {
            if self.is_punct(&token, "(") {
                self.next()?; // consume '('
                loop {
                    if let Some(token) = self.peek()? {
                        if self.is_punct(&token, ")") {
                            self.next()?;
                            break;
                        }
                    }
                    args.push(self.parse_expression()?);
                    if let Some(token) = self.peek()? {
                        if self.is_punct(&token, ",") {
                            self.next()?;
                        } else if self.is_punct(&token, ")") {
                            self.next()?;
                            break;
                        } else {
                            return Err(self.tokens.croak(
                                format!("Expected ',' or ')' in attribute args but got {}", token),
                                None,
                            ));
                        }
                    } else {
                        return Err(self.tokens.croak(
                            "Expected ',' or ')' in attribute args but got EOF".to_string(),
                            None,
                        ));
                    }
                }
            }
        }

        self.expect_punct("]")?;
        let stmt = self.parse_statement()?;
        Ok(Stmt::AttrStmt {
            attr: Attribute { name, args },
            stmt: Box::new(stmt),
        })
    }

    fn parse_var_decl(&mut self, mutable: bool) -> Result<Stmt> {
        // Expect 'let'
        let keyword_token = self.next()?;
        if !matches!(keyword_token, Some(Token::Keyword(ref k)) if k == "let") {
            return Err(self.tokens.croak("Expected 'let'".to_string(), None));
        }

        let name_token = self.next()?;
        let name = match name_token {
            Some(Token::Ident(name)) => name,
            Some(token) => {
                let error_msg = format!("Expected identifier but got {}", token);
                return Err(self.tokens.croak(error_msg, None));
            }
            None => {
                return Err(self
                    .tokens
                    .croak("Expected identifier but got EOF".to_string(), None))
            }
        };

        // Check for type annotation
        let var_type = if let Some(Token::Punct(p)) = self.peek()? {
            if p == ":" {
                self.next()?; // consume ':'
                Some(self.parse_type()?)
            } else {
                None
            }
        } else {
            None
        };

        // Check for initialization
        let value = if let Some(Token::Op(op)) = self.peek()? {
            if op == "=" {
                self.next()?; // consume '='
                Some(self.parse_expression()?)
            } else {
                None
            }
        } else {
            None
        };

        self.maybe_consume_semicolon();
        Ok(Stmt::VarDecl {
            name,
            var_type,
            mutable,
            value,
        })
    }

    fn parse_const_decl(&mut self) -> Result<Stmt> {
        self.expect_keyword("const")?;

        let name_token = self.next()?;
        let name = match name_token {
            Some(Token::Ident(name)) => name,
            Some(token) => {
                let error_msg = format!("Expected identifier but got {}", token);
                return Err(self.tokens.croak(error_msg, None));
            }
            None => {
                return Err(self
                    .tokens
                    .croak("Expected identifier but got EOF".to_string(), None))
            }
        };

        // Check for type annotation
        let const_type = if let Some(Token::Punct(p)) = self.peek()? {
            if p == ":" {
                self.next()?; // consume ':'
                Some(self.parse_type()?)
            } else {
                None
            }
        } else {
            None
        };

        // Constants must have a value
        self.expect_op("=")?;
        let value = self.parse_expression()?;

        self.maybe_consume_semicolon();
        Ok(Stmt::ConstDecl {
            name,
            const_type,
            value,
        })
    }

    fn parse_type(&mut self) -> Result<Type> {
        let name_token = self.next()?;
        let name = match name_token {
            Some(Token::Ident(name)) => name,
            Some(token) => {
                let error_msg = format!("Expected type name but got {}", token);
                return Err(self.tokens.croak(error_msg, None));
            }
            None => {
                return Err(self
                    .tokens
                    .croak("Expected type name but got EOF".to_string(), None))
            }
        };

        // Check for generic type arguments
        if let Some(Token::Op(op)) = self.peek()? {
            if op == "<" {
                self.next()?; // consume '<'
                let mut type_args = Vec::new();

                loop {
                    type_args.push(self.parse_type()?);

                    if let Some(token) = self.peek()? {
                        if self.is_punct(&token, ",") {
                            self.next()?; // consume ','
                        } else if self.is_op(&token, ">") {
                            self.next()?; // consume '>'
                            break;
                        } else {
                            return Err(self
                                .tokens
                                .croak("Expected ',' or '>' in generic type".to_string(), None));
                        }
                    } else {
                        return Err(self
                            .tokens
                            .croak("Unexpected EOF in generic type".to_string(), None));
                    }
                }

                return Ok(Type::Generic {
                    base: name,
                    type_args,
                });
            }
        }

        Ok(Type::Named(name))
    }

    fn parse_if_statement(&mut self) -> Result<Stmt> {
        self.expect_keyword("if")?;
        self.expect_punct("(")?;
        let condition = self.parse_expression()?;
        self.expect_punct(")")?;

        let then_branch = Box::new(self.parse_statement()?);

        let else_branch = if let Some(token) = self.peek()? {
            if self.is_keyword(&token, "else") {
                self.next()?; // consume 'else'
                Some(Box::new(self.parse_statement()?))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Stmt> {
        self.expect_keyword("while")?;
        self.expect_punct("(")?;
        let condition = self.parse_expression()?;
        self.expect_punct(")")?;
        let body = Box::new(self.parse_statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn parse_return_statement(&mut self) -> Result<Stmt> {
        self.expect_keyword("return")?;

        // Check if there's an expression to return
        let value = if let Some(token) = self.peek()? {
            if self.is_punct(&token, ";") {
                None
            } else {
                Some(self.parse_expression()?)
            }
        } else {
            None
        };

        self.maybe_consume_semicolon();
        Ok(Stmt::Return(value))
    }

    fn parse_import_statement(&mut self) -> Result<Stmt> {
        self.expect_keyword("learn")?;

        let module_token = self.next()?;
        let module_str = match module_token {
            Some(Token::String(s)) => s,
            Some(token) => {
                return Err(self.tokens.croak(
                    format!("Expected string literal after 'learn' but got {}", token),
                    None,
                ))
            }
            None => {
                return Err(self.tokens.croak(
                    "Expected string literal after 'learn' but got EOF".to_string(),
                    None,
                ))
            }
        };

        // Parse the module path, splitting by "::"
        let path: Vec<String> = module_str.split("::").map(|s| s.to_string()).collect();

        if path.is_empty() {
            return Err(self
                .tokens
                .croak("Import path cannot be empty".to_string(), None));
        }

        self.maybe_consume_semicolon();
        Ok(Stmt::ImportDecl { path })
    }

    fn parse_function_decl(&mut self, is_async: bool, is_exported: bool) -> Result<Stmt> {
        self.expect_keyword("fn")?;

        let name_token = self.next()?;
        let name = match name_token {
            Some(Token::Ident(name)) => name,
            Some(token) => {
                return Err(self
                    .tokens
                    .croak(format!("Expected function name but got {}", token), None))
            }
            None => {
                return Err(self
                    .tokens
                    .croak("Expected function name but got EOF".to_string(), None))
            }
        };

        // Parse optional type parameters
        let mut type_params = Vec::new();
        if let Some(Token::Op(op)) = self.peek()? {
            if op == "<" {
                self.next()?; // consume '<'
                loop {
                    let param_token = self.next()?;
                    if let Some(Token::Ident(param)) = param_token {
                        type_params.push(param);
                    } else {
                        return Err(self
                            .tokens
                            .croak("Expected type parameter name".to_string(), None));
                    }

                    if let Some(token) = self.peek()? {
                        if self.is_punct(&token, ",") {
                            self.next()?; // consume ','
                        } else if self.is_op(&token, ">") {
                            self.next()?; // consume '>'
                            break;
                        } else {
                            return Err(self.tokens.croak(
                                "Expected ',' or '>' in type parameters".to_string(),
                                None,
                            ));
                        }
                    }
                }
            }
        }

        // Parse parameters
        self.expect_punct("(")?;
        let mut params = Vec::new();

        while let Some(token) = self.peek()? {
            if self.is_punct(&token, ")") {
                break;
            }

            let param_name = match self.next()? {
                Some(Token::Ident(name)) => name,
                Some(token) => {
                    return Err(self
                        .tokens
                        .croak(format!("Expected parameter name but got {}", token), None))
                }
                None => {
                    return Err(self
                        .tokens
                        .croak("Expected parameter name but got EOF".to_string(), None))
                }
            };

            // Special case: 'self' parameter may not need type annotation in impl blocks
            // But can have one if explicitly specified
            let param_type = if param_name == "self" {
                // Check if there's a ':' following self
                if let Some(Token::Punct(p)) = self.peek()? {
                    if p == ":" {
                        // Has type annotation, parse it
                        self.next()?; // consume ':'
                        self.parse_type()?
                    } else {
                        // No type annotation, use placeholder
                        crate::parser::Type::Named("Self".to_string())
                    }
                } else {
                    // No type annotation, use placeholder
                    crate::parser::Type::Named("Self".to_string())
                }
            } else {
                self.expect_punct(":")?;
                self.parse_type()?
            };

            params.push((param_name, param_type));

            if let Some(token) = self.peek()? {
                if self.is_punct(&token, ",") {
                    self.next()?; // consume ','
                }
            }
        }

        self.expect_punct(")")?;

        // Parse optional return type
        let return_type = if let Some(Token::Op(op)) = self.peek()? {
            if op == "->" {
                self.next()?; // consume '->'
                Some(self.parse_type()?)
            } else {
                None
            }
        } else {
            None
        };

        // Parse body
        let body = Box::new(self.parse_block_statement()?);

        Ok(Stmt::FunctionDecl {
            name,
            type_params,
            params,
            return_type,
            body,
            is_async,
            is_exported,
        })
    }

    fn parse_struct_decl(&mut self) -> Result<Stmt> {
        self.expect_keyword("def")?;

        let name_token = self.next()?;
        let name = match name_token {
            Some(Token::Ident(name)) => name,
            Some(token) => {
                return Err(self
                    .tokens
                    .croak(format!("Expected struct name but got {}", token), None))
            }
            None => {
                return Err(self
                    .tokens
                    .croak("Expected struct name but got EOF".to_string(), None))
            }
        };

        self.expect_punct("{")?;
        let mut fields = Vec::new();

        while let Some(token) = self.peek()? {
            if self.is_punct(&token, "}") {
                break;
            }

            let field_name = match self.next()? {
                Some(Token::Ident(name)) => name,
                Some(token) => {
                    return Err(self
                        .tokens
                        .croak(format!("Expected field name but got {}", token), None))
                }
                None => {
                    return Err(self
                        .tokens
                        .croak("Expected field name but got EOF".to_string(), None))
                }
            };

            self.expect_punct(":")?;
            let field_type = self.parse_type()?;

            fields.push((field_name, field_type));

            if let Some(token) = self.peek()? {
                if self.is_punct(&token, ",") {
                    self.next()?; // consume ','
                }
            }
        }

        self.expect_punct("}")?;

        Ok(Stmt::StructDecl { name, fields })
    }

    fn parse_enum_decl(&mut self) -> Result<Stmt> {
        self.expect_keyword("enum")?;

        let name_token = self.next()?;
        let name = match name_token {
            Some(Token::Ident(name)) => name,
            Some(token) => {
                return Err(self
                    .tokens
                    .croak(format!("Expected enum name but got {}", token), None))
            }
            None => {
                return Err(self
                    .tokens
                    .croak("Expected enum name but got EOF".to_string(), None))
            }
        };

        self.expect_punct("{")?;
        let mut variants = Vec::new();

        while let Some(token) = self.peek()? {
            if self.is_punct(&token, "}") {
                break;
            }

            let variant_name = match self.next()? {
                Some(Token::Ident(name)) => name,
                Some(token) => {
                    return Err(self
                        .tokens
                        .croak(format!("Expected variant name but got {}", token), None))
                }
                None => {
                    return Err(self
                        .tokens
                        .croak("Expected variant name but got EOF".to_string(), None))
                }
            };

            // Check for tuple variant
            let types = if let Some(token) = self.peek()? {
                if self.is_punct(&token, "(") {
                    self.next()?; // consume '('
                    let mut variant_types = Vec::new();

                    while let Some(token) = self.peek()? {
                        if self.is_punct(&token, ")") {
                            break;
                        }

                        variant_types.push(self.parse_type()?);

                        if let Some(token) = self.peek()? {
                            if self.is_punct(&token, ",") {
                                self.next()?; // consume ','
                            }
                        }
                    }

                    self.expect_punct(")")?;
                    Some(variant_types)
                } else {
                    None
                }
            } else {
                None
            };

            variants.push((variant_name, types));

            if let Some(token) = self.peek()? {
                if self.is_punct(&token, ",") {
                    self.next()?; // consume ','
                }
            }
        }

        self.expect_punct("}")?;

        Ok(Stmt::EnumDecl { name, variants })
    }

    fn parse_trait_decl(&mut self) -> Result<Stmt> {
        self.expect_keyword("trait")?;

        let name_token = self.next()?;
        let name = match name_token {
            Some(Token::Ident(name)) => name,
            Some(token) => {
                return Err(self
                    .tokens
                    .croak(format!("Expected trait name but got {}", token), None))
            }
            None => {
                return Err(self
                    .tokens
                    .croak("Expected trait name but got EOF".to_string(), None))
            }
        };

        self.expect_punct("{")?;
        let mut methods = Vec::new();

        while let Some(token) = self.peek()? {
            if self.is_punct(&token, "}") {
                break;
            }

            self.expect_keyword("fn")?;

            let method_name = match self.next()? {
                Some(Token::Ident(name)) => name,
                Some(token) => {
                    return Err(self
                        .tokens
                        .croak(format!("Expected method name but got {}", token), None))
                }
                None => {
                    return Err(self
                        .tokens
                        .croak("Expected method name but got EOF".to_string(), None))
                }
            };

            self.expect_punct("(")?;
            let mut params = Vec::new();

            while let Some(token) = self.peek()? {
                if self.is_punct(&token, ")") {
                    break;
                }

                let param_name = match self.next()? {
                    Some(Token::Ident(name)) => name,
                    Some(token) => {
                        return Err(self
                            .tokens
                            .croak(format!("Expected parameter name but got {}", token), None))
                    }
                    None => {
                        return Err(self
                            .tokens
                            .croak("Expected parameter name but got EOF".to_string(), None))
                    }
                };

                // Type annotation is optional for 'self'
                let param_type = if let Some(Token::Punct(p)) = self.peek()? {
                    if p == ":" {
                        self.next()?; // consume ':'
                        self.parse_type()?
                    } else {
                        // Implicit type for 'self'
                        Type::Named(param_name.clone())
                    }
                } else {
                    // No type annotation, assume it's 'self' type
                    Type::Named(param_name.clone())
                };

                params.push((param_name, param_type));

                if let Some(token) = self.peek()? {
                    if self.is_punct(&token, ",") {
                        self.next()?; // consume ','
                    }
                }
            }

            self.expect_punct(")")?;

            self.expect_op("->")?;
            let return_type = self.parse_type()?;

            // Check if this is a signature or default implementation
            if let Some(token) = self.peek()? {
                if self.is_punct(&token, ";") {
                    self.next()?; // consume ';'
                    methods.push(TraitMethod::Signature {
                        name: method_name,
                        params,
                        return_type,
                    });
                } else if self.is_punct(&token, "{") {
                    // Default implementation
                    let body = Box::new(self.parse_block_statement()?);
                    methods.push(TraitMethod::Default {
                        name: method_name,
                        params,
                        return_type,
                        body,
                    });
                } else {
                    return Err(self.tokens.croak(
                        "Expected ';' or '{' after trait method signature".to_string(),
                        None,
                    ));
                }
            } else {
                return Err(self.tokens.croak(
                    "Expected ';' or '{' after trait method signature".to_string(),
                    None,
                ));
            }
        }

        self.expect_punct("}")?;

        Ok(Stmt::TraitDecl { name, methods })
    }

    fn parse_impl_block(&mut self) -> Result<Stmt> {
        self.expect_keyword("impl")?;

        // Check if implementing a trait
        let first_name = match self.next()? {
            Some(Token::Ident(name)) => name,
            Some(token) => {
                return Err(self.tokens.croak(
                    format!("Expected type or trait name but got {}", token),
                    None,
                ))
            }
            None => {
                return Err(self
                    .tokens
                    .croak("Expected type or trait name but got EOF".to_string(), None))
            }
        };

        let (trait_name, type_name) = if let Some(Token::Keyword(k)) = self.peek()? {
            if k == "for" {
                self.next()?; // consume 'for'
                let type_name = match self.next()? {
                    Some(Token::Ident(name)) => name,
                    _ => return Err(self.tokens.croak("Expected type name".to_string(), None)),
                };
                (Some(first_name), type_name)
            } else {
                (None, first_name)
            }
        } else {
            (None, first_name)
        };

        self.expect_punct("{")?;
        let mut methods = Vec::new();

        while let Some(token) = self.peek()? {
            if self.is_punct(&token, "}") {
                break;
            }

            methods.push(self.parse_function_decl(false, false)?);
        }

        self.expect_punct("}")?;

        Ok(Stmt::ImplBlock {
            type_name,
            trait_name,
            methods,
        })
    }

    fn parse_for_statement(&mut self) -> Result<Stmt> {
        self.expect_keyword("for")?;

        let var = match self.next()? {
            Some(Token::Ident(name)) => name,
            _ => {
                return Err(self
                    .tokens
                    .croak("Expected variable name".to_string(), None))
            }
        };

        self.expect_keyword("in")?;
        let iterable = self.parse_expression()?;

        let body = Box::new(self.parse_block_statement()?);

        Ok(Stmt::For {
            var,
            iterable,
            body,
        })
    }

    fn parse_match_expr(&mut self) -> Result<Expr> {
        // 'match' keyword already consumed
        // Parse the match subject without allowing struct literal syntax
        // since the { for the match block would be confused with struct literal
        let expr = self.parse_match_subject()?;

        self.expect_punct("{")?;
        let mut arms = Vec::new();

        while let Some(token) = self.peek()? {
            if self.is_punct(&token, "}") {
                break;
            }

            // Parse pattern - use a limited expression parser that doesn't treat { as struct literal
            let pattern = self.parse_pattern_expr()?;
            self.expect_op("=>")?;

            // Parse the body as an expression (not a statement)
            let body_expr = self.parse_expression()?;

            arms.push((pattern, body_expr));

            if let Some(token) = self.peek()? {
                if self.is_punct(&token, ",") {
                    self.next()?; // consume ','
                }
            }
        }

        self.expect_punct("}")?;

        Ok(Expr::Match {
            expr: Box::new(expr),
            arms,
        })
    }

    // Parse match subject expression without struct literal postfix
    fn parse_match_subject(&mut self) -> Result<Expr> {
        let mut left = self.parse_primary_expr()?;

        // Handle postfix operations but NOT struct literals
        loop {
            if let Some(token) = self.peek()? {
                match token {
                    Token::Punct(ref p) if p == "(" => {
                        // Function call
                        self.next()?; // consume '('
                        let mut args = Vec::new();

                        while let Some(token) = self.peek()? {
                            if self.is_punct(&token, ")") {
                                break;
                            }
                            args.push(self.parse_expression()?);
                            if let Some(token) = self.peek()? {
                                if self.is_punct(&token, ",") {
                                    self.next()?; // consume ','
                                }
                            }
                        }

                        self.expect_punct(")")?;
                        left = Expr::Call {
                            func: Box::new(left),
                            args,
                        };
                    }
                    Token::Op(ref p) if p == "." => {
                        // Field access
                        self.next()?; // consume '.'
                        let field_token = self.next()?;
                        let field = match field_token {
                            Some(Token::Ident(name)) => name,
                            _ => {
                                return Err(self
                                    .tokens
                                    .croak("Expected field name after '.'".to_string(), None))
                            }
                        };
                        left = Expr::FieldAccess {
                            object: Box::new(left),
                            field,
                        };
                    }
                    Token::Punct(ref p) if p == "[" => {
                        // Array index
                        self.next()?; // consume '['
                        let index = self.parse_expression()?;
                        self.expect_punct("]")?;
                        left = Expr::Index {
                            array: Box::new(left),
                            index: Box::new(index),
                        };
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }

        // Now handle binary operations
        self.parse_binary_expr_with_left(left, 0)
    }

    // Parse a pattern expression (similar to primary expr but without struct literal parsing)
    fn parse_pattern_expr(&mut self) -> Result<Expr> {
        let mut expr = self.parse_pattern_primary()?;

        // Handle postfix operations but don't interpret { as struct literal
        loop {
            if let Some(token) = self.peek()? {
                match token {
                    Token::Punct(ref p) if p == "(" => {
                        // Function call or enum constructor
                        self.next()?; // consume '('
                        let mut args = Vec::new();

                        while let Some(token) = self.peek()? {
                            if self.is_punct(&token, ")") {
                                break;
                            }
                            args.push(self.parse_pattern_expr()?);
                            if let Some(token) = self.peek()? {
                                if self.is_punct(&token, ",") {
                                    self.next()?; // consume ','
                                }
                            }
                        }

                        self.expect_punct(")")?;
                        expr = Expr::Call {
                            func: Box::new(expr),
                            args,
                        };
                    }
                    Token::Op(ref p) if p == "." => {
                        // Field access
                        self.next()?; // consume '.'
                        let field_token = self.next()?;
                        let field = match field_token {
                            Some(Token::Ident(name)) => name,
                            _ => {
                                return Err(self
                                    .tokens
                                    .croak("Expected field name after '.'".to_string(), None))
                            }
                        };
                        expr = Expr::FieldAccess {
                            object: Box::new(expr),
                            field,
                        };
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_pattern_primary(&mut self) -> Result<Expr> {
        let token_opt = self.next()?;
        match token_opt {
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::String(s)) => Ok(Expr::String(s)),
            Some(Token::Keyword(k)) if k == "true" => Ok(Expr::Boolean(true)),
            Some(Token::Keyword(k)) if k == "false" => Ok(Expr::Boolean(false)),
            Some(Token::Ident(name)) => Ok(Expr::Ident(name)),
            Some(Token::Punct(p)) if p == "(" => {
                let expr = self.parse_pattern_expr()?;
                self.expect_punct(")")?;
                Ok(expr)
            }
            Some(token) => Err(self
                .tokens
                .croak(format!("Unexpected token in pattern: {}", token), None)),
            None => Err(self
                .tokens
                .croak("Unexpected EOF in pattern".to_string(), None)),
        }
    }

    fn parse_match_statement(&mut self) -> Result<Stmt> {
        self.expect_keyword("match")?;

        let expr = self.parse_match_subject()?;

        self.expect_punct("{")?;
        let mut arms = Vec::new();

        while let Some(token) = self.peek()? {
            if self.is_punct(&token, "}") {
                break;
            }

            let pattern = self.parse_pattern_expr()?;
            self.expect_op("=>")?;
            let body = self.parse_statement()?;

            arms.push((pattern, body));

            if let Some(token) = self.peek()? {
                if self.is_punct(&token, ",") {
                    self.next()?; // consume ','
                }
            }
        }

        self.expect_punct("}")?;

        Ok(Stmt::Match { expr, arms })
    }

    fn parse_block_statement(&mut self) -> Result<Stmt> {
        let statements = self.parse_block()?;
        Ok(Stmt::Block(statements))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        self.expect_punct("{")?;
        let mut statements = Vec::new();

        while let Some(token) = self.peek()? {
            if self.is_punct(&token, "}") {
                break;
            }
            statements.push(self.parse_statement()?);
        }

        self.expect_punct("}")?;
        Ok(statements)
    }

    // Expression parsing with operator precedence
    pub fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_prec: u8) -> Result<Expr> {
        let mut left = self.parse_primary_expr()?;

        // Handle postfix operations first (highest precedence)
        left = self.parse_postfix(left)?;

        while let Some(token) = self.peek()? {
            if let Token::Op(op) = token {
                let prec = self.get_precedence(&op);
                if prec < min_prec {
                    break;
                }

                self.next()?; // consume operator
                let right = self.parse_binary_expr(prec + 1)?;
                left = Expr::BinOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    // Parse binary expressions without postfix operations (for array elements, function args, etc.)
    fn parse_binary_expr_with_left(&mut self, mut left: Expr, min_prec: u8) -> Result<Expr> {
        while let Some(token) = self.peek()? {
            if let Token::Op(op) = token {
                let prec = self.get_precedence(&op);
                if prec < min_prec {
                    break;
                }

                self.next()?; // consume operator
                              // For the right side, parse primary + postfix + binary
                let right_primary = self.parse_primary_expr()?;
                let right_postfix = self.parse_postfix(right_primary)?;
                let right = self.parse_binary_expr_with_left(right_postfix, prec + 1)?;
                left = Expr::BinOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_binary_expr_no_postfix(&mut self, min_prec: u8) -> Result<Expr> {
        let mut left = self.parse_primary_expr()?;

        while let Some(token) = self.peek()? {
            if let Token::Op(op) = token {
                let prec = self.get_precedence(&op);
                if prec < min_prec {
                    break;
                }

                self.next()?; // consume operator
                let right = self.parse_binary_expr_no_postfix(prec + 1)?;
                left = Expr::BinOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr> {
        let token_opt = self.next()?;
        match token_opt {
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::String(s)) => Ok(Expr::String(s)),
            Some(Token::TemplateStart) => {
                // Parse template literal
                self.parse_template_literal()
            }
            Some(Token::Keyword(k)) if k == "true" => Ok(Expr::Boolean(true)),
            Some(Token::Keyword(k)) if k == "false" => Ok(Expr::Boolean(false)),
            Some(Token::Keyword(k)) if k == "await" => {
                // Parse await expression: await <expr>
                // We need to parse the primary expression and then handle postfix operations
                let expr = self.parse_primary_expr()?;
                let expr_with_postfix = self.parse_postfix(expr)?;
                Ok(Expr::Await(Box::new(expr_with_postfix)))
            }
            Some(Token::Keyword(k)) if k == "async" => {
                // Parse async expression: async <expr>
                // This creates an eagerly-started async expression
                let expr = self.parse_primary_expr()?;
                let expr_with_postfix = self.parse_postfix(expr)?;
                Ok(Expr::Async(Box::new(expr_with_postfix)))
            }
            Some(Token::Keyword(k)) if k == "lazy" => {
                // Parse lazy expression: lazy <expr>
                // This creates a lazily-evaluated async expression
                let expr = self.parse_primary_expr()?;
                let expr_with_postfix = self.parse_postfix(expr)?;
                Ok(Expr::Lazy(Box::new(expr_with_postfix)))
            }
            Some(Token::Keyword(k)) if k == "match" => {
                // Parse match expression
                self.parse_match_expr()
            }
            Some(Token::Ident(name)) => {
                // Check if this is a lambda expression (v => ...)
                if let Some(Token::Op(op)) = self.peek()? {
                    if op == "=>" {
                        self.next()?; // consume '=>'
                                      // This is a lambda: v => body
                        return self.parse_lambda_body(vec![(name, None)], None);
                    }
                }
                // Return identifier, postfix parsing will be handled by parse_binary_expr
                Ok(Expr::Ident(name))
            }
            Some(Token::Punct(p)) if p == "(" => {
                // Could be: (expr), (params) => body, or function call
                // We need to peek ahead to see if this is a lambda
                let is_lambda = self.is_lambda_params()?;

                if is_lambda {
                    self.parse_lambda_with_parens()
                } else {
                    let expr = self.parse_expression()?;
                    self.expect_punct(")")?;
                    Ok(expr)
                }
            }
            Some(Token::Punct(p)) if p == "[" => {
                // Array literal
                let mut elements = Vec::new();

                while let Some(token) = self.peek()? {
                    if self.is_punct(&token, "]") {
                        break;
                    }

                    // Use parse_binary_expr_no_postfix to avoid consuming postfix ops
                    elements.push(self.parse_binary_expr_no_postfix(0)?);

                    if let Some(token) = self.peek()? {
                        if self.is_punct(&token, ",") {
                            self.next()?; // consume ','
                        }
                    }
                }

                self.expect_punct("]")?;
                Ok(Expr::ArrayLiteral(elements))
            }
            Some(Token::Punct(p)) if p == "{" => {
                // Put back the '{' token for block parsing
                self.tokens.push_back(Token::Punct(p));
                let statements = self.parse_block()?;
                Ok(Expr::Block(statements))
            }
            Some(token) => {
                let error_msg = format!("Unexpected token in expression: {}", token);
                Err(self.tokens.croak(error_msg, None))
            }
            None => Err(self
                .tokens
                .croak("Unexpected end of input in expression".to_string(), None)),
        }
    }

    // Parse postfix operations like function calls, field access, and indexing
    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr> {
        loop {
            if let Some(token) = self.peek()? {
                match token {
                    Token::Punct(ref p) if p == "(" => {
                        expr = self.parse_call(expr)?;
                    }
                    Token::Op(ref op) if op == "." => {
                        self.next()?; // consume '.'
                        let field = match self.next()? {
                            Some(Token::Ident(name)) => name,
                            _ => {
                                return Err(self
                                    .tokens
                                    .croak("Expected field name after '.'".to_string(), None))
                            }
                        };
                        expr = Expr::FieldAccess {
                            object: Box::new(expr),
                            field,
                        };
                    }
                    Token::Punct(ref p) if p == "[" => {
                        self.next()?; // consume '['
                        let index = self.parse_expression()?;
                        self.expect_punct("]")?;
                        expr = Expr::Index {
                            array: Box::new(expr),
                            index: Box::new(index),
                        };
                    }
                    Token::Punct(ref p) if p == "{" => {
                        // Check if this is a struct literal (identifier followed by {)
                        if let Expr::Ident(name) = expr {
                            expr = self.parse_struct_literal(name)?;
                        } else {
                            // Not a struct literal, break out
                            break;
                        }
                    }
                    Token::Op(ref op) if op == "?" => {
                        // Error propagation operator
                        self.next()?; // consume '?'
                        expr = Expr::Try(Box::new(expr));
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_struct_literal(&mut self, name: String) -> Result<Expr> {
        self.expect_punct("{")?;
        let mut fields = Vec::new();

        while let Some(token) = self.peek()? {
            if self.is_punct(&token, "}") {
                break;
            }

            // Parse field name
            let field_name = match self.next()? {
                Some(Token::Ident(name)) => name,
                Some(token) => {
                    return Err(self
                        .tokens
                        .croak(format!("Expected field name but got {}", token), None))
                }
                None => {
                    return Err(self
                        .tokens
                        .croak("Expected field name but got EOF".to_string(), None))
                }
            };

            self.expect_punct(":")?;

            // Parse field value expression - need to handle postfix operations
            // Parse primary expression then apply postfix operations
            let primary = self.parse_primary_expr()?;
            let field_value = self.parse_postfix(primary)?;

            // Now handle binary operations (but stop at comma or closing brace)
            let field_value = self.parse_binary_expr_with_left(field_value, 0)?;

            fields.push((field_name, field_value));

            if let Some(token) = self.peek()? {
                if self.is_punct(&token, ",") {
                    self.next()?; // consume ','
                }
            }
        }

        self.expect_punct("}")?;

        Ok(Expr::StructLiteral { name, fields })
    }

    fn parse_template_literal(&mut self) -> Result<Expr> {
        let mut parts = Vec::new();

        loop {
            match self.next()? {
                Some(Token::TemplateString(text)) => {
                    parts.push(TemplatePart::Text(text));
                }
                Some(Token::TemplateExprStart) => {
                    // Parse the expression inside ${}
                    let expr = self.parse_expression()?;
                    parts.push(TemplatePart::Expression(expr));

                    // Expect the closing brace
                    match self.next()? {
                        Some(Token::TemplateExprEnd) => {
                            // Continue parsing
                        }
                        Some(token) => {
                            return Err(self.tokens.croak(
                                format!("Expected '}}' after template expression, found {}", token),
                                None,
                            ));
                        }
                        None => {
                            return Err(self.tokens.croak(
                                "Unexpected end of input in template expression".to_string(),
                                None,
                            ));
                        }
                    }
                }
                Some(Token::TemplateEnd) => {
                    // End of template literal
                    break;
                }
                Some(token) => {
                    return Err(self.tokens.croak(
                        format!("Unexpected token in template literal: {}", token),
                        None,
                    ));
                }
                None => {
                    return Err(self.tokens.croak(
                        "Unexpected end of input in template literal".to_string(),
                        None,
                    ));
                }
            }
        }

        Ok(Expr::TemplateLiteral { parts })
    }

    // Helper function to check if we're looking at lambda params
    fn is_lambda_params(&mut self) -> Result<bool> {
        // Save the current state by collecting tokens
        let mut tokens_to_restore = Vec::new();
        let mut depth = 0;
        let mut found_arrow = false;

        // Look ahead to find ) => pattern
        loop {
            match self.next()? {
                Some(token) => {
                    tokens_to_restore.push(token.clone());

                    match &token {
                        Token::Punct(p) if p == ")" && depth == 0 => {
                            // Check next token for =>
                            if let Some(Token::Op(op1)) = self.next()? {
                                tokens_to_restore.push(Token::Op(op1.clone()));
                                if op1 == "=>" {
                                    found_arrow = true;
                                }
                            }
                            break;
                        }
                        Token::Punct(p) if p == "(" => depth += 1,
                        Token::Punct(p) if p == ")" => depth -= 1,
                        _ => {}
                    }

                    // Safety limit
                    if tokens_to_restore.len() > 100 {
                        break;
                    }
                }
                None => break,
            }
        }

        // Restore all tokens in reverse order (inserting at the front)
        for token in tokens_to_restore.into_iter().rev() {
            self.tokens.buffer.insert(0, token);
        }

        Ok(found_arrow)
    }

    fn parse_lambda_with_parens(&mut self) -> Result<Expr> {
        // The '(' has already been consumed by parse_primary_expr
        // Parse parameters
        let mut params = Vec::new();

        while let Some(token) = self.peek()? {
            if self.is_punct(&token, ")") {
                self.next()?; // consume ')'
                break;
            }

            let param_name = match self.next()? {
                Some(Token::Ident(name)) => name,
                Some(token) => {
                    return Err(self
                        .tokens
                        .croak(format!("Expected parameter name but got {}", token), None));
                }
                None => {
                    return Err(self
                        .tokens
                        .croak("Expected parameter name but got EOF".to_string(), None));
                }
            };

            // Optional type annotation
            let param_type = if let Some(Token::Punct(p)) = self.peek()? {
                if p == ":" {
                    self.next()?; // consume ':'
                    Some(self.parse_type()?)
                } else {
                    None
                }
            } else {
                None
            };

            params.push((param_name, param_type));

            if let Some(token) = self.peek()? {
                if self.is_punct(&token, ",") {
                    self.next()?; // consume ','
                }
            }
        }

        // Expect =>
        self.expect_op("=>")?;

        // Optional return type (not common but supported)
        let return_type = None;

        self.parse_lambda_body(params, return_type)
    }

    fn parse_lambda_body(
        &mut self,
        params: Vec<(String, Option<Type>)>,
        return_type: Option<Type>,
    ) -> Result<Expr> {
        // Body
        let body = if let Some(Token::Punct(p)) = self.peek()? {
            if p == "{" {
                // Block body
                let statements = self.parse_block()?;
                Box::new(Expr::Block(statements))
            } else {
                // Expression body
                Box::new(self.parse_expression()?)
            }
        } else {
            Box::new(self.parse_expression()?)
        };

        Ok(Expr::Lambda {
            params,
            return_type,
            body,
        })
    }

    fn parse_call(&mut self, func: Expr) -> Result<Expr> {
        self.expect_punct("(")?;
        let mut args = Vec::new();

        while let Some(token) = self.peek()? {
            if self.is_punct(&token, ")") {
                break;
            }

            args.push(self.parse_expression()?);

            if let Some(token) = self.peek()? {
                if self.is_punct(&token, ",") {
                    self.next()?; // consume comma
                } else if !self.is_punct(&token, ")") {
                    return Err(self
                        .tokens
                        .croak("Expected ',' or ')' in function call".to_string(), None));
                }
            }
        }

        self.expect_punct(")")?;
        Ok(Expr::Call {
            func: Box::new(func),
            args,
        })
    }

    fn get_precedence(&self, op: &str) -> u8 {
        match op {
            "||" => 1,
            "&&" => 2,
            "==" | "!=" => 3,
            "<" | "<=" | ">" | ">=" => 4,
            "|" => 5,
            "^" => 6,
            "&" => 7,
            "<<" | ">>" => 8,
            "+" | "-" => 9,
            "*" | "/" | "%" => 10,
            _ => 0,
        }
    }

    fn expect_op(&mut self, op: &str) -> Result<()> {
        let token_opt = self.next()?;
        match token_opt {
            Some(Token::Op(o)) if o == op => Ok(()),
            Some(token) => {
                let error_msg = format!("Expected operator '{}' but got {}", op, token);
                Err(self.tokens.croak(error_msg, None))
            }
            None => {
                let error_msg = format!("Expected operator '{}' but got EOF", op);
                Err(self.tokens.croak(error_msg, None))
            }
        }
    }

    fn maybe_consume_semicolon(&mut self) {
        if let Ok(Some(token)) = self.peek() {
            if self.is_punct(&token, ";") {
                let _ = self.next();
            }
        }
    }
}

// Extension methods for TokenStream to support lookahead
impl<'a> TokenStream<'a> {
    pub fn new(input: InputStream<'a>) -> Self {
        Self {
            input,
            buffer: Vec::new(),
            last_doc_comment: None,
        }
    }

    pub fn croak(&self, msg: String, len: Option<usize>) -> Error {
        self.input.croak(msg, len)
    }

    // Placeholder methods for position saving/restoring
    // You'd need to implement these based on your specific needs
    pub fn save_position(&self) -> usize {
        // Return current position in input stream
        0 // Placeholder
    }

    pub fn restore_position(&mut self, _pos: usize) {
        // Restore to saved position
        // Placeholder implementation
    }
}

#[cfg(test)]
mod tests {
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
}

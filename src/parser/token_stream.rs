use std::str::FromStr;
use std::fmt;

use rust_decimal::Decimal;

use crate::parser::input_stream::{InputStream, Result};

#[derive(Clone, Debug)]
pub enum Token {
    Number(Decimal),
    Keyword(String),
    Ident(String),
    String(String),
    Punct(String),
    Op(String),
    DocComment(String),  // For doc comments like /// or /** */
    Comment(String),     // For regular comments like // or /* */
    TemplateStart,       // `
    TemplateString(String), // Text part of template literal
    TemplateExprStart,   // ${
    TemplateExprEnd,     // }
    TemplateEnd,         // `
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Keyword(k) => write!(f, "'{}'", k),
            Token::Ident(i) => write!(f, "'{}'", i),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Punct(p) => write!(f, "'{}'", p),
            Token::Op(o) => write!(f, "'{}'", o),
            Token::DocComment(_) => write!(f, "doc comment"),
            Token::Comment(_) => write!(f, "comment"),
            Token::TemplateStart => write!(f, "'`'"),
            Token::TemplateString(s) => write!(f, "template text \"{}\"", s),
            Token::TemplateExprStart => write!(f, "'${{'"),
            Token::TemplateExprEnd => write!(f, "'}}'"),
            Token::TemplateEnd => write!(f, "'`'"),
        }
    }
}

pub struct TokenStream<'a> {
    pub(crate) input: InputStream<'a>,
    pub(crate) buffer: Vec<Token>,
    pub(crate) last_doc_comment: Option<String>,  // Store the last doc comment
}

pub const KEYWORDS: &[&str] = &["let", "const", "fn", "if", "else", "while", "for", "in", "return", "break", "continue", "match", "def", "enum", "impl", "trait", "async", "await", "lazy", "mut", "true", "false", "learn", "teach"];
pub const OPERATORS: &[char] = &['+', '-', '*', '/', '%', '=', '!', '<', '>', '&', '|', '^', '~', '.', '@', '?'];
pub const PUNCT: &[char] = &[',', ';', ':', '(', ')', '{', '}', '[', ']'];

impl TokenStream<'_> {
    pub fn is_keyword(s: &str) -> bool {
        KEYWORDS.contains(&s)
    }

    pub fn is_digit(c: char) -> bool {
        c.is_numeric()
    }

    pub fn is_ident_start(c: char) -> bool {
        c.is_alphabetic()
    }

    pub fn is_ident_body(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    pub fn is_op(c: char) -> bool {
        OPERATORS.contains(&c)
    }

    pub fn is_punct(c: char) -> bool {
        PUNCT.contains(&c)
    }

    pub fn is_whitespace(c: char) -> bool {
        c.is_whitespace()
    }

    pub fn read_while(&mut self, mut p: impl FnMut(char) -> bool) -> String {
        let mut s = String::new();
        while !self.input.eof() && p(self.input.peek().unwrap()) {
            s.push(self.input.next().unwrap());
        }
        s
    }

    pub fn read_number(&mut self) -> Result<Token> {
        let mut is_float = false;
        let num = self.read_while(|c| {
            if c == '.' {
                if is_float {
                    false
                } else {
                    is_float = true;
                    true
                }
            } else {
                c.is_numeric()
            }
        });

        Ok(Token::Number(Decimal::from_str(&num).map_err(|e| {
            self.input.croak(e.to_string(), Some(num.len()))
        })?))
    }

    pub fn read_ident(&mut self) -> Result<Token> {
        let mut first_char = true;
        let id = self.read_while(|c| {
            if first_char {
                first_char = false;
                Self::is_ident_start(c)
            } else {
                Self::is_ident_body(c)
            }
        });

        if Self::is_keyword(&id) {
            Ok(Token::Keyword(id))
        } else {
            Ok(Token::Ident(id))
        }
    }

    pub fn read_escaped(&mut self, end: char) -> String {
        let mut esc = false;
        let mut s = String::new();
        self.input.next();
        while !self.input.eof() {
            let c = self.input.next().unwrap();
            if esc {
                s.push(c);
            } else if c == '\\' {
                esc = false;
            } else if c == end {
                break;
            } else {
                s.push(c);
            }
        }

        return s;
    }

    pub fn read_string(&mut self) -> Result<Token> {
        Ok(Token::String(self.read_escaped('"')))
    }

    pub fn read_template_literal(&mut self) -> Result<Vec<Token>> {
        let mut tokens = vec![Token::TemplateStart];
        
        // Skip the opening backtick
        self.input.next();
        
        let mut text = String::new();
        
        while !self.input.eof() {
            let c = self.input.peek().unwrap();
            
            if c == '`' {
                // End of template literal
                if !text.is_empty() {
                    tokens.push(Token::TemplateString(text));
                }
                tokens.push(Token::TemplateEnd);
                self.input.next(); // consume closing backtick
                break;
            } else if c == '$' {
                // Check for interpolation start
                let pos = self.input.save_position();
                self.input.next(); // consume '$'
                
                if !self.input.eof() && self.input.peek().unwrap() == '{' {
                    // This is an interpolation
                    if !text.is_empty() {
                        tokens.push(Token::TemplateString(text));
                        text = String::new();
                    }
                    
                    tokens.push(Token::TemplateExprStart);
                    self.input.next(); // consume '{'
                    
                    // Parse the expression inside ${} by collecting characters until balanced }
                    let expr_tokens = self.parse_template_expression()?;
                    tokens.extend(expr_tokens);
                    
                    tokens.push(Token::TemplateExprEnd);
                    // The closing '}' is consumed by parse_template_expression
                } else {
                    // Not interpolation, restore position and treat as regular character
                    self.input.restore_position(pos);
                    text.push(self.input.next().unwrap());
                }
            } else if c == '\\' {
                // Handle escape sequences
                self.input.next(); // consume '\'
                if !self.input.eof() {
                    let escaped = self.input.next().unwrap();
                    match escaped {
                        'n' => text.push('\n'),
                        't' => text.push('\t'),
                        'r' => text.push('\r'),
                        '\\' => text.push('\\'),
                        '`' => text.push('`'),
                        '$' => text.push('$'),
                        c => {
                            text.push('\\');
                            text.push(c);
                        }
                    }
                }
            } else {
                text.push(self.input.next().unwrap());
            }
        }
        
        if !matches!(tokens.last(), Some(Token::TemplateEnd)) {
            return Err(self.input.croak("Unterminated template literal".to_string(), None));
        }
        
        Ok(tokens)
    }

    fn parse_template_expression(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut brace_count = 1;
        let mut expr_text = String::new();
        
        // Collect the expression text until we find the matching closing brace
        while !self.input.eof() && brace_count > 0 {
            let c = self.input.peek().unwrap();
            
            if c == '{' {
                brace_count += 1;
                expr_text.push(self.input.next().unwrap());
            } else if c == '}' {
                brace_count -= 1;
                if brace_count > 0 {
                    expr_text.push(self.input.next().unwrap());
                } else {
                    // This is the closing brace - consume it but don't add to expr_text
                    self.input.next();
                }
            } else {
                expr_text.push(self.input.next().unwrap());
            }
        }
        
        if brace_count > 0 {
            return Err(self.input.croak("Unterminated template expression".to_string(), None));
        }
        
        // Now parse the expression text as a separate token stream
        if !expr_text.is_empty() {
            let expr_input = InputStream::new("template_expr", &expr_text);
            let mut expr_stream = TokenStream::new(expr_input);
            
            // Parse all tokens from the expression
            while let Some(token_result) = expr_stream.next() {
                tokens.push(token_result?);
            }
        }
        
        Ok(tokens)
    }

    pub fn skip_whitespace_and_comments(&mut self) -> Result<()> {
        loop {
            // Skip regular whitespace
            self.read_while(|c| Self::is_whitespace(c));
            
            if self.input.eof() {
                break;
            }
            
            // Check for comments
            let current = self.input.peek().unwrap();
            if current == '/' {
                // Look ahead to see if it's a comment
                let pos = self.input.save_position();
                self.input.next(); // consume first '/'
                
                if !self.input.eof() {
                    let next = self.input.peek().unwrap();
                    if next == '/' {
                        self.input.next(); // consume second '/'
                        
                        // Check if it's a doc comment (///)
                        if !self.input.eof() && self.input.peek().unwrap() == '/' {
                            self.input.next(); // consume third '/'
                            // This is a doc comment, capture it
                            let doc_text = self.read_while(|c| c != '\n').trim().to_string();
                            self.last_doc_comment = Some(doc_text);
                            continue;
                        } else {
                            // Regular inline comment: skip to end of line
                            self.read_while(|c| c != '\n');
                            continue;
                        }
                    } else if next == '*' {
                        self.input.next(); // consume '*'
                        
                        // Check if it's a doc comment (/** */)
                        let is_doc_comment = !self.input.eof() && self.input.peek().unwrap() == '*';
                        if is_doc_comment {
                            self.input.next(); // consume third '*'
                        }
                        
                        // Read block comment content
                        let mut comment_text = String::new();
                        let mut found_end = false;
                        while !self.input.eof() {
                            let c = self.input.next().unwrap();
                            if c == '*' && !self.input.eof() {
                                if self.input.peek().unwrap() == '/' {
                                    self.input.next(); // consume '/'
                                    found_end = true;
                                    break;
                                }
                            }
                            if is_doc_comment {
                                comment_text.push(c);
                            }
                        }
                        
                        if !found_end {
                            return Err(self.input.croak("Unterminated block comment".to_string(), None));
                        }
                        
                        if is_doc_comment {
                            self.last_doc_comment = Some(comment_text.trim().to_string());
                        }
                        continue;
                    }
                }
                
                // Not a comment, restore position
                self.input.restore_position(pos);
                break;
            } else {
                // No more whitespace or comments
                break;
            }
        }
        Ok(())
    }

    pub fn parse_next(&mut self) -> Result<Option<Token>> {
        // Check if there's a token in the buffer first
        if !self.buffer.is_empty() {
            return Ok(Some(self.buffer.remove(0)));
        }
        
        self.skip_whitespace_and_comments()?;
        if self.input.eof() {
            return Ok(None);
        }

        let tok = match self.input.peek().unwrap() {
            '"' => self.read_string(),
            '`' => {
                // Handle template literals
                let template_tokens = self.read_template_literal()?;
                // Add all tokens except the first one to the buffer
                for token in template_tokens.into_iter().skip(1).rev() {
                    self.buffer.insert(0, token);
                }
                // Return the first token (TemplateStart)
                Ok(Token::TemplateStart)
            }
            c if Self::is_digit(c) => self.read_number(),
            c if Self::is_ident_start(c) => self.read_ident(),
            c if Self::is_punct(c) => {
                self.input.next(); // consume the character
                Ok(Token::Punct(c.to_string()))
            }
            c if Self::is_op(c) => {
                self.input.next(); // consume the character
                Ok(Token::Op(c.to_string()))
            }
            c => {
                return Err(self
                    .input
                    .croak(format!("Unexpected token '{}'", c), Some(1)));
            }
        };

        Some(tok).transpose()
    }

    pub fn next(&mut self) -> Option<Result<Token>> {
        self.parse_next().transpose()
    }
    
    pub fn push_back(&mut self, token: Token) {
        self.buffer.insert(0, token);
    }
    
    pub fn take_last_doc_comment(&mut self) -> Option<String> {
        self.last_doc_comment.take()
    }
}

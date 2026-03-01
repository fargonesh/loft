use crate::parser::input_stream::InputStream;
use crate::parser::token_stream::Token;
use rust_decimal::Decimal;

/// Token-based formatter that preserves comments and handles parse errors.
///
/// This formatter works directly with tokens rather than AST, allowing it to:
/// - Preserve all comments (both regular and doc comments)
/// - Format code even when there are parse errors
/// - Maintain more control over whitespace and formatting
pub struct TokenFormatter {
    indent_size: usize,
}

impl Default for TokenFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFormatter {
    pub fn new() -> Self {
        Self { indent_size: 4 }
    }

    /// Format source code using token-based approach
    pub fn format(&self, source: &str) -> Result<String, String> {
        let tokens = self.tokenize_with_comments(source)?;
        Ok(self.format_tokens(&tokens))
    }

    /// Tokenize source code while preserving all comments
    fn tokenize_with_comments(&self, source: &str) -> Result<Vec<TokenWithWhitespace>, String> {
        let mut tokens = Vec::new();
        let source_string = source.to_string();
        let mut input = InputStream::new("formatter", &source_string);

        while !input.eof() {
            // Track position before skipping whitespace
            let _ws_start = input.save_position();

            // Collect whitespace
            let mut whitespace = String::new();
            while !input.eof() && input.peek().unwrap().is_whitespace() {
                whitespace.push(input.next().unwrap());
            }

            if input.eof() {
                break;
            }

            // Check for comments
            let current = input.peek().unwrap();
            if current == '/' {
                let pos = input.save_position();
                input.next(); // consume first '/'

                if !input.eof() {
                    let next = input.peek().unwrap();
                    if next == '/' {
                        // Line comment
                        input.next(); // consume second '/'

                        // Check if it's a doc comment (///)
                        let is_doc = !input.eof() && input.peek().unwrap() == '/';
                        if is_doc {
                            input.next(); // consume third '/'
                        }

                        let mut comment_text = String::from("//");
                        if is_doc {
                            comment_text.push('/');
                        }

                        while !input.eof() && input.peek().unwrap() != '\n' {
                            comment_text.push(input.next().unwrap());
                        }

                        let token = if is_doc {
                            Token::DocComment(comment_text[3..].trim().to_string())
                        } else {
                            Token::Comment(comment_text)
                        };

                        tokens.push(TokenWithWhitespace {
                            token,
                            leading_whitespace: whitespace,
                        });
                        continue;
                    } else if next == '*' {
                        // Block comment
                        input.next(); // consume '*'

                        // Check if it's a doc comment (/** */)
                        let is_doc = !input.eof() && input.peek().unwrap() == '*';
                        if is_doc {
                            input.next(); // consume third '*'
                        }

                        let mut comment_text = String::from("/*");
                        if is_doc {
                            comment_text.push('*');
                        }

                        let mut found_end = false;
                        while !input.eof() {
                            let c = input.next().unwrap();
                            comment_text.push(c);
                            if c == '*' && !input.eof() && input.peek().unwrap() == '/' {
                                comment_text.push(input.next().unwrap());
                                found_end = true;
                                break;
                            }
                        }

                        if !found_end {
                            return Err("Unterminated block comment".to_string());
                        }

                        let token = if is_doc {
                            Token::DocComment(
                                comment_text[3..comment_text.len() - 2].trim().to_string(),
                            )
                        } else {
                            Token::Comment(comment_text)
                        };

                        tokens.push(TokenWithWhitespace {
                            token,
                            leading_whitespace: whitespace,
                        });
                        continue;
                    }
                }

                // Not a comment, restore position
                input.restore_position(pos);
            }

            // Parse regular token
            if let Some(token) = self.read_token(&mut input)? {
                tokens.push(TokenWithWhitespace {
                    token,
                    leading_whitespace: whitespace,
                });
            }
        }

        Ok(tokens)
    }

    /// Read a single token from the input
    fn read_token(&self, input: &mut InputStream) -> Result<Option<Token>, String> {
        if input.eof() {
            return Ok(None);
        }

        let c = input.peek().unwrap();

        // Numbers
        if c.is_numeric() {
            return Ok(Some(self.read_number(input)?));
        }

        // Strings
        if c == '"' {
            input.next();
            let mut s = String::new();
            let mut escaped = false;
            while !input.eof() {
                let ch = input.next().unwrap();
                if escaped {
                    s.push(ch);
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    break;
                } else {
                    s.push(ch);
                }
            }
            return Ok(Some(Token::String(s)));
        }

        // Template literals
        if c == '`' {
            input.next();
            return Ok(Some(Token::TemplateStart));
        }

        // Identifiers and keywords
        if c.is_alphabetic() || c == '_' {
            return Ok(Some(self.read_ident(input)));
        }

        // Operators and punctuation
        if self.is_op_char(c) || self.is_punct_char(c) {
            let mut op_str = String::new();
            op_str.push(input.next().unwrap());

            // Check for multi-char operators
            if !input.eof() {
                let next = input.peek().unwrap();
                let two_char = format!("{}{}", op_str, next);
                if self.is_multi_char_op(&two_char) {
                    op_str.push(input.next().unwrap());
                    return Ok(Some(Token::Op(op_str)));
                }
            }

            if self.is_op_char(c) {
                return Ok(Some(Token::Op(op_str)));
            } else {
                return Ok(Some(Token::Punct(op_str)));
            }
        }

        Err(format!("Unexpected character: {}", c))
    }

    fn read_number(&self, input: &mut InputStream) -> Result<Token, String> {
        let mut num = String::new();
        while !input.eof() {
            let c = input.peek().unwrap();
            if c.is_numeric() || c == '.' {
                num.push(input.next().unwrap());
            } else {
                break;
            }
        }

        Decimal::from_str_exact(&num)
            .map(Token::Number)
            .map_err(|e| format!("Invalid number: {}", e))
    }

    fn read_ident(&self, input: &mut InputStream) -> Token {
        let mut id = String::new();
        while !input.eof() {
            let c = input.peek().unwrap();
            if c.is_alphanumeric() || c == '_' {
                id.push(input.next().unwrap());
            } else {
                break;
            }
        }

        if self.is_keyword(&id) {
            Token::Keyword(id)
        } else {
            Token::Ident(id)
        }
    }

    fn is_keyword(&self, s: &str) -> bool {
        matches!(
            s,
            "let"
                | "const"
                | "fn"
                | "if"
                | "else"
                | "while"
                | "for"
                | "in"
                | "return"
                | "break"
                | "continue"
                | "match"
                | "def"
                | "enum"
                | "impl"
                | "trait"
                | "async"
                | "await"
                | "lazy"
                | "mut"
                | "true"
                | "false"
                | "learn"
                | "teach"
        )
    }

    fn is_op_char(&self, c: char) -> bool {
        matches!(
            c,
            '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~' | '.' | '@'
        )
    }

    fn is_punct_char(&self, c: char) -> bool {
        matches!(c, ',' | ';' | ':' | '(' | ')' | '{' | '}' | '[' | ']')
    }

    fn is_multi_char_op(&self, s: &str) -> bool {
        matches!(
            s,
            "->" | "=>"
                | "=="
                | "!="
                | "<="
                | ">="
                | "&&"
                | "||"
                | "+="
                | "-="
                | "*="
                | "/="
                | "::"
        )
    }

    /// Format tokens with proper spacing and indentation
    fn format_tokens(&self, tokens: &[TokenWithWhitespace]) -> String {
        let mut output = String::new();
        let mut indent_level = 0;
        let mut at_line_start = true;
        let mut prev_token: Option<&Token> = None;
        let mut scope_stack: Vec<char> = Vec::new();

        for (i, tw) in tokens.iter().enumerate() {
            let token = &tw.token;

            // Handle comments specially
            match token {
                Token::Comment(text) | Token::DocComment(text) => {
                    if !at_line_start {
                        output.push_str("  ");
                    } else {
                        output.push_str(&" ".repeat(indent_level * self.indent_size));
                    }

                    if matches!(token, Token::Comment(_)) {
                        output.push_str(text);
                    } else {
                        output.push_str("/// ");
                        output.push_str(text);
                    }

                    output.push('\n');
                    at_line_start = true;
                    prev_token = Some(token);
                    continue;
                }
                _ => {}
            }

            // Handle extra newlines from source
            let newlines = tw.leading_whitespace.chars().filter(|&c| c == '\n').count();
            if newlines >= 2 {
                if !at_line_start {
                    output.push('\n');
                }
                output.push('\n');
                at_line_start = true;
            }

            // Handle indentation decreases
            if matches!(token, Token::Punct(p) if p == "}" || p == "]" || p == ")") {
                if matches!(token, Token::Punct(p) if p == "}") {
                    indent_level = indent_level.saturating_sub(1);
                }

                // Pop scope
                match token {
                    Token::Punct(p) if p == "}" => {
                        if let Some(&'{') = scope_stack.last() {
                            scope_stack.pop();
                        }
                    }
                    Token::Punct(p) if p == "]" => {
                        if let Some(&'[') = scope_stack.last() {
                            scope_stack.pop();
                        }
                    }
                    Token::Punct(p) if p == ")" => {
                        if let Some(&'(') = scope_stack.last() {
                            scope_stack.pop();
                        }
                    }
                    _ => {}
                }

                // Ensure closing brace is on its own line
                if matches!(token, Token::Punct(p) if p == "}")
                    && !at_line_start {
                        output.push('\n');
                        at_line_start = true;
                    }

                if at_line_start {
                    output.push_str(&" ".repeat(indent_level * self.indent_size));
                    at_line_start = false;
                }
            }

            // Add the token
            let token_str = self.token_to_string(token);

            // Add spacing before token
            if !at_line_start && self.needs_space_before(token, prev_token) {
                output.push(' ');
            } else if at_line_start {
                output.push_str(&" ".repeat(indent_level * self.indent_size));
            }

            output.push_str(&token_str);

            // Update scope stack for openers
            match token {
                Token::Punct(p) if p == "{" => scope_stack.push('{'),
                Token::Punct(p) if p == "(" => scope_stack.push('('),
                Token::Punct(p) if p == "[" => scope_stack.push('['),
                _ => {}
            }

            // Handle newlines and indentation
            match token {
                Token::Punct(p) if p == ";" => {
                    output.push('\n');
                    at_line_start = true;
                }
                Token::Punct(p) if p == "{" => {
                    indent_level += 1;
                    output.push('\n');
                    at_line_start = true;
                }
                Token::Punct(p) if p == "," => {
                    // Force newline if inside braces (struct fields)
                    if let Some(&'{') = scope_stack.last() {
                        output.push('\n');
                        at_line_start = true;
                    }
                }
                Token::Punct(p) if p == "}" => {
                    // Check next token
                    let next_token = tokens.get(i + 1).map(|tw| &tw.token);
                    let should_newline = !matches!(
                        next_token,
                        Some(Token::Punct(p)) if p == ";" || p == "," || p == ")"
                    );

                    if should_newline {
                        output.push('\n');
                        at_line_start = true;
                    } else {
                        at_line_start = false;
                    }
                }
                _ => {
                    at_line_start = false;
                }
            }

            prev_token = Some(token);
        }

        output.trim_end().to_string() + "\n"
    }

    fn token_to_string(&self, token: &Token) -> String {
        match token {
            Token::Number(n) => n.to_string(),
            Token::Keyword(k) => k.clone(),
            Token::Ident(i) => i.clone(),
            Token::String(s) => format!("\"{}\"", s),
            Token::Punct(p) => p.clone(),
            Token::Op(o) => o.clone(),
            Token::DocComment(c) => format!("/// {}", c),
            Token::Comment(c) => c.clone(),
            Token::TemplateStart => "`".to_string(),
            Token::TemplateString(s) => s.clone(),
            Token::TemplateExprStart => "${".to_string(),
            Token::TemplateExprEnd => "}".to_string(),
            Token::TemplateEnd => "`".to_string(),
        }
    }

    fn needs_space_before(&self, token: &Token, prev: Option<&Token>) -> bool {
        let prev = match prev {
            Some(p) => p,
            None => return false,
        };

        match (prev, token) {
            // No space after opening brackets
            (Token::Punct(p), _) if p == "(" || p == "[" || p == "{" => false,
            // No space before closing brackets or punctuation (except colons)
            (_, Token::Punct(p)) if p == ")" || p == "]" || p == "}" || p == "," || p == ";" => {
                false
            }
            // Space before opening brace
            (_, Token::Punct(p)) if p == "{" => true,
            // Space after colons (for type annotations)
            (Token::Punct(p), _) if p == ":" => true,
            // Space after commas
            (Token::Punct(p), _) if p == "," => true,
            // No space before colons
            (_, Token::Punct(p)) if p == ":" => false,
            // No space around dots
            (Token::Op(o), _) if o == "." => false,
            (_, Token::Op(o)) if o == "." => false,
            // Space around operators
            (_, Token::Op(_)) => true,
            (Token::Op(_), _) => true,
            // Space after keywords (except before opening paren/bracket)
            (Token::Keyword(_), Token::Punct(p)) if p == "(" || p == "{" => true,
            (Token::Keyword(_), _) => true,
            // No space between function name and opening paren
            (Token::Ident(_), Token::Punct(p)) if p == "(" => false,
            // Default: add space for identifiers and keywords
            (Token::Ident(_), Token::Ident(_)) => true,
            (Token::Ident(_), Token::Keyword(_)) => true,
            // (Token::Keyword(_), Token::Ident(_)) => true,  // Removed: unreachable pattern
            _ => false,
        }
    }
}

#[derive(Debug)]
struct TokenWithWhitespace {
    token: Token,
    #[allow(dead_code)]
    leading_whitespace: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_comments() {
        let input = "// This is a comment\nlet x=42;// inline comment\nlet y=100;";
        let formatter = TokenFormatter::new();
        let formatted = formatter.format(input).unwrap();

        assert!(formatted.contains("// This is a comment"));
        assert!(formatted.contains("// inline comment"));
    }

    #[test]
    fn test_format_with_doc_comments() {
        let input = "/// Documentation\nfn test()->void{}";
        let formatter = TokenFormatter::new();
        let formatted = formatter.format(input).unwrap();

        assert!(formatted.contains("/// Documentation"));
    }
}

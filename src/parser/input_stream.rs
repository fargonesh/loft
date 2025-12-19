use std::fmt::{Debug, Display};

use miette::{Diagnostic, LabeledSpan, NamedSource};

#[derive(Debug, Clone, Copy)]
pub struct Position {
    position: usize,
    line: usize,
    column: usize,
}

pub struct InputStream<'a> {
    path: String,
    input: &'a [u8],
    position: usize,
    line: usize,
    column: usize,
}

impl Debug for InputStream<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputStream")
            .field("path", &self.path)
            .field("input", &"[..elided..]")
            .field("position", &self.position)
            .field("line", &self.line)
            .field("column", &self.column)
            .finish()
    }
}

impl<'a> InputStream<'a> {
    pub fn new(path: impl Display, st: &'a String) -> Self {
        Self {
            path: path.to_string(),
            input: st.as_bytes(),
            position: 0,
            line: 0,
            column: 0,
        }
    }
}

impl Iterator for InputStream<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.input.get(self.position) {
            let ch = *c as char;
            self.position += 1;
            
            // Update line and column tracking
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
            
            return Some(ch);
        };

        None
    }
}

impl InputStream<'_> {
    pub fn peek(&self) -> Option<char> {
        self.input.get(self.position).map(|v| *v as char)
    }

    pub fn eof(&self) -> bool {
        self.peek().is_none()
    }

    pub fn save_position(&self) -> Position {
        Position {
            position: self.position,
            line: self.line,
            column: self.column,
        }
    }

    pub fn restore_position(&mut self, pos: Position) {
        self.position = pos.position;
        self.line = pos.line;
        self.column = pos.column;
    }

    pub fn croak(&self, msg: impl Display, len: Option<usize>) -> Error {
        let source_text = String::from_utf8_lossy(self.input).to_string();
        Error {
            path: self.path.clone(),
            position: self.position,
            line: self.line,
            column: self.column,
            message: msg.to_string(),
            help: None,
            len,
            source: NamedSource::new(self.path.clone(), source_text),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub path: String,
    pub position: usize,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub help: Option<String>,
    pub len: Option<usize>,
    pub source: NamedSource<String>,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error in {} @ {}:{}:\n{}",
            self.path, self.line, self.column, self.message
        )
    }
}

impl std::error::Error for Error {}
impl Diagnostic for Error {
    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help
            .as_ref()
            .map(|c| Box::new(c) as Box<dyn std::fmt::Display>)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let len = self.len.as_ref().copied().unwrap_or_default();

        Some(Box::new(
            vec![LabeledSpan::new(
                Some(self.message.clone()),
                self.position.saturating_sub(len),
                len,
            )]
            .into_iter(),
        ))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

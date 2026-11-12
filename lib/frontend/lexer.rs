use std::path::PathBuf;

// Expand each parser into a direct call, stopping at the first success.
macro_rules! or_fn_chain {
    ($state:expr, [$($parser:expr),+ $(,)?]) => {{
        let state = &mut *($state);
        None$(.or_else(|| ($parser)(&mut *state)))+
    }};
}

pub trait ParserExt<S, T>: Fn(&mut S) -> Option<T> + Sized {
    fn map<U, F>(self, f: F) -> impl Fn(&mut S) -> Option<U>
    where
        F: Fn(T) -> U,
    {
        move |state| self(state).map(&f)
    }
}

impl<S, T, P> ParserExt<S, T> for P where P: Fn(&mut S) -> Option<T> {}

/// Lexer holds lexing state
pub struct Lexer<'a> {
    source: &'a str,
    error: bool,
    pos: u64,
}

/// Token holds the direct token as well as
/// its location data
#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    raw_str: &'a str,
    offset: usize,
    kind: TokenValue<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenValue<'a> {
    Program,
    Comment(&'a str),
    Var,
    Ident(&'a str),
    Colon,
    StrTyp,
    Digit(&'a str),
    WriteLn,
    ReadLn,
    Output,
    Input,
    Begin,
    End,
    Dot,
    Comma,
    Semicolon,
    LParen,
    RParen,
    Plus,
    Quote(&'a str),
}

impl<'a> Token<'a> {
    pub fn raw_str(&self) -> &'a str {
        self.raw_str
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn kind(&self) -> &TokenValue<'a> {
        &self.kind
    }
}

pub struct FileReader {
    contents: Vec<u8>,
    line_offset: u64,
    line_count: u64,
}

impl FileReader {
    pub fn new(file_name: PathBuf) -> Self {
        let data = std::fs::read(file_name).expect("was unable to read input file");
        FileReader {
            contents: data,
            line_count: 0,
            line_offset: 0,
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            error: false,
            pos: 0,
        }
    }

    pub fn has_error(&self) -> bool {
        self.error
    }

    /// Current byte offset, including the location of an unrecognized token.
    pub fn offset(&self) -> usize {
        self.pos as usize
    }

    pub fn consume_string<'s>(s: &'s str) -> impl Fn(&mut Self) -> Option<Token<'a>> + 's {
        move |this| Self::try_string(this, s)
    }

    /// Match a keyword regardless of ASCII case, without consuming an identifier prefix.
    pub fn consume_keyword<'s>(s: &'s str) -> impl Fn(&mut Self) -> Option<Token<'a>> + 's {
        move |this| {
            let pos = usize::try_from(this.pos).ok()?;
            let remaining = this.source.get(pos..)?;
            let raw_str = remaining.get(..s.len())?;
            if !raw_str.eq_ignore_ascii_case(s) {
                return None;
            }
            let suffix = &remaining[s.len()..];
            if suffix
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                return None;
            }
            this.try_string(raw_str)
        }
    }

    pub fn try_string(&mut self, s: &str) -> Option<Token<'a>> {
        let pos = usize::try_from(self.pos).ok()?;
        let remaining = self.source.get(pos..)?;
        if s.is_empty() || !remaining.starts_with(s) {
            return None;
        }

        let raw_str = &remaining[..s.len()];
        self.pos += s.len() as u64;
        Some(Token {
            raw_str,
            offset: pos,
            kind: TokenValue::Ident(raw_str),
        })
    }

    /// Match an ASCII identifier, preserving its original spelling.
    pub fn try_ident(&mut self) -> Option<Token<'a>> {
        let pos = usize::try_from(self.pos).ok()?;
        let remaining = self.source.get(pos..)?;
        let first = remaining.as_bytes().first()?;
        if !first.is_ascii_alphabetic() && *first != b'_' {
            return None;
        }

        let len = remaining
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            .count();
        let raw_str = &remaining[..len];
        self.pos += len as u64;
        Some(Token {
            raw_str,
            offset: pos,
            kind: TokenValue::Ident(raw_str),
        })
    }

    pub fn try_digit(&mut self) -> Option<Token<'a>> {
        let pos = usize::try_from(self.pos).ok()?;
        let remaining = self.source.get(pos..)?;
        let len = remaining
            .bytes()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if len == 0 {
            return None;
        }

        let raw_str = &remaining[..len];
        self.pos += len as u64;
        Some(Token {
            raw_str,
            offset: pos,
            kind: TokenValue::Digit(raw_str),
        })
    }

    pub fn try_quote(&mut self) -> Option<Token<'a>> {
        let pos = usize::try_from(self.pos).ok()?;
        let remaining = self.source.get(pos..)?;
        let bytes = remaining.as_bytes();
        if bytes.first() != Some(&b'\'') {
            return None;
        }

        let mut end = 1;
        while end < bytes.len() {
            match bytes[end] {
                b'\'' => {
                    if bytes.get(end + 1) == Some(&b'\'') {
                        end += 2;
                    } else {
                        let raw_str = &remaining[..end + 1];
                        self.pos += raw_str.len() as u64;
                        return Some(Token {
                            raw_str,
                            offset: pos,
                            kind: TokenValue::Quote(&remaining[1..end]),
                        });
                    }
                }
                b'\r' | b'\n' => return None,
                _ => end += 1,
            }
        }
        None
    }

    pub fn next_token(&mut self) -> Option<Token<'a>> {
        if self.error {
            return None;
        }

        let remaining = self.source.get(self.offset()..)?;
        let whitespace_len = remaining.len() - remaining.trim_start().len();
        self.pos += whitespace_len as u64;
        if whitespace_len == remaining.len() {
            return None;
        }

        let program = Self::consume_keyword("program").map(|token| Token {
            kind: TokenValue::Program,
            ..token
        });
        let var = Self::consume_keyword("var").map(|token| Token {
            kind: TokenValue::Var,
            ..token
        });
        let writeln = Self::consume_keyword("writeln").map(|token| Token {
            kind: TokenValue::WriteLn,
            ..token
        });

        let readln = Self::consume_keyword("readln").map(|token| Token {
            kind: TokenValue::ReadLn,
            ..token
        });

        let string = Self::consume_keyword("string").map(|token| Token {
            kind: TokenValue::StrTyp,
            ..token
        });
        let output = Self::consume_keyword("output").map(|token| Token {
            kind: TokenValue::Output,
            ..token
        });
        let input = Self::consume_keyword("input").map(|token| Token {
            kind: TokenValue::Input,
            ..token
        });
        let begin = Self::consume_keyword("begin").map(|token| Token {
            kind: TokenValue::Begin,
            ..token
        });
        let end = Self::consume_keyword("end").map(|token| Token {
            kind: TokenValue::End,
            ..token
        });
        let dot = Self::consume_string(".").map(|token| Token {
            kind: TokenValue::Dot,
            ..token
        });
        let colon = Self::consume_string(":").map(|token| Token {
            kind: TokenValue::Colon,
            ..token
        });
        let comma = Self::consume_string(",").map(|token| Token {
            kind: TokenValue::Comma,
            ..token
        });
        let semicolon = Self::consume_string(";").map(|token| Token {
            kind: TokenValue::Semicolon,
            ..token
        });
        let lparen = Self::consume_string("(").map(|token| Token {
            kind: TokenValue::LParen,
            ..token
        });
        let rparen = Self::consume_string(")").map(|token| Token {
            kind: TokenValue::RParen,
            ..token
        });
        let plus = Self::consume_string("+").map(|token| Token {
            kind: TokenValue::Plus,
            ..token
        });

        let token = or_fn_chain!(
            self,
            [
                program,
                var,
                writeln,
                readln,
                string,
                output,
                input,
                begin,
                end,
                dot,
                colon,
                comma,
                semicolon,
                lparen,
                rparen,
                plus,
                Self::try_digit,
                Self::try_quote,
                Self::try_ident,
            ]
        );
        if token.is_none() {
            self.error = true;
        }
        token
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

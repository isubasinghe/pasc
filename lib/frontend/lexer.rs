use std::{fs::File, io::{BufRead, BufReader, Read}, marker::PhantomData, path::PathBuf};

/// Lexer holds lexing state
pub struct Lexer<'a> {
    source: &'a str,
    error: bool,
    pos: u64
}

/// Token holds the direct token as well as
/// its location data
pub struct Token<'a> {
    raw_str: &'a str,
    offset: usize,
    kind: TokenValue<'a>
}

pub enum TokenValue<'a> {
    Program,
    Comment(&'a str),
    Var,
    Ident(&'a str),
    Colon,
    StrTyp,
    Digit(&'a str)
}

impl<'a> Token<'a> {
    pub fn offset(&self) -> usize {
        self.offset
    }
}


pub struct FileReader { 
    contents: Vec<u8>,
    line_offset: u64,
    line_count: u64
}

impl FileReader {
    pub fn new(file_name: PathBuf) -> Self {
        let data = std::fs::read(file_name).expect("was unable to read input file");
        FileReader {contents: data, line_count: 0, line_offset: 0}
    }
}

pub fn or_fn_chain<F, T>(or_fns: &[F]) -> F
    where F: FnOnce() -> Option<T>
{
    for val in or_fns {
        let x = val();
    }

    todo!()
}

impl<'a> Lexer<'a> {
    pub fn try_string(&mut self) -> Option<Token<'a>> {
        let pos = self.pos;
        None
    }
    pub fn next_token(&mut self) -> Token<'a> {
        
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.error {
            return None
        }
        todo!()
    }
}

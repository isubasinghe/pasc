use crate::frontend::{self, ast::*};
use crate::frontend::lexer::*;

pub struct Parser<T> 
{
    run_fn: fn() -> T
}

pub fn parse_statement<'a> (toks: Vec<Token<'a>>) -> Parser<Program<'a>> {
    todo!()
}

pub fn parse_program<'a> (toks: Vec<Token<'a>>) -> Parser<Program<'a>> {
    todo!()
}

use crate::frontend::ast::*;
use crate::frontend::lexer::*;
use std::convert::Infallible;
use std::ops::{ControlFlow, FromResidual, Residual, Try};

#[derive(Debug)]
pub struct ParserState<'a> {
    toks: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> ParserState<'a> {
    pub fn new(toks: Vec<Token<'a>>) -> Self {
        Self { toks, pos: 0 }
    }

    fn peek(&self) -> Option<TokenValue<'a>> {
        self.toks.get(self.pos).map(|token| *token.kind())
    }
}

#[derive(Debug)]
pub enum Parser<'a, T> {
    Ok { value: T, state: ParserState<'a> },
    Error { state: ParserState<'a>, msg: String },
}

impl<'a, T> Try for Parser<'a, T> {
    // Keep the updated state available when `?` extracts a successful value.
    type Output = (ParserState<'a>, T);
    type Residual = Parser<'a, Infallible>;

    fn from_output((state, value): Self::Output) -> Self {
        Self::Ok { state, value }
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok { state, value } => ControlFlow::Continue((state, value)),
            Self::Error { state, msg } => ControlFlow::Break(Parser::Error { state, msg }),
        }
    }
}

impl<'a, T> Residual<(ParserState<'a>, T)> for Parser<'a, Infallible> {
    type TryType = Parser<'a, T>;
}

impl<'a, T> FromResidual<Parser<'a, Infallible>> for Parser<'a, T> {
    fn from_residual(residual: Parser<'a, Infallible>) -> Self {
        match residual {
            Parser::Error { state, msg } => Self::Error { state, msg },
            Parser::Ok { value, .. } => match value {},
        }
    }
}

impl<'a, T> Parser<'a, T> {
    fn ok(&self) -> bool {
        matches!(self, Self::Ok { .. })
    }
    fn error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    fn pos(&self) -> usize {
        match self {
            Self::Ok { state, .. } | Self::Error { state, .. } => state.pos,
        }
    }
}

#[inline]
pub fn parse_single_tok<'a>(
    mut state: ParserState<'a>,
    tok_kind: TokenValue<'_>,
) -> Parser<'a, Token<'a>> {
    let Some(token) = state.toks.get(state.pos).copied() else {
        return Parser::Error {
            state,
            msg: format!("Expected {tok_kind:?}, found end of input"),
        };
    };
    if *token.kind() != tok_kind {
        return Parser::Error {
            state,
            msg: format!("Expected {tok_kind:?}, found {:?}", token.kind()),
        };
    }
    state.pos += 1;
    Parser::Ok {
        value: token,
        state,
    }
}

#[inline]
/// Run a parser and restore the attempt's starting position on failure.
pub fn try_parse<'a, T>(
    state: ParserState<'a>,
    f: impl FnOnce(ParserState<'a>) -> Parser<'a, T>,
) -> Parser<'a, T> {
    let start = state.pos;
    match f(state) {
        success @ Parser::Ok { .. } => success,
        Parser::Error { mut state, msg } => {
            state.pos = start;
            Parser::Error { state, msg }
        }
    }
}

pub fn separated_by1<'a, T>(
    delimiter: TokenValue<'a>,
    mut parse_item: impl FnMut(ParserState<'a>) -> Parser<'a, T>,
) -> impl FnMut(ParserState<'a>) -> Parser<'a, Vec<T>> {
    move |mut state| {
        let mut items = Vec::new();
        loop {
            let start = state.pos;
            let (next, item) = try_parse(state, &mut parse_item)?;
            state = next;
            if state.pos <= start {
                return Parser::Error {
                    state,
                    msg: "Repeated parser must consume at least one token".to_string(),
                };
            }
            items.push(item);
            if state.peek() != Some(delimiter) {
                return Parser::Ok {
                    state,
                    value: items,
                };
            }
            state.pos += 1;
        }
    }
}

pub fn parse_expr<'a>(mut state: ParserState<'a>) -> Parser<'a, Expr<'a>> {
    match state.peek() {
        Some(TokenValue::Quote(value)) => {
            state.pos += 1;
            Parser::Ok {
                state,
                value: Expr::ConstStr(value),
            }
        }
        found => Parser::Error {
            state,
            msg: format!("Expected a string literal, found {found:?}"),
        },
    }
}

pub fn parse_writeln<'a>(state: ParserState<'a>) -> Parser<'a, Statement<'a>> {
    let (state, _) = try_parse(state, |state| parse_single_tok(state, TokenValue::WriteLn))?;
    let (mut state, _) = try_parse(state, |state| parse_single_tok(state, TokenValue::LParen))?;
    let explicit_output = state.peek() == Some(TokenValue::Output);
    if explicit_output {
        (state, _) = try_parse(state, |state| parse_single_tok(state, TokenValue::Output))?;
    }
    let (state, args) = if state.peek() == Some(TokenValue::RParen) {
        (state, Vec::new())
    } else {
        if explicit_output {
            (state, _) = try_parse(state, |state| parse_single_tok(state, TokenValue::Comma))?;
        }
        parse_args(state)?
    };
    let (state, _) = try_parse(state, |state| parse_single_tok(state, TokenValue::RParen))?;
    Parser::Ok {
        state,
        value: Statement::WriteLn(Descriptor::Output, args),
    }
}

fn parse_args<'a>(state: ParserState<'a>) -> Parser<'a, Vec<Expr<'a>>> {
    try_parse(state, separated_by1(TokenValue::Comma, parse_expr))
}

pub fn parse_statement<'a>(toks: Vec<Token<'a>>) -> Parser<Program<'a>> {
    todo!()
}

pub fn parse_program<'a>(toks: Vec<Token<'a>>) -> Parser<Program<'a>> {
    todo!()
}

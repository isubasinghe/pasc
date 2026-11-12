use crate::cli::Cli;
use pasc::frontend::lexer::*;



pub fn drive(c: Cli) {
    let contents = std::fs::read_to_string(c.input).expect("was unable to read file");
    let mut lexer = Lexer::new(&contents);
    let toks: Vec<Token<'_>> = lexer.collect();
    println!("{:?}", toks);
}

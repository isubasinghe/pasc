mod cli;
mod driver;

use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    if let Err(error) = cli.validate() {
        error.exit();
    };
    driver::drive(cli);
    ExitCode::SUCCESS
}

use std::fmt;
use std::path::PathBuf;

use clap::{ArgAction, ArgGroup, CommandFactory, Parser, ValueEnum, error::ErrorKind};

/// Command-line interface for the pasc compiler.
#[derive(Debug, Parser)]
#[command(
    name = "pasc",
    version,
    about = "Compile source code and inspect intermediate compiler stages",
    long_about = "Compile source code and inspect intermediate compiler stages.\n\n\
        Compiler stages are under development. Source emission is available; \
        later stages currently report an error. Dumps of completed stages are \
        retained when a later stage fails.",
    group(ArgGroup::new("dump_selection").args(["dump", "dump_all"])),
    after_help = "Examples:\n  \
        pasc program.pasc --emit source\n  \
        pasc program.pasc --emit source -o source.pasc\n  \
        pasc program.pasc --check --dump-all\n  \
        pasc program.pasc -O2 -g --emit asm -o program.s\n  \
        pasc program.pasc --dump-all --dump-dir build/debug -vv --timings\n  \
        cat program.pasc | pasc - --emit source\n\n\
        Exit status: 0 success, 1 compiler or I/O error, 2 invalid arguments.\n\
        Emitted data goes to stdout by default; diagnostics always go to stderr."
)]
pub struct Cli {
    /// Source file, or '-' to read stdin
    #[arg(value_name = "INPUT", value_hint = clap::ValueHint::FilePath)]
    pub input: PathBuf,

    /// Final output file, or '-' for stdout
    #[arg(short, long, default_value = "-", value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    pub output: PathBuf,

    /// Emit this stage and stop (later stages are not yet implemented)
    #[arg(long, value_enum, default_value_t = Stage::Asm, value_name = "STAGE")]
    pub emit: Stage,

    /// Stop after semantic analysis without emitting final output
    #[arg(long, conflicts_with_all = ["emit", "output"])]
    pub check: bool,

    /// Dump selected stages; repeat or separate stages with commas
    #[arg(long, value_enum, value_delimiter = ',', value_name = "STAGE")]
    pub dump: Vec<Stage>,

    /// Dump every completed stage through --emit
    #[arg(long)]
    pub dump_all: bool,

    /// Directory for stage dumps [default: .pasc-dumps]
    #[arg(long, requires = "dump_selection", value_name = "DIR", value_hint = clap::ValueHint::DirPath)]
    pub dump_dir: Option<PathBuf>,

    /// Code-generation target
    #[arg(long, value_enum, default_value_t = Target::X86_64)]
    pub target: Target,

    /// Optimisation level: -O means -O2; explicit levels use -O2 or --opt-level=2
    #[arg(short = 'O', long = "opt-level", value_enum, default_value_t = OptimizationLevel::O0,
        num_args = 0..=1, default_missing_value = "2", require_equals = true, value_name = "LEVEL")]
    pub opt_level: OptimizationLevel,

    /// Request debug information from the backend (independent of optimisation)
    #[arg(short = 'g', long)]
    pub debug_info: bool,

    /// Show stage progress; repeat for input and output details
    #[arg(short, long, action = ArgAction::Count, conflicts_with = "quiet")]
    pub verbose: u8,

    /// Suppress progress messages (errors are still reported)
    #[arg(short, long, conflicts_with = "timings")]
    pub quiet: bool,

    /// Report the elapsed time for each attempted stage
    #[arg(long)]
    pub timings: bool,
}

impl Cli {
    pub fn validate(&self) -> Result<(), clap::Error> {
        if let Some(stage) = self.dump.iter().find(|&&stage| stage > self.final_stage()) {
            return Err(Self::command().error(
                ErrorKind::ArgumentConflict,
                format!(
                    "cannot dump '{stage}' when stopping after '{}'; choose a later --emit stage",
                    self.final_stage()
                ),
            ));
        }
        Ok(())
    }

    pub fn dumps(&self, stage: Stage) -> bool {
        self.dump_all || self.dump.contains(&stage)
    }

    pub fn final_stage(&self) -> Stage {
        if self.check { Stage::TypedAst } else { self.emit }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Stage {
    Source,
    Tokens,
    Ast,
    TypedAst,
    StackIr,
    Ssa,
    Asm,
}

impl Stage {
    pub const ALL: [Self; 7] = [
        Self::Source,
        Self::Tokens,
        Self::Ast,
        Self::TypedAst,
        Self::StackIr,
        Self::Ssa,
        Self::Asm,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Tokens => "tokens",
            Self::Ast => "ast",
            Self::TypedAst => "typed-ast",
            Self::StackIr => "stack-ir",
            Self::Ssa => "ssa",
            Self::Asm => "asm",
        }
    }

    pub fn dump_filename(self) -> &'static str {
        match self {
            Self::Source => "00-source.pasc",
            Self::Tokens => "01-tokens.txt",
            Self::Ast => "02-ast.txt",
            Self::TypedAst => "03-typed-ast.txt",
            Self::StackIr => "04-stack-ir.txt",
            Self::Ssa => "05-ssa.txt",
            Self::Asm => "06-asm.s",
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Target {
    #[value(name = "x86_64", alias = "x86-64")]
    X86_64,
    Riscv64,
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::X86_64 => "x86_64",
            Self::Riscv64 => "riscv64",
        })
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OptimizationLevel {
    #[value(name = "0")]
    O0,
    #[value(name = "1")]
    O1,
    #[value(name = "2")]
    O2,
    #[value(name = "3")]
    O3,
    #[value(name = "s")]
    Size,
    #[value(name = "z")]
    MinSize,
}

impl fmt::Display for OptimizationLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::O0 => "0",
            Self::O1 => "1",
            Self::O2 => "2",
            Self::O3 => "3",
            Self::Size => "s",
            Self::MinSize => "z",
        })
    }
}

use std::path::Path;

use clap::Parser;

#[derive(Parser)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub command: Subcommand,

    /// LSP server command.
    #[arg(long, default_value_t = Box::from("rust-analyzer"), global = true)]
    pub lsp: Box<str>,

    /// Enable verbose debug output.
    #[arg(long, short, default_value_t = false, global = true)]
    pub debug: bool,
}

#[derive(Parser)]
pub(crate) enum Subcommand {
    Verify(VerifyArgs),
    MakeRef(MakeRefArgs),
}

#[derive(Parser)]
pub(crate) struct VerifyArgs {
    /// Target file to apply sync results to.
    pub target: Box<Path>,

    /// Apply changes to the target. If not enabled, only a validation will be performed.
    #[arg(long, short, default_value_t = false)]
    pub update: bool,
}

#[derive(Parser)]
pub(crate) struct MakeRefArgs {
    /// Target position in code to create reference to.
    /// Run in interactive mode if it's not specified.
    pub target: Option<Box<str>>,
}

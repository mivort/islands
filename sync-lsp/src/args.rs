use std::path::Path;

use clap::Parser;

#[derive(Parser)]
pub(crate) struct Args {
    /// Target file to apply sync results to.
    pub target: Box<Path>,

    /// LSP server command.
    #[arg(long, default_value_t = Box::from("rust-analyzer"))]
    pub lsp: Box<str>,

    /// Apply changes to the target. If not enabled, only a validation will be performed.
    #[arg(long, short, default_value_t = false)]
    pub update: bool,
}

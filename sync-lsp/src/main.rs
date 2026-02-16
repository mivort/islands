use std::process::{Command, Stdio};

use lsp_types::InitializeParams;

fn main() -> anyhow::Result<()> {
    let mut _child = Command::new("rust-analyzer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let _init_params = InitializeParams::default();

    Ok(())
}

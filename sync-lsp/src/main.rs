use std::process::{Command, Stdio};

fn main() -> anyhow::Result<()> {
    let mut _child = Command::new("rust-analyzer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(())
}

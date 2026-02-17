mod client;

fn main() -> anyhow::Result<()> {
    let _client = client::LspClient::new("rust-analyzer")?;

    Ok(())
}

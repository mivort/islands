mod client;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let mut client = client::LspClient::new("rust-analyzer")?;

    client.initialize().await?;
    client.wait_index().await?;
    println!("Indexing complete");

    client.find_symbol().await?;

    client.exit().await
}

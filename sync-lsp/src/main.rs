use clap::Parser as _;

mod args;
mod client;
mod graph;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = args::Args::parse();

    let mut client = client::LspClient::new(&args.lsp)?;

    client.initialize().await?;
    client.wait_index().await?;
    println!("Indexing complete");

    client.find_symbol().await?;

    client.exit().await
}

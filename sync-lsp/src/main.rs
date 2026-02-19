use clap::Parser as _;

mod args;
mod client;
mod graph;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = args::Args::parse();

    let graph = graph::Graph::from_json(&args.target)?;
    println!(
        "Graph loaded, nodes: {}, edges: {}",
        graph.nodes.len(),
        graph.edges.len()
    );

    let mut client = client::LspClient::new(&args.lsp)?;

    client.initialize().await?;
    client.wait_index().await?;
    println!("Indexing complete");

    client.find_symbol("LspClient").await?;

    client.exit().await
}

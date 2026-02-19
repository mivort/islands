use clap::Parser as _;
use unwrap_or::unwrap_some_or;

mod args;
mod client;
mod graph;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = args::Args::parse();

    let mut graph = graph::Graph::from_json(&args.target)?;
    println!(
        "Graph loaded, nodes: {}, edges: {}",
        graph.nodes.len(),
        graph.edges.len()
    );

    let mut client = client::LspClient::new(&args.lsp)?;

    client.initialize().await?;
    client.wait_index().await?;
    println!("Indexing complete");

    for node in &mut graph.nodes {
        let node_ref = unwrap_some_or!(&node.data.r#ref, { continue });
        match parse_ref(&node_ref) {
            (RefType::Lsp, _value) => {}
            (RefType::File, _value) => {}
            (RefType::Unknown, value) => {
                eprintln!("Unknown reference type: {}", value);
            }
        }
    }

    client.find_symbol("LspClient").await?;

    client.exit().await
}

enum RefType {
    Lsp,
    File,
    Unknown,
}

/// Process the reference and handle the supported ones.
fn parse_ref(node_ref: &str) -> (RefType, &str) {
    let node_ref = node_ref.trim_start();
    if let Some(value) = node_ref.strip_prefix("lsp://") {
        (RefType::Lsp, value)
    } else if let Some(value) = node_ref.strip_prefix("file://") {
        (RefType::File, value)
    } else {
        (RefType::Unknown, node_ref)
    }
}

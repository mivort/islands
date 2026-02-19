mod args;
mod client;
mod graph;
mod noderef;

use clap::Parser as _;
use noderef::{NodeRef, NodeRefParams};
use unwrap_or::{unwrap_ok_or, unwrap_some_or};

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
            (RefType::Lsp, node_ref) => {
                client.find_symbol(node_ref).await?;
            }
            (RefType::File, _node_ref) => {}
            (RefType::Unknown, node_ref) => {
                eprintln!("Unknown reference type: {}", node_ref.base);
            }
        }
    }

    client.exit().await
}

enum RefType {
    Lsp,
    File,
    Unknown,
}

/// Process the reference and handle the supported ones.
fn parse_ref(node_ref: &str) -> (RefType, NodeRef) {
    let node_ref = node_ref.trim_start();
    if let Some(value) = node_ref.strip_prefix("lsp://") {
        if let Some((base_ref, params)) = value.split_once('?') {
            let params = NodeRefParams::from_str(params);
            let params = unwrap_ok_or!(params, _, {
                return (RefType::Unknown, NodeRef::base_ref(node_ref.into()));
            });
            (RefType::Lsp, NodeRef::params_ref(base_ref.into(), params))
        } else {
            (RefType::Lsp, NodeRef::base_ref(value.into()))
        }
    } else if let Some(value) = node_ref.strip_prefix("file://") {
        (RefType::File, NodeRef::base_ref(value.into()))
    } else {
        (RefType::Unknown, NodeRef::base_ref(node_ref.into()))
    }
}

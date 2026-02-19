mod args;
mod client;
mod graph;
mod noderef;

use std::fs;

use anyhow::Context as _;
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

    let mut checked_refs = 0usize;
    let mut missing_refs = 0usize;

    for node in &mut graph.nodes {
        let ref_uri = unwrap_some_or!(&node.data.r#ref, { continue });
        checked_refs += 1;
        match parse_ref(ref_uri) {
            (RefType::Lsp, node_ref) => {
                let data = client.find_symbol(node_ref).await?;
                if let Some(data) = data {
                    if !args.update {
                        continue;
                    }

                    node.data.doc = Some(data.hover);
                    node.data.valid = Some(true);
                } else {
                    missing_refs += 1;
                    eprintln!("Reference not found: {}", ref_uri);
                    if !args.update {
                        continue;
                    }

                    node.data.valid = Some(false);
                }
            }
            (RefType::File, node_ref) => {
                eprintln!("File refs are not supported yet: {}", node_ref.base);
                missing_refs += 1;
            }
            (RefType::Unknown, node_ref) => {
                eprintln!("Unknown reference type: {}", node_ref.base);
                missing_refs += 1;
            }
        }
    }

    client.exit().await?;

    println!("References validated: {checked_refs}");

    if missing_refs > 0 {
        eprintln!("Found {missing_refs} missing references");

        if !args.update {
            std::process::exit(1);
        }
    } else {
        println!("No missing references found");
    }

    if args.update {
        let output =
            serde_json::to_string_pretty(&graph).context("Unable to serialize graph data")?;
        fs::write(&args.target, &output)?;
    }

    Ok(())
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

mod args;
mod client;
mod graph;
mod noderef;

use std::fs;

use anyhow::Context as _;
use clap::Parser as _;
use noderef::{NodeRef, RefType};
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

    let mut client = client::LspClient::new(&args.lsp, args.debug)?;
    client.initialize().await?;
    client.wait_index().await?;
    println!("Indexing complete");

    let mut checked_refs = 0usize;
    let mut missing_refs = 0usize;
    let mut updated_docs = 0usize;
    let mut updated_locs = 0usize;

    for node in &mut graph.nodes {
        let ref_uri = unwrap_some_or!(&node.data.r#ref, { continue });
        checked_refs += 1;

        let node_ref = unwrap_ok_or!(NodeRef::parse_ref(ref_uri), _, {
            eprintln!("Unable to parse reference: {}", ref_uri);
            continue;
        });

        match node_ref.schema {
            RefType::Lsp => {
                let data = client.find_symbol(&node_ref).await?;
                if let Some(data) = data {
                    if !args.update {
                        continue;
                    }

                    if node.data.doc.as_ref() != Some(&data.hover) {
                        node.data.doc = Some(data.hover);
                        updated_docs += 1;
                    }
                    if node.data.location.as_ref() != Some(&data.location) {
                        node.data.location = Some(data.location);
                        updated_locs += 1;
                    }
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
            RefType::File => {
                eprintln!("File refs are not supported yet: {}", ref_uri);
                missing_refs += 1;
            }
            RefType::Unknown => {
                eprintln!("Unknown reference type: {}", ref_uri);
                missing_refs += 1;
            }
        }
    }

    client.exit().await?;

    println!("References validated: {checked_refs}");
    if updated_docs > 0 {
        println!("Docs updated: {updated_docs}");
    }
    if updated_locs > 0 {
        println!("Locations updated: {updated_locs}");
    }

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

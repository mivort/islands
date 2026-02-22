mod args;
mod client;
mod graph;
mod noderef;

use std::io::Write as _;
use std::{fs, io};

use anyhow::Context as _;
use args::{Args, MakeRefArgs, Subcommand, VerifyArgs};
use clap::Parser as _;
use noderef::{NodeRef, RefType};
use unwrap_or::{unwrap_ok_or, unwrap_some_or};

#[derive(Default)]
struct Stats {
    checked_refs: usize,
    missing_refs: usize,
    updated_docs: usize,
    updated_locs: usize,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = args::Args::parse();

    match &args.command {
        Subcommand::Verify(verify_args) => verify(&args, verify_args).await,
        Subcommand::MakeRef(make_ref_args) => make_ref(&args, make_ref_args).await,
    }
}

/// Iterate over graph nodes and check the referenced entries.
async fn verify(args: &Args, verify: &VerifyArgs) -> anyhow::Result<()> {
    let mut graph = graph::Graph::from_json(&verify.target)?;
    println!(
        "Graph loaded, nodes: {}, edges: {}",
        graph.nodes.len(),
        graph.edges.len()
    );

    let mut client = client::LspClient::new(&args.lsp, args.debug)?;
    client.initialize().await?;
    client.wait_index().await?;
    println!("Indexing complete");

    let mut stats = Stats::default();

    for node in &mut graph.nodes {
        let ref_uri = unwrap_some_or!(&node.data.r#ref, { continue });
        stats.checked_refs += 1;

        let node_ref = unwrap_ok_or!(NodeRef::parse_ref(ref_uri), _, {
            eprintln!("Unable to parse reference: {}", ref_uri);
            continue;
        });

        match node_ref.schema {
            RefType::Lsp => {
                let data = client.find_symbol(&node_ref).await?;
                if let Some(data) = data {
                    if !verify.update {
                        continue;
                    }

                    if node.data.doc.as_ref() != Some(&data.hover) {
                        node.data.doc = Some(data.hover);
                        stats.updated_docs += 1;
                    }
                    if node.data.location.as_ref() != Some(&data.location) {
                        node.data.location = Some(data.location);
                        stats.updated_locs += 1;
                    }
                    node.data.valid = Some(true);
                } else {
                    stats.missing_refs += 1;
                    eprintln!("Reference not found: {}", ref_uri);
                    if !verify.update {
                        continue;
                    }

                    node.data.valid = Some(false);
                }
            }
            RefType::File => {
                eprintln!("File refs are not supported yet: {}", ref_uri);
                stats.missing_refs += 1;
            }
            RefType::Unknown => {
                eprintln!("Unknown reference type: {}", ref_uri);
                stats.missing_refs += 1;
            }
        }
    }

    client.exit().await?;

    println!("References validated: {}", stats.checked_refs);
    if stats.updated_docs > 0 {
        println!("Docs updated: {}", stats.updated_docs);
    }
    if stats.updated_locs > 0 {
        println!("Locations updated: {}", stats.updated_locs);
    }

    if stats.missing_refs > 0 {
        eprintln!("Found {} missing references", stats.missing_refs);

        if !verify.update {
            std::process::exit(1);
        }
    } else {
        println!("All references were resolved");
    }

    if verify.update {
        let output =
            serde_json::to_string_pretty(&graph).context("Unable to serialize graph data")?;
        fs::write(&verify.target, &output)?;
    }

    Ok(())
}

/// Produce a reference to the given place in code.
async fn make_ref(args: &Args, make_ref: &MakeRefArgs) -> anyhow::Result<()> {
    let mut client = client::LspClient::new(&args.lsp, args.debug)?;
    client.initialize().await?;
    client.wait_index().await?;
    println!("Indexing complete");

    if let Some(target) = &make_ref.target {
        if let Some((path, line, char)) = extract_path(target) {
            match client.make_ref(path, line, char).await {
                Ok(Some(reference)) => println!("Reference: {reference}"),
                _ => eprintln!("Location not resolved: {}", target),
            }
        } else {
            eprintln!("Unable to extract path, line and character numbers from input");
        }
    } else {
        println!(
            "Interactive mode started, enter 'path/to/file:line:char' to resolve into reference"
        );
        let mut input = String::new();

        loop {
            input.clear();
            print!("> ");
            io::stdout().flush()?;
            if io::stdin().read_line(&mut input)? == 0 {
                break;
            };

            let (path, line, char) = unwrap_some_or!(extract_path(&input.trim()), {
                println!("Unable to extract path");
                continue;
            });
            match client.make_ref(path, line, char).await {
                Ok(Some(reference)) => println!("Reference: {reference}"),
                _ => eprintln!("Location not resolved: {}", &input),
            }
        }
    }

    client.exit().await?;

    Ok(())
}

/// Extract line number and character number from the input parameter.
/// Numbers are coverted to be 0-based to be compatible with LSP output.
fn extract_path(full_path: &str) -> Option<(&str, u32, u32)> {
    let (remainder, char_no) = full_path.rsplit_once(':')?;
    let (path, line_no) = remainder.rsplit_once(':')?;

    let line_no: u32 = line_no.parse().ok()?;
    let char_no: u32 = char_no.parse().ok()?;
    Some((path, line_no.saturating_sub(1), char_no.saturating_sub(1)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn extract_path() {
        assert_eq!(
            super::extract_path("src/main.rs:32:5"),
            Some(("src/main.rs", 31, 4))
        );
        assert_eq!(super::extract_path("src/main.rs:32"), None);
        assert_eq!(super::extract_path("src/main.rs:-32:-5"), None);
    }
}

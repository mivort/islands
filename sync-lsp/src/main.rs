use anyhow::Context as _;
use lsp_types::{
    ClientCapabilities, DocumentSymbolParams, DocumentSymbolResponse, InitializeParams,
    InitializedParams, TextDocumentIdentifier, Uri, WorkDoneProgressParams, WorkspaceFolder,
    WorkspaceSymbolParams, WorkspaceSymbolResponse, notification, request,
};

mod client;

fn main() -> anyhow::Result<()> {
    let mut client = client::LspClient::new("rust-analyzer")?;

    let cwd = std::env::current_dir()?;
    let root_uri: Uri = format!("file://{}", cwd.display()).parse()?;

    let init = client
        .request::<request::Initialize>(InitializeParams {
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: root_uri,
                name: cwd.file_name().unwrap().to_string_lossy().into_owned(),
            }]),
            capabilities: ClientCapabilities::default(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            ..Default::default()
        })
        .context("Init request failed")?;

    if let Some(info) = &init.server_info {
        println!(
            "Connected to {} {}",
            info.name,
            info.version.as_deref().unwrap_or_default()
        );
    } else {
        println!("Connected to server, no server info provided");
    }

    client
        .notify::<notification::Initialized>(InitializedParams {})
        .context("Ready notification failed")?;

    let query = client
        .request::<request::WorkspaceSymbolRequest>(WorkspaceSymbolParams {
            query: "main".into(),
            ..Default::default()
        })
        .context("Unable to perform symbol query")?;

    match query {
        Some(WorkspaceSymbolResponse::Flat(symbols)) => {
            println!("{} flat symbols", symbols.len());
        }
        _ => {}
    }

    let query = client
        .request::<request::DocumentSymbolRequest>(DocumentSymbolParams {
            text_document: TextDocumentIdentifier {
                uri: "file://src/main.rs".parse()?,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .context("Unable to perform symbol query")?;

    match query {
        Some(DocumentSymbolResponse::Flat(symbols)) => {
            println!("{} flat document symbols", symbols.len());
        }
        _ => {}
    }

    Ok(())
}

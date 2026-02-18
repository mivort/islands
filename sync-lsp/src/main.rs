use anyhow::Context as _;
use lsp_types::{
    ClientCapabilities, InitializeParams, Uri, WorkDoneProgressParams, WorkspaceFolder, request,
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

    Ok(())
}

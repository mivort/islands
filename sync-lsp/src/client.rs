use crate::noderef::NodeRef;
use std::ops::ControlFlow;
use std::process::Stdio;

use anyhow::Context as _;
use async_lsp::concurrency::ConcurrencyLayer;
use async_lsp::lsp_types::{
    ClientCapabilities, HoverParams, InitializeParams, InitializedParams, NumberOrString,
    ProgressParams, ProgressParamsValue, SymbolInformation, TextDocumentIdentifier,
    TextDocumentPositionParams, Url, WindowClientCapabilities, WorkDoneProgress, WorkspaceFolder,
    WorkspaceSymbolParams, WorkspaceSymbolResponse,
};
use async_lsp::panic::CatchUnwindLayer;
use async_lsp::router::Router;
use async_lsp::tracing::TracingLayer;
use async_lsp::{LanguageClient, LanguageServer, ResponseError, ServerSocket};
use async_process::Child;
use futures::channel::oneshot;
use tokio::task::JoinHandle;
use tower::ServiceBuilder;

/// Client context instance.
pub(crate) struct LspClient {
    #[expect(unused)]
    child: Child,
    workdir: Url,
    server: ServerSocket,
    indexed_recv: Option<oneshot::Receiver<()>>,
    join: Option<JoinHandle<()>>,
}

/// List of known indexing tokens.
const INDEXING_TOKENS: &[&str] = &["rustAnalyzer/Indexing", "rustAnalyzer/cachePriming"];

impl LspClient {
    /// Spawn LSP server child process.
    pub fn new(cmd: &str) -> anyhow::Result<Self> {
        let cwd = std::env::current_dir()?;
        let workdir: Url = format!("file://{}", cwd.display()).parse()?;

        let (indexed_send, indexed_recv) = oneshot::channel();
        let mut router = Router::from_language_client(LspState {
            indexed_send: Some(indexed_send),
        });
        router.event(LspState::stop);

        let mut child = async_process::Command::new(cmd)
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .context("Failed run rust-analyzer")?;

        let (mainloop, server) = async_lsp::MainLoop::new_client(|_server| {
            ServiceBuilder::new()
                .layer(TracingLayer::default())
                .layer(CatchUnwindLayer::default())
                .layer(ConcurrencyLayer::default())
                .service(router)
        });

        let stdout = child
            .stdout
            .take()
            .context("Unable to get child process stdout")?;
        let stdin = child
            .stdin
            .take()
            .context("Unable to get child process stdin")?;

        let mainloop_handle = tokio::spawn(async move {
            mainloop
                .run_buffered(stdout, stdin)
                .await
                .unwrap_or_else(|_| {
                    eprintln!("Unable to fetch data from language server process");
                });
        });

        Ok(Self {
            child,
            workdir,
            server,
            join: Some(mainloop_handle),
            indexed_recv: Some(indexed_recv),
        })
    }

    /// Send initialization request and notification.
    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        let init = self
            .server
            .initialize(InitializeParams {
                workspace_folders: Some(vec![WorkspaceFolder {
                    name: "root".into(),
                    uri: self.workdir.clone(),
                }]),
                capabilities: ClientCapabilities {
                    window: Some(WindowClientCapabilities {
                        work_done_progress: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .context("Unable to process initialize request")?;

        let (name, version) = if let Some(info) = &init.server_info {
            (
                info.name.as_str(),
                info.version.as_deref().unwrap_or("(unknown)"),
            )
        } else {
            ("unknown", "(unknown)")
        };

        println!("Initialized: {name} {version}");
        self.server.initialized(InitializedParams {}).unwrap();

        Ok(())
    }

    /// Wait for LSP server to report index readyness.
    pub async fn wait_index(&mut self) -> anyhow::Result<()> {
        println!("Waiting for index to be loaded...");

        let recv = self
            .indexed_recv
            .take()
            .context("Unable to get indexed recv")?;
        recv.await.context("Unable to wait for indexing completion")
    }

    /// Perform a workspace lookup for specific symbol.
    pub async fn find_symbol(&mut self, node_ref: NodeRef) -> anyhow::Result<()> {
        let symbol = self
            .server
            .symbol(WorkspaceSymbolParams {
                query: node_ref.base.clone(),
                ..Default::default()
            })
            .await
            .context("Unable to search for workspace symbol")?;

        match symbol {
            Some(WorkspaceSymbolResponse::Flat(symbols)) => {
                self.match_symbol(symbols, &node_ref).await
            }
            _ => {
                println!("No symbol found");
                Ok(())
            }
        }
    }

    /// Wait for LSP server child process completion.
    pub async fn exit(&mut self) -> anyhow::Result<()> {
        self.server.shutdown(()).await?;
        self.server.emit(LspStop)?;
        self.server.exit(())?;

        let join = self
            .join
            .take()
            .context("Unable to get child process join handle")?;
        join.await
            .context("Unable to wait for server process completion")
    }
}

impl LspClient {
    /// Iterate over list of found symbols and try to match the parameters.
    async fn match_symbol(
        &mut self,
        symbols: Vec<SymbolInformation>,
        node_ref: &NodeRef,
    ) -> anyhow::Result<()> {
        for s in symbols {
            if s.name != node_ref.base {
                continue;
            }
            if !node_ref.params.matches_kind(s.kind) {
                continue;
            }
            if !node_ref.params.matches_uri(&s.location.uri) {
                continue;
            }

            println!("Symbol: {:?}", s);

            let hover = self
                .server
                .hover(HoverParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier {
                            uri: s.location.uri,
                        },
                        position: s.location.range.start,
                    },
                    work_done_progress_params: Default::default(),
                })
                .await?;

            let hover = match hover {
                Some(hover) => hover,
                None => continue,
            };

            println!("Hover: {:?}", hover.contents);
        }

        Ok(())
    }
}

struct LspStop;

struct LspState {
    indexed_send: Option<oneshot::Sender<()>>,
}

impl LspState {
    fn stop(&mut self, _: LspStop) -> ControlFlow<async_lsp::Result<()>> {
        ControlFlow::Break(Ok(()))
    }
}

impl LanguageClient for LspState {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<async_lsp::Result<()>>;

    fn progress(&mut self, params: ProgressParams) -> Self::NotifyResult {
        if matches!(
            params.value,
            ProgressParamsValue::WorkDone(WorkDoneProgress::End(_))
        ) && matches!(params.token, NumberOrString::String(ref s) if INDEXING_TOKENS.contains(&&**s))
        {
            if let Some(tx) = self.indexed_send.take() {
                let _ = tx.send(());
            }
        }
        ControlFlow::Continue(())
    }
}

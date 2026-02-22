use crate::noderef::NodeRef;
use std::ops::ControlFlow;
use std::process::Stdio;

use anyhow::Context as _;
use async_lsp::concurrency::ConcurrencyLayer;
use async_lsp::lsp_types::{
    ClientCapabilities, DocumentSymbol, DocumentSymbolClientCapabilities, DocumentSymbolParams,
    DocumentSymbolResponse, Hover, HoverClientCapabilities, HoverContents, HoverParams,
    InitializeParams, InitializedParams, MarkupKind, NumberOrString, ProgressParams,
    ProgressParamsValue, SymbolInformation, TextDocumentClientCapabilities, TextDocumentIdentifier,
    TextDocumentPositionParams, Url, WindowClientCapabilities, WorkDoneProgress, WorkspaceFolder,
    WorkspaceSymbolParams, WorkspaceSymbolResponse,
};
use async_lsp::panic::CatchUnwindLayer;
use async_lsp::router::Router;
use async_lsp::tracing::TracingLayer;
use async_lsp::{LanguageClient, LanguageServer, ResponseError, ServerSocket};
use async_process::Child;
use futures::channel::oneshot;
use log::{debug, error, info};
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use unwrap_or::unwrap_some_or;

/// Client context instance.
pub(crate) struct LspClient {
    #[expect(unused)]
    child: Child,
    workdir: Url,
    server: ServerSocket,
    indexed_recv: Option<oneshot::Receiver<()>>,
    join: Option<JoinHandle<()>>,

    // TODO: introduce a logging facility
    debug: bool,
}

/// List of known indexing tokens.
const INDEXING_TOKENS: &[&str] = &["rustAnalyzer/Indexing", "rustAnalyzer/cachePriming"];

impl LspClient {
    /// Spawn LSP server child process.
    pub fn new(cmd: &str, debug: bool) -> anyhow::Result<Self> {
        let cwd = std::env::current_dir()?;
        let workdir: Url = format!("file://{}/", cwd.display()).parse()?;

        let (indexed_send, indexed_recv) = oneshot::channel();
        let mut router = Router::from_language_client(LspState {
            indexed_send: Some(indexed_send),
        });
        router.event(LspState::stop);

        let mut child = async_process::Command::new(cmd)
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(if debug {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
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
                    error!("Unable to fetch data from language server process");
                });
        });

        Ok(Self {
            child,
            workdir,
            server,
            join: Some(mainloop_handle),
            indexed_recv: Some(indexed_recv),
            debug,
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
                    text_document: Some(TextDocumentClientCapabilities {
                        hover: Some(HoverClientCapabilities {
                            content_format: Some(vec![MarkupKind::Markdown]),
                            ..Default::default()
                        }),
                        document_symbol: Some(DocumentSymbolClientCapabilities {
                            hierarchical_document_symbol_support: Some(true),
                            ..Default::default()
                        }),
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

        info!("Initialized: {name} {version}");
        self.server.initialized(InitializedParams {}).unwrap();

        Ok(())
    }

    /// Wait for LSP server to report index readyness.
    pub async fn wait_index(&mut self) -> anyhow::Result<()> {
        info!("Waiting for index to be loaded...");

        let recv = self
            .indexed_recv
            .take()
            .context("Unable to get indexed recv")?;
        recv.await.context("Unable to wait for indexing completion")
    }

    /// Perform a workspace lookup for specific symbol.
    /// Server response items are treated as 'missing symbols' to handle changed
    /// file paths.
    pub async fn find_symbol(&mut self, node_ref: &NodeRef) -> anyhow::Result<Option<LspData>> {
        if node_ref.path.len() > 0 {
            match self.find_document_symbol(&node_ref).await {
                Ok(symbol) => return Ok(symbol),
                Err(err) => {
                    for cause in err.chain() {
                        if let Some(_err) = cause.downcast_ref::<ResponseError>() {
                            return Ok(None);
                        }
                    }
                    return Err(err);
                }
            }
        }
        self.find_workspace_symbol(&node_ref).await
    }

    /// Use provided file path, line and char number to produce LSP reference (`lsp://...`).
    pub async fn make_ref(
        &mut self,
        path: &str,
        line: u32,
        char: u32,
    ) -> anyhow::Result<Option<String>> {
        debug!("Querying symbols for {}", path);

        let uri = self.workdir.join(path)?;
        let symbol = self
            .server
            .document_symbol(DocumentSymbolParams {
                text_document: TextDocumentIdentifier { uri },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            })
            .await?;

        match symbol {
            Some(DocumentSymbolResponse::Nested(symbols)) => {
                debug!("Top level symbols in the document: {}", symbols.len());
                let stack = self.find_nested_symbol_in_position(&symbols, line, char);
                if stack.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(convert_stack(path, stack)));
            }
            Some(DocumentSymbolResponse::Flat(_)) => todo!(),
            None => todo!(),
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
    /// Query the specified workspace path.
    async fn find_workspace_symbol(
        &mut self,
        node_ref: &NodeRef,
    ) -> anyhow::Result<Option<LspData>> {
        let symbol = self
            .server
            .symbol(WorkspaceSymbolParams {
                query: node_ref.hash.clone(),
                ..Default::default()
            })
            .await
            .context("Unable to search for workspace symbol")?;

        match symbol {
            Some(WorkspaceSymbolResponse::Flat(symbols)) => {
                self.match_flat_symbol(symbols, &node_ref).await
            }
            _ => Ok(None),
        }
    }

    /// Query the specified document path.
    async fn find_document_symbol(
        &mut self,
        node_ref: &NodeRef,
    ) -> anyhow::Result<Option<LspData>> {
        if self.debug {
            info!("Query document symbols: {}", node_ref.path);
        }
        let uri = self.workdir.join(&node_ref.path)?;
        let symbol = self
            .server
            .document_symbol(DocumentSymbolParams {
                text_document: TextDocumentIdentifier { uri },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            })
            .await?;

        match symbol {
            Some(DocumentSymbolResponse::Flat(symbols)) => {
                self.match_flat_symbol(symbols, &node_ref).await
            }
            Some(DocumentSymbolResponse::Nested(symbols)) => {
                self.match_nested_symbol(symbols, &node_ref).await
            }
            _ => Ok(None),
        }
    }

    /// Iterate over list of found symbols and try to match the parameters.
    async fn match_flat_symbol(
        &mut self,
        symbols: Vec<SymbolInformation>,
        node_ref: &NodeRef,
    ) -> anyhow::Result<Option<LspData>> {
        for s in symbols {
            if s.name != node_ref.hash {
                continue;
            }
            if !node_ref.params.matches_kind(s.kind) {
                continue;
            }

            let hover = self
                .server
                .hover(HoverParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier {
                            uri: s.location.uri.clone(),
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

            // TODO: convert location into relative path
            return Ok(Some(LspData::from_hover(
                hover,
                s.location.uri.as_str(),
                s.location.range.start.line,
            )));
        }

        Ok(None)
    }

    /// Iterate over list of found symbols and try to match the parameters.
    async fn match_nested_symbol(
        &mut self,
        symbols: Vec<DocumentSymbol>,
        node_ref: &NodeRef,
    ) -> anyhow::Result<Option<LspData>> {
        let symbol = self.find_nested_symbol(&symbols, node_ref, &node_ref.hash);
        let symbol = unwrap_some_or!(symbol, { return Ok(None) });

        // TODO: match kind in addition to name on the last part

        let uri = self.workdir.join(&node_ref.path)?;

        let hover = self
            .server
            .hover(HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri },
                    position: symbol.selection_range.start,
                },
                work_done_progress_params: Default::default(),
            })
            .await?;

        match hover {
            Some(hover) => Ok(Some(LspData::from_hover(
                hover,
                &node_ref.path,
                symbol.selection_range.start.line,
            ))),
            None => Ok(Some(LspData::default())),
        }
    }

    /// Iterate over list of found symbols and try to match the parameters.
    fn find_nested_symbol<'a>(
        &self,
        symbols: &'a [DocumentSymbol],
        node_ref: &NodeRef,
        path: &str,
    ) -> Option<&'a DocumentSymbol> {
        let (current, remainder) = path.split_once('/').unwrap_or((path, ""));

        for symbol in symbols {
            let name = convert_name(&symbol.name);

            debug!(
                "Matching symbol '{name}' ({:?}) with '{}' + '{}'",
                symbol.kind, current, remainder
            );

            if name != current {
                continue;
            }
            if remainder.is_empty() && node_ref.params.matches_kind(symbol.kind) {
                return Some(symbol);
            }
            if let Some(symbols) = &symbol.children {
                let nested = self.find_nested_symbol(symbols, node_ref, remainder);
                if nested.is_some() {
                    return nested;
                }
            }
        }

        None
    }

    /// Iterate recursevly over nested document symbols to find one under the cursor.
    fn find_nested_symbol_in_position<'a>(
        &self,
        symbols: &'a [DocumentSymbol],
        line: u32,
        char: u32,
    ) -> Vec<&'a DocumentSymbol> {
        let mut stack = Vec::new();

        fn lookup<'al>(
            list: &'al [DocumentSymbol],
            stack: &mut Vec<&'al DocumentSymbol>,
            line: u32,
            char: u32,
        ) -> bool {
            for symbol in list {
                let range = &symbol.selection_range;

                debug!(
                    "Matching symbol: {}, range: {}:{}..{}:{}",
                    symbol.name,
                    range.start.line,
                    range.start.character,
                    range.end.line,
                    range.end.character,
                );

                if range.start.line <= line
                    && range.start.character <= char
                    && range.end.line >= line
                    && range.end.character >= char
                {
                    stack.push(symbol);
                    return true;
                }

                let symbols = unwrap_some_or!(&symbol.children, { continue });
                stack.push(symbol);

                if !lookup(symbols, stack, line, char) {
                    stack.pop();
                }
            }
            false
        }

        lookup(symbols, &mut stack, line, char);
        stack
    }
}

/// Remove extra symbols from name and replace spaces and special chars with '-'.
fn convert_name(name: &str) -> String {
    let mut out = String::new();
    let mut last = '-';
    for c in name.trim().chars() {
        let char = match c {
            '?' | '@' | ':' | '!' | '$' | '&' | '(' | ')' | '*' | '-' | '+' | '=' | '~' | '.'
            | '_' => c,
            _ if c.is_alphanumeric() => c,
            _ => '+',
        };
        if char != '+' || last != '+' {
            out.push(char);
        }
        last = char;
    }
    out
}

/// Produce LSP reference from nested stack.
fn convert_stack(path: &str, stack: Vec<&DocumentSymbol>) -> String {
    let mut base = format!("lsp://{path}#");
    let mut iter = stack.iter().peekable();
    while let Some(symbol) = iter.next() {
        base.push_str(&convert_name(&symbol.name));
        if iter.peek().is_some() {
            base.push('/');
        }
    }
    base
}

#[test]
fn name_conversion() {
    assert_eq!(
        convert_name("impl MyTrait   for   MyType<R>"),
        "impl+MyTrait+for+MyType+R+"
    );
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

#[derive(Default)]
pub(crate) struct LspData {
    pub hover: String,
    pub location: String,
}

impl LspData {
    fn from_hover(hover: Hover, path: &str, line: u32) -> Self {
        match hover.contents {
            HoverContents::Markup(content) => Self {
                hover: content.value,
                location: format!("{}:{}", path, line + 1),
            },
            _ => Default::default(),
        }
    }
}

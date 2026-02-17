use std::io::{self, BufReader, Write as _};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

use anyhow::Context as _;
use lsp_types::{notification::Notification, request::Request};
use serde_json::json;

/// LSP client instance with in-out references.
pub(crate) struct LspClient {
    #[expect(dead_code)]
    reader: BufReader<ChildStdout>,
    writer: ChildStdin,

    /// Message ID which gets auto-incremented on each written message.
    message_id: i64,
}

impl LspClient {
    /// Spawn LSP server child process and attach stdin/stdout.
    pub(crate) fn new(cmd: &str) -> anyhow::Result<Self> {
        let mut child = Command::new(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("Unable to spawn LSP server: {cmd}"))?;

        let reader = BufReader::new(
            child
                .stdout
                .take()
                .context("Unable to attach child stdout")?,
        );
        let writer = child.stdin.take().context("Unable to attach child stdin")?;

        Ok(Self {
            reader,
            writer,
            message_id: 1,
        })
    }

    /// Call provided method with parameters and wait for the reply. Increase
    /// message ID count.
    #[expect(dead_code)]
    pub(crate) fn request<R: Request>(&mut self, _method: &str) -> anyhow::Result<()> {
        let body = serde_json::to_string(&json!({
            "jsonrpc": "2.0",
            "id": self.message_id,
            "method": R::METHOD,
            "params": "",
        }))?;
        self.write_content(&body)?;

        // TODO: wait for reply.

        self.message_id += 1;
        Ok(())
    }

    /// Send a notification to LSP server.
    #[expect(dead_code)]
    pub(crate) fn notify<N: Notification>(&mut self) -> anyhow::Result<()> {
        let body = serde_json::to_string(&json!({
            "jsonrpc": "2.0",
            "method": N::METHOD,
            "params": "",
        }))?;
        self.write_content(&body)
            .context("Unable to write notification")
    }
}

impl LspClient {
    /// Write data along with content length.
    fn write_content(&mut self, content: &str) -> io::Result<()> {
        write!(
            self.writer,
            "Content-Length: {}\r\n\r\n{}",
            content.len(),
            content
        )?;
        self.writer.flush()
    }
}

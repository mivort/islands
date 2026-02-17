use std::io::{self, BufRead as _, BufReader, Read as _, Write as _};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

use anyhow::Context as _;
use lsp_types::{notification::Notification, request::Request};
use serde_json::{Value, json};

/// LSP client instance with in-out references.
pub(crate) struct LspClient {
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
    pub(crate) fn request<R: Request>(&mut self, params: R::Params) -> anyhow::Result<R::Result> {
        let body = serde_json::to_string(&json!({
            "jsonrpc": "2.0",
            "id": self.message_id,
            "method": R::METHOD,
            "params": serde_json::to_value(&params)?,
        }))?;
        self.write_content(&body)?;
        self.message_id += 1;

        loop {
            let mut content = self.read_content()?;
            if content.get("id").and_then(Value::as_i64) != Some(self.message_id) {
                continue;
            }
            let result = content
                .get_mut("result")
                .context("Missing result value")?
                .take();
            return Ok(serde_json::from_value(result).context("Result parsing failed")?);
        }
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

    /// Read response and parse as JSON.
    fn read_content(&mut self) -> anyhow::Result<serde_json::Value> {
        let mut content_length: Option<u64> = None;
        let mut line = String::new();

        loop {
            line.clear();
            self.reader.read_line(&mut line)?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some(val) = trimmed.strip_prefix("Content-Length: ") {
                content_length = Some(val.parse()?);
            }
        }

        let content_length = content_length.context("Missing Content-Length header")?;

        line.clear();

        // TODO: look for a way to avoid resize
        let mut buf = line.into_bytes();
        buf.resize(content_length as usize, 0);
        self.reader.read_exact(&mut buf)?;

        Ok(serde_json::from_slice(&buf).context("Response content parsing failed")?)
    }
}

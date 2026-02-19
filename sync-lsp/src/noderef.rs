use anyhow::Context as _;
use async_lsp::lsp_types::{SymbolKind, Url};
use serde_derive::Deserialize;
use unwrap_or::unwrap_some_or;

pub(crate) struct NodeRef {
    pub base: String,
    pub params: NodeRefParams,
}

impl NodeRef {
    pub fn base_ref(base: String) -> Self {
        Self {
            base,
            params: Default::default(),
        }
    }

    pub fn params_ref(base: String, params: NodeRefParams) -> Self {
        Self { base, params }
    }
}

#[derive(Default, Deserialize)]
pub(crate) struct NodeRefParams {
    pub kind: Option<SymbolKind>,
    pub path: Option<String>,
}

impl NodeRefParams {
    pub fn from_str(line: &str) -> anyhow::Result<Self> {
        serde_urlencoded::from_str(line).context("Unable to parse reference")
    }

    /// Check if node reference matches the specific symbol kind.
    pub fn matches_kind(&self, kind: SymbolKind) -> bool {
        let match_kind = unwrap_some_or!(self.kind, return true);
        match_kind == kind
    }

    /// Check if node reference matches symbol location.
    pub fn matches_uri(&self, uri: &Url) -> bool {
        let path = unwrap_some_or!(&self.path, return true);
        uri.path() == path
    }
}

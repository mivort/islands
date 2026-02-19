use anyhow::Context as _;
use serde_derive::Deserialize;

#[expect(unused)]
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

#[expect(unused)]
#[derive(Default, Deserialize)]
pub(crate) struct NodeRefParams {
    kind: Option<String>,
    file: Option<String>,
}

impl NodeRefParams {
    pub fn from_str(line: &str) -> anyhow::Result<Self> {
        serde_urlencoded::from_str(line).context("Unable to parse reference")
    }
}

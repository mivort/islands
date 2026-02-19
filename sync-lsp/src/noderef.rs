use anyhow::Context as _;
use async_lsp::lsp_types::{SymbolKind, Url};
use serde::Deserialize;
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
    pub kind: Option<KindMarker>,
    pub path: Option<String>,
}

impl NodeRefParams {
    pub fn from_str(line: &str) -> anyhow::Result<Self> {
        serde_urlencoded::from_str(line).context("Unable to parse reference")
    }

    /// Check if node reference matches the specific symbol kind.
    pub fn matches_kind(&self, kind: SymbolKind) -> bool {
        let match_kind = unwrap_some_or!(&self.kind, return true);
        match_kind.to_kind() == kind
    }

    /// Check if node reference matches symbol location.
    pub fn matches_uri(&self, uri: &Url) -> bool {
        let path = unwrap_some_or!(&self.path, return true);
        uri.path() == path
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum KindMarker {
    File,
    Module,
    Namespace,
    Package,
    Class,
    Method,
    Property,
    Field,
    Constructor,
    Enum,
    Interface,
    Function,
    Variable,
    Constant,
    String,
    Number,
    Boolean,
    Array,
    Object,
    Key,
    Null,
    EnumMember,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

impl KindMarker {
    pub fn to_kind(&self) -> SymbolKind {
        match self {
            Self::File => SymbolKind::FILE,
            Self::Module => SymbolKind::MODULE,
            Self::Namespace => SymbolKind::NAMESPACE,
            Self::Package => SymbolKind::PACKAGE,
            Self::Class => SymbolKind::CLASS,
            Self::Method => SymbolKind::METHOD,
            Self::Property => SymbolKind::PROPERTY,
            Self::Field => SymbolKind::FIELD,
            Self::Constructor => SymbolKind::CONSTRUCTOR,
            Self::Enum => SymbolKind::ENUM,
            Self::Interface => SymbolKind::INTERFACE,
            Self::Function => SymbolKind::FUNCTION,
            Self::Variable => SymbolKind::VARIABLE,
            Self::Constant => SymbolKind::CONSTANT,
            Self::String => SymbolKind::STRING,
            Self::Number => SymbolKind::NUMBER,
            Self::Boolean => SymbolKind::BOOLEAN,
            Self::Array => SymbolKind::ARRAY,
            Self::Object => SymbolKind::OBJECT,
            Self::Key => SymbolKind::KEY,
            Self::Null => SymbolKind::NULL,
            Self::EnumMember => SymbolKind::ENUM_MEMBER,
            Self::Struct => SymbolKind::STRUCT,
            Self::Event => SymbolKind::EVENT,
            Self::Operator => SymbolKind::OPERATOR,
            Self::TypeParameter => SymbolKind::TYPE_PARAMETER,
        }
    }
}

use anyhow::Context as _;
use async_lsp::lsp_types::SymbolKind;
use serde::Deserialize;
use unwrap_or::unwrap_some_or;

#[derive(Default)]
pub(crate) struct NodeRef {
    pub schema: RefType,
    pub path: String,
    pub params: NodeRefParams,
    pub hash: String,
}

impl NodeRef {
    /// Process the reference and handle the supported ones.
    pub fn parse_ref(node_ref: &str) -> anyhow::Result<Self> {
        let node_ref = node_ref.trim();
        let (schema, path) = node_ref
            .split_once(':')
            .context("Reference has missing schema")?;

        let schema = match schema {
            "lsp" => RefType::Lsp,
            "file" => {
                return Ok(Self {
                    schema: RefType::File,
                    path: path.strip_prefix("//").unwrap_or(path).into(),
                    ..Default::default()
                });
            }
            _ => return Ok(Self::default()),
        };

        let path = path.strip_prefix("//").unwrap_or(path);
        let (path, params, hash) = if let Some((path, all_params)) = path.split_once('?') {
            if let Some((params, hash)) = all_params.split_once('#') {
                (path, params, hash)
            } else {
                (path, all_params, "")
            }
        } else {
            if let Some((path, hash)) = path.split_once('#') {
                (path, "", hash)
            } else {
                (path, "", "")
            }
        };

        Ok(Self {
            schema,
            path: path.into(),
            params: NodeRefParams::from_str(params)?,
            hash: urlencoding::decode(hash)?.into(),
        })
    }
}

#[derive(Default)]
pub(crate) enum RefType {
    Lsp,
    File,
    #[default]
    Unknown,
}

#[derive(Default, Deserialize)]
pub(crate) struct NodeRefParams {
    /// Symbol kind.
    pub kind: Option<KindMarker>,

    /// Symbol container value.
    #[expect(unused)]
    pub container: Option<String>,
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

#[test]
fn parse_ref() {
    let node_ref = NodeRef::parse_ref("lsp://src/main.rs?kind=function#main").unwrap();
    assert!(matches!(node_ref.schema, RefType::Lsp));
    assert!(matches!(node_ref.params.kind, Some(KindMarker::Function)));
    assert_eq!(node_ref.path, "src/main.rs");
    assert_eq!(node_ref.hash, "main");
}

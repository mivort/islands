use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Context as _;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

/// Graph data which loosely follows Cytoscape.js format, along with some
/// additional properties.
#[derive(Serialize, Deserialize)]
pub(crate) struct Graph {
    pub nodes: Vec<Entry>,
    pub edges: Vec<Entry>,

    // TODO: store settings on this level
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

/// Node or edge entry.
#[derive(Serialize, Deserialize)]
pub(crate) struct Entry {
    pub group: Group,
    pub data: Data,
    pub position: Position,
    pub removed: bool,
    pub selected: bool,
    pub selectable: bool,
    pub locked: bool,
    pub grabbable: bool,
    pub pannable: bool,
    pub classes: String,
}

/// Single entry, either node or edge.
#[derive(Serialize, Deserialize)]
pub(crate) struct Data {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub r#ref: Option<String>,

    /// Result of reference check - applied during sync.
    pub valid: Option<bool>,

    /// Location of referenced item - applied during sync.
    pub refloc: Option<String>,

    /// Attached document for referenced item - applied during sync.
    pub refdoc: Option<String>,

    /// Remaining fields to maintain the custom metadata on serialization.
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Group {
    Nodes,
    Edges,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Position {
    x: f64,
    y: f64,
}

impl Graph {
    pub fn from_json(path: &Path) -> anyhow::Result<Self> {
        serde_json::from_str(&fs::read_to_string(path).context("Unable to read graph JSON file")?)
            .context("Unable to parse graph JSON file")
    }
}

use std::fs;
use std::path::Path;

use anyhow::Context as _;
use indexmap::IndexMap;
use serde::Serializer;
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
    pub data: IndexMap<String, Value>,
}

/// Node or edge entry.
#[derive(Serialize, Deserialize)]
pub(crate) struct Entry {
    pub data: Data,
    pub position: Position,
    pub group: Group,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,

    /// Result of reference check - applied during sync.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid: Option<bool>,

    /// Location of referenced item - applied during sync.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// Attached document for referenced item - applied during sync.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,

    /// Remaining fields to maintain the custom metadata on serialization.
    #[serde(flatten)]
    pub data: IndexMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Group {
    Nodes,
    Edges,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Position {
    #[serde(serialize_with = "serialize_zero_as_int")]
    x: f64,
    #[serde(serialize_with = "serialize_zero_as_int")]
    y: f64,
}

impl Graph {
    pub fn from_json(path: &Path) -> anyhow::Result<Self> {
        serde_json::from_str(&fs::read_to_string(path).context("Unable to read graph JSON file")?)
            .context("Unable to parse graph JSON file")
    }
}

fn serialize_zero_as_int<S: Serializer>(x: &f64, s: S) -> Result<S::Ok, S::Error> {
    if *x == 0.0 {
        s.serialize_i64(0)
    } else {
        s.serialize_f64(*x)
    }
}

use serde_derive::{Deserialize, Serialize};

/// Graph data which loosely follows Cytoscape.js format, along with some
/// additional properties.
#[expect(unused)]
#[derive(Serialize, Deserialize)]
pub(crate) struct Graph {
    nodes: Vec<Entry>,
    edges: Vec<Entry>,
    // TODO: store settings on this level
}

/// Node or edge entry.
#[derive(Serialize, Deserialize)]
pub(crate) struct Entry {
    group: Group,
    data: Data,
    position: Position,
    removed: bool,
    selected: bool,
    selectable: bool,
    locked: bool,
    grabbable: bool,
    pannable: bool,
    classes: String,
}

/// Single entry, either node or edge.
#[derive(Serialize, Deserialize)]
pub(crate) struct Data {
    name: String,
    desc: String,
    r#ref: String,

    /// Location of referenced item - applied during sync.
    refloc: Option<String>,

    /// Attached document for referenced item - applied during sync.
    refdoc: Option<String>,
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

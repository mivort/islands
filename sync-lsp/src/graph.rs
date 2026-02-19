use serde_derive::{Deserialize, Serialize};

/// Graph data which follows Cytoscape.js format.
#[expect(unused)]
#[derive(Serialize, Deserialize)]
pub(crate) struct Graph {
    group: Group,
    nodes: Vec<Entry>,
}

/// Single entry, either node or edge.
#[derive(Serialize, Deserialize)]
pub(crate) struct Entry {}

#[derive(Serialize, Deserialize)]
pub(crate) enum Group {
    Nodes,
    Edges,
}

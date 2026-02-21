# Islands

Network graph drawing tool with support for LSP-based node relevance
validation.

The repo consists of the several modules:
* `/editor`: the graph editor based on [Cytoscape.js][cytoscape].
* `/sync-lsp`: sync utility which communicate with LSP server to resolve
  `lsp://` references in the graph.

* [cytoscape]: https://js.cytoscape.org/

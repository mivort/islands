<img src="docs/assets/logo.png" alt="Islands logo" />

----

Network graph drawing tool with support for LSP-based node relevance
validation.

The repo consists of the several modules:
* `/editor`: the graph editor based on [Cytoscape.js][cytoscape]. The editor is
  compiled as self-contained HTML file, with static instance hosted on [Github
  pages][editor]. The instance page can be downloaded and run locally.
* `/sync-lsp`: sync utility which communicate with LSP server to resolve
  `lsp://` references in the graph.

[editor]: https://mivort.github.io/islands-editor/
[cytoscape]: https://js.cytoscape.org/

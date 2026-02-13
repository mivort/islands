import cytoscape from "./cytoscape.umd.js";

document.addEventListener('DOMContentLoaded', () => {
  cytoscape({
    container: document.getElementById('root'),
    elements: [
      {
        group: 'nodes',
        data: { id: 'n1' },
        position: { x: 0, y: 0 },
      },
      {
        group: 'nodes',
        data: { id: 'n2' },
        position: { x: 100, y: 100 },
      },
      {
        group: 'nodes',
        data: { id: 'n3' },
        position: { x: 0, y: 100 },
      },
      {
        group: 'nodes',
        data: { id: 'n4' },
        position: { x: 0, y: 200 },
      },
    ],
    layout: { name: 'preset' },
  });
});

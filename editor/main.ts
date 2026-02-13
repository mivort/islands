import cytoscape from "cytoscape";

document.addEventListener('DOMContentLoaded', () => {
  cytoscape({
    container: document.getElementById('root'),
    elements: [
      { data: { id: 'n1' }, position: { x: 0, y: 0 } },
      { data: { id: 'n2' }, position: { x: 100, y: 100 } },
      { data: { id: 'n3', parent: 'n5' }, position: { x: 0, y: 100 } },
      { data: { id: 'n4', parent: 'n5' }, position: { x: 0, y: 200 } },
      { data: { id: 'n5' }, position: { x: 100, y: 200 } },

      { group: 'edges', data: { id: 'n1n2', source: 'n1', target: 'n2' } },
      { group: 'edges', data: { id: 'n1n5', source: 'n1', target: 'n5' } },
      { group: 'edges', data: { id: 'n3n4', source: 'n3', target: 'n4' } },
    ],
    layout: { name: 'preset' },
  });
});

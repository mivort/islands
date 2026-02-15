import cytoscape from 'cytoscape';
import contextMenus from 'cytoscape-context-menus';
import { SidePanel } from './side';

cytoscape.use(contextMenus);

import 'cytoscape-context-menus/cytoscape-context-menus.css';

document.addEventListener('DOMContentLoaded', () => {
  const cy = cytoscape({
    container: document.getElementById('root'),
    elements: [],
    layout: { name: 'preset' },
    style: [
      {
        selector: 'node',
        css: {
          'border-color': '#333',
          'border-width': '2',
        },
      },
      {
        selector: 'node[name]',
        css: {
          'label': 'data(name)',
          'font-family': 'monospace',
          'text-wrap': 'wrap',
          'text-margin-y': -2,
        },
      },
      {
        selector: ':parent',
        css: {
          'shape': 'round-rectangle',
          'corner-radius': '10',
          'border-color': '#333',
          'background-color': '#fff',
          'background-opacity': 0.1,
        },
      },
      {
        selector: ':parent:selected',
        css: {
          'background-color': '#0169d9',
          'background-opacity': 1,
        },
      },
      {
        selector: 'edge',
        css: {
          'curve-style': 'straight',
          'target-arrow-shape': 'tee',
          'target-arrow-color': '#333',
          'line-cap': 'square',
          'line-outline-width': '3',
          'line-outline-color': '#333',
          'source-distance-from-node': 4,
          'target-distance-from-node': 4,
          'arrow-scale': 0.6,
        },
      },
    ],
  });

  const side = new SidePanel(cy);

  cy.on('cxttap', () => {
    const nodeSelected = cy.nodes(':selected').length > 0;
    if (nodeSelected) {
      menus.showMenuItem('link');
      menus.showMenuItem('parent');
      menus.showMenuItem('add-node-linked');
      menus.showMenuItem('add-node-child');
      menus.showMenuItem('unparent');
    } else {
      menus.hideMenuItem('link');
      menus.hideMenuItem('parent');
      menus.hideMenuItem('add-node-linked');
      menus.hideMenuItem('add-node-child');
      menus.hideMenuItem('unparent');
    }
  });

  cy.on('select', () => side.showSelected());
  cy.on('unselect', () => side.showSelected());

  /** Create edges starting at selected nodes. */
  const linkWithSelected = (target: string) => {
    const nodes = cy.nodes(':selected');
    for (const node of nodes) {
      cy.add({ data: { group: 'edges', source: node.id(), target } });
    }
  };

  const menus = cy.contextMenus({
    menuItems: [
      {
        id: 'add-node',
        content: 'Add node',
        coreAsWell: true,
        selector: '',
        onClickFunction: (event) => {
          cy.add({
            data: { group: 'nodes' },
            position: {
              x: event.position.x,
              y: event.position.y,
            },
          });
        },
      },
      {
        id: 'add-node-linked',
        content: 'Add linked node',
        coreAsWell: true,
        selector: '',
        onClickFunction: (event) => {
          const nodes = cy.add({
            data: { group: 'nodes' },
            position: {
              x: event.position.x,
              y: event.position.y,
            },
          });
          for (const node of nodes) {
            linkWithSelected(node.id());
          }
        },
      },
      {
        id: 'add-node-child',
        content: 'Add child node',
        coreAsWell: true,
        selector: '',
        onClickFunction: (event) => {
          const selected = cy.nodes(':selected')[0];
          if (!selected) return;
          cy.add({
            data: { group: 'nodes', parent: selected.id() },
            position: {
              x: event.position.x,
              y: event.position.y,
            },
          });
        },
      },
      {
        id: 'unparent',
        content: 'Unparent',
        coreAsWell: true,
        selector: '',
        onClickFunction: () => {
          const nodes = cy.nodes(':selected');
          for (const node of nodes) {
            node.move({ parent: null });
          }
        },
      },
      {
        id: 'link',
        content: 'Link',
        selector: 'node',
        onClickFunction: (event) => {
          const target = event.target.id();
          linkWithSelected(target);
        },
      },
      {
        id: 'parent',
        content: 'Parent',
        selector: 'node',
        onClickFunction: (event) => {
          const parent = event.target.id();
          const nodes = cy.nodes(':selected');
          for (const node of nodes) {
            node.move({ parent });
          }
        },
      },
      {
        id: 'select',
        content: 'Select',
        selector: 'node, edge',
        onClickFunction: (event) => {
          event.target.select();
        },
      },
      {
        id: 'reverse',
        content: 'Reverse',
        selector: 'edge',
        onClickFunction: (event) => {
          const target = event.target.source().id();
          const source = event.target.target().id();
          const data = event.target.data();
          event.target.remove();
          cy.add({ data: { ...data, group: 'edges', source, target } });
        },
      },
      {
        id: 'remove',
        content: 'Remove',
        tooltipText: 'remove',
        selector: 'node, edge',
        onClickFunction: (event) => {
          if (!event.target.selected()) {
            event.target.remove();
            return;
          }
          for (const elem of cy.elements(':selected')) {
            elem.remove();
          }
        },
      },
    ],
  });

  document.body.addEventListener('keyup', (event) => {
    if (event.target !== document.body) {
      return;
    }
    switch (event.key) {
      case 'Delete':
        for (const elem of cy.elements(':selected')) {
          elem.remove();
        }
        break;
      default:
    }
  });
});

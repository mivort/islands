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
          'background-color': '#aaa',
        },
      },
      {
        selector: 'node[name]',
        css: {
          'label': 'data(name)',
          'font-family': 'monospace',
          'text-wrap': 'wrap',
          'text-margin-y': -2,
          'text-outline-width': 2,
          'text-outline-color': '#aaa',
        },
      },
      {
        selector: ':parent',
        css: {
          'shape': 'round-rectangle',
          'corner-radius': '10',
          'border-color': '#333',
          'background-color': '#999',
          'background-opacity': 0.3,
        },
      },
      {
        selector: 'node:parent:selected, node:selected',
        css: {
          'background-color': '#0169d9',
          'background-opacity': 1,
        },
      },
      {
        selector: 'edge',
        css: {
          'line-color': '#999',
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
      {
        selector: 'node.fade',
        css: {
          'background-opacity': 0.5,
          'border-opacity': 0.5,
        },
      },
      {
        selector: 'node.draft',
        css: {
          'border-style': 'dashed',
          'border-width': 3,
          'border-dash-pattern': [6, 3],
        },
      },
      {
        selector: 'edge.fade',
        css: {
          'line-opacity': 0.5,
        },
      },
      {
        selector: 'edge.draft',
        css: {
          'line-color': '#333',
          'line-style': 'dashed',
          'line-cap': 'butt',
          'line-outline-width': 0,
          'arrow-scale': 1,
        },
      },
      {
        selector: 'edge:selected',
        css: {
          'line-color': '#0169d9',
        },
      },
    ],
  });

  const side = new SidePanel(cy);

  /** Update multiple menu items visibility. */
  const setMenuItemsVisible = (items: string[], state: boolean) => {
    if (state) {
      for (const item of items) {
        menus.showMenuItem(item);
      }
    } else {
      for (const item of items) {
        menus.hideMenuItem(item);
      }
    }
  };

  cy.on('cxttap', () => {
    const nodes = cy.nodes(':selected');
    const nodeSelected = nodes.length > 0;
    const isParent = nodes.some((node) => (node as any).isParent());
    const isChild = nodes.some((node) => (node as any).isChild());

    setMenuItemsVisible(['link', 'parent', 'add-node-linked'], nodeSelected);
    setMenuItemsVisible(['add-node-child'], nodeSelected && isParent);
    setMenuItemsVisible(['unparent'], nodeSelected && isChild);
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
          const classes = event.target.classes();
          const data = event.target.data();
          event.target.remove();
          cy.add({ data: { ...data, group: 'edges', source, target }, classes });
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

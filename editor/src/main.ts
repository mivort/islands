import cytoscape from 'cytoscape';
import undoRedo from 'cytoscape-undo-redo';
import contextMenus from 'cytoscape-context-menus';
import { SidePanel } from './side';

cytoscape.use(contextMenus);
cytoscape.use(undoRedo);

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
        selector: 'node[ref]',
        css: {
          'outline-color': '#333',
          'outline-width': 2,
          'outline-style': 'dashed',
          'outline-offset': 2,
          'outline-opacity': 0.5,
        },
      },
      {
        selector: 'node[shape]',
        css: {
          'shape': 'data(shape)' as cytoscape.Css.NodeShape,
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

  const undo = (cy as any).undoRedo({
    stackSizeLimit: 1000,
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

    setMenuItemsVisible(['link-to', 'link-from', 'parent', 'add-node-linked'], nodeSelected);
    setMenuItemsVisible(['add-node-child'], nodeSelected && isParent);
    setMenuItemsVisible(['unparent'], nodeSelected && isChild);
  });

  cy.on('select', () => side.showSelected());
  cy.on('unselect', () => side.showSelected());

  /** Create edges starting at selected nodes. */
  const linkWithSelected = (target: string, reverse?: boolean) => {
    const nodes = cy.nodes(':selected');
    if (reverse) {
      for (const node of nodes) {
        cy.add({ group: 'edges', data: { target: node.id(), source: target } });
      }
      return;
    }
    for (const node of nodes) {
      cy.add({ group: 'edges', data: { source: node.id(), target } });
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
          undo.do('add', {
            group: 'nodes',
            data: {},
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
            group: 'nodes',
            data: {},
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
            group: 'nodes',
            data: { parent: selected.id() },
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
        id: 'link-to',
        content: 'Link to',
        selector: 'node',
        onClickFunction: (event) => {
          const target = event.target.id();
          linkWithSelected(target);
        },
      },
      {
        id: 'link-from',
        content: 'Link from',
        selector: 'node',
        onClickFunction: (event) => {
          const target = event.target.id();
          linkWithSelected(target, true);
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
          cy.add({ group: 'edges', data: { ...data, source, target }, classes });
        },
      },
      {
        id: 'lock',
        content: 'Lock',
        selector: 'node',
        onClickFunction: (event) => {
          if (event.target.locked()) {
            event.target.unlock();
            return;
          }
          event.target.lock();
        },
      },
      {
        id: 'remove',
        content: 'Remove',
        selector: 'node, edge',
        onClickFunction: (event) => {
          if (!event.target.selected()) {
            undo.do('remove', event.target);
            return;
          }
          undo.do('remove', cy.elements(':selected'));
        },
      },
    ],
  });

  document.body.addEventListener('keyup', (event) => {
    if (event.target !== document.body) {
      return;
    }
    switch (event.key) {
      case 'Backspace':
      case 'Delete':
        undo.do('remove', cy.elements(':selected'));
        break;
      default:
    }
  });
});

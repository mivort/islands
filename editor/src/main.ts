import cytoscape from 'cytoscape';
import contextMenus from 'cytoscape-context-menus';
import { SidePanel } from './side';

cytoscape.use(contextMenus);

import 'cytoscape-context-menus/cytoscape-context-menus.css';

document.addEventListener('DOMContentLoaded', () => {
  const style = (cytoscape as any).stylesheet();
  const cy = cytoscape({
    container: document.getElementById('root'),
    elements: [],
    layout: { name: 'preset' },
    style: style.selector('node[name]').css({
      'label': 'data(name)',
      'font-family': 'monospace',
    }),
  });

  const side = new SidePanel(cy);

  cy.on('cxttap', () => {
    const nodeSelected = cy.nodes(':selected').length > 0;
    if (nodeSelected) {
      menus.showMenuItem('link');
      menus.showMenuItem('parent');
      menus.showMenuItem('add-node-linked');
    } else {
      menus.hideMenuItem('link');
      menus.hideMenuItem('parent');
      menus.hideMenuItem('add-node-linked');
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
        id: 'remove',
        content: 'Remove',
        tooltipText: 'remove',
        selector: 'node, edge',
        onClickFunction: (event) => {
          if (!event.target.selected()) {
            event.target.remove();
            return;
          }
          const edges = cy.edges(':selected');
          for (const edge of edges) {
            edge.remove();
          }
          const nodes = cy.nodes(':selected');
          for (const node of nodes) {
            node.remove();
          }
        },
      },
    ],
  });
});

import cytoscape from 'cytoscape';
import contextMenus from 'cytoscape-context-menus';

cytoscape.use(contextMenus);

import 'cytoscape-context-menus/cytoscape-context-menus.css';

document.addEventListener('DOMContentLoaded', () => {
  const cy = cytoscape({
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

  cy.on('cxttap', () => {
    const nodeSelected = cy.nodes(':selected').length > 0;
    if (nodeSelected) {
      menus.showMenuItem('link');
      menus.showMenuItem('parent');
    } else {
      menus.hideMenuItem('link');
      menus.hideMenuItem('parent');
    }
  });

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
        id: 'link',
        content: 'Link',
        selector: 'node',
        onClickFunction: (event) => {
          const target = event.target.id();
          const nodes = cy.nodes(':selected');
          for (const node of nodes) {
            cy.add({ data: { group: 'edges', source: node.id(), target: target } });
          }
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
          const nodes = cy.nodes(':selected');
          for (const node of nodes) {
            node.remove();
          }
        },
      },
    ],
  });

  document.getElementById('side-fit-button')?.addEventListener('click', () => {
    cy.fit();
  });
});

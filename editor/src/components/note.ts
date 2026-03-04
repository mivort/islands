import cytoscape from 'cytoscape';
import { Data, ElementChangeEvent, Events } from '../data';

/**
 * Node/edge attached note editor.
 */
export const noteEdit = (cy: cytoscape.Core) => {
  const note = document.getElementById('side-edit-note') as HTMLTextAreaElement | null;

  if (!note) return;

  note.addEventListener('change', (event) => {
    const nodes = cy.nodes(':selected');
    const value = (event.target as HTMLTextAreaElement).value;

    for (const node of nodes) {
      node.data(Data.NOTE, value);
    }
  });

  window.addEventListener(Events.GRAPH_ELEMENT_CHANGE, (event) => {
    const element = (event as CustomEvent<ElementChangeEvent>).detail.element;
    note.disabled = !element;
    note.value = element?.data(Data.NOTE) ?? '';
  });
};

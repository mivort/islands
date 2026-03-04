import cytoscape from 'cytoscape';
import { Data, ElementChangeEvent, Events } from '../data';

/** Node/edge label editor. */
export const labelEdit = (cy: cytoscape.Core) => {
  const label = document.getElementById('side-edit-label') as HTMLTextAreaElement;

  if (!label) return;

  label.addEventListener('change', (event) => {
    const nodes = cy.elements(':selected');
    const value = (event.target as HTMLInputElement).value;
    if (value) {
      for (const node of nodes) {
        node.data(Data.LABEL, value);
      }
      return;
    }
    for (const node of nodes) {
      node.removeData(Data.LABEL);
    }
  });

  window.addEventListener(Events.GRAPH_ELEMENT_CHANGE, (event) => {
    const element = (event as CustomEvent<ElementChangeEvent>).detail.element;
    label.disabled = !element;
    label.value = element?.data(Data.LABEL) ?? '';
  });
};

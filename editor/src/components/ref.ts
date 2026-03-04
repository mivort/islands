import cytoscape from 'cytoscape';
import { Data, ElementChangeEvent, Events } from '../data';

/**
 * Node/edge reference link editor. Each time reference value changes, 'valid'
 * flag should be reset.
 */
export const refEdit = (cy: cytoscape.Core) => {
  const ref = document.getElementById('side-edit-ref') as HTMLInputElement | null;

  if (!ref) return;

  ref.addEventListener('change', (event) => {
    const nodes = cy.elements(':selected');
    const value = (event.target as HTMLInputElement).value.replace(/ /g, '');
    if (value) {
      for (const node of nodes) {
        node.data(Data.REF, value);
        node.removeData(Data.VALID);
      }
      return;
    }
    for (const node of nodes) {
      node.removeData(Data.REF);
      node.removeData(Data.VALID);
    }
  });

  window.addEventListener(Events.GRAPH_ELEMENT_CHANGE, (event) => {
    const element = (event as CustomEvent<ElementChangeEvent>).detail.element;
    ref.disabled = !element;
    ref.value = element?.data(Data.REF) ?? '';
  });
};

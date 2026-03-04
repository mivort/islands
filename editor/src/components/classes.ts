import cytoscape from 'cytoscape';
import { ElementChangeEvent, Events } from '../data';

/** Checkboxes for common node/edge classes. */
export const classesEdit = (cy: cytoscape.Core) => {
  const fade = document.getElementById('side-edit-class-fade') as HTMLInputElement | null;
  const draft = document.getElementById('side-edit-class-draft') as HTMLInputElement | null;
  const comment = document.getElementById('side-edit-class-comment') as HTMLInputElement | null;

  if (!fade || !draft || !comment) return;

  /** Toggle class depending on checkbox value. */
  const toggleClass = (name: string, checked: boolean) => {
    const elements = cy.elements(':selected');
    if (checked) {
      for (const elem of elements) {
        elem.addClass(name);
      }
      return;
    }
    for (const elem of elements) {
      elem.removeClass(name);
    }
  };

  fade.addEventListener('change', ({ target }) => {
    toggleClass('fade', (target as HTMLInputElement).checked);
  });
  draft.addEventListener('change', ({ target }) => {
    toggleClass('draft', (target as HTMLInputElement).checked);
  });
  comment.addEventListener('change', ({ target }) => {
    toggleClass('comment', (target as HTMLInputElement).checked);
  });

  window.addEventListener(Events.GRAPH_ELEMENT_CHANGE, (event) => {
    const element = (event as CustomEvent<ElementChangeEvent>).detail.element;
    fade.disabled = !element;
    draft.disabled = !element;
    comment.disabled = !element;

    fade.checked = element?.hasClass('fade') ?? false;
    draft.checked = element?.hasClass('draft') ?? false;
    comment.checked = element?.hasClass('comment') ?? false;
  });
};

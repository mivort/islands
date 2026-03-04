import { ElementChangeEvent, Events, Data } from '../data';
import { marked } from 'marked';

export const docView = () => {
  /** Reference to the view pane. */
  const view = document.getElementById('side-view-doc');
  if (!view) return;

  window.addEventListener(Events.GRAPH_ELEMENT_CHANGE, (event) => {
    const element = (event as CustomEvent<ElementChangeEvent>).detail.element;
    view.innerHTML = marked.parse(element?.data(Data.DOC) ?? '') as string;
  });
};

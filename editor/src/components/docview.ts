import { ElementChangeEvent, Events, Data } from '../data';
import { marked } from 'marked';

export class DocView {
  /** Last rendered ID. */
  id: string;

  /** Reference to the view pane. */
  view: HTMLElement;

  constructor() {
    const view = document.getElementById('side-view-doc');
    if (!view) return;

    this.view = view;

    window.addEventListener(Events.GRAPH_ELEMENT_CHANGE, (event) => {
      const element = (event as CustomEvent<ElementChangeEvent>).detail.element;
      this.view.innerHTML = marked.parse(element?.data(Data.DOC) ?? '') as string;
    });
  }
}

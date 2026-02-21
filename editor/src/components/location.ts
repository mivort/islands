import { Data, ElementChangeEvent, Events } from "../data";

/** Location view with the copy button alongside. */
export class LocationView {
  container: HTMLElement;
  value: HTMLElement;

  constructor() {
    const container = document.getElementById('side-view-location');
    const value = document.getElementById('side-view-location-value');
    const copy = document.getElementById('side-view-location-copy');

    if (!value || !container || !copy) return;

    this.container = container;
    this.value = value;

    window.addEventListener(Events.GRAPH_ELEMENT_CHANGE, (event) => {
      const element = (event as CustomEvent<ElementChangeEvent>).detail.element;
      this.value.innerText = element?.data(Data.LOCATION) ?? '';
    });

    copy.addEventListener('click', () => {
      navigator.clipboard.writeText(this.value.innerText);
    });
  }
}

import { Data } from "../data";

/** Location view with the copy button alongside. */
export class LocationView {
  container: HTMLElement;
  value: HTMLElement;

  constructor(cy: cytoscape.Core) {
    const container = document.getElementById('side-view-location');
    const value = document.getElementById('side-view-location-value');
    const copy = document.getElementById('side-view-location-copy');

    if (!value || !container || !copy) return;

    this.container = container;
    this.value = value;

    // TODO: subscribe to custom 'current node' event
    cy.addListener('select unselect', () => {
      const elements = cy.elements(':selected');
      console.log('select' + elements.length);
      if (elements.length === 0) return;

      this.value.innerText = elements[0].data(Data.LOCATION) ?? '';
    });

    copy.addEventListener('click', () => {
      navigator.clipboard.writeText(this.value.innerText);
    });
  }
}

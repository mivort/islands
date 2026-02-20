/** Location view with the copy button alongside. */
export class LocationView {
  container: HTMLElement;
  value: HTMLElement;
  copy: HTMLElement;

  constructor() {
    const container = document.getElementById('side-view-location');
    const value = document.getElementById('side-view-location-value');
    const copy = document.getElementById('side-view-location-copy');

    if (!value || !container || !copy) return;

    this.container = container;
    this.value = value;
    this.copy = copy;

    // TODO: subscribe to selection event
  }
}

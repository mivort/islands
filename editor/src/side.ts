import cytoscape from 'cytoscape';
import { Data, ElementChangeEvent, Events } from './data';
import { classesEdit } from './components/classes';
import { docView } from './components/docview';
import { labelEdit } from './components/label';
import { locationView } from './components/location';

/** Side panel state. */
export class SidePanel {
  cy: cytoscape.Core;
  id: HTMLElement | null;
  ref: HTMLInputElement | null;
  shape: HTMLSelectElement | null;
  size: HTMLInputElement | null;
  desc: HTMLTextAreaElement | null;

  /** ID of the currently displayed node or edge in the side panel. */
  current: string | null;

  /** Current file name for saving. */
  filename = 'islands.json';

  /** Update the display of selected nodes. */
  showSelected() {
    const elements = this.cy.elements(':selected');

    let selection: string | null;

    if (elements.length === 0) {
      selection = null;
      this.showId('');
      this.hideElement();
    } else if (elements.length === 1) {
      selection = elements[0].id();
      this.showId(selection);
      this.showElement(elements[0]);
    } else {
      selection = elements[0].id();
      this.showId(`<${elements.length} selected>`);
      this.showElement(elements[0]);
    }

    if (this.current != selection) {
      this.current = selection;
      window.dispatchEvent(new CustomEvent<ElementChangeEvent>(Events.GRAPH_ELEMENT_CHANGE, {
        detail: { element: elements[0] },
      }));
    }
  }

  /** Show the title of the currently open file. */
  showTitle() {
    document.title = `Islands - ${this.filename}`;
  }

  /** Show selected node ID. */
  showId(id: string) {
    if (!this.id) return;
    this.id.innerText = id;
  }

  /** Show specified node's attached data. */
  showElement(node: cytoscape.NodeSingular) {
    if (this.ref) this.ref.value = node.data(Data.REF) ?? '';
    if (this.shape) this.shape.value = node.data(Data.SHAPE) ?? '';
    if (this.size) this.size.value = node.data(Data.SIZE) ?? 25;
    if (this.desc) this.desc.value = node.data(Data.NOTE) ?? '';
  }

  /** Hide display of node data. */
  hideElement() {
    if (this.ref) this.ref.value = '';
    if (this.desc) this.desc.value = '';
  }

  constructor(cy: cytoscape.Core) {
    this.cy = cy;
    this.id = document.getElementById('side-edit-id');
    this.ref = document.getElementById('side-edit-ref') as HTMLInputElement;
    this.shape = document.getElementById('side-edit-shape') as HTMLSelectElement;
    this.size = document.getElementById('side-edit-size') as HTMLInputElement;
    this.desc = document.getElementById('side-edit-desc') as HTMLTextAreaElement;

    labelEdit(cy);
    locationView();
    docView();
    classesEdit(cy);

    this.ref?.addEventListener('change', (event) => {
      const nodes = this.cy.elements(':selected');
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
    this.shape?.addEventListener('change', (event) => {
      const nodes = this.cy.nodes(':selected');
      const value = (event.target as any).value;
      if (value) {
        for (const node of nodes) {
          node.data(Data.SHAPE, value);
        }
        return;
      }
      for (const node of nodes) {
        node.removeData(Data.SHAPE);
      }
    });
    this.size?.addEventListener('change', (event) => {
      const nodes = this.cy.nodes(':selected');
      const value = parseFloat((event.target as any).value);
      if (!isNaN(value) && value > 0) {
        for (const node of nodes) {
          node.data(Data.SIZE, value);
        }
        return;
      }
      for (const node of nodes) {
        node.removeData(Data.SIZE);
      }
    });
    this.desc?.addEventListener('change', (event) => {
      const nodes = this.cy.elements(':selected');
      const value = (event.target as HTMLInputElement).value;
      for (const node of nodes) {
        node.data(Data.NOTE, value);
      }
    });

    document.getElementById('side-undo-button')?.addEventListener('click', () => {
      (this.cy as any).undoRedo({}, true).undo();
    });
    document.getElementById('side-redo-button')?.addEventListener('click', () => {
      (this.cy as any).undoRedo({}, true).redo();
    });

    document.getElementById('side-fit-button')?.addEventListener('click', () => {
      cy.fit();
    });
    document.getElementById('side-fit-sel-button')?.addEventListener('click', () => {
      cy.fit(cy.elements(':selected'));
    });

    document.getElementById('side-export-button')?.addEventListener('click', () => {
      const json = JSON.stringify(cy.json().elements, null, 2);
      const data = json[Symbol.iterator]();
      const file = new File(data as any, this.filename, { type: 'application/octet-stream' });
      const url = URL.createObjectURL(file);

      const a = document.createElement('a');
      a.href = url;
      a.download = this.filename;

      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);

      URL.revokeObjectURL(url);
    });

    const filePicker = document.getElementById('side-file-picker') as HTMLInputElement;
    filePicker?.addEventListener('change', () => {
      const files = filePicker.files;
      if (!files || files?.length === 0) {
        return;
      }
      cy.elements().remove();
      for (const file of files) {
        this.filename = file.name;
        const reader = new FileReader();
        reader.readAsText(file, 'UTF-8');
        reader.onload = ({ target }) => {
          if (!target?.result) return;
          try {
            const json = JSON.parse(target.result as string);
            cy.json({
              elements: json
            });
          }
          catch {
            console.log("JSON parsing failed");
          }
        };
      }
      filePicker.value = '';
      this.showTitle();
    });
  }
}

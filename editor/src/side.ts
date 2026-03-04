import cytoscape from 'cytoscape';
import { Data, ElementChangeEvent, Events } from './data';
import { classesEdit } from './components/classes';
import { docView } from './components/docview';
import { labelEdit } from './components/label';
import { locationView } from './components/location';
import { refEdit } from './components/ref';
import { noteEdit } from './components/note';

/** Side panel state. */
export class SidePanel {
  cy: cytoscape.Core;
  id: HTMLElement | null;
  shape: HTMLSelectElement | null;
  size: HTMLInputElement | null;

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
    if (this.shape) this.shape.value = node.data(Data.SHAPE) ?? '';
    if (this.size) this.size.value = node.data(Data.SIZE) ?? 25;
  }

  constructor(cy: cytoscape.Core) {
    this.cy = cy;
    this.id = document.getElementById('side-edit-id');
    this.shape = document.getElementById('side-edit-shape') as HTMLSelectElement;
    this.size = document.getElementById('side-edit-size') as HTMLInputElement;

    labelEdit(cy);
    refEdit(cy);
    noteEdit(cy);
    locationView();
    docView();
    classesEdit(cy);

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

import cytoscape from 'cytoscape';

/** Side panel state. */
export class SidePanel {
  cy: cytoscape.Core;
  id: HTMLElement | null;
  name: HTMLTextAreaElement | null;
  ref: HTMLInputElement | null;
  desc: HTMLTextAreaElement | null;

  /** Update the display of selected nodes. */
  showSelected() {
    const nodes = this.cy.nodes(':selected');
    if (nodes.length === 0) {
      this.showId('');
      this.hideNode();
    } else if (nodes.length === 1) {
      this.showId(nodes[0].id());
      this.showNode(nodes[0]);
    } else {
      this.showId(`<${nodes.length} selected>`);
      this.showNode(nodes[0]);
    }
  }

  /** Show selected node ID. */
  showId(id: string) {
    if (!this.id) return;
    this.id.innerText = id;
  }

  /** Show specified node's attached data. */
  showNode(node: cytoscape.NodeSingular) {
    if (this.name) this.name.value = node.data('name') ?? '';
    if (this.ref) this.ref.value = node.data('ref') ?? '';
    if (this.desc) this.desc.value = node.data('desc') ?? '';
  }

  /** Hide display of node data. */
  hideNode() {
    if (this.name) this.name.value = '';
    if (this.ref) this.ref.value = '';
    if (this.desc) this.desc.value = '';
  }

  constructor(cy: cytoscape.Core) {
    this.cy = cy;
    this.id = document.getElementById('side-edit-id');
    this.name = document.getElementById('side-edit-name') as HTMLTextAreaElement;
    this.ref = document.getElementById('side-edit-ref') as HTMLInputElement;
    this.desc = document.getElementById('side-edit-desc') as HTMLTextAreaElement;

    this.name?.addEventListener('change', (event) => {
      const nodes = this.cy.nodes(':selected');
      for (const node of nodes) {
        node.data('name', (event.target as any).value);
      }
    });
    this.ref?.addEventListener('change', (event) => {
      const nodes = this.cy.nodes(':selected');
      for (const node of nodes) {
        node.data('ref', (event.target as any).value);
      }
    });
    this.desc?.addEventListener('change', (event) => {
      const nodes = this.cy.nodes(':selected');
      for (const node of nodes) {
        node.data('desc', (event.target as any).value);
      }
    });

    document.getElementById('side-fit-button')?.addEventListener('click', () => {
      cy.fit();
    });
    document.getElementById('side-fit-sel-button')?.addEventListener('click', () => {
      cy.fit(cy.nodes(':selected'));
    });

    document.getElementById('side-export-button')?.addEventListener('click', () => {
      const json = JSON.stringify(cy.json().elements, null, 2);
      const data = json[Symbol.iterator]();
      const file = new File(data as any, "islands.json", { type: 'application/octet-stream' });
      const url = URL.createObjectURL(file);

      window.open(url);
      URL.revokeObjectURL(url);
    });

    const filePicker = document.getElementById('side-file-picker');
    filePicker?.addEventListener('change', () => {
      const files = (filePicker as any).files;
      if (files.length === 0) {
        return;
      }
      cy.elements().remove();
      for (const file of files) {
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
    });
  }
}

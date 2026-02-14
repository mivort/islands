import cytoscape from 'cytoscape';

/** Side panel state. */
export class SidePanel {
  cy: cytoscape.Core;
  id: HTMLElement | null;

  /** Update the display of selected nodes. */
  showSelected() {
    const nodes = this.cy.nodes(':selected');
    if (nodes.length === 0) {
      this.showId('');
    } else if (nodes.length === 1) {
      this.showId(nodes[0].id());
    } else {
      this.showId(`<${nodes.length} selected>`);
    }
  }

  /** Show selected node ID. */
  showId(id: string) {
    if (!this.id) return;
    this.id.innerText = id;
  }

  constructor(cy: cytoscape.Core) {
    this.cy = cy;
    this.id = document.getElementById('side-edit-id');

    document.getElementById('side-fit-button')?.addEventListener('click', () => {
      cy.fit();
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

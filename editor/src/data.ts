/** Available data fields. */
export enum Data {
  /** Text label drawn over the node. */
  NAME = 'name',
  /** Selected node shape. */
  SHAPE = 'shape',
  /** Node visible size. */
  SIZE = 'size',

  REF = 'ref',
  VALID = 'valid',
  DESC = 'desc',
  DOC = 'doc',
  LOCATION = 'location',
}

/** List of custom events. */
export enum Events {
  /** Currently selected graph node has changed. */
  GRAPH_ELEMENT_CHANGE = 'graph_element_change',
}

/** Notify that side panel active element has changed. */
export interface ElementChangeEvent {
  element: (cytoscape.NodeSingular & cytoscape.EdgeSingular) | null;
}

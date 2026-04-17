/**
 * EdgeLines — unified edge style definitions (synced with Admin)
 *
 * Sequence: neon cyan bezier, solid, 2px
 * Reference: dark gray bezier, dashed, 1.5px
 */

/** Sequence edge style — neon cyan bezier */
export const SEQUENCE_EDGE = {
  stroke: "#00e5ff",
  strokeWidth: 2,
  type: "default" as const,
  animated: false,
  style: {
    stroke: "#00e5ff",
    strokeWidth: 2,
  },
};

/** Reference edge style — dark gray dashed bezier */
export const REFERENCE_EDGE = {
  stroke: "#555555",
  strokeWidth: 1.5,
  type: "default" as const,
  animated: false,
  style: {
    stroke: "#555555",
    strokeWidth: 1.5,
    strokeDasharray: "5 5",
  },
};

/** Get edge style by relation type */
export function getEdgeStyle(relation: string) {
  return relation === "sequence" ? SEQUENCE_EDGE : REFERENCE_EDGE;
}

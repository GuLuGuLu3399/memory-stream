/**
 * 🌟 EdgeLines — 统一的连线样式定义
 *
 * 主干 Sequence：深蓝粗实线 + 流动动画
 * 旁支 Reference：灰色细虚线 + 静止
 */

/** Sequence 主干线样式 — 霓虹青 */
export const SEQUENCE_EDGE = {
  stroke: "#00e5ff", // neon
  strokeWidth: 3,
  type: "smoothstep" as const,
  animated: true,
  style: {
    stroke: "#00e5ff",
    strokeWidth: 3,
  },
};

/** Reference 旁支线样式 — zinc 灰 */
export const REFERENCE_EDGE = {
  stroke: "#71717a", // zinc-500
  strokeWidth: 1.5,
  type: "straight" as const,
  animated: false,
  style: {
    stroke: "#71717a",
    strokeWidth: 1.5,
    strokeDasharray: "6 4",
  },
};

/** 根据关系类型返回边样式 */
export function getEdgeStyle(relation: string) {
  return relation === "sequence" ? SEQUENCE_EDGE : REFERENCE_EDGE;
}

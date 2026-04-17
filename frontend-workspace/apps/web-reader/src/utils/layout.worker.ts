/**
 * layout.worker — Web Worker 线程：图谱布局离屏计算
 *
 * 将 dagre + graphology + potpack 计算移出主线程，
 * 避免大图谱（500+ 节点）布局时 UI 冻结。
 *
 * 零 Vue Flow 依赖 — 只导入 computePositions 纯函数。
 */

import { computePositions } from "./graphLayout";

self.onmessage = (
  e: MessageEvent<{
    nodes: Array<{ id: string; width: number; height: number }>;
    edges: Array<{ source: string; target: string; type?: string }>;
  }>,
) => {
  const { nodes, edges } = e.data;
  const positions = computePositions(nodes, edges);
  self.postMessage(positions);
};

/**
 * seqTraversal — 拓扑潜流：SEQ 链双向遍历
 *
 * 从任意节点出发，沿着 sequence 类型的边双向遍历，
 * 收集完整的 SEQ 链条（前驱 + 当前 + 后继）。
 *
 * 返回有序数组，保持链的阅读顺序。
 */

import type { Edge } from "@vue-flow/core";

interface AdjacencyMap {
    forward: Map<string, string[]>;  // node → successors along SEQ
    backward: Map<string, string[]>; // node → predecessors along SEQ
}

/**
 * 从边列表构建 SEQ 邻接表
 */
function buildSeqAdjacency(edges: Edge[]): AdjacencyMap {
    const forward = new Map<string, string[]>();
    const backward = new Map<string, string[]>();

    for (const edge of edges) {
        const relation = (edge.data as { type?: string })?.type;
        if (relation !== "sequence") continue;

        const src = edge.source;
        const tgt = edge.target;

        if (!forward.has(src)) forward.set(src, []);
        forward.get(src)!.push(tgt);

        if (!backward.has(tgt)) backward.set(tgt, []);
        backward.get(tgt)!.push(src);
    }

    return { forward, backward };
}

/**
 * 沿某个方向做 BFS 遍历，返回遇到的所有节点（有序）
 */
function bfs(
    startId: string,
    adj: Map<string, string[]>,
): string[] {
    const visited = new Set<string>();
    const queue = [startId];
    const ordered: string[] = [];
    let head = 0;

    while (head < queue.length) {
        const id = queue[head++];
        if (visited.has(id)) continue;
        visited.add(id);
        ordered.push(id);

        const neighbors = adj.get(id);
        if (neighbors) {
            for (const n of neighbors) {
                if (!visited.has(n)) queue.push(n);
            }
        }
    }

    return ordered;
}

/**
 * 提取从 startId 出发的完整 SEQ 链
 *
 * 遍历策略：
 * 1. 从 startId 向后找所有前驱节点（reverse BFS）
 * 2. 从 startId 向前找所有后继节点（forward BFS）
 * 3. 拼接：前驱（逆序）+ [startId] + 后继
 *
 * @returns 有序节点 ID 数组，从链头到链尾
 */
export function extractSeqChain(
    startId: string,
    edges: Edge[],
): string[] {
    const { forward, backward } = buildSeqAdjacency(edges);

    // 向后找前驱 → 得到从 startId 往回的 BFS 序列
    const predecessorsReversed = bfs(startId, backward);
    // 向前找后继 → 得到从 startId 往前的 BFS 序列
    const successors = bfs(startId, forward);

    // 前驱序列去掉 startId（它同时出现在两个 BFS 中）并反转
    const predecessors = predecessorsReversed
        .filter((id) => id !== startId)
        .reverse();

    return [...predecessors, ...successors];
}

/**
 * 检测 startId 是否处于任何 SEQ 链中
 */
export function isInSeqChain(
    startId: string,
    edges: Edge[],
): boolean {
    const { forward, backward } = buildSeqAdjacency(edges);
    return (forward.has(startId) || backward.has(startId));
}

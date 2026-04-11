/**
 * useOblivionHeatmap — 遗忘热力学
 *
 * 基于 localStorage 的节点浏览时间追踪，
 * 计算每个节点的记忆衰退系数（opacity）。
 *
 * 衰减曲线：
 *   0-1 天  → 1.0（刚看过，记忆鲜活）
 *   1-3 天  → 0.85
 *   3-7 天  → 0.65
 *   7-14 天 → 0.45
 *   14+ 天  → 0.25（濒临遗忘）
 *
 * 用法：
 *   const { recordView, getDecayMap } = useOblivionHeatmap()
 *   recordView(nodeId)                    // 点击节点时调用
 *   const decayMap = getDecayMap(nodes)   // 返回 { id: opacity } 映射
 */

const STORAGE_KEY = "ms-oblivion-heatmap";

interface ViewTimestamps {
    [nodeId: string]: number; // Unix timestamp (ms)
}

function loadTimestamps(): ViewTimestamps {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        return raw ? JSON.parse(raw) : {};
    } catch {
        return {};
    }
}

function saveTimestamps(data: ViewTimestamps): void {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
    } catch {
        // localStorage full or unavailable — silently ignore
    }
}

/**
 * 根据距上次浏览的天数计算衰减 opacity
 */
function computeOpacity(daysSince: number): number {
    if (daysSince <= 1) return 1.0;
    if (daysSince <= 3) return 0.85;
    if (daysSince <= 7) return 0.65;
    if (daysSince <= 14) return 0.45;
    return 0.25;
}

export function useOblivionHeatmap() {
    /**
     * 记录节点被浏览（点击/查看）的时间戳
     */
    function recordView(nodeId: string): void {
        const timestamps = loadTimestamps();
        timestamps[nodeId] = Date.now();
        saveTimestamps(timestamps);
    }

    /**
     * 获取单个节点的衰减 opacity
     * @returns 0.25 ~ 1.0，从未浏览过的节点返回 0.25
     */
    function getDecay(nodeId: string): number {
        const timestamps = loadTimestamps();
        const lastViewed = timestamps[nodeId];
        if (!lastViewed) return 0.25; // 从未浏览 — 最暗
        const daysSince = (Date.now() - lastViewed) / (1000 * 60 * 60 * 24);
        return computeOpacity(daysSince);
    }

    /**
     * 批量获取衰减映射
     * @param nodeIds - 节点 ID 列表
     * @returns Map<nodeId, opacity>
     */
    function getDecayMap(nodeIds: string[]): Map<string, number> {
        const timestamps = loadTimestamps();
        const now = Date.now();
        const map = new Map<string, number>();

        for (const id of nodeIds) {
            const lastViewed = timestamps[id];
            if (!lastViewed) {
                map.set(id, 0.25);
            } else {
                const daysSince = (now - lastViewed) / (1000 * 60 * 60 * 24);
                map.set(id, computeOpacity(daysSince));
            }
        }

        return map;
    }

    return { recordView, getDecay, getDecayMap };
}

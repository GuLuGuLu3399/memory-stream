/**
 * 🌊 useGraphSync — WebSocket 实时增量同步
 *
 * 连接 Go 后端 WS，监听 CARD_CREATED / CARD_UPDATED / CARD_DELETED 事件。
 * 收到事件后直接操作 useGraph 中的 nodes/edges，实现图谱实时更新。
 *
 * 认证流程：
 * 1. 连接 WS（不带 token）
 * 2. 立即发送 AUTH 消息（携带 localStorage 中的 JWT）
 * 3. 等待 AUTH_OK 确认
 * 4. 超时 5s 断连重试
 *
 * 泛型设计：
 * - N extends MinimalNode — 只约束 id 字段，兼容 VueFlow Node 等复杂类型
 * - E extends MinimalEdge — 只约束 id/source/target，避免类型实例化过深
 * - 内部通过辅助函数安全访问动态属性
 */

import { ref, type Ref } from "vue";
import { api, getAuthToken } from "../api";

// ── 最小约束接口（泛型边界，避免导入 VueFlow 复杂泛型） ──

/** 节点最小约束：必须有 id — 导出供调用方做类型缩窄 */
export interface MinimalNode {
  id: string;
}

/** 边最小约束：必须有 id / source / target — 导出供调用方做类型缩窄 */
export interface MinimalEdge {
  id: string;
  source: string;
  target: string;
}

// ── 安全属性访问辅助函数 ──

function getNodeData(node: MinimalNode): Record<string, unknown> {
  const record = node as unknown as Record<string, unknown>;
  return (record["data"] as Record<string, unknown> | undefined) ?? {};
}

function setNodeField<T extends MinimalNode>(
  node: T,
  field: string,
  value: unknown,
): T {
  return { ...node, [field]: value };
}

function setNodeData<T extends MinimalNode>(
  node: T,
  patch: Record<string, unknown>,
): T {
  const existing = getNodeData(node);
  return setNodeField(node, "data", { ...existing, ...patch });
}

// ── WS 事件类型 ──

/** WS 事件载荷 — 卡片变更 */
interface CardEventPayload {
  card_id: string;
  title?: string;
  excerpt?: string;
  category_id?: string;
}

/** WS 事件载荷 — 边变更 */
interface EdgeEventPayload {
  source_id: string;
  target_id: string;
  relation_type?: string;
}

/** WS 事件结构 */
interface WSEvent {
  event: string;
  payload: CardEventPayload | EdgeEventPayload | unknown;
}

/** WS Action 结构（发送） */
interface WSAction {
  action: string;
  payload: Record<string, string>;
}

const WS_BASE_URL =
  import.meta.env.VITE_WS_URL || "ws://localhost:8080/api/v1/ws";

// ── 全局连接状态（模块级单例，供 LeftDock 等组件读取） ──
export const wsConnected = ref(false);
export const wsAuthenticated = ref(false);
export const wsLatency = ref(0);

const RECONNECT_BASE_DELAY = 3000;
const RECONNECT_MAX_DELAY = 30000;
const AUTH_TIMEOUT = 3000;

// 标志：是否为首次连接（用于区分首次加载与重连补偿）
let wasFirstConnect = true;

/**
 * 创建 WS 实时同步实例
 *
 * @param nodes - VueFlow 节点 Ref（只约束 id 字段）
 * @param edges - VueFlow 边 Ref（只约束 id/source/target）
 */
export function useGraphSync<N extends MinimalNode, E extends MinimalEdge>(
  nodes: Ref<N[]>,
  edges: Ref<E[]>,
) {
  const connected = ref(false);
  const authenticated = ref(false);
  const latency = ref(0); // RTT in ms

  let ws: WebSocket | null = null;
  let reconnectDelay = RECONNECT_BASE_DELAY;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let authTimer: ReturnType<typeof setTimeout> | null = null;
  let pingTimer: ReturnType<typeof setInterval> | null = null;
  let lastPingTime = 0;

  // ── 连接管理 ──

  function connect() {
    // 首次连接时重置标志
    wasFirstConnect = true;

    if (
      ws?.readyState === WebSocket.OPEN ||
      ws?.readyState === WebSocket.CONNECTING
    ) {
      return;
    }

    try {
      ws = new WebSocket(WS_BASE_URL);
    } catch (e) {
      console.error("[WS] create failed:", e);
      scheduleReconnect();
      return;
    }

    ws.onopen = () => {
      console.log("[WS] connected");
      connected.value = true;
      wsConnected.value = true;
      reconnectDelay = RECONNECT_BASE_DELAY;

      // 立即发送 AUTH 消息
      const token = getAuthToken();
      if (token) {
        sendAuth(token);
        authTimer = setTimeout(() => {
          if (!authenticated.value) {
            console.warn("[WS] auth timeout — closing");
            ws?.close();
          }
        }, AUTH_TIMEOUT);
      }
    };

    ws.onmessage = (event) => {
      try {
        const msg: WSEvent = JSON.parse(event.data);
        handleMessage(msg);
      } catch (e) {
        console.warn("[WS] parse error:", e);
      }
    };

    ws.onclose = () => {
      console.log("[WS] disconnected");
      connected.value = false;
      wsConnected.value = false;
      authenticated.value = false;
      wsAuthenticated.value = false;
      wsLatency.value = 0;
      clearAuthTimer();
      scheduleReconnect();
    };

    ws.onerror = (e) => {
      console.error("[WS] error:", e);
    };
  }

  function sendAuth(token: string) {
    const action: WSAction = {
      action: "AUTH",
      payload: { token },
    };
    ws?.send(JSON.stringify(action));
    console.log("[WS] AUTH sent");
  }

  // ── 消息分发 ──

  function handleMessage(msg: WSEvent) {
    switch (msg.event) {
      case "AUTH_OK":
        authenticated.value = true;
        authenticated.value = true;
        wsAuthenticated.value = true;
        clearAuthTimer();
        startPingPong();
        console.log("[WS] authenticated ✓");
        // 重连补偿：仅在非首次连接时才拉取最新图谱全量数据
        if (!wasFirstConnect) {
          reconcileGraph();
        }
        wasFirstConnect = false;
        break;

      case "PONG":
        if (lastPingTime > 0) {
          latency.value = Date.now() - lastPingTime;
          wsLatency.value = latency.value;
          lastPingTime = 0;
        }
        break;

      case "CARD_CREATED":
        handleCardCreated(msg.payload as CardEventPayload);
        break;

      case "CARD_UPDATED":
        handleCardUpdated(msg.payload as CardEventPayload);
        break;

      case "CARD_DELETED":
        handleCardDeleted(msg.payload as CardEventPayload);
        break;

      case "EDGE_CREATED":
        handleEdgeCreated(msg.payload as EdgeEventPayload);
        break;

      case "EDGE_DELETED":
        handleEdgeDeleted(msg.payload as EdgeEventPayload);
        break;

      case "EDGE_UPDATED":
        handleEdgeUpdated(msg.payload as EdgeEventPayload);
        break;

      case "ERROR": {
        console.warn("[WS] server error:", msg.payload);
        const payload = msg.payload as { message?: string };
        if (
          payload?.message?.includes("token") ||
          payload?.message?.includes("认证")
        ) {
          ws?.close();
        }
        break;
      }

      default:
        break;
    }
  }

  // ── 重连补偿：从 REST API 拉取最新全量图谱 ──

  async function reconcileGraph() {
    try {
      const result = await api.getFullGraph();
      if (!result.nodes || result.nodes.length === 0) return;

      // 构建节点 ID 集合，用于快速查找
      const serverNodeIds = new Set(result.nodes.map((n) => n.id));

      // 同步节点：添加缺失的，移除已删除的
      const currentNodeIds = new Set(nodes.value.map((n) => n.id));
      for (const node of result.nodes) {
        if (!currentNodeIds.has(node.id)) {
          const newNode = {
            id: node.id,
            type: "card",
            position: { x: Math.random() * 400, y: Math.random() * 400 },
            data: {
              title: node.title,
              date: "",
              isOrphan: true,
            },
          } as unknown as N;
          nodes.value = [...nodes.value, newNode];
        }
      }
      // 移除服务端已不存在的节点
      const beforeCount = nodes.value.length;
      nodes.value = nodes.value.filter((n) => serverNodeIds.has(n.id));
      if (nodes.value.length !== beforeCount) {
        console.log(
          `[WS] reconcile: removed ${beforeCount - nodes.value.length} stale nodes`,
        );
      }

      // 同步边：构建边 ID 集合
      const serverEdgeIds = new Set(
        result.edges.map((e) => `e-${e.source}-${e.target}`),
      );

      // 添加缺失的边
      for (const edge of result.edges) {
        const edgeId = `e-${edge.source}-${edge.target}`;
        if (!edges.value.some((e) => e.id === edgeId)) {
          const relation = edge.relation || "reference";
          const newEdge = {
            id: edgeId,
            source: edge.source,
            target: edge.target,
            animated: relation === "sequence",
            data: { type: relation },
            type: "smoothstep",
            style:
              relation === "sequence"
                ? { stroke: "#00e5ff", strokeWidth: 2 }
                : { stroke: "#71717a", strokeWidth: 1.5 },
          } as unknown as E;
          edges.value = [...edges.value, newEdge];
        }
      }
      // 移除已不存在的边
      const beforeEdgeCount = edges.value.length;
      edges.value = edges.value.filter((e) => serverEdgeIds.has(e.id));
      if (edges.value.length !== beforeEdgeCount) {
        console.log(
          `[WS] reconcile: removed ${beforeEdgeCount - edges.value.length} stale edges`,
        );
      }

      console.log(
        `[WS] reconcile done: ${nodes.value.length} nodes, ${edges.value.length} edges`,
      );
    } catch (err) {
      console.warn("[WS] reconcile failed:", err);
    }
  }

  // ── 增量更新 handlers ──

  function handleCardCreated(payload: CardEventPayload) {
    if (!payload.card_id) return;
    if (nodes.value.some((n) => n.id === payload.card_id)) return;

    // 构造最小新节点（兼容 VueFlow Node 结构）
    const newNode = {
      id: payload.card_id,
      type: "card",
      position: { x: Math.random() * 400, y: Math.random() * 400 },
      data: {
        title: payload.title || "New Card",
        date: "",
        isOrphan: true,
      },
    } as unknown as N;

    nodes.value = [...nodes.value, newNode];
    console.log(`[WS] +node: ${payload.card_id}`);
  }

  function handleCardUpdated(payload: CardEventPayload) {
    if (!payload.card_id) return;
    const idx = nodes.value.findIndex((n) => n.id === payload.card_id);
    if (idx < 0) return;

    const updated = [...nodes.value];
    const patch: Record<string, unknown> = {};
    if (payload.title) patch.title = payload.title;
    updated[idx] = setNodeData(updated[idx], patch);
    nodes.value = updated;
    console.log(`[WS] ~node: ${payload.card_id}`);
  }

  function handleCardDeleted(payload: CardEventPayload) {
    if (!payload.card_id) return;
    const cardId = payload.card_id;

    nodes.value = nodes.value.filter((n) => n.id !== cardId);
    edges.value = edges.value.filter(
      (e) => e.source !== cardId && e.target !== cardId,
    );
    console.log(`[WS] -node: ${cardId}`);
  }

  function handleEdgeCreated(payload: EdgeEventPayload) {
    if (!payload.source_id || !payload.target_id) return;
    const edgeId = `e-${payload.source_id}-${payload.target_id}`;

    if (edges.value.some((e) => e.id === edgeId)) return;

    const relation = payload.relation_type || "reference";
    const newEdge = {
      id: edgeId,
      source: payload.source_id,
      target: payload.target_id,
      animated: relation === "sequence",
      data: { type: relation },
      type: "smoothstep",
      style:
        relation === "sequence"
          ? { stroke: "#00e5ff", strokeWidth: 2 }
          : { stroke: "#71717a", strokeWidth: 1.5 },
    } as unknown as E;

    edges.value = [...edges.value, newEdge];

    updateOrphanStatus(payload.source_id, false);
    updateOrphanStatus(payload.target_id, false);

    console.log(`[WS] +edge: ${payload.source_id} → ${payload.target_id}`);
  }

  function handleEdgeDeleted(payload: EdgeEventPayload) {
    if (!payload.source_id || !payload.target_id) return;
    const edgeId = `e-${payload.source_id}-${payload.target_id}`;

    edges.value = edges.value.filter((e) => e.id !== edgeId);

    checkAndUpdateOrphan(payload.source_id);
    checkAndUpdateOrphan(payload.target_id);

    console.log(`[WS] -edge: ${payload.source_id} → ${payload.target_id}`);
  }

  function handleEdgeUpdated(payload: EdgeEventPayload) {
    if (!payload.source_id || !payload.target_id) return;
    const edgeId = `e-${payload.source_id}-${payload.target_id}`;
    const idx = edges.value.findIndex((e) => e.id === edgeId);
    if (idx < 0) return;

    const relation = payload.relation_type || "reference";
    const updated = [...edges.value];
    updated[idx] = {
      ...updated[idx],
      animated: relation === "sequence",
      data: { type: relation },
      style:
        relation === "sequence"
          ? { stroke: "#00e5ff", strokeWidth: 2 }
          : { stroke: "#71717a", strokeWidth: 1.5 },
    } as unknown as E;
    edges.value = updated;
    console.log(
      `[WS] ~edge: ${payload.source_id} → ${payload.target_id} (${relation})`,
    );
  }

  // ── 孤岛状态管理 ──

  function updateOrphanStatus(nodeId: string, isOrphan: boolean) {
    const idx = nodes.value.findIndex((n) => n.id === nodeId);
    if (idx < 0) return;
    const updated = [...nodes.value];
    updated[idx] = setNodeData(updated[idx], { isOrphan });
    nodes.value = updated;
  }

  function checkAndUpdateOrphan(nodeId: string) {
    const hasEdge = edges.value.some(
      (e) => e.source === nodeId || e.target === nodeId,
    );
    updateOrphanStatus(nodeId, !hasEdge);
  }

  // ── Ping/Pong RTT ──

  function startPingPong() {
    stopPingPong();
    pingTimer = setInterval(() => {
      if (ws?.readyState === WebSocket.OPEN) {
        lastPingTime = Date.now();
        ws.send(JSON.stringify({ action: "PING", payload: {} }));
      }
    }, 15_000);
  }

  function stopPingPong() {
    if (pingTimer) {
      clearInterval(pingTimer);
      pingTimer = null;
    }
    lastPingTime = 0;
  }

  // ── 重连与清理 ──

  function scheduleReconnect() {
    if (reconnectTimer) return;
    console.log(`[WS] reconnect in ${reconnectDelay / 1000}s...`);
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null;
      connect();
      reconnectDelay = Math.min(reconnectDelay * 2, RECONNECT_MAX_DELAY);
    }, reconnectDelay);
  }

  function clearAuthTimer() {
    if (authTimer) {
      clearTimeout(authTimer);
      authTimer = null;
    }
  }

  function disconnect() {
    stopPingPong();
    clearAuthTimer();
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    if (ws) {
      ws.onclose = null;
      ws.close();
      ws = null;
    }
    connected.value = false;
    wsConnected.value = false;
    authenticated.value = false;
    wsAuthenticated.value = false;
    latency.value = 0;
    wsLatency.value = 0;
  }

  return {
    connected,
    authenticated,
    latency,
    connect,
    disconnect,
  };
}

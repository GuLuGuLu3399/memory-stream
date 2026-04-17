/**
 * WebSocket Pinia Store — 全局状态管理与连接生命周期
 *
 * 职责：
 * - 管理WebSocket连接生命周期
 * - 提供全局事件订阅接口
 * - 缓存来自服务器的实时消息状态
 * - 与认证状态联动（Token更新时重连）
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import * as wsComposable from "../composables/useWebSocket";

export interface CardEvent {
  type: "CARD_CREATED" | "CARD_DELETED" | "CARD_UPDATED";
  card_id: string;
  data?: any;
}

export interface EdgeEvent {
  type: "EDGE_CREATED" | "EDGE_DELETED";
  source_id: string;
  target_id: string;
}

export const useWebSocketStore = defineStore("websocket", () => {
  // ═══════════════════════════════════════════════════════════════════════════
  // 状态
  // ═══════════════════════════════════════════════════════════════════════════

  const isConnected = ref(false);
  const isConnecting = ref(false);
  const lastError = ref<string | null>(null);
  const retryCount = ref(0);

  // 缓存最近的实时事件
  const lastCardEvent = ref<CardEvent | null>(null);
  const lastEdgeEvent = ref<EdgeEvent | null>(null);
  const pendingMessages = ref<any[]>([]);

  // ═══════════════════════════════════════════════════════════════════════════
  // 暴露的方法
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * 使用Token连接到WebSocket服务器
   * 【三次握手】在这一刻开始
   */
  async function connectWithToken(token: string): Promise<void> {
    isConnecting.value = true;
    lastError.value = null;

    try {
      await wsComposable.connect({ token, maxRetries: 5, retryDelayMs: 3000 });

      isConnected.value = true;
      isConnecting.value = false;

      // 注册所有事件监听器
      setupEventListeners();

      console.log("✅ [WebSocket] 已连接并准备就绪");
    } catch (error) {
      isConnected.value = false;
      isConnecting.value = false;
      lastError.value = error instanceof Error ? error.message : String(error);

      console.error("❌ [WebSocket] 连接失败:", lastError.value);
      throw error;
    }
  }

  /**
   * 断开连接
   */
  function disconnect(): void {
    wsComposable.disconnect();
    isConnected.value = false;
    isConnecting.value = false;
    pendingMessages.value = [];
  }

  /**
   * 发送消息到服务器
   */
  function sendMessage(message: { type: string; payload?: any }): void {
    if (isConnected.value) {
      wsComposable.send(message);
    } else {
      // 如果未连接，将消息缓存到待发送队列
      pendingMessages.value.push(message);
      console.warn("[WebSocket] 未连接，消息已缓存到待发送队列");
    }
  }

  /**
   * 订阅事件（返回取消订阅函数）
   */
  function on(eventType: string, handler: (data: any) => void): () => void {
    return wsComposable.on(eventType, handler);
  }

  /**
   * 设置重连配置
   */
  function setRetryConfig(maxRetries: number, retryDelayMs: number): void {
    wsComposable.setRetryConfig(maxRetries, retryDelayMs);
  }

  /**
   * 获取连接状态快照
   */
  function getConnectionState() {
    return wsComposable.connectionState.value;
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // 事件监听器设置
  // ═══════════════════════════════════════════════════════════════════════════

  function setupEventListeners(): void {
    // 卡片事件
    wsComposable.on("CARD_CREATED", (data) => {
      lastCardEvent.value = {
        type: "CARD_CREATED",
        card_id: data.card_id,
        data,
      };
      console.log("📝 [实时] 卡片已创建:", data.card_id);
    });

    wsComposable.on("CARD_DELETED", (data) => {
      lastCardEvent.value = { type: "CARD_DELETED", card_id: data.card_id };
      console.log("🗑️ [实时] 卡片已删除:", data.card_id);
    });

    wsComposable.on("CARD_UPDATED", (data) => {
      lastCardEvent.value = {
        type: "CARD_UPDATED",
        card_id: data.card_id,
        data,
      };
      console.log("✏️ [实时] 卡片已更新:", data.card_id);
    });

    // 边事件
    wsComposable.on("EDGE_CREATED", (data) => {
      lastEdgeEvent.value = {
        type: "EDGE_CREATED",
        source_id: data.source_id,
        target_id: data.target_id,
      };
      console.log(
        "🔗 [实时] 关系已创建:",
        data.source_id,
        "=>",
        data.target_id,
      );
    });

    wsComposable.on("EDGE_DELETED", (data) => {
      lastEdgeEvent.value = {
        type: "EDGE_DELETED",
        source_id: data.source_id,
        target_id: data.target_id,
      };
      console.log(
        "❌ [实时] 关系已删除:",
        data.source_id,
        "=>",
        data.target_id,
      );
    });

    // 错误事件
    wsComposable.on("ERROR", (data) => {
      console.error("⚠️ [WebSocket Error]:", data.message);
      lastError.value = data.message;
    });
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // 计算属性
  // ═══════════════════════════════════════════════════════════════════════════

  const connectionStatus = computed(() => ({
    isConnected: isConnected.value,
    isConnecting: isConnecting.value,
    lastError: lastError.value,
    retryCount: retryCount.value,
    hasPendingMessages: pendingMessages.value.length > 0,
  }));

  const realTimeEvents = computed(() => ({
    lastCardEvent: lastCardEvent.value,
    lastEdgeEvent: lastEdgeEvent.value,
  }));

  // ═══════════════════════════════════════════════════════════════════════════
  // 生命周期清理
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * 销毁Store时的清理
   */
  function cleanup(): void {
    wsComposable.cleanup();
    isConnected.value = false;
    pendingMessages.value = [];
  }

  return {
    // 状态
    isConnected,
    isConnecting,
    lastError,
    retryCount,
    lastCardEvent,
    lastEdgeEvent,
    pendingMessages,

    // 方法
    connectWithToken,
    disconnect,
    sendMessage,
    on,
    setRetryConfig,
    getConnectionState,
    cleanup,

    // 计算属性
    connectionStatus,
    realTimeEvents,
  };
});

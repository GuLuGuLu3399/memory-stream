/**
 * 🌀 useWebSocket — Token鉴权后的极致WebSocket管道
 *
 * 三次握手与心跳契约：
 * 1. 握手验证：Token在HTTP升级握手阶段验证（Go防线）
 * 2. 连接建立：前端携带Token建立WebSocket连接
 * 3. 心跳防线：Ping/Pong保活机制缺一不可
 *
 * 设计约束：
 * - 单例模式：全应用共享一个WebSocket连接
 * - 自动重连：异常断开时自动重连（指数退避策略）
 * - 事件驱动：基于TypeScript类型系统的事件分发
 * - 资源清理：确保没有泄漏的定时器与监听器
 */

import { ref, computed } from 'vue';

// ═══════════════════════════════════════════════════════════════════════════
// 类型定义
// ═══════════════════════════════════════════════════════════════════════════

export interface WSMessage {
  type: string;
  payload?: any;
}

export type WSEventHandler = (data: any) => void;

interface ConnectionConfig {
  token: string;
  maxRetries?: number;
  retryDelayMs?: number;
}

// ═══════════════════════════════════════════════════════════════════════════
// 响应式状态管理
// ═══════════════════════════════════════════════════════════════════════════

const ws = ref<WebSocket | null>(null);
const isConnected = ref(false);
const isConnecting = ref(false);
const lastError = ref<string | null>(null);
const retryCount = ref(0);
const maxRetries = ref(5);
const retryDelayMs = ref(3000);

// 事件监听器映射（订阅发布）
const eventHandlers = ref<Map<string, Set<WSEventHandler>>>(new Map());

// 心跳相关
let pingInterval: ReturnType<typeof setInterval> | null = null;
let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
let pongTimeout: ReturnType<typeof setTimeout> | null = null;

// ═══════════════════════════════════════════════════════════════════════════
// 核心连接管理
// ═══════════════════════════════════════════════════════════════════════════

/**
 * 建立WebSocket连接（携带Token进行握手验证）
 *
 * 【第二纪元】：前端注入Token，Go端在握手阶段验证
 * 如果Token无效，Go会直接返回401，连接失败
 */
export function connect(config: ConnectionConfig): Promise<void> {
  return new Promise((resolve, reject) => {
    if (isConnecting.value) {
      reject(new Error('连接正在进行中，请勿重复调用'));
      return;
    }

    isConnecting.value = true;
    lastError.value = null;

    try {
      // 动态判断协议：https -> wss, http -> ws
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const baseUrl = `${protocol}//${window.location.host}`;
      
      // Token通过URL Query参数携带（浏览器原生WebSocket API的最佳实践）
      const wsUrl = `${baseUrl}/api/v1/ws?token=${encodeURIComponent(config.token)}`;

      ws.value = new WebSocket(wsUrl);

      // 本质上是upgrade request被Go服务器验证的阶段
      ws.value.onopen = () => {
        console.log('🔗 [WS] 跃迁通道已建立 (握手验证通过)');
        isConnected.value = true;
        isConnecting.value = false;
        retryCount.value = 0; // 重置重试计数
        
        // 启动心跳防线
        startHeartbeat();
        
        resolve();
      };

      ws.value.onmessage = (event) => {
        const message: WSMessage = JSON.parse(event.data);
        
        // 发派事件给所有监听者
        dispatchEvent(message.type, message.payload);
      };

      ws.value.onclose = (event) => {
        console.warn(`❌ [WS] 连接断开 (code=${event.code}, reason=${event.reason || '无'})`);
        isConnected.value = false;
        isConnecting.value = false;
        stopHeartbeat();

        // 1000 = 正常关闭 (由用户主动调用disconnect)
        // 其他code = 异常段开，需要重连
        if (event.code !== 1000) {
          attemptReconnect(config);
        }
      };

      ws.value.onerror = (error) => {
        const errorMsg = '【WebSocket连接失败】握手验证可能被拒绝（Token无效或Origin被禁）';
        console.error(errorMsg, error);
        lastError.value = errorMsg;
        isConnecting.value = false;
        reject(new Error(errorMsg));
      };

    } catch (error) {
      isConnecting.value = false;
      reject(error);
    }
  });
}

/**
 * 断开WebSocket连接（主动退出，不触发重连）
 */
export function disconnect(): void {
  stopHeartbeat();
  if (reconnectTimeout) {
    clearTimeout(reconnectTimeout);
    reconnectTimeout = null;
  }
  
  if (ws.value && ws.value.readyState === WebSocket.OPEN) {
    ws.value.close(1000, '用户主动退出');
  }
  
  ws.value = null;
  isConnected.value = false;
  retryCount.value = 0;
}

/**
 * 自动重连机制（指数退避策略）
 * 重连延迟：3s -> 6s -> 12s -> 24s -> 48s (上限)
 */
function attemptReconnect(config: ConnectionConfig): void {
  if (retryCount.value >= maxRetries.value) {
    lastError.value = `连接失败，已重试${maxRetries.value}次，请检查网络`;
    console.error('[WS] 达到최대重试次数，放弃重连');
    return;
  }

  retryCount.value++;
  const delayMs = Math.min(retryDelayMs.value * Math.pow(2, retryCount.value - 1), 60000);
  
  console.log(`[WS] ${delayMs}ms后进行第${retryCount.value}/${maxRetries.value}次重连...`);
  
  reconnectTimeout = setTimeout(() => {
    connect(config).catch((error) => {
      console.error('[WS] 重连失败:', error.message);
    });
  }, delayMs);
}

// ═══════════════════════════════════════════════════════════════════════════
// 【第三纪元】心跳防线：Ping/Pong保活
// ═══════════════════════════════════════════════════════════════════════════

/**
 * 启动心跳防线：每30秒发送一次Ping
 * Go端收到Ping后会重置ReadDeadline，实现活性检测
 */
function startHeartbeat(): void {
  stopHeartbeat(); // 确保没有旧的心跳定时器
  
  pingInterval = setInterval(() => {
    if (ws.value && isConnected.value) {
      try {
        ws.value.send(JSON.stringify({ type: 'ping' }));
        console.log('💓 [WS] 心跳pulse已发送');

        // 设置Pong超时：如果5秒内未收到Pong，认为连接死亡
        setPongTimeout();
      } catch (error) {
        console.error('[WS] 心跳发送失败:', error);
        stopHeartbeat();
      }
    }
  }, 30000); // 每30秒一次
}

/**
 * 停止心跳防线（连接关闭或重新启动时调用）
 */
function stopHeartbeat(): void {
  if (pingInterval) {
    clearInterval(pingInterval);
    pingInterval = null;
  }
  
  if (pongTimeout) {
    clearTimeout(pongTimeout);
    pongTimeout = null;
  }
}

/**
 * 设置Pong超时检测：5秒内必须收到Pong响应
 * 如果超时，认为连接已死亡，主动断开
 */
function setPongTimeout(): void {
  if (pongTimeout) {
    clearTimeout(pongTimeout);
  }
  
  pongTimeout = setTimeout(() => {
    console.warn('[WS] ⏰ Pong超时，连接可能已死亡，主动断开');
    if (ws.value && isConnected.value) {
      ws.value.close(4000, 'Pong timeout');
    }
  }, 5000);
}

/**
 * 作为心跳Pong响应的处理
 * 当收到Go端的PONG消息时，取消Pong超时检测
 */
function handlePongResponse(): void {
  if (pongTimeout) {
    clearTimeout(pongTimeout);
    pongTimeout = null;
  }
  console.log('💚 [WS] 收到心跳回应 (Pong)');
}

// ═══════════════════════════════════════════════════════════════════════════
// 消息发送与事件分派
// ═══════════════════════════════════════════════════════════════════════════

/**
 * 发送消息到服务器
 */
export function send(message: WSMessage): void {
  if (!ws.value || !isConnected.value) {
    console.warn('[WS] WebSocket未连接，无法发送消息:', message);
    return;
  }

  try {
    ws.value.send(JSON.stringify(message));
  } catch (error) {
    console.error('[WS] 消息发送失败:', error);
  }
}

/**
 * 订阅特定事件
 */
export function on(eventType: string, handler: WSEventHandler): () => void {
  if (!eventHandlers.value.has(eventType)) {
    eventHandlers.value.set(eventType, new Set());
  }
  
  const handlers = eventHandlers.value.get(eventType)!;
  handlers.add(handler);

  // 返回取消订阅函数
  return () => {
    handlers.delete(handler);
  };
}

/**
 * 取消订阅所有该事件的监听器
 */
export function off(eventType: string): void {
  eventHandlers.value.delete(eventType);
}

/**
 * 分派事件给所有监听者
 */
function dispatchEvent(eventType: string, payload: any): void {
  // 特殊处理心跳Pong
  if (eventType === 'PONG') {
    handlePongResponse();
  }

  const handlers = eventHandlers.value.get(eventType);
  if (handlers && handlers.size > 0) {
    handlers.forEach((handler) => {
      try {
        handler(payload);
      } catch (error) {
        console.error(`[WS] 事件处理器执行失败 (${eventType}):`, error);
      }
    });
  } else {
    console.debug(`[WS] 未知事件或无监听器: ${eventType}`);
  }
}

// ═══════════════════════════════════════════════════════════════════════════
// 导出计算属性与状态
// ═══════════════════════════════════════════════════════════════════════════

export const connectionState = computed(() => ({
  isConnected: isConnected.value,
  isConnecting: isConnecting.value,
  lastError: lastError.value,
  retryCount: retryCount.value,
  readyState: ws.value?.readyState,
}));

// 设置最大重试次数与延迟参数
export function setRetryConfig(max: number, delayMs: number): void {
  maxRetries.value = max;
  retryDelayMs.value = delayMs;
}

/**
 * 清理所有资源（应用卸载时调用）
 */
export function cleanup(): void {
  disconnect();
  eventHandlers.value.clear();
}

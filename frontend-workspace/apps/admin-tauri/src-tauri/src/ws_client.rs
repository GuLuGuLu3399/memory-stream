use crate::auth::AuthState;
use crate::cache::CacheManager;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const WS_URL: &str = "ws://localhost:8080/api/v1/ws";
const AUTH_TIMEOUT_SECS: u64 = 3;

#[derive(Serialize, Deserialize, Debug)]
struct WSAction {
    action: String,
    payload: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct WSEvent {
    event: String,
    payload: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LayoutItem {
    id: String,
    x: f64,
    y: f64,
}

pub struct WsSender {
    tx: mpsc::Sender<String>,
}

pub fn start_ws_client(
    app_handle: AppHandle,
    cache: Arc<Mutex<Option<CacheManager>>>,
    auth_state: Arc<AuthState>,
    ws_url: String,
) -> WsSender {
    let (tx, mut rx) = mpsc::channel::<String>(256);
    let app_h = app_handle.clone();
    let cache_c = cache.clone();
    let http_client = reqwest::Client::new();
    let url = if ws_url.is_empty() {
        WS_URL.to_string()
    } else {
        ws_url
    };

    tauri::async_runtime::spawn(async move {
        let mut retry_delay = std::time::Duration::from_secs(3);
        let mut retry_count: u32 = 0;
        const MAX_RETRIES: u32 = 30;
        loop {
                eprintln!("[WS] connecting to {}...", url);

                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        eprintln!("[WS] connected! sending AUTH...");
                        retry_delay = std::time::Duration::from_secs(3);
                        retry_count = 0;
                        let (mut write, mut read) = ws_stream.split();
                        let app2 = app_h.clone();
                        let cache2 = cache_c.clone();

                        let auth_sent = send_auth_message(&mut write, &auth_state).await;
                        if !auth_sent {
                            eprintln!("[WS] no token available, waiting for auth...");
                        }

                        let auth_deadline = tokio::time::Instant::now()
                            + std::time::Duration::from_secs(AUTH_TIMEOUT_SECS);
                        let mut auth_ok_received = !auth_sent;
                        let mut auth_deadline_checked = !auth_sent;

                        loop {
                            tokio::select! {
                                msg = read.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(text))) => {
                                            if !auth_ok_received {
                                                if let Ok(evt) = serde_json::from_str::<WSEvent>(&text) {
                                                    if evt.event == "AUTH_OK" {
                                                        auth_ok_received = true;
                                                        auth_deadline_checked = true;
                                                        eprintln!("[WS] authentication confirmed");
                                                    } else if evt.event == "ERROR" {
                                                        eprintln!("[WS] auth rejected, attempting token refresh...");
                                                        drop(write);
                                                        break;
                                                    }
                                                }
                                            }
                                            handle_msg(&text, &app2, &cache2)
                                        }
                                        Some(Ok(Message::Close(_))) => {
                                            eprintln!("[WS] closed by server");
                                            break;
                                        }
                                        Some(Err(e)) => {
                                            eprintln!("[WS] read error: {}", e);
                                            break;
                                        }
                                        None => break,
                                        _ => {}
                                    }
                                }
                                text = rx.recv() => {
                                    match text {
                                        Some(t) => {
                                            if write
                                                .send(Message::Text(t.into()))
                                                .await
                                                .is_err()
                                            {
                                                break;
                                            }
                                        }
                                        None => {
                                            eprintln!("[WS] channel closed");
                                            return;
                                        }
                                    }
                                }
                                _ = tokio::time::sleep_until(auth_deadline), if !auth_deadline_checked => {
                                    if !auth_ok_received {
                                        eprintln!("[WS] auth timeout — no AUTH_OK received within {}s", AUTH_TIMEOUT_SECS);
                                        break;
                                    }
                                    auth_deadline_checked = true;
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("[WS] connect failed: {}", e),
                }

                let should_refresh = auth_state.try_acquire_refresh_lock();
                if should_refresh {
                    let refreshed = crate::auth::do_refresh(&http_client, &auth_state).await;
                    auth_state.release_refresh_lock();
                    if !refreshed {
                        let has_refresh = auth_state.get_refresh_token().is_some();
                        if has_refresh {
                            eprintln!("[WS] token refresh failed — emitting auth_expired");
                            let _ = app_h.emit("auth_expired", ());
                        }
                    }
                }

                retry_count += 1;
                if retry_count >= MAX_RETRIES {
                    eprintln!("[WS] max retries ({}) reached — stopping", MAX_RETRIES);
                    break;
                }
                eprintln!("[WS] retry {}/{} in {}s...", retry_count, MAX_RETRIES, retry_delay.as_secs());
                tokio::time::sleep(retry_delay).await;
                retry_delay =
                    (retry_delay * 2).min(std::time::Duration::from_secs(30));
            }
            });

    WsSender { tx }
}

async fn send_auth_message(
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    auth_state: &Arc<AuthState>,
) -> bool {
    let token = match auth_state.get_access_token() {
        Some(t) => t,
        None => return false,
    };

    let auth_msg = WSAction {
        action: "AUTH".to_string(),
        payload: serde_json::json!({ "token": token }),
    };

    match serde_json::to_string(&auth_msg) {
        Ok(json) => {
            if write.send(Message::Text(json.into())).await.is_err() {
                eprintln!("[WS] failed to send AUTH message");
                return false;
            }
            eprintln!("[WS] AUTH message sent");
            true
        }
        Err(e) => {
            eprintln!("[WS] failed to serialize AUTH: {}", e);
            false
        }
    }
}

fn handle_msg(
    text: &str,
    app: &AppHandle,
    cache: &Arc<Mutex<Option<CacheManager>>>,
) {
    let event: WSEvent = match serde_json::from_str(text) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[WS] JSON error: {}", e);
            return;
        }
    };

    match event.event.as_str() {
        "LAYOUT_UPDATED" => {
            let layouts: Vec<LayoutItem> = match serde_json::from_value(event.payload) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("[WS] parse error: {}", e);
                    return;
                }
            };
            if let Ok(g) = cache.lock() {
                if let Some(ref cm) = *g {
                    let cached: Vec<crate::cache::CachedLayout> = layouts
                        .iter()
                        .map(|l| crate::cache::CachedLayout {
                            card_id: l.id.clone(),
                            x: l.x,
                            y: l.y,
                            title: String::new(),
                            category_id: None,
                            hot_score: 0.0,
                            updated_at: fmt_time(),
                        })
                        .collect();
                    if let Err(e) = cm.upsert_layouts(&cached) {
                        eprintln!("[WS] sqlite error: {}", e);
                    }
                }
            }
            let _ = app.emit("layout_synced", &layouts);
            eprintln!("[WS] synced {} nodes", layouts.len());
        }
        "AUTH_OK" => {
            eprintln!("[WS] authenticated: {:?}", event.payload);
        }
        "ERROR" => {
            eprintln!("[WS] server error: {:?}", event.payload);
        }
        // Forward card events to Vue frontend
        "CARD_CREATED" | "CARD_UPDATED" | "CARD_DELETED" | "CARDS_MERGED" => {
            let event_name = event.event.clone();
            let _ = app.emit(&format!("ws_{}", event_name.to_lowercase()), &event.payload);
            eprintln!("[WS] forwarded {} to frontend", event_name);
        }
        _ => {
            eprintln!("[WS] unknown event: {}", event.event);
        }
    }
}

impl WsSender {
    pub fn send_action(
        &self,
        action: &str,
        payload: serde_json::Value,
    ) -> Result<(), String> {
        let json = serde_json::to_string(&WSAction {
            action: action.to_string(),
            payload,
        })
        .map_err(|e| format!("{}", e))?;
        self.tx.try_send(json).map_err(|e| format!("{}", e))
    }
}

fn fmt_time() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}", d.as_secs(), d.subsec_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_time_format() {
        let time = fmt_time();
        let parts: Vec<&str> = time.split('.').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].parse::<u64>().is_ok());
        assert_eq!(parts[1].len(), 3);
    }

    #[test]
    fn test_ws_sender_send_action() {
        let (tx, mut rx) = mpsc::channel::<String>(256);
        let sender = WsSender { tx };
        sender.send_action("PING", serde_json::json!({})).unwrap();
        let msg = rx.try_recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["action"], "PING");
    }

    #[test]
    fn test_ws_sender_with_payload() {
        let (tx, mut rx) = mpsc::channel::<String>(256);
        let sender = WsSender { tx };
        sender.send_action("CREATE_EDGE", serde_json::json!({"source_id": "s1", "target_id": "t1"})).unwrap();
        let msg = rx.try_recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["action"], "CREATE_EDGE");
        assert_eq!(parsed["payload"]["source_id"], "s1");
    }
}

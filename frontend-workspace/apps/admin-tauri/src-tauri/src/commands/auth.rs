use crate::state::AppState;

const KEYRING_SERVICE: &str = "memory-stream";
const KEYRING_ACCESS_TOKEN: &str = "access_token";

#[derive(serde::Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct LoginData {
    access_token: String,
    #[allow(dead_code)]
    refresh_token: String,
}

#[derive(serde::Deserialize)]
struct LoginResponse {
    data: LoginData,
}

pub fn get_access_token() -> Option<String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCESS_TOKEN)
        .ok()
        .and_then(|e| e.get_password().ok())
}

pub fn clear_access_token() {
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCESS_TOKEN) {
        let _ = entry.delete_password();
    }
}

#[tauri::command]
pub async fn login_to_server(
    state: tauri::State<'_, AppState>,
    username: String,
    password: String,
) -> Result<String, String> {
    let base = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.api_base_url.clone()
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base}/auth/login"))
        .json(&LoginRequest { username, password })
        .send()
        .await
        .map_err(|e| format!("Login request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| format!("Read error body failed: {e}"))?;
        return Err(format!("Login failed: {status} — {body}"));
    }

    let login_resp = resp
        .json::<LoginResponse>()
        .await
        .map_err(|e| format!("Login response parse failed: {e}"))?;

    let token = login_resp.data.access_token;

    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCESS_TOKEN)
        .map_err(|e| format!("Keyring init failed: {e}"))?;
    entry
        .set_password(&token)
        .map_err(|e| format!("Keyring store failed: {e}"))?;

    Ok(token)
}

#[tauri::command]
pub fn logout_from_server() -> Result<(), String> {
    clear_access_token();
    Ok(())
}

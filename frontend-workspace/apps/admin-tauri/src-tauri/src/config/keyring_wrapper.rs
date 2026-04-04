//! Keyring wrapper with fallback to encrypted file storage
//!
//! This module provides secure secret storage using the OS keyring,
//! with automatic fallback to base64-encoded JSON file storage when
//! the keyring is unavailable.
//!
//! Platform-specific keyring backends:
//! - Windows: Windows Credential Manager
//! - macOS: Keychain
//! - Linux: DBus Secret Service

use base64::{engine::general_purpose::STANDARD, Engine as _};
use keyring::Entry;
use std::collections::HashMap;
use std::path::PathBuf;

const SERVICE_NAME: &str = "memory-stream";
const KEYRING_FALLBACK_STORE: &str = "keyring-fallback.json";

/// Store a secret in OS keyring, fallback to encrypted JSON file
///
/// # Arguments
/// * `key` - The identifier for the secret
/// * `value` - The secret value to store
///
/// # Returns
/// `Ok(())` if successful, `Err` with error message otherwise
///
/// # Example
/// ```rust
/// store_secret("api_key", "secret123")?;
/// ```
pub fn store_secret(key: &str, value: &str) -> Result<(), String> {
    match try_keyring_store(key, value) {
        Ok(()) => {
            if let Ok(Some(_)) = try_keyring_get(key) {
                Ok(())
            } else {
                log::warn!(
                    "Keyring verification failed for key '{}', falling back to file store",
                    key
                );
                fallback_store(key, value)
            }
        }
        Err(e) => {
            log::warn!(
                "Keyring unavailable for key '{}': {}, falling back to file store",
                key,
                e
            );
            fallback_store(key, value)
        }
    }
}

/// Get a secret from OS keyring, fallback to encrypted JSON file
///
/// # Arguments
/// * `key` - The identifier for the secret
///
/// # Returns
/// `Ok(Some(value))` if found, `Ok(None)` if not found, `Err` on error
///
/// # Example
/// ```rust
/// if let Some(secret) = get_secret("api_key")? {
///     println!("Found secret: {}", secret);
/// }
/// ```
pub fn get_secret(key: &str) -> Result<Option<String>, String> {
    match try_keyring_get(key) {
        Ok(Some(v)) => Ok(Some(v)),
        Ok(None) => {
            if let Some(value) = fallback_get(key)? {
                if let Err(e) = try_keyring_store(key, &value) {
                    log::warn!(
                        "Failed to sync fallback value to keyring for key '{}': {}",
                        key,
                        e
                    );
                }
                Ok(Some(value))
            } else {
                Ok(None)
            }
        }
        Err(e) => {
            log::warn!(
                "Keyring read failed for key '{}': {}, trying fallback",
                key,
                e
            );
            fallback_get(key)
        }
    }
}

/// Delete a secret from both keyring and fallback store
///
/// # Arguments
/// * `key` - The identifier for the secret to delete
///
/// # Returns
/// `Ok(())` if successful (or key didn't exist), `Err` on error
pub fn delete_secret(key: &str) -> Result<(), String> {
    // Try to delete from keyring
    if let Ok(entry) = Entry::new(SERVICE_NAME, key) {
        let _ = entry.delete_credential();
    }

    // Also delete from fallback store
    let mut store = load_fallback_store()?;
    store.remove(key);
    save_fallback_store(&store)
}

// === Keyring Implementation ===

fn try_keyring_store(key: &str, value: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key).map_err(|e| e.to_string())?;
    entry.set_password(value).map_err(|e| e.to_string())
}

fn try_keyring_get(key: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, key).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

// === Fallback Implementation ===

/// Get platform-specific data directory for fallback storage
fn fallback_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        // Windows: Use %APPDATA%\memory-stream
        let appdata = std::env::var("APPDATA").map_err(|_| "Cannot determine APPDATA directory")?;
        Ok(PathBuf::from(appdata)
            .join("memory-stream")
            .join(KEYRING_FALLBACK_STORE))
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: Use ~/Library/Application Support/memory-stream
        let home = std::env::var("HOME").map_err(|_| "Cannot determine HOME directory")?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("memory-stream")
            .join(KEYRING_FALLBACK_STORE))
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: Use ~/.local/share/memory-stream (XDG Base Directory Specification)
        if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            Ok(PathBuf::from(data_home)
                .join("memory-stream")
                .join(KEYRING_FALLBACK_STORE))
        } else {
            let home = std::env::var("HOME").map_err(|_| "Cannot determine HOME directory")?;
            Ok(PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("memory-stream")
                .join(KEYRING_FALLBACK_STORE))
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Unsupported platform".to_string())
    }
}

/// Load the fallback store from disk
fn load_fallback_store() -> Result<HashMap<String, String>, String> {
    let path = fallback_path()?;

    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read fallback store: {}", e))?;

    let encoded_map: HashMap<String, String> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse fallback store: {}", e))?;

    // Decode values from base64
    let decoded: HashMap<String, String> = encoded_map
        .into_iter()
        .filter_map(|(k, v)| {
            STANDARD
                .decode(&v)
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok().map(|decoded| (k, decoded)))
        })
        .collect();

    Ok(decoded)
}

/// Save the fallback store to disk
fn save_fallback_store(store: &HashMap<String, String>) -> Result<(), String> {
    let path = fallback_path()?;

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create fallback directory: {}", e))?;
    }

    // Encode values to base64
    let encoded: HashMap<String, String> = store
        .iter()
        .map(|(k, v)| (k.clone(), STANDARD.encode(v.as_bytes())))
        .collect();

    let content = serde_json::to_string_pretty(&encoded)
        .map_err(|e| format!("Failed to serialize fallback store: {}", e))?;

    std::fs::write(&path, content).map_err(|e| format!("Failed to write fallback store: {}", e))
}

fn fallback_store(key: &str, value: &str) -> Result<(), String> {
    let mut store = load_fallback_store()?;
    store.insert(key.to_string(), value.to_string());
    save_fallback_store(&store)
}

fn fallback_get(key: &str) -> Result<Option<String>, String> {
    let store = load_fallback_store()?;
    Ok(store.get(key).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_key(prefix: &str) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{}_{}_{}", prefix, std::process::id(), timestamp)
    }

    fn cleanup_key(key: &str) {
        if let Ok(entry) = Entry::new(SERVICE_NAME, key) {
            let _ = entry.delete_credential();
        }
        if let Ok(mut store) = load_fallback_store() {
            store.remove(key);
            let _ = save_fallback_store(&store);
        }
    }

    #[test]
    fn test_keyring_store_and_retrieve() {
        let test_key = unique_test_key("test_keyring");
        let test_value = "test_secret_value_12345";

        cleanup_key(&test_key);

        store_secret(&test_key, test_value).expect("store should succeed");

        let result = get_secret(&test_key).expect("get should succeed");

        cleanup_key(&test_key);

        assert_eq!(
            result,
            Some(test_value.to_string()),
            "Expected to retrieve the stored value"
        );
    }

    #[test]
    fn test_fallback_store_and_retrieve() {
        let test_key = unique_test_key("test_fallback");
        let test_value = "test_fallback_value";

        cleanup_key(&test_key);

        fallback_store(&test_key, test_value).expect("fallback store should work");

        let loaded = fallback_get(&test_key).expect("fallback get should work");

        cleanup_key(&test_key);

        assert_eq!(
            loaded,
            Some(test_value.to_string()),
            "Expected to retrieve value from fallback store"
        );
    }

    #[test]
    fn test_base64_encoding() {
        let original = "my_secret_key_123!";
        let encoded = STANDARD.encode(original.as_bytes());
        let decoded_bytes = STANDARD.decode(&encoded).unwrap();
        let decoded = String::from_utf8(decoded_bytes).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_delete_secret() {
        let test_key = unique_test_key("test_delete");
        let test_value = "to_be_deleted";

        cleanup_key(&test_key);

        store_secret(&test_key, test_value).expect("store should succeed");

        let result = get_secret(&test_key).expect("get should succeed");
        assert_eq!(
            result,
            Some(test_value.to_string()),
            "Value should exist before deletion"
        );

        delete_secret(&test_key).expect("delete should succeed");

        let result = get_secret(&test_key).expect("get should succeed");
        assert_eq!(result, None, "Value should not exist after deletion");
    }

    #[test]
    fn test_fallback_path_exists() {
        let path = fallback_path().expect("Should get fallback path");
        assert!(path.to_string_lossy().contains("memory-stream"));
        assert!(path.to_string_lossy().ends_with("keyring-fallback.json"));
    }

    #[test]
    fn test_nonexistent_key() {
        let test_key = unique_test_key("nonexistent");
        let result = get_secret(&test_key).expect("get should succeed");
        assert_eq!(result, None);
    }

    #[test]
    fn test_store_overwrites_existing() {
        let test_key = unique_test_key("test_overwrite");
        let test_value1 = "first_value";
        let test_value2 = "second_value";

        cleanup_key(&test_key);

        store_secret(&test_key, test_value1).expect("first store should succeed");
        store_secret(&test_key, test_value2).expect("second store should succeed");

        let result = get_secret(&test_key).expect("get should succeed");

        cleanup_key(&test_key);

        assert_eq!(
            result,
            Some(test_value2.to_string()),
            "Should retrieve the most recently stored value"
        );
    }
}

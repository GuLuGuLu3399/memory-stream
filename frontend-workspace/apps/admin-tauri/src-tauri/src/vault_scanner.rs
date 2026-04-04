//! # Vault Scanner Module
//!
//! Configuration validation scanner that checks S3, API, and WebSocket config completeness.
//! Returns a list of config issues with severity levels for the frontend to display.

use serde::Serialize;

// ============================================================================
// Types
// ============================================================================

/// Severity level for configuration issues
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum IssueSeverity {
    /// App cannot function without this config
    Critical,
    /// Degraded functionality
    Warning,
    /// Optional config not set
    Info,
}

/// A single configuration issue detected by the scanner
#[derive(Debug, Clone, Serialize)]
pub struct ConfigIssue {
    /// e.g., "S3_ACCESS_KEY", "WS_URL"
    pub field: String,
    /// Severity level
    pub severity: IssueSeverity,
    /// Human-readable description
    pub message: String,
    /// How to fix, e.g., "Open Settings > Storage"
    pub fix_hint: String,
}

/// Result of scanning configuration
#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    /// List of detected issues
    pub issues: Vec<ConfigIssue>,
    /// true if no Critical issues
    pub is_healthy: bool,
}

/// System configuration struct for scanning
/// This represents the config state loaded from environment or store
#[derive(Debug, Clone, Default)]
pub struct SysConfig {
    // S3 Configuration
    pub s3_endpoint: Option<String>,
    pub s3_region: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
    pub s3_public_url_base: Option<String>,
    // API Configuration
    pub api_base_url: Option<String>,
    // WebSocket Configuration
    pub ws_url: Option<String>,
}

impl ScanResult {
    /// Create a new ScanResult with the given issues
    pub fn new(issues: Vec<ConfigIssue>) -> Self {
        let is_healthy = !issues.iter().any(|i| i.severity == IssueSeverity::Critical);
        Self { issues, is_healthy }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if a string is a valid URL
fn is_valid_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ws://")
        || s.starts_with("wss://")
}

/// Check if a string is empty or whitespace only
fn is_empty(s: &Option<String>) -> bool {
    match s {
        None => true,
        Some(v) => v.trim().is_empty(),
    }
}

// ============================================================================
// Scanner Functions
// ============================================================================

/// Scan a SysConfig for configuration issues
///
/// # Checks
/// - S3 fields: endpoint, bucket, access_key, secret_key — if ANY missing → Critical
/// - API base URL: if empty or malformed → Critical
/// - WS URL: if empty or malformed → Warning (can use default)
/// - S3 region: if empty → Info (has default "us-east-1")
/// - S3 public_url_base: if empty → Info (optional)
pub fn scan_for_config_issues(config: &SysConfig) -> ScanResult {
    let mut issues: Vec<ConfigIssue> = Vec::new();

    // ── S3 Configuration ────────────────────────────────────────────────
    // S3 is critical - all required fields must be present

    if is_empty(&config.s3_endpoint) {
        issues.push(ConfigIssue {
            field: "S3_ENDPOINT".to_string(),
            severity: IssueSeverity::Critical,
            message: "S3 endpoint URL is not configured".to_string(),
            fix_hint: "Set S3_ENDPOINT in environment or Settings > Storage".to_string(),
        });
    } else if let Some(ref endpoint) = config.s3_endpoint {
        if !is_valid_url(endpoint) {
            issues.push(ConfigIssue {
                field: "S3_ENDPOINT".to_string(),
                severity: IssueSeverity::Critical,
                message: format!("S3 endpoint '{}' is not a valid URL", endpoint),
                fix_hint: "Ensure S3_ENDPOINT starts with http:// or https://".to_string(),
            });
        }
    }

    if is_empty(&config.s3_bucket) {
        issues.push(ConfigIssue {
            field: "S3_BUCKET".to_string(),
            severity: IssueSeverity::Critical,
            message: "S3 bucket name is not configured".to_string(),
            fix_hint: "Set S3_BUCKET in environment or Settings > Storage".to_string(),
        });
    }

    if is_empty(&config.s3_access_key) {
        issues.push(ConfigIssue {
            field: "S3_ACCESS_KEY".to_string(),
            severity: IssueSeverity::Critical,
            message: "S3 access key is not configured".to_string(),
            fix_hint: "Set S3_ACCESS_KEY in environment or Settings > Storage".to_string(),
        });
    }

    if is_empty(&config.s3_secret_key) {
        issues.push(ConfigIssue {
            field: "S3_SECRET_KEY".to_string(),
            severity: IssueSeverity::Critical,
            message: "S3 secret key is not configured".to_string(),
            fix_hint: "Set S3_SECRET_KEY in environment or Settings > Storage".to_string(),
        });
    }

    // S3 region has a default, so it's just Info
    if is_empty(&config.s3_region) {
        issues.push(ConfigIssue {
            field: "S3_REGION".to_string(),
            severity: IssueSeverity::Info,
            message: "S3 region not set, will use default 'us-east-1'".to_string(),
            fix_hint: "Optionally set S3_REGION for better performance".to_string(),
        });
    }

    // S3 public URL base is optional
    if is_empty(&config.s3_public_url_base) {
        issues.push(ConfigIssue {
            field: "S3_PUBLIC_URL_BASE".to_string(),
            severity: IssueSeverity::Info,
            message: "S3 public URL base not configured (optional CDN support)".to_string(),
            fix_hint: "Optionally set S3_PUBLIC_URL_BASE for CDN URLs".to_string(),
        });
    }

    // ── API Configuration ────────────────────────────────────────────────
    // API base URL is critical

    if is_empty(&config.api_base_url) {
        issues.push(ConfigIssue {
            field: "API_BASE_URL".to_string(),
            severity: IssueSeverity::Critical,
            message: "API base URL is not configured".to_string(),
            fix_hint: "Set API_BASE_URL at compile time or in Settings > Connection".to_string(),
        });
    } else if let Some(ref url) = config.api_base_url {
        if !is_valid_url(url) {
            issues.push(ConfigIssue {
                field: "API_BASE_URL".to_string(),
                severity: IssueSeverity::Critical,
                message: format!("API base URL '{}' is not a valid URL", url),
                fix_hint: "Ensure API_BASE_URL starts with http:// or https://".to_string(),
            });
        }
    }

    // ── WebSocket Configuration ──────────────────────────────────────────
    // WS URL has a default, so it's just a Warning

    if is_empty(&config.ws_url) {
        issues.push(ConfigIssue {
            field: "WS_URL".to_string(),
            severity: IssueSeverity::Warning,
            message: "WebSocket URL not set, will use default 'ws://localhost:8080/api/v1/ws'"
                .to_string(),
            fix_hint: "Set WS_URL in Settings > Connection if using a different server".to_string(),
        });
    } else if let Some(ref url) = config.ws_url {
        if !url.starts_with("ws://") && !url.starts_with("wss://") {
            issues.push(ConfigIssue {
                field: "WS_URL".to_string(),
                severity: IssueSeverity::Warning,
                message: format!("WebSocket URL '{}' is not a valid WS URL", url),
                fix_hint: "Ensure WS_URL starts with ws:// or wss://".to_string(),
            });
        }
    }

    ScanResult::new(issues)
}

/// Scan environment variables for configuration
///
/// Reads current env vars (S3_ENDPOINT, S3_BUCKET, etc.) and builds
/// a temporary SysConfig, then scans it for issues.
pub fn scan_env_config() -> ScanResult {
    // Load .env files if available (development friendly)
    for dir in &[".", ".."] {
        for name in &[".env", ".env.development", ".env.local"] {
            let path = std::path::Path::new(dir).join(name);
            let _ = dotenv::from_path(&path);
        }
    }

    let config = SysConfig {
        s3_endpoint: std::env::var("S3_ENDPOINT").ok(),
        s3_region: std::env::var("S3_REGION").ok(),
        s3_bucket: std::env::var("S3_BUCKET").ok(),
        s3_access_key: std::env::var("S3_ACCESS_KEY").ok(),
        s3_secret_key: std::env::var("S3_SECRET_KEY").ok(),
        s3_public_url_base: std::env::var("S3_PUBLIC_URL_BASE").ok(),
        api_base_url: option_env!("API_BASE_URL").map(|s| s.to_string()),
        ws_url: std::env::var("WS_URL").ok(),
    };

    scan_for_config_issues(&config)
}

// ============================================================================
// Tauri Command
// ============================================================================

/// Tauri command to scan configuration
///
/// Scans environment variables for configuration issues.
/// Returns a ScanResult with all detected issues.
#[tauri::command]
pub fn scan_config() -> Result<ScanResult, String> {
    Ok(scan_env_config())
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_config_present() {
        let config = SysConfig {
            s3_endpoint: Some("https://s3.example.com".to_string()),
            s3_region: Some("us-west-2".to_string()),
            s3_bucket: Some("my-bucket".to_string()),
            s3_access_key: Some("access123".to_string()),
            s3_secret_key: Some("secret456".to_string()),
            s3_public_url_base: Some("https://cdn.example.com".to_string()),
            api_base_url: Some("https://api.example.com/api/v1".to_string()),
            ws_url: Some("wss://api.example.com/api/v1/ws".to_string()),
        };

        let result = scan_for_config_issues(&config);

        // Should have no Critical issues
        assert!(
            result.is_healthy,
            "Config should be healthy when all fields present"
        );

        // May have Info issues (like optional fields), but no Critical/Warning
        let has_critical_or_warning = result
            .issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Critical || i.severity == IssueSeverity::Warning);
        assert!(
            !has_critical_or_warning,
            "Should have no Critical or Warning issues"
        );
    }

    #[test]
    fn test_missing_s3_endpoint() {
        let config = SysConfig {
            s3_endpoint: None, // Missing
            s3_region: Some("us-east-1".to_string()),
            s3_bucket: Some("my-bucket".to_string()),
            s3_access_key: Some("access123".to_string()),
            s3_secret_key: Some("secret456".to_string()),
            s3_public_url_base: None,
            api_base_url: Some("https://api.example.com/api/v1".to_string()),
            ws_url: Some("wss://api.example.com/api/v1/ws".to_string()),
        };

        let result = scan_for_config_issues(&config);

        assert!(
            !result.is_healthy,
            "Config should not be healthy with missing S3 endpoint"
        );

        let endpoint_issue = result.issues.iter().find(|i| i.field == "S3_ENDPOINT");
        assert!(endpoint_issue.is_some(), "Should have S3_ENDPOINT issue");
        assert_eq!(
            endpoint_issue.unwrap().severity,
            IssueSeverity::Critical,
            "S3_ENDPOINT missing should be Critical"
        );
    }

    #[test]
    fn test_missing_api_url() {
        let config = SysConfig {
            s3_endpoint: Some("https://s3.example.com".to_string()),
            s3_region: Some("us-east-1".to_string()),
            s3_bucket: Some("my-bucket".to_string()),
            s3_access_key: Some("access123".to_string()),
            s3_secret_key: Some("secret456".to_string()),
            s3_public_url_base: None,
            api_base_url: None, // Missing
            ws_url: Some("wss://api.example.com/api/v1/ws".to_string()),
        };

        let result = scan_for_config_issues(&config);

        assert!(
            !result.is_healthy,
            "Config should not be healthy with missing API URL"
        );

        let api_issue = result.issues.iter().find(|i| i.field == "API_BASE_URL");
        assert!(api_issue.is_some(), "Should have API_BASE_URL issue");
        assert_eq!(
            api_issue.unwrap().severity,
            IssueSeverity::Critical,
            "API_BASE_URL missing should be Critical"
        );
    }

    #[test]
    fn test_missing_ws_url() {
        let config = SysConfig {
            s3_endpoint: Some("https://s3.example.com".to_string()),
            s3_region: Some("us-east-1".to_string()),
            s3_bucket: Some("my-bucket".to_string()),
            s3_access_key: Some("access123".to_string()),
            s3_secret_key: Some("secret456".to_string()),
            s3_public_url_base: None,
            api_base_url: Some("https://api.example.com/api/v1".to_string()),
            ws_url: None, // Missing
        };

        let result = scan_for_config_issues(&config);

        // WS URL missing should NOT make config unhealthy (it's a Warning, not Critical)
        assert!(
            result.is_healthy,
            "Config should still be healthy with missing WS URL (Warning only)"
        );

        let ws_issue = result.issues.iter().find(|i| i.field == "WS_URL");
        assert!(ws_issue.is_some(), "Should have WS_URL issue");
        assert_eq!(
            ws_issue.unwrap().severity,
            IssueSeverity::Warning,
            "WS_URL missing should be Warning, not Critical"
        );
    }

    #[test]
    fn test_missing_all_s3_required() {
        let config = SysConfig {
            s3_endpoint: None,
            s3_region: None,
            s3_bucket: None,
            s3_access_key: None,
            s3_secret_key: None,
            s3_public_url_base: None,
            api_base_url: Some("https://api.example.com/api/v1".to_string()),
            ws_url: None,
        };

        let result = scan_for_config_issues(&config);

        assert!(!result.is_healthy, "Config should not be healthy");

        // Should have 4 Critical issues for S3 (endpoint, bucket, access_key, secret_key)
        let critical_s3_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .collect();

        assert!(
            critical_s3_issues.len() >= 4,
            "Should have at least 4 Critical S3 issues, got {}",
            critical_s3_issues.len()
        );
    }

    #[test]
    fn test_invalid_url_format() {
        let config = SysConfig {
            s3_endpoint: Some("not-a-url".to_string()), // Invalid URL
            s3_region: Some("us-east-1".to_string()),
            s3_bucket: Some("my-bucket".to_string()),
            s3_access_key: Some("access123".to_string()),
            s3_secret_key: Some("secret456".to_string()),
            s3_public_url_base: None,
            api_base_url: Some("also-not-a-url".to_string()), // Invalid URL
            ws_url: Some("http://should-be-ws".to_string()),  // Invalid WS URL
        };

        let result = scan_for_config_issues(&config);

        assert!(
            !result.is_healthy,
            "Config should not be healthy with invalid URLs"
        );

        // Should have Critical for S3 endpoint and API URL
        let endpoint_issue = result.issues.iter().find(|i| i.field == "S3_ENDPOINT");
        assert!(endpoint_issue.is_some());
        assert!(endpoint_issue.unwrap().message.contains("not a valid URL"));

        let api_issue = result.issues.iter().find(|i| i.field == "API_BASE_URL");
        assert!(api_issue.is_some());
        assert!(api_issue.unwrap().severity == IssueSeverity::Critical);

        // WS URL should be Warning (not ws:// or wss://)
        let ws_issue = result.issues.iter().find(|i| i.field == "WS_URL");
        assert!(ws_issue.is_some());
        assert!(ws_issue.unwrap().severity == IssueSeverity::Warning);
    }

    #[test]
    fn test_empty_strings_treated_as_missing() {
        let config = SysConfig {
            s3_endpoint: Some("   ".to_string()), // Whitespace only
            s3_region: Some("".to_string()),      // Empty string
            s3_bucket: Some("my-bucket".to_string()),
            s3_access_key: Some("access123".to_string()),
            s3_secret_key: Some("secret456".to_string()),
            s3_public_url_base: None,
            api_base_url: Some("https://api.example.com/api/v1".to_string()),
            ws_url: None,
        };

        let result = scan_for_config_issues(&config);

        // Should detect whitespace-only endpoint as missing
        let endpoint_issue = result.issues.iter().find(|i| i.field == "S3_ENDPOINT");
        assert!(endpoint_issue.is_some());
        assert_eq!(endpoint_issue.unwrap().severity, IssueSeverity::Critical);

        // Empty region should be Info (has default)
        let region_issue = result.issues.iter().find(|i| i.field == "S3_REGION");
        assert!(region_issue.is_some());
        assert_eq!(region_issue.unwrap().severity, IssueSeverity::Info);
    }
}

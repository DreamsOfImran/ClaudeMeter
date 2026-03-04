use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// Usage data returned by the session-scraping approach
// ---------------------------------------------------------------------------

/// Mirrors the usage data available on claude.ai/settings/usage.
/// All token-limit fields are expressed as percentages (0–100) so they work
/// regardless of the user's plan tier.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageData {
    /// Current-session usage % (rate-limit window)
    pub session_percent: Option<i32>,
    pub session_reset_time: Option<String>,

    /// Weekly usage % across all models
    pub weekly_all_models_percent: Option<i32>,
    pub weekly_all_models_reset: Option<String>,

    /// Weekly usage % for Sonnet only
    pub weekly_sonnet_percent: Option<i32>,
    pub weekly_sonnet_reset: Option<String>,

    /// Extra / add-on usage (Pro users)
    pub extra_spent: Option<f64>,
    pub extra_limit: Option<f64>,
    pub extra_balance: Option<f64>,
    pub extra_percent: Option<i32>,
    pub extra_reset: Option<String>,

    pub is_logged_in: bool,
    pub error: Option<String>,

    // Not present in the scraped JSON — Rust fills this in after parsing.
    #[serde(with = "chrono::serde::ts_seconds_option", default)]
    pub last_updated: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Commands sent to the background polling task
// ---------------------------------------------------------------------------

#[derive(Debug)]
#[allow(dead_code)] // Stop is reserved for graceful shutdown
pub enum PollingCommand {
    SetInterval(u64),
    RefreshNow,
    Stop,
}

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

pub struct AppState {
    pub usage: Option<UsageData>,
    pub refresh_interval_secs: u64,
    pub polling_tx: Option<mpsc::Sender<PollingCommand>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            usage: None,
            refresh_interval_secs: 300,
            polling_tx: None,
        }
    }
}

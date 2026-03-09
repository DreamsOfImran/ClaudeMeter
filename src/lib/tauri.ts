/**
 * Typed wrappers around Tauri `invoke()`.
 * All Claude API communication happens through the Rust backend.
 */
import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Data types (must mirror src-tauri/src/state.rs UsageData)
// ---------------------------------------------------------------------------

export interface UsageData {
  /** Session usage percentage (0–100), or null if unavailable */
  sessionPercent: number | null;
  sessionResetTime: string | null;

  weeklyAllModelsPercent: number | null;
  weeklyAllModelsReset: string | null;

  weeklySonnetPercent: number | null;
  weeklySonnetReset: string | null;

  /** Extra / add-on usage */
  extraSpent: number | null;
  extraLimit: number | null;
  extraBalance: number | null;
  extraPercent: number | null;
  extraReset: string | null;

  isLoggedIn: boolean;
  error: string | null;

  /** Unix timestamp seconds, or null */
  lastUpdated: number | null;
}

export interface UsageResponse {
  data: UsageData | null;
  last_refresh: string | null;
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

export const getUsage = (): Promise<UsageResponse> =>
  invoke<UsageResponse>("get_usage");

export const refreshNow = (): Promise<void> => invoke<void>("refresh_now");

export const setRefreshInterval = (seconds: number): Promise<void> =>
  invoke<void>("set_refresh_interval", { seconds });

export const getRefreshInterval = (): Promise<number> =>
  invoke<number>("get_refresh_interval");

/** Open the claude.ai login page in a hidden WebView. */
export const openLoginWindow = (): Promise<void> =>
  invoke<void>("open_login_window");

/** Manually trigger an update check (emits update-available or update-not-available). */
export const checkUpdate = (): Promise<void> => invoke<void>("check_update");

/** Download and install the latest update, then restart the app. */
export const installUpdate = (): Promise<void> => invoke<void>("install_update");

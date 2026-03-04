/**
 * Convenience hook — wraps the store and adds refresh-interval management.
 */
import { useCallback, useEffect, useState } from "react";
import { useUsageStore } from "../stores/usageStore";
import {
  getRefreshInterval,
  setRefreshInterval,
} from "../lib/tauri";

export const INTERVAL_OPTIONS = [
  { label: "1 min", value: 60 },
  { label: "5 min", value: 300 },
  { label: "15 min", value: 900 },
  { label: "30 min", value: 1800 },
  { label: "60 min", value: 3600 },
] as const;

export function useUsage() {
  const store = useUsageStore();
  const [interval, setIntervalState] = useState<number>(300);

  // Load current interval from Rust on mount.
  useEffect(() => {
    getRefreshInterval()
      .then(setIntervalState)
      .catch(() => {});
  }, []);

  const changeInterval = useCallback(async (seconds: number) => {
    setIntervalState(seconds);
    // Persist to localStorage (UI pref) and notify Rust backend.
    localStorage.setItem("cm-interval", String(seconds));
    try {
      await setRefreshInterval(seconds);
    } catch (e) {
      console.error("Failed to set refresh interval:", e);
    }
  }, []);

  return {
    ...store,
    interval,
    changeInterval,
  };
}

import { useState, useEffect, useCallback } from "react";

export type ThemePreference = "system" | "light" | "dark";

const STORAGE_KEY = "cm-theme";

function applyTheme(pref: ThemePreference): void {
  const dark =
    pref === "dark" ||
    (pref === "system" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);

  document.documentElement.classList.toggle("dark", dark);
}

export function useTheme() {
  const [preference, setPreferenceState] = useState<ThemePreference>(
    () => (localStorage.getItem(STORAGE_KEY) as ThemePreference) || "system"
  );

  // Apply theme whenever preference changes.
  useEffect(() => {
    applyTheme(preference);
  }, [preference]);

  // Re-apply when system appearance changes (only relevant for "system" pref).
  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      if (preference === "system") applyTheme("system");
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [preference]);

  // Enable smooth transitions after the initial render.
  useEffect(() => {
    const t = setTimeout(() => {
      document.documentElement.classList.add("transitions-ready");
    }, 100);
    return () => clearTimeout(t);
  }, []);

  const setPreference = useCallback((pref: ThemePreference) => {
    localStorage.setItem(STORAGE_KEY, pref);
    setPreferenceState(pref);
  }, []);

  const isDark =
    preference === "dark" ||
    (preference === "system" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);

  return { preference, setPreference, isDark };
}

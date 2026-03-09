import { useEffect, useCallback, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { UsageProvider } from "./stores/usageStore";
import { useTheme, ThemePreference } from "./hooks/useTheme";
import { useUsage } from "./hooks/useUsage";

import { Header } from "./components/Header";
import { UsageCard } from "./components/UsageCard";
import { CostDisplay } from "./components/CostDisplay";
import { Footer } from "./components/Footer";
import { ErrorState } from "./components/ErrorState";
import { LoadingState } from "./components/LoadingState";
import { LoginSetup } from "./components/LoginSetup";
import { UpdateBanner } from "./components/UpdateBanner";

// ─── Inner app (has access to store context) ──────────────────────────────────

function AppInner() {
  const { preference, setPreference } = useTheme();
  const {
    data,
    lastRefresh,
    isLoading,
    refresh,
    triggerRefresh,
    interval,
    changeInterval,
  } = useUsage();

  // Whether the user has kicked off the login flow (waiting for auth).
  const [awaitingLogin, setAwaitingLogin] = useState(false);

  // Available update version, if any.
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);

  // Hide window on blur — macOS popover style.
  useEffect(() => {
    const win = getCurrentWindow();
    let unlisten: (() => void) | undefined;
    win
      .onFocusChanged(({ payload: focused }) => {
        if (!focused) setTimeout(() => win.hide(), 80);
      })
      .then((fn) => { unlisten = fn; });
    return () => unlisten?.();
  }, []);

  // Listen for the login-success event emitted by the Rust backend.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("login-success", async () => {
      setAwaitingLogin(false);
      await triggerRefresh();
      await refresh();
    }).then((fn) => { unlisten = fn; });
    return () => unlisten?.();
  }, [triggerRefresh, refresh]);

  // Listen for update events from the Rust backend.
  useEffect(() => {
    let unlistenAvailable: (() => void) | undefined;
    let unlistenNone: (() => void) | undefined;
    listen<{ version: string }>("update-available", (e) => {
      setUpdateVersion(e.payload.version);
    }).then((fn) => { unlistenAvailable = fn; });
    listen("update-not-available", () => {
      // Briefly show "up to date" — handled by tray tooltip; nothing to show here.
    }).then((fn) => { unlistenNone = fn; });
    return () => { unlistenAvailable?.(); unlistenNone?.(); };
  }, []);

  const handleThemeCycle = useCallback(() => {
    const cycle: ThemePreference[] = ["system", "light", "dark"];
    const next = cycle[(cycle.indexOf(preference) + 1) % cycle.length];
    setPreference(next);
  }, [preference, setPreference]);

  const handleRefresh = useCallback(() => { triggerRefresh(); }, [triggerRefresh]);

  const handleLoginStarted = useCallback(() => { setAwaitingLogin(true); }, []);

  // ── Decide which body to render ───────────────────────────────────────────
  const notLoggedIn = data && !data.isLoggedIn;
  const hasUsableData = data && data.isLoggedIn;
  const showLogin = !data || notLoggedIn;
  const showLoading = !showLogin && isLoading && !data?.isLoggedIn;
  const showError = hasUsableData && !!data?.error && !isLoading;
  const showData = hasUsableData;

  return (
    <div
      className="
        flex h-screen w-full flex-col overflow-hidden
        rounded-popover bg-white text-gray-900
        border border-black/[0.1]
        dark:bg-[#1a1a1c] dark:text-gray-50 dark:border-white/[0.1]
        animate-fade-in
      "
    >
      <Header
        isLoading={isLoading}
        onRefresh={handleRefresh}
        theme={preference}
        onThemeCycle={handleThemeCycle}
      />

      <div className="mx-4 h-px bg-gray-100 dark:bg-gray-800" />

      <div className="flex-1 overflow-y-auto py-3">
        {/* Not signed in → show login prompt */}
        {showLogin && !awaitingLogin && (
          <LoginSetup onLoginStarted={handleLoginStarted} />
        )}

        {/* Login flow in progress */}
        {awaitingLogin && (
          <div className="flex flex-col items-center gap-3 px-6 py-10 text-center">
            <svg
              className="animate-spin text-amber-500"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
            >
              <circle
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="3"
                strokeOpacity="0.25"
              />
              <path
                d="M12 2a10 10 0 0 1 10 10"
                stroke="currentColor"
                strokeWidth="3"
                strokeLinecap="round"
              />
            </svg>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Waiting for sign-in…
            </p>
            <button
              onClick={() => setAwaitingLogin(false)}
              className="mt-2 text-xs text-gray-400 underline underline-offset-2 hover:text-gray-600 dark:hover:text-gray-300"
            >
              Cancel
            </button>
          </div>
        )}

        {showLoading && <LoadingState />}

        {showData && (
          <div className="flex flex-col gap-4">
            <UsageCard data={data!} />

            {showError && (
              <div className="mx-4">
                <ErrorState message={data!.error!} onRetry={handleRefresh} />
              </div>
            )}

            <CostDisplay data={data!} />
          </div>
        )}
      </div>

      {updateVersion && (
        <UpdateBanner
          version={updateVersion}
          onDismiss={() => setUpdateVersion(null)}
        />
      )}

      <div className="mx-4 h-px bg-gray-100 dark:bg-gray-800" />

      <Footer
        lastRefresh={lastRefresh}
        isLoading={isLoading}
        interval={interval}
        onIntervalChange={changeInterval}
      />
    </div>
  );
}

// ─── Root ─────────────────────────────────────────────────────────────────────

export default function App() {
  return (
    <UsageProvider>
      <AppInner />
    </UsageProvider>
  );
}

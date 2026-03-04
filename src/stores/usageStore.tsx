import {
  createContext,
  useContext,
  useReducer,
  useEffect,
  useCallback,
  useRef,
} from "react";
import { listen } from "@tauri-apps/api/event";
import { getUsage, refreshNow as tauriRefreshNow, UsageData, UsageResponse } from "../lib/tauri";

// ─── Types ───────────────────────────────────────────────────────────────────

interface UsageState {
  data: UsageData | null;
  lastRefresh: string | null;
  isLoading: boolean;
}

type Action =
  | { type: "FETCH_START" }
  | { type: "FETCH_SUCCESS"; payload: UsageResponse }
  | { type: "FETCH_ERROR" }
  | { type: "LIVE_UPDATE"; payload: UsageData };

interface UsageContextValue extends UsageState {
  refresh: () => Promise<void>;
  triggerRefresh: () => Promise<void>;
}

// ─── Reducer ─────────────────────────────────────────────────────────────────

function reducer(state: UsageState, action: Action): UsageState {
  switch (action.type) {
    case "FETCH_START":
      return { ...state, isLoading: true };
    case "FETCH_SUCCESS":
      return {
        isLoading: false,
        data: action.payload.data,
        lastRefresh: action.payload.last_refresh,
      };
    case "FETCH_ERROR":
      return { ...state, isLoading: false };
    case "LIVE_UPDATE":
      return {
        isLoading: false,
        data: action.payload,
        lastRefresh: action.payload.lastUpdated
          ? new Date(action.payload.lastUpdated * 1000).toISOString()
          : new Date().toISOString(),
      };
    default:
      return state;
  }
}

const initialState: UsageState = {
  data: null,
  lastRefresh: null,
  isLoading: false,
};

// ─── Context ─────────────────────────────────────────────────────────────────

const UsageContext = createContext<UsageContextValue | null>(null);

export function UsageProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);
  const mounted = useRef(true);

  const refresh = useCallback(async () => {
    if (!mounted.current) return;
    dispatch({ type: "FETCH_START" });
    try {
      const resp = await getUsage();
      if (mounted.current) dispatch({ type: "FETCH_SUCCESS", payload: resp });
    } catch {
      if (mounted.current) dispatch({ type: "FETCH_ERROR" });
    }
  }, []);

  const triggerRefresh = useCallback(async () => {
    dispatch({ type: "FETCH_START" });
    try {
      await tauriRefreshNow();
    } catch (e) {
      console.error("Refresh failed:", e);
      if (mounted.current) dispatch({ type: "FETCH_ERROR" });
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    let unlistenUpdate: (() => void) | undefined;
    let unlistenError: (() => void) | undefined;

    listen<UsageData>("usage-updated", (event) => {
      if (mounted.current) dispatch({ type: "LIVE_UPDATE", payload: event.payload });
    }).then((fn) => { unlistenUpdate = fn; });

    listen<string>("usage-error", () => {
      // Re-pull the cached state so the error field propagates.
      if (mounted.current) refresh();
    }).then((fn) => { unlistenError = fn; });

    return () => {
      mounted.current = false;
      unlistenUpdate?.();
      unlistenError?.();
    };
  }, [refresh]);

  return (
    <UsageContext.Provider value={{ ...state, refresh, triggerRefresh }}>
      {children}
    </UsageContext.Provider>
  );
}

export function useUsageStore(): UsageContextValue {
  const ctx = useContext(UsageContext);
  if (!ctx) throw new Error("useUsageStore must be used within UsageProvider");
  return ctx;
}

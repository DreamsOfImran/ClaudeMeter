import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { installUpdate } from "../lib/tauri";

interface UpdateBannerProps {
  version: string;
  onDismiss: () => void;
}

export function UpdateBanner({ version, onDismiss }: UpdateBannerProps) {
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!installing) return;
    let unlisten: (() => void) | undefined;
    listen<number>("update-progress", (e) => {
      setProgress(e.payload);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [installing]);

  const handleInstall = async () => {
    setInstalling(true);
    setError(null);
    try {
      await installUpdate();
      // app restarts — this line is never reached
    } catch (e) {
      setInstalling(false);
      setProgress(null);
      setError(String(e));
    }
  };

  return (
    <div className="mx-3 mb-1 rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 dark:border-amber-500/20 dark:bg-amber-500/10">
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <p className="text-[11px] font-semibold text-amber-700 dark:text-amber-400">
            Update available — v{version}
          </p>

          {installing ? (
            <div className="mt-1.5">
              <p className="text-[10px] text-amber-600 dark:text-amber-500">
                {progress !== null ? `Downloading… ${progress}%` : "Preparing download…"}
              </p>
              {progress !== null && (
                <div className="mt-1 h-1 w-full overflow-hidden rounded-full bg-amber-200 dark:bg-amber-900/50">
                  <div
                    className="h-full rounded-full bg-amber-500 transition-all duration-300"
                    style={{ width: `${progress}%` }}
                  />
                </div>
              )}
            </div>
          ) : (
            <button
              onClick={handleInstall}
              className="mt-1 text-[10px] font-medium text-amber-600 underline underline-offset-2 hover:text-amber-800 dark:text-amber-400 dark:hover:text-amber-200"
            >
              Install &amp; Restart
            </button>
          )}

          {error && (
            <p className="mt-1 text-[10px] text-red-500 dark:text-red-400 truncate">
              {error}
            </p>
          )}
        </div>

        {!installing && (
          <button
            onClick={onDismiss}
            className="mt-0.5 flex-shrink-0 text-amber-400 hover:text-amber-600 dark:text-amber-600 dark:hover:text-amber-400"
            title="Dismiss"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        )}
      </div>
    </div>
  );
}

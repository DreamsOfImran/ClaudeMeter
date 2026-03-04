import { useState } from "react";
import { openLoginWindow } from "../lib/tauri";

interface LoginSetupProps {
  onLoginStarted: () => void;
}

export function LoginSetup({ onLoginStarted }: LoginSetupProps) {
  const [opening, setOpening] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSignIn = async () => {
    setOpening(true);
    setError(null);
    try {
      await openLoginWindow();
      onLoginStarted();
    } catch (e) {
      setError(String(e));
      setOpening(false);
    }
  };

  return (
    <div className="flex flex-col items-center gap-5 px-5 py-8">
      {/* Anthropic / Claude brand mark */}
      <div className="flex h-14 w-14 items-center justify-center rounded-2xl bg-amber-50 text-amber-500 dark:bg-amber-500/10 dark:text-amber-400">
        <svg
          width="30"
          height="30"
          viewBox="0 0 100 100"
          fill="currentColor"
          xmlns="http://www.w3.org/2000/svg"
          aria-hidden="true"
        >
          {/* Simple stylised diamond as a Claude-ish logo placeholder */}
          <polygon points="50,5 95,50 50,95 5,50" opacity="0.9" />
        </svg>
      </div>

      <div className="text-center">
        <h2 className="mb-1 text-sm font-semibold text-gray-900 dark:text-gray-50">
          Sign in to Claude
        </h2>
        <p className="text-xs leading-relaxed text-gray-500 dark:text-gray-400">
          ClaudeMeter opens claude.ai in a private browser window. Your credentials
          are handled by the browser and never stored by this app.
        </p>
      </div>

      {error && (
        <p className="text-xs text-red-500 dark:text-red-400">{error}</p>
      )}

      <button
        onClick={handleSignIn}
        disabled={opening}
        className="w-full rounded-xl bg-gray-900 py-2.5 text-sm font-medium text-white transition-colors hover:bg-gray-700 disabled:cursor-not-allowed disabled:opacity-40 dark:bg-gray-100 dark:text-gray-900 dark:hover:bg-white"
      >
        {opening ? "Opening sign-in page…" : "Sign in with Claude"}
      </button>

      <p className="text-[11px] leading-relaxed text-center text-gray-400 dark:text-gray-600">
        A browser window will open for you to sign in. It will close automatically
        once you are authenticated.
      </p>
    </div>
  );
}

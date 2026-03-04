
interface ErrorStateProps {
  message: string;
  onRetry: () => void;
}

export function ErrorState({ message, onRetry }: ErrorStateProps) {
  const isAuthError =
    message.toLowerCase().includes("401") ||
    message.toLowerCase().includes("unauthorized") ||
    message.toLowerCase().includes("invalid api key");

  return (
    <div className="flex flex-col items-center gap-4 px-6 py-8 text-center">
      <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-red-50 dark:bg-red-500/10">
        <svg
          width="22"
          height="22"
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          className="text-red-500 dark:text-red-400"
        >
          <path
            d="M12 9v4M12 17h.01M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </div>

      <div>
        <p className="mb-1 text-sm font-semibold text-gray-800 dark:text-gray-200">
          {isAuthError ? "Authentication Failed" : "Refresh Failed"}
        </p>
        <p className="text-xs text-gray-500 dark:text-gray-400 leading-relaxed">
          {isAuthError
            ? "Your API key may be invalid or expired."
            : message}
        </p>
      </div>

      <button
        onClick={onRetry}
        className="rounded-lg bg-gray-900 px-4 py-2 text-xs font-medium text-white transition-colors hover:bg-gray-700 dark:bg-gray-100 dark:text-gray-900 dark:hover:bg-white"
      >
        Retry
      </button>
    </div>
  );
}

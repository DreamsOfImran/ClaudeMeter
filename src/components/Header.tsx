import { ThemePreference } from "../hooks/useTheme";

interface HeaderProps {
  isLoading: boolean;
  onRefresh: () => void;
  theme: ThemePreference;
  onThemeCycle: () => void;
}

const THEME_ICONS: Record<ThemePreference, string> = {
  system: "⊙",
  light: "☀",
  dark: "☽",
};
const THEME_LABELS: Record<ThemePreference, string> = {
  system: "System",
  light: "Light",
  dark: "Dark",
};

export function Header({ isLoading, onRefresh, theme, onThemeCycle }: HeaderProps) {
  return (
    <header className="flex items-center justify-between px-3 py-2.5">
      {/* Logo + name */}
      <div className="flex items-center gap-2">
        <img
          src="/logo.png"
          alt="ClaudeMeter"
          className="h-5 w-5 object-contain dark:invert"
        />
        <span className="text-[13px] font-semibold tracking-tight text-gray-900 dark:text-gray-50">
          ClaudeMeter
        </span>
      </div>

      {/* Controls */}
      <div className="flex items-center gap-0.5">
        <button
          onClick={onThemeCycle}
          title={`Theme: ${THEME_LABELS[theme]}`}
          className="flex h-6 w-6 items-center justify-center rounded text-[11px] text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:text-gray-500 dark:hover:bg-gray-700 dark:hover:text-gray-300"
        >
          {THEME_ICONS[theme]}
        </button>

        <button
          onClick={onRefresh}
          disabled={isLoading}
          title="Refresh now"
          className="flex h-6 w-6 items-center justify-center rounded text-gray-400 hover:bg-gray-100 hover:text-gray-600 disabled:opacity-40 dark:text-gray-500 dark:hover:bg-gray-700 dark:hover:text-gray-300"
        >
          <svg
            width="12"
            height="12"
            viewBox="0 0 16 16"
            fill="none"
            className={isLoading ? "animate-spin" : ""}
          >
            <path
              d="M13.5 8A5.5 5.5 0 1 1 8 2.5M13.5 2.5v3h-3"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>
      </div>
    </header>
  );
}

import { INTERVAL_OPTIONS } from "../hooks/useUsage";

interface FooterProps {
  lastRefresh: string | null;
  isLoading: boolean;
  interval: number;
  onIntervalChange: (seconds: number) => void;
}

function formatTime(iso: string | null): string {
  if (!iso) return "Never";
  const d = new Date(iso);
  const h = d.getHours();
  const m = String(d.getMinutes()).padStart(2, "0");
  const ampm = h >= 12 ? "pm" : "am";
  return `${h % 12 || 12}:${m}${ampm}`;
}

export function Footer({ lastRefresh, isLoading, interval, onIntervalChange }: FooterProps) {
  const label = isLoading ? "Refreshing…" : `Last updated: ${formatTime(lastRefresh)}`;

  return (
    <footer className="flex items-center justify-between px-3 py-2 border-t border-gray-100 dark:border-gray-800">
      <span className="tabular text-[10px] text-gray-500 dark:text-gray-400">
        {label}
      </span>

      <select
        value={interval}
        onChange={(e) => onIntervalChange(Number(e.target.value))}
        className="rounded border border-gray-200 bg-white px-1.5 py-0.5 text-[10px] text-gray-500 outline-none hover:border-gray-300 focus:border-amber-500 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-400"
        title="Refresh interval"
      >
        {INTERVAL_OPTIONS.map(({ label, value }) => (
          <option key={value} value={value}>
            {label}
          </option>
        ))}
      </select>
    </footer>
  );
}

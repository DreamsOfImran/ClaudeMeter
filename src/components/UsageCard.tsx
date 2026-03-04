import { UsageData } from "../lib/tauri";

interface UsageCardProps {
  data: UsageData;
}

interface GaugeProps {
  label: string;
  percent: number | null;
  resetTime: string | null;
  color: string;
}

function Gauge({ label, percent, resetTime, color }: GaugeProps) {
  const pct = percent ?? 0;
  const barColor =
    pct >= 95 ? "bg-red-500" : pct >= 70 ? "bg-amber-500" : color;
  const pctColor =
    pct >= 95
      ? "text-red-600 dark:text-red-400"
      : pct >= 70
      ? "text-amber-600 dark:text-amber-400"
      : "text-gray-800 dark:text-gray-100";

  return (
    <div className="flex flex-col gap-1 rounded-lg bg-gray-50 px-3 py-2 dark:bg-gray-800/60">
      <div className="flex items-center justify-between">
        <span className="text-[11px] font-medium text-gray-500 dark:text-gray-400">
          {label}
        </span>
        <span className={`tabular text-[13px] font-bold ${pctColor}`}>
          {percent != null ? `${percent}%` : "—"}
        </span>
      </div>

      <div className="h-1 w-full overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
        <div
          className={`h-full rounded-full transition-all duration-700 ${barColor}`}
          style={{ width: `${Math.min(pct, 100)}%` }}
        />
      </div>

      {resetTime && (
        <p className="text-[10px] text-gray-400 dark:text-gray-600 truncate">
          Resets {resetTime}
        </p>
      )}
    </div>
  );
}

export function UsageCard({ data }: UsageCardProps) {
  return (
    <section className="px-3 flex flex-col gap-1.5">
      <Gauge
        label="Session"
        percent={data.sessionPercent}
        resetTime={data.sessionResetTime}
        color="bg-blue-400 dark:bg-blue-500"
      />
      <Gauge
        label="Weekly · All Models"
        percent={data.weeklyAllModelsPercent}
        resetTime={data.weeklyAllModelsReset}
        color="bg-violet-400 dark:bg-violet-500"
      />
    </section>
  );
}

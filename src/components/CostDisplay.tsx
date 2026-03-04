import { UsageData } from "../lib/tauri";

interface CostDisplayProps {
  data: UsageData;
}

function fmt(v: number | null, prefix = "$"): string {
  if (v == null) return "—";
  return `${prefix}${v.toFixed(2)}`;
}

function ExtraBar({ spent, limit }: { spent: number | null; limit: number | null }) {
  if (spent == null || limit == null || limit === 0) return null;
  const pct = Math.min((spent / limit) * 100, 100);
  return (
    <div className="mt-2 h-1.5 w-full overflow-hidden rounded-full bg-gray-100 dark:bg-gray-800">
      <div
        className="h-full rounded-full bg-amber-400 dark:bg-amber-500 transition-all duration-700"
        style={{ width: `${pct.toFixed(1)}%` }}
      />
    </div>
  );
}

export function CostDisplay({ data }: CostDisplayProps) {
  const hasExtra =
    data.extraSpent != null ||
    data.extraLimit != null ||
    data.extraBalance != null;

  if (!hasExtra) return null;

  return (
    <section className="px-4">
      <h2 className="mb-3 text-[10px] font-semibold uppercase tracking-widest text-gray-400 dark:text-gray-500">
        Extra Usage
      </h2>

      <div className="rounded-xl border border-gray-100 dark:border-gray-700/60 overflow-hidden">
        {[
          { label: "Spent", value: fmt(data.extraSpent) },
          { label: "Limit", value: fmt(data.extraLimit) },
          { label: "Balance", value: fmt(data.extraBalance) },
        ].map(({ label, value }, i, arr) => (
          <div
            key={label}
            className={`flex items-center justify-between px-3 py-2 ${
              i < arr.length - 1
                ? "border-b border-gray-100 dark:border-gray-700/60"
                : ""
            }`}
          >
            <span className="text-xs text-gray-500 dark:text-gray-400">{label}</span>
            <span className="tabular text-xs font-medium text-gray-700 dark:text-gray-300">
              {value}
            </span>
          </div>
        ))}

        {data.extraLimit != null && (
          <div className="px-3 pb-3">
            <ExtraBar spent={data.extraSpent} limit={data.extraLimit} />
            {data.extraReset && (
              <p className="mt-1 text-[10px] text-gray-400 dark:text-gray-500">
                Resets {data.extraReset}
              </p>
            )}
          </div>
        )}
      </div>
    </section>
  );
}

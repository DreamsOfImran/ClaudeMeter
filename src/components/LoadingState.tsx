
export function LoadingState() {
  return (
    <div className="flex flex-col gap-3 px-4 animate-pulse">
      {/* Stats row */}
      <div className="grid grid-cols-2 gap-2">
        {[...Array(2)].map((_, i) => (
          <div
            key={i}
            className="h-16 rounded-xl bg-gray-100 dark:bg-gray-800"
          />
        ))}
      </div>
      <div className="grid grid-cols-2 gap-2">
        {[...Array(2)].map((_, i) => (
          <div
            key={i}
            className="h-16 rounded-xl bg-gray-100 dark:bg-gray-800"
          />
        ))}
      </div>

      {/* Divider */}
      <div className="h-px bg-gray-100 dark:bg-gray-800" />

      {/* Token bars */}
      <div className="flex flex-col gap-3">
        {[...Array(2)].map((_, i) => (
          <div key={i} className="flex flex-col gap-1.5">
            <div className="h-3 w-24 rounded bg-gray-100 dark:bg-gray-800" />
            <div className="h-1.5 rounded-full bg-gray-100 dark:bg-gray-800" />
          </div>
        ))}
      </div>
    </div>
  );
}

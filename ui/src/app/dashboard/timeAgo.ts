// Freshness label for the Home readiness line ("Last check 12s ago"). Uses
// second granularity while recent so it visibly counts up and surfaces a stalled
// poll, then falls back to minutes/hours. Empty string for a bad/missing value.
export function lastCheckLabel(iso: string, now: number = Date.now()): string {
  const then = Date.parse(iso)
  if (Number.isNaN(then)) {
    return ""
  }
  const seconds = Math.max(0, Math.round((now - then) / 1000))
  if (seconds < 10) return "just now"
  if (seconds < 60) return `${seconds}s ago`
  const minutes = Math.round(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.round(minutes / 60)
  return `${hours}h ago`
}

// Compact "time ago" for run timestamps (design doc metadata: "2m ago",
// "1h ago"). Backend timestamps are ISO strings; a bad or missing value yields
// an em dash rather than "NaN". Pure and deterministic given `now`, so it is
// unit-testable without faking the clock.
export function timeAgo(iso: string, now: number = Date.now()): string {
  const then = Date.parse(iso)
  if (Number.isNaN(then)) {
    return "—"
  }
  const seconds = Math.max(0, Math.round((now - then) / 1000))
  if (seconds < 45) return "just now"
  const minutes = Math.round(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.round(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.round(hours / 24)
  return `${days}d ago`
}

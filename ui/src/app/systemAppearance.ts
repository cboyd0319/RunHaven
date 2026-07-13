type AppearanceQuery = {
  matches: boolean
  addEventListener: (
    type: "change",
    listener: (event: { matches: boolean }) => void,
  ) => void
  removeEventListener: (
    type: "change",
    listener: (event: { matches: boolean }) => void,
  ) => void
}

// Follow the Mac's light/dark appearance. RunHaven has no separate theme
// preference, so the native system setting stays the single source of truth.
export function installSystemAppearance(
  root = document.documentElement,
  query: AppearanceQuery = window.matchMedia("(prefers-color-scheme: dark)"),
): () => void {
  const apply = (dark: boolean) => root.classList.toggle("dark", dark)
  const onChange = (event: { matches: boolean }) => apply(event.matches)
  apply(query.matches)
  query.addEventListener("change", onChange)
  return () => query.removeEventListener("change", onChange)
}

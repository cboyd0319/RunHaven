import { useEffect, useRef } from "react"

// Move focus to a region when the flow moves into it, so keyboard and screen
// reader users are not dropped onto <body> at the key moments of the launch and
// session flow. Attach the returned ref to a `tabIndex={-1}` element (a heading
// or the view container) and pass a `key` that changes on the transition.
//
// `focusOnMount` controls whether the very first render focuses: a view that
// only mounts on a real transition (the session view) wants focus on mount,
// while a view that can be the initial screen (the launch view) should only take
// focus when its `key` later changes, never on first paint.
export function useFocusOnChange<T extends HTMLElement>(
  key: unknown,
  { focusOnMount = false }: { focusOnMount?: boolean } = {},
) {
  const ref = useRef<T>(null)
  const shouldFocus = useRef(focusOnMount)

  useEffect(() => {
    if (shouldFocus.current) {
      ref.current?.focus()
    }
    shouldFocus.current = true
  }, [key])

  return ref
}

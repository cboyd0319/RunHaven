// Minimal hook renderer for behavioral tests. The repo has no React DOM test
// library, so this drives a hook through react-dom/client with React's own
// `act`. Tests that use it must declare `// @vitest-environment happy-dom`.
import { act, createElement } from "react"
import { createRoot } from "react-dom/client"

export { act }

// React uses this flag to allow state updates outside a real event loop.
;(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true

export type RenderHookResult<TProps, TResult> = {
  result: { current: TResult }
  rerender: (props: TProps) => Promise<void>
  unmount: () => Promise<void>
}

export async function renderHook<TProps, TResult>(
  render: (props: TProps) => TResult,
  initialProps: TProps,
): Promise<RenderHookResult<TProps, TResult>> {
  const container = document.createElement("div")
  const root = createRoot(container)
  const result = { current: undefined as TResult }

  function TestComponent({ hookProps }: { hookProps: TProps }): null {
    result.current = render(hookProps)
    return null
  }

  await act(async () => {
    root.render(createElement(TestComponent, { hookProps: initialProps }))
  })

  return {
    result,
    async rerender(props: TProps) {
      await act(async () => {
        root.render(createElement(TestComponent, { hookProps: props }))
      })
    },
    async unmount() {
      await act(async () => {
        root.unmount()
      })
    },
  }
}

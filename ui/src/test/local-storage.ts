// Minimal localStorage stand-in for happy-dom tests. Node's experimental
// global localStorage (undefined without --localstorage-file) shadows
// happy-dom's own Storage, so tests that exercise persistence install this
// in-memory stub on window instead.
export function installLocalStorageStub(): void {
  const store = new Map<string, string>()
  Object.defineProperty(window, "localStorage", {
    configurable: true,
    value: {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => {
        store.set(key, String(value))
      },
      removeItem: (key: string) => {
        store.delete(key)
      },
      clear: () => {
        store.clear()
      },
    },
  })
}

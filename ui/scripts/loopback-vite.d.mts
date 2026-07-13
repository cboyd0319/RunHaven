export interface LoopbackViteServer {
  url: string
  close(): Promise<void>
}

export interface LoopbackViteOptions {
  root: string
  logLevel?: "info" | "warn" | "error" | "silent"
}

export function startLoopbackVite(
  options: LoopbackViteOptions,
): Promise<LoopbackViteServer>

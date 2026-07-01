// Types for the in-app maintenance surface: agent image rebuild and network
// cleanup. Both mirror the sanitized Tauri command responses; counts and
// bounded, redacted output only, never raw host paths or network names.

export type ImageBuildStartResponse = {
  buildId: string;
  agent: string;
  tag: string;
  state: string;
  label: string;
  detail: string;
};

export type ImageBuildStatusResponse = {
  buildId: string;
  agent: string;
  tag: string;
  state: "building" | "succeeded" | "failed" | string;
  label: string;
  detail: string;
  outputTail: string;
  updatedAt: string;
};

export type NetworkPrunePreview = {
  count: number;
};

export type NetworkPruneResponse = {
  removed: number;
  failed: number;
};

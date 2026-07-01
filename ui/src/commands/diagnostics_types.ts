// Diagnostics command response types: egress log, auth log, auth status, and
// auth explain. These mirror the sanitized Tauri diagnostics contracts; no
// tokens, keys, or raw host paths are present. Re-exported through types.ts and
// the runhaven barrel so existing imports stay stable.

export type EgressLogEntry = {
  timestamp: string;
  profile: string;
  decision: string;
  host: string;
  port: number;
  count: number;
  reason: string;
  matchedRule: string;
  runId: string;
};

export type EgressLogResponse = {
  entries: EgressLogEntry[];
};

export type AuthLogEntry = {
  timestamp: string;
  profile: string;
  broker: string;
  decision: string;
  method: string;
  path: string;
  upstreamStatus: number | null;
  count: number;
  reason: string;
  runId: string;
};

export type AuthLogResponse = {
  entries: AuthLogEntry[];
};

export type AuthProfileStatus = {
  name: string;
  status: string;
};

export type AuthStatusResponse = {
  status: string;
  runtime: string;
  profiles: AuthProfileStatus[];
};

export type AuthExplainResponse = {
  name: string;
  status: string;
  runtime: string;
  supportedAuth: string[];
  hostKeeps: string[];
  guestReceives: string[];
  providerHosts: string[];
  currentSafePath: string;
  notes: string[];
};

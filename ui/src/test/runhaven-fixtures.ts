import type { AgentProfile } from "../commands/runhaven";

export const providerAgent: AgentProfile = {
  name: "codex",
  description: "Codex",
  image: "runhaven/codex:0.7.0",
  defaultCommand: ["codex"],
  providerHosts: ["api.openai.com"],
  signIn: "runhaven login",
  broker: "yes",
  supportsInAppSignIn: true
};

export const localAgent: AgentProfile = {
  name: "shell",
  description: "Shell",
  image: "runhaven/base:0.7.0",
  defaultCommand: ["/bin/bash"],
  providerHosts: [],
  signIn: "n/a",
  broker: "n/a",
  supportsInAppSignIn: false
};

import { call } from "./client";
import type { RunPlanRequest, SavedSetupSummary } from "./types";

export async function getSavedSetups(): Promise<SavedSetupSummary[]> {
  return call("get_saved_setups", {}, () => []);
}

export async function pinSavedSetup(
  assistant: string,
  projectPath: string
): Promise<string> {
  return call(
    "pin_saved_setup",
    { assistant, projectPath },
    () => "preview-saved-setup"
  );
}

export async function selectSavedSetup(id: string): Promise<RunPlanRequest> {
  return call("select_saved_setup", { id }, () => {
    throw new Error("Saved setups need the desktop app.");
  });
}

export async function setSavedSetupPinned(
  id: string,
  pinned: boolean
): Promise<void> {
  return call("set_saved_setup_pinned", { id, pinned }, () => undefined);
}

import { listen } from "@tauri-apps/api/event";

import { hasTauriRuntime } from "./client";

// The single Rust-to-WebView menu channel. The Rust menu bar
// (src-tauri/src/menu.rs) emits each activated item's id string on this
// event; nothing else rides it. The browser preview has no Tauri runtime and
// no menu bar, so listening is a safe no-op there.
export const MENU_COMMAND_EVENT = "menu://command";

export type MenuCommandHandler = (id: string) => void;

// Subscribe to menu item activations. Resolves to an unsubscribe function;
// in the browser preview it resolves immediately to a no-op.
export async function listenForMenuCommands(
  handler: MenuCommandHandler
): Promise<() => void> {
  if (!hasTauriRuntime()) {
    return () => {};
  }
  return listen<string>(MENU_COMMAND_EVENT, (event) => handler(event.payload));
}

// Persistence helpers mapping schema-defined entities to disk per data/iodd_form_schema.json (.temp.details/iodd_xsd_comprehensive.csv compatibility).
// Stores state using Tauri filesystem when available, otherwise falls back to localStorage for web preview.

import type { RootDocumentState } from "$lib/models/generated";

type TauriFsModule = typeof import("@tauri-apps/plugin-fs");

const STATE_FILE = "iodd-builder-state.json";

function isTauri(): boolean {
  return typeof window !== "undefined" && !!window.__TAURI_IPC__;
}

async function resolveFsModule(): Promise<TauriFsModule | null> {
  const moduleName = "@tauri-apps/plugin-fs";
  try {
    const fs = (await import(/* @vite-ignore */ moduleName)) as TauriFsModule;
    return fs;
  } catch (error) {
    console.debug("Tauri FS plugin unavailable, falling back to web persistence.", error);
    return null;
  }
}

export async function loadState(): Promise<RootDocumentState | null> {
  if (isTauri()) {
    try {
      const fs = await resolveFsModule();
      if (!fs) {
        return null;
      }
      const { exists, readTextFile, BaseDirectory } = fs;
      const stateExists = await exists(STATE_FILE, { baseDir: BaseDirectory.AppData });
      if (!stateExists) {
        return null;
      }
      const content = await readTextFile(STATE_FILE, { baseDir: BaseDirectory.AppData });
      return JSON.parse(content) as RootDocumentState;
    } catch (error) {
      console.warn("Failed to load persisted state", error);
      return null;
    }
  }

  if (typeof localStorage !== "undefined") {
    const raw = localStorage.getItem(STATE_FILE);
    if (!raw) return null;
    try {
      return JSON.parse(raw) as RootDocumentState;
    } catch (error) {
      console.warn("Failed to parse localStorage state", error);
      return null;
    }
  }

  return null;
}

export async function saveState(state: RootDocumentState): Promise<void> {
  const payload = JSON.stringify(state, null, 2);

  if (isTauri()) {
    try {
      const fs = await resolveFsModule();
      if (!fs) {
        throw new Error("Tauri FS module missing");
      }
      const { writeTextFile, BaseDirectory } = fs;
      await writeTextFile(STATE_FILE, payload, { baseDir: BaseDirectory.AppData });
      return;
    } catch (error) {
      console.error("Failed to persist state via Tauri FS", error);
    }
  }

  if (typeof localStorage !== "undefined") {
    localStorage.setItem(STATE_FILE, payload);
  }
}

declare global {
  interface Window {
    __TAURI_IPC__?: unknown;
  }
}

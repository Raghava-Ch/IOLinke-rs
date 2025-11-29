// Document store binds schema-defined entities (data/iodd_form_schema.json, .temp.details/iodd_xsd_comprehensive.csv) to Svelte state.
// Responsible for persistence, undo stack seeds, and import/export management.

import { writable } from "svelte/store";
import sampleStateRaw from "../../../data/sample_state.json?raw";
import type { RootDocumentState } from "$lib/models/generated";
import { loadState, saveState } from "$lib/utils/persistence";

const SAMPLE_STATE = JSON.parse(sampleStateRaw) as RootDocumentState;

function cloneDeep<T>(value: T): T {
  if (typeof structuredClone === "function") {
    return structuredClone(value);
  }
  return JSON.parse(JSON.stringify(value)) as T;
}

const documentStore = writable<RootDocumentState>(cloneDeep(SAMPLE_STATE));

let hydrated = false;

loadState()
  .then((persisted) => {
    if (persisted) {
      documentStore.set(persisted);
    }
  })
  .finally(() => {
    hydrated = true;
  });

documentStore.subscribe((state) => {
  if (!hydrated) return;
  void saveState(state);
});

export { documentStore };

export function resetToSample(): void {
  documentStore.set(cloneDeep(SAMPLE_STATE));
}

export function importState(state: RootDocumentState): void {
  documentStore.set(cloneDeep(state));
}

export function exportState(): Promise<RootDocumentState> {
  return new Promise((resolve) => {
    let lastValue: RootDocumentState | null = null;
    const unsubscribe = documentStore.subscribe((value) => {
      lastValue = cloneDeep(value);
    });
    unsubscribe();
    resolve(lastValue ?? cloneDeep(SAMPLE_STATE));
  });
}

export function updateEntity<K extends keyof RootDocumentState>(
  entityId: K,
  updater: (value: RootDocumentState[K]) => RootDocumentState[K]
): void {
  documentStore.update((state) => {
    const next = cloneDeep(state);
    next[entityId] = updater(cloneDeep(state[entityId]));
    return next;
  });
}


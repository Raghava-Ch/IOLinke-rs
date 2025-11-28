// UI-focused store orchestrating selection, tabs, and panel state from data/iodd_form_schema.json (.temp.details/iodd_xsd_comprehensive.csv alignment).
// Supports activity bar toggles, open editors, and inspector focus.

import { derived, get, writable } from "svelte/store";
import type { HierarchyInstanceNode } from "$lib/stores/hierarchy";
import { hierarchyStore } from "$lib/stores/hierarchy";

export interface EditorTabDescriptor {
  id: string;
  entity: string;
  label: string;
  path: string[];
}

export const selectedPath = writable<string | null>(null);
export const activitySection = writable<"hierarchy" | "validation" | "preview">("hierarchy");
export const openTabs = writable<EditorTabDescriptor[]>([]);
export const activeTabId = writable<string | null>(null);

function flattenHierarchy(nodes: HierarchyInstanceNode[]): HierarchyInstanceNode[] {
  const result: HierarchyInstanceNode[] = [];
  const queue = [...nodes];

  while (queue.length) {
    const node = queue.shift();
    if (!node) continue;
    result.push(node);
    queue.push(...node.children);
  }

  return result;
}

export const flatHierarchy = derived(hierarchyStore, (value) => flattenHierarchy(value));

export const selectedNode = derived([flatHierarchy, selectedPath], ([$flatHierarchy, $selectedPath]) => {
  if (!$selectedPath) return null;
  return $flatHierarchy.find((node) => node.id === $selectedPath || node.path.join("/") === $selectedPath) ?? null;
});

selectedNode.subscribe((node) => {
  if (!node) return;
  const tabId = node.id;
  openTabs.update((tabs) => {
    if (tabs.find((tab) => tab.id === tabId)) {
      return tabs;
    }
    return [
      ...tabs,
      {
        id: tabId,
        entity: node.entity,
        label: node.label,
        path: node.path,
      },
    ];
  });
  activeTabId.set(tabId);
});

export function closeTab(id: string): void {
  const currentActive = get(activeTabId);
  let filtered: EditorTabDescriptor[] = [];
  openTabs.update((tabs) => {
    filtered = tabs.filter((tab) => tab.id !== id);
    return filtered;
  });

  if (currentActive === id) {
    const nextActive = filtered[filtered.length - 1]?.id ?? null;
    activeTabId.set(nextActive);
    if (nextActive) {
      selectedPath.set(nextActive);
    }
  }
}

export function activateTab(id: string): void {
  activeTabId.set(id);
  selectedPath.set(id);
}

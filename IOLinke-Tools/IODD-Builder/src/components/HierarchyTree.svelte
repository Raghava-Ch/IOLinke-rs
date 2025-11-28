<script lang="ts">
  // Hierarchy tree mirrors schema relationships for quick navigation through document entities.
  import { onMount } from "svelte";
  import { get, writable } from "svelte/store";
  import { hierarchyStore } from "$lib/stores/hierarchy";
  import { selectedNode, selectedPath } from "$lib/stores/ui";
  import HierarchyTreeNode from "./HierarchyTreeNode.svelte";

  const collapsed = writable<Set<string>>(new Set());

  function toggle(id: string) {
    collapsed.update((set) => {
      const next = new Set(set);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }

  function select(id: string) {
    selectedPath.set(id);
  }

  onMount(() => {
    const unsubscribe = hierarchyStore.subscribe((nodes) => {
      if (!nodes.length) return;
      const current = get(selectedNode);
      if (!current) {
        const firstId = nodes[0]?.id;
        if (firstId) {
          select(firstId);
        }
      }
    });
    return () => unsubscribe();
  });
</script>

<div class="flex h-full flex-col overflow-hidden">
  <div class="px-4 pt-4">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-slate-300">Hierarchy</h2>
  </div>
  <div class="mt-3 flex-1 overflow-y-auto px-2 pb-4">
    <div aria-label="Document hierarchy" role="tree" class="space-y-1">
      {#each $hierarchyStore as node}
        <HierarchyTreeNode
          {node}
          collapsedSet={$collapsed}
          {toggle}
          selectedId={$selectedPath}
          on:select={(event) => select(event.detail)}
        />
      {/each}
      {#if !$hierarchyStore.length}
        <p class="px-2 text-xs text-slate-500">No entities have been loaded.</p>
      {/if}
    </div>
  </div>
</div>

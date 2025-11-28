<script lang="ts">
  import type { HierarchyInstanceNode } from "$lib/stores/hierarchy";
  import { createEventDispatcher } from "svelte";

  export let node: HierarchyInstanceNode;
  export let depth = 0;
  export let collapsedSet: Set<string>;
  export let toggle: (id: string) => void;
  export let selectedId: string | null = null;

  const dispatch = createEventDispatcher<{ select: string }>();

  $: isCollapsed = collapsedSet.has(node.id);
  $: isSelected = selectedId === node.id;
  $: hasChildren = node.children.length > 0;

  function handleSelect() {
    dispatch("select", node.id);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      handleSelect();
    }
  }
</script>

<div class="flex flex-col">
  <div
    class="flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-sm transition hover:bg-slate-800"
    class:bg-slate-700={isSelected}
    class:text-white={isSelected}
    style={`margin-left: ${depth * 0.75}rem;`}
    on:click={handleSelect}
    on:keydown={handleKeydown}
    role="treeitem"
    aria-expanded={hasChildren ? !isCollapsed : undefined}
    aria-selected={isSelected}
    tabindex="0"
  >
    {#if hasChildren}
      <button
        type="button"
        class="flex h-5 w-5 items-center justify-center rounded bg-slate-800 text-xs text-slate-200"
        on:click|stopPropagation={() => toggle(node.id)}
        aria-label={isCollapsed ? `Expand ${node.label}` : `Collapse ${node.label}`}
      >
        {isCollapsed ? "+" : "-"}
      </button>
    {:else}
      <span class="h-5 w-5" aria-hidden="true"></span>
    {/if}
    <span class="font-medium text-slate-200">{node.label}</span>
    {#if node.count !== undefined && node.count > 0}
      <span class="rounded bg-slate-900/80 px-1 text-xs text-slate-300">{node.count}</span>
    {/if}
  </div>

  {#if hasChildren && !isCollapsed}
    <div role="group">
      {#each node.children as child}
        <svelte:self
          node={child}
          depth={depth + 1}
          {collapsedSet}
          {toggle}
          {selectedId}
          on:select
        />
      {/each}
    </div>
  {/if}
</div>

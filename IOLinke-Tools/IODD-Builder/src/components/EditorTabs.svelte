<script lang="ts">
  // Editor tabs reflect active form editors opened from the hierarchy tree.
  import { closeTab, activateTab, activeTabId, openTabs } from "$lib/stores/ui";
</script>

<div class="flex items-center gap-2 overflow-x-auto border-b border-slate-800 bg-slate-950 px-3 py-2 text-sm">
  {#if !$openTabs.length}
    <span class="text-xs text-slate-500">Select an entity from the hierarchy to begin editing.</span>
  {/if}
  {#each $openTabs as tab}
    {@const isActive = $activeTabId === tab.id}
    <div
      class="flex shrink-0 items-center gap-2 rounded border border-transparent bg-slate-900/60 px-3 py-1 text-slate-300 transition"
      class:border-slate-400={isActive}
      class:bg-slate-800={isActive}
      class:text-white={isActive}
    >
      <button
        type="button"
        class="text-left text-sm font-medium"
        on:click={() => activateTab(tab.id)}
      >
        {tab.label}
      </button>
      <button
        type="button"
        class="text-xs text-slate-400 transition hover:text-slate-200"
        on:click={() => closeTab(tab.id)}
        aria-label={`Close ${tab.label}`}
      >
        ×
      </button>
    </div>
  {/each}
</div>

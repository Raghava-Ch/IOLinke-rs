<script lang="ts">
  // Toolbar surfaces validation actions and persistence helpers for the renderer workspace.
  import { createEventDispatcher } from "svelte";
  import { Button } from "flowbite-svelte";
  import { resetToSample } from "$lib/stores/documents";
  import { errorCount, runValidation, warningCount, lastValidatedAt } from "$lib/stores/validation";

  const dispatch = createEventDispatcher<{ export: void }>();

  let validating = false;

  async function handleValidate() {
    validating = true;
    await runValidation();
    validating = false;
  }

  function handleExport() {
    dispatch("export");
  }

  function handleReset() {
    resetToSample();
  }
</script>

<div class="flex items-center justify-between border-b border-slate-800 bg-slate-950 px-4 py-3 text-sm">
  <div class="flex items-center gap-3">
    <Button size="sm" color="blue" on:click={handleValidate} disabled={validating}>
      {#if validating}
        Validating...
      {:else}
        Run Validation
      {/if}
    </Button>
    <Button size="sm" color="green" on:click={handleExport}>
      Export XML
    </Button>
    <Button size="sm" color="light" on:click={handleReset}>
      Reset to Sample
    </Button>
  </div>
  <div class="flex items-center gap-4 text-xs text-slate-400">
    <span>Error count: <span class="text-rose-400 font-semibold">{$errorCount}</span></span>
    <span>Warnings: <span class="text-amber-300 font-semibold">{$warningCount}</span></span>
    {#if $lastValidatedAt}
      <span>Last run: {$lastValidatedAt.toLocaleTimeString()}</span>
    {/if}
  </div>
</div>

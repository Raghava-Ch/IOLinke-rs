<script lang="ts">
  // Validation panel aggregates outstanding issues across the entire document for quick triage.
  import { Button } from "flowbite-svelte";
  import { runValidation, validationIssues } from "$lib/stores/validation";

  let validating = false;

  async function handleValidate() {
    validating = true;
    await runValidation();
    validating = false;
  }
</script>

<div class="flex h-full flex-col overflow-hidden">
  <div class="flex items-center justify-between border-b border-slate-800 px-4 py-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-slate-300">Validation</h2>
    <Button size="xs" color="blue" on:click={handleValidate} disabled={validating}>
      {validating ? "Running..." : "Re-run"}
    </Button>
  </div>
  <div class="flex-1 overflow-y-auto px-4 py-4 text-sm text-slate-200">
    {#if !$validationIssues.length}
      <p class="text-xs text-slate-500">Run validation to populate this list.</p>
    {:else}
      <ul class="space-y-3">
        {#each $validationIssues as issue}
          <li class="rounded border border-slate-700 px-3 py-2">
            <p class="text-xs uppercase tracking-wide"
              class:text-rose-400={issue.severity === "error"}
              class:text-amber-300={issue.severity === "warning"}
            >
              {issue.severity}
            </p>
            <p class="mt-1 text-sm font-semibold text-slate-100">{issue.entity}</p>
            <p class="mt-1 text-xs text-slate-300">{issue.message}</p>
            <p class="mt-2 font-mono text-[0.65rem] text-slate-500">{issue.path}</p>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

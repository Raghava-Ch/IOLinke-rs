<script lang="ts">
  // Inspector surfaces schema details and validation messages for the selected entity node.
  import { selectedNode } from "$lib/stores/ui";
  import { validationIssues, lastValidatedAt } from "$lib/stores/validation";
  import { getEntitySchema } from "$lib/schema/loader";

  $: node = $selectedNode;
  $: schema = node ? getEntitySchema(node.entity) : null;
  $: multiplicity = node?.multiplicity;
  $: issues = node
    ? $validationIssues.filter((issue) => issue.entity === node?.entity || issue.path.startsWith(node?.entity ?? ""))
    : [];
  $: validatedAt = $lastValidatedAt;
</script>

<div class="flex h-full flex-col overflow-hidden bg-slate-950">
  <div class="border-b border-slate-800 px-4 py-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-slate-300">Inspector</h2>
  </div>
  <div class="flex-1 overflow-y-auto px-4 py-4 text-sm text-slate-200">
    {#if node}
      <section class="space-y-2">
        <div>
          <p class="text-xs uppercase tracking-wide text-slate-400">Entity</p>
          <p class="font-semibold text-slate-100">{node.entity}</p>
        </div>
        <div>
          <p class="text-xs uppercase tracking-wide text-slate-400">Label</p>
          <p>{node.label}</p>
        </div>
        <div>
          <p class="text-xs uppercase tracking-wide text-slate-400">Path</p>
          <p class="font-mono text-xs text-slate-300">{node.path.join(" / ")}</p>
        </div>
        {#if multiplicity}
          <div class="grid grid-cols-2 gap-2">
            <div>
              <p class="text-xs uppercase tracking-wide text-slate-400">Min</p>
              <p>{multiplicity.min ?? 0}</p>
            </div>
            <div>
              <p class="text-xs uppercase tracking-wide text-slate-400">Max</p>
              <p>{multiplicity.max ?? "No limit"}</p>
            </div>
          </div>
        {/if}
        {#if schema?.documentation}
          <div>
            <p class="text-xs uppercase tracking-wide text-slate-400">Documentation</p>
            <p class="text-slate-300">{schema.documentation}</p>
          </div>
        {/if}
      </section>
      <section class="mt-6 space-y-2">
        <div class="flex items-center justify-between">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-slate-400">Validation</h3>
          {#if validatedAt}
            <span class="text-[0.65rem] text-slate-500">{validatedAt.toLocaleTimeString()}</span>
          {/if}
        </div>
        {#if issues.length}
          <ul class="space-y-2">
            {#each issues as issue}
              <li class="rounded border px-2 py-2"
                class:border-rose-500={issue.severity === "error"}
                class:border-amber-400={issue.severity === "warning"}
              >
                <p class="text-xs font-semibold uppercase tracking-wide"
                  class:text-rose-400={issue.severity === "error"}
                  class:text-amber-300={issue.severity === "warning"}
                >
                  {issue.severity}
                </p>
                <p class="mt-1 text-xs text-slate-300">{issue.message}</p>
                <p class="mt-1 font-mono text-[0.65rem] text-slate-500">{issue.path}</p>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="text-xs text-slate-500">No validation issues recorded for this entity.</p>
        {/if}
      </section>
    {:else}
      <p class="text-xs text-slate-500">Select an entity to review its schema details.</p>
    {/if}
  </div>
</div>

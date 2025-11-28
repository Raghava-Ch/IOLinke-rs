<script lang="ts">
  // XML preview renders latest export value for quick inspection and copy.
  import { Button, Textarea } from "flowbite-svelte";

  export let value = "";
  export let lastUpdated: Date | null = null;

  let copied = false;

  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(value);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 1500);
    } catch (error) {
      console.error("Failed to copy XML", error);
    }
  }
</script>

<div class="flex h-full flex-col">
  <div class="flex items-center justify-between border-b border-slate-800 px-4 py-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-slate-300">XML Preview</h2>
    <div class="flex items-center gap-2 text-xs text-slate-400">
      {#if lastUpdated}
        <span>Updated {lastUpdated.toLocaleTimeString()}</span>
      {/if}
      <Button size="xs" color="dark" on:click={copyToClipboard}>
        {copied ? "Copied" : "Copy"}
      </Button>
    </div>
  </div>
  <div class="flex-1 overflow-hidden p-4">
    <Textarea
      class="h-full min-h-[16rem] resize-none text-xs font-mono text-slate-200"
      value={value}
      readonly
    />
  </div>
</div>

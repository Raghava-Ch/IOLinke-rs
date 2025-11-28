<script lang="ts">
  // Activity bar toggles high-level workspace sections defined in data/iodd_form_schema.json.
  import { activitySection } from "$lib/stores/ui";
  import { derived } from "svelte/store";

  type ActivitySection = "hierarchy" | "validation" | "preview";

  interface ActivityItem {
    id: ActivitySection;
    label: string;
    short: string;
  }

  const items: ActivityItem[] = [
    { id: "hierarchy", label: "Hierarchy", short: "H" },
    { id: "validation", label: "Validation", short: "V" },
    { id: "preview", label: "Preview", short: "P" }
  ];

  const activeSection = derived(activitySection, ($section) => $section ?? items[0].id);

  function selectSection(next: ActivitySection) {
    activitySection.set(next);
  }
</script>

<div class="flex h-full w-14 flex-col items-center gap-2 bg-slate-950 py-4">
  {#each items as item}
    {@const isActive = $activeSection === item.id}
    <button
      type="button"
      class="flex h-12 w-12 items-center justify-center rounded-md text-xs font-semibold transition focus:outline-none focus:ring-2 focus:ring-slate-400/60"
      class:bg-slate-700={isActive}
      class:text-white={isActive}
      class:text-slate-400={!isActive}
      class:hover:bg-slate-800={!isActive}
      on:click={() => selectSection(item.id)}
      aria-pressed={isActive}
      aria-label={item.label}
      title={item.label}
    >
      {item.short}
    </button>
  {/each}
</div>

<script lang="ts">
	// Main page composes hierarchy, editor, inspector, and preview panes powered by data/iodd_form_schema.json state.
	import ActivityBar from "../components/ActivityBar.svelte";
	import HierarchyTree from "../components/HierarchyTree.svelte";
	import EditorTabs from "../components/EditorTabs.svelte";
	import InspectorPanel from "../components/InspectorPanel.svelte";
	import Toolbar from "../components/Toolbar.svelte";
	import XMLPreview from "../components/XMLPreview.svelte";
	import ValidationPanel from "../components/ValidationPanel.svelte";
	import EntityForm from "../components/EntityForm.svelte";
	import { activitySection, selectedNode } from "$lib/stores/ui";
	import { documentStore } from "$lib/stores/documents";
	import { getValueAtPath, setValueAtPath, type StatePath } from "$lib/utils/statePath";
	import { exportToXml } from "$lib/utils/xml";
	import type { RootDocumentState } from "$lib/models/generated";

	let statePath: StatePath | null = null;
	let formValue: unknown = null;
	let resolvedValue: Record<string, unknown> = {};
	let formEntity: string | null = null;
	let xmlPreview = "";
	let xmlUpdatedAt: Date | null = null;

	$: docState = $documentStore;
	$: node = $selectedNode;
	$: statePath = (node?.statePath ?? null) as StatePath | null;
	$: formValue = statePath ? getValueAtPath(docState, statePath) : docState[node?.entity as keyof RootDocumentState] ?? null;
	$: resolvedValue = (formValue && typeof formValue === "object" ? formValue : ({} as Record<string, unknown>)) as Record<string, unknown>;
	$: formEntity = node?.entity ?? null;
	$: {
		xmlPreview = exportToXml(docState);
		xmlUpdatedAt = new Date();
	}

	function handleFormUpdate(event: CustomEvent<{ value: Record<string, unknown> }>): void {
		if (!statePath) return;
		const nextValue = event.detail.value;
		documentStore.update((state) => setValueAtPath(state, statePath, nextValue) as RootDocumentState);
	}

	function handleExport(): void {
		const xml = exportToXml(docState);
		const blob = new Blob([xml], { type: "application/xml" });
		const url = URL.createObjectURL(blob);
		const anchor = document.createElement("a");
		anchor.href = url;
		anchor.download = "iodd_export.xml";
		anchor.click();
		URL.revokeObjectURL(url);
	}
</script>

<div class="flex h-screen min-h-screen bg-slate-900 text-slate-100">
	<ActivityBar />
	<aside class="flex h-full w-72 flex-col border-r border-slate-800 bg-slate-950">
		{#if $activitySection === "hierarchy"}
			<HierarchyTree />
		{:else if $activitySection === "validation"}
			<ValidationPanel />
		{:else if $activitySection === "preview"}
			<XMLPreview value={xmlPreview} lastUpdated={xmlUpdatedAt} />
		{/if}
	</aside>
	<main class="flex h-full flex-1 flex-col">
		<Toolbar on:export={handleExport} />
		<EditorTabs />
		<div class="flex flex-1 overflow-hidden">
			<section class="flex-1 overflow-y-auto px-8 py-6">
				{#if formEntity}
					<EntityForm
						entityId={formEntity}
						value={resolvedValue}
						path={[]}
						rootValue={docState as unknown as Record<string, unknown>}
						validationPrefix={node?.entity ?? null}
						on:update={handleFormUpdate}
					/>
				{:else}
					<div class="flex h-full items-center justify-center">
						<p class="text-sm text-slate-500">Select an entity from the hierarchy to begin editing.</p>
					</div>
				{/if}
			</section>
			<aside class="flex w-96 flex-col border-l border-slate-800">
				<InspectorPanel />
			</aside>
		</div>
	</main>
</div>

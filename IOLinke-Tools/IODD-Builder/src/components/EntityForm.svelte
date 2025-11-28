<!-- Dynamic form renderer mapping schema-defined fields from data/iodd_form_schema.json (.temp.details/iodd_xsd_comprehensive.csv). -->
<script lang="ts">
  import { createEventDispatcher, onDestroy } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import {
    Button,
    Checkbox,
    Input,
    Label,
    Select,
    TabItem,
    Tabs,
    Textarea
  } from "flowbite-svelte";
  import { documentStore } from "$lib/stores/documents";
  import { getEntitySchema } from "$lib/schema/loader";
  import type {
    CollectionFieldSchema,
    EntityFieldSchema,
    EnumFieldSchema,
    FieldSchema,
    ObjectFieldSchema,
    ReferenceFieldSchema
  } from "$lib/schema/types";
  import {
    createDefaultValue,
    ensureCollectionArray,
    updateAtPath
  } from "$lib/utils/form";

  type UpdateDetail = { value: Record<string, unknown> };

  export let entityId: string;
  export let value: Record<string, unknown> = {};
  export let readOnly = false;
  export let path: string[] = [];
  export let fields: FieldSchema[] | null = null;
  export let rootValue: Record<string, unknown> | null = null;

  const dispatch = createEventDispatcher<{ update: UpdateDetail }>();
  let schema = getEntitySchema(entityId);

  $: schema = getEntitySchema(entityId);

  $: {
    if (!schema && !fields) {
      console.warn(`Missing schema for entity ${entityId}`);
    }
  }

  let currentDocument: Record<string, unknown> = {};
  const unsubscribe = documentStore.subscribe((state) => {
    currentDocument = state as unknown as Record<string, unknown>;
  });

  onDestroy(() => {
    unsubscribe();
  });

  let resolvedFields: FieldSchema[] = fields ?? schema?.fields ?? [];
  $: resolvedFields = fields ?? schema?.fields ?? [];

  let effectiveRoot: Record<string, unknown> = rootValue ?? value;
  $: effectiveRoot = (rootValue ?? value) as Record<string, unknown>;

  function commit(next: Record<string, unknown>): void {
    dispatch("update", { value: next });
  }

  function getFieldValue(field: FieldSchema): unknown {
    return value?.[field.id];
  }

  function formattedInputValue(field: FieldSchema): string {
    const raw = getFieldValue(field);
    if (raw === undefined || raw === null) return "";
    if (typeof raw === "number") return raw.toString();
    return String(raw);
  }

  function controlId(field: FieldSchema): string {
    return [...path, field.id].join("__");
  }

  function isLabelable(field: FieldSchema): boolean {
    return field.type !== "collection" && field.type !== "object" && field.type !== "entity";
  }

  function handlePrimitiveChange(field: FieldSchema, newValue: unknown): void {
    if (readOnly) return;
    let formatted: unknown = newValue;
    if (field.type === "integer" || field.type === "number") {
      const raw = typeof newValue === "string" ? newValue.trim() : newValue;
      if (raw === "") {
        formatted = null;
      } else {
        const numeric = Number(raw);
        formatted = Number.isNaN(numeric) ? null : numeric;
      }
    }
    const next = updateAtPath(value, [...path, field.id], formatted);
    commit(next);
  }

  function getValueByPath(target: Record<string, unknown>, fieldPath: string): unknown {
    const segments = fieldPath.split(".").filter(Boolean);
    let cursor: unknown = target;
    for (const segment of segments) {
      if (!cursor || typeof cursor !== "object") {
        return undefined;
      }
      cursor = (cursor as Record<string, unknown>)[segment];
    }
    return cursor;
  }

  function shouldRender(field: FieldSchema): boolean {
    const visibility = field.visibility;
    if (!visibility) return true;
    let candidate = getValueByPath(effectiveRoot, visibility.field);
    if (candidate === undefined) {
      candidate = getValueByPath(value, visibility.field);
    }
    if (visibility.equals !== undefined) {
      return candidate === visibility.equals;
    }
    if (visibility.notEquals !== undefined) {
      return candidate !== visibility.notEquals;
    }
    return true;
  }

  function renderHint(field: FieldSchema): string | null {
    if (field.help) return field.help;
    if (field.optionalReason) return field.optionalReason;
    if (field.checkerRules?.length) return field.checkerRules.join("; ");
    return null;
  }

  function resolveReferenceOptions(reference: ReferenceFieldSchema["reference"]): { label: string; value: string }[] {
    const pool = currentDocument[reference.entityId as keyof typeof currentDocument];
    if (!pool) return [];

    if (Array.isArray(pool)) {
      return pool
        .map((entry) => {
          if (!entry || typeof entry !== "object") return null;
          const record = entry as Record<string, unknown>;
          const label = record[reference.displayField];
          if (typeof label !== "string") return null;
          return { label, value: label };
        })
        .filter(Boolean) as { label: string; value: string }[];
    }

    if (typeof pool === "object") {
      const record = pool as Record<string, unknown>;
      const label = record[reference.displayField];
      if (typeof label === "string") {
        return [{ label, value: label }];
      }
    }

    return [];
  }

  function addCollectionItem(field: CollectionFieldSchema): void {
    if (readOnly) return;
    const items = ensureCollectionArray(field, getFieldValue(field));
    const nextItems = [...items, createDefaultValue(field.item)];
    const next = updateAtPath(value, [...path, field.id], nextItems);
    commit(next);
  }

  function removeCollectionItem(field: CollectionFieldSchema, index: number): void {
    if (readOnly) return;
    const items = ensureCollectionArray(field, getFieldValue(field)).filter((_, idx) => idx !== index);
    const next = updateAtPath(value, [...path, field.id], items);
    commit(next);
  }

  function reorderCollection(field: CollectionFieldSchema, items: { id: string; value: unknown }[]): void {
    if (readOnly) return;
    const reordered = items.map((entry) => entry.value);
    const next = updateAtPath(value, [...path, field.id], reordered);
    commit(next);
  }
</script>

{#if resolvedFields.length === 0}
  <div class="rounded border border-dashed border-slate-600 p-4 text-sm text-slate-400">
    No fields defined for {entityId}.
  </div>
{:else}
  <div class="space-y-6">
    {#each resolvedFields.filter((field) => shouldRender(field)) as field (field.id)}
      <div class="space-y-2">
        <div class="flex items-center justify-between gap-2">
          {#if isLabelable(field)}
            <Label for={controlId(field)} class="text-sm font-semibold text-slate-200">{field.label}</Label>
          {:else}
            <div class="text-sm font-semibold text-slate-200">{field.label}</div>
          {/if}
          {#if field.required}
            <span class="text-[10px] uppercase tracking-wide text-amber-400">Required</span>
          {/if}
        </div>

        {#if field.type === "string" || field.type === "integer" || field.type === "number" || field.type === "date" || field.type === "file"}
          <Input
            id={controlId(field)}
            name={controlId(field)}
            type={field.type === "date" ? "date" : field.type === "integer" || field.type === "number" ? "number" : "text"}
            value={formattedInputValue(field)}
            on:input={(event: Event) => {
              const target = event.currentTarget as HTMLInputElement | null;
              if (!target) return;
              handlePrimitiveChange(field, target.value);
            }}
            disabled={readOnly}
            placeholder={field.help ?? field.label}
          />
        {:else if field.type === "text"}
          <Textarea
            id={controlId(field)}
            name={controlId(field)}
            rows={4}
            value={formattedInputValue(field)}
            on:input={(event: Event) => {
              const target = event.currentTarget as HTMLTextAreaElement | null;
              if (!target) return;
              handlePrimitiveChange(field, target.value);
            }}
            disabled={readOnly}
          />
        {:else if field.type === "enum"}
          {@const enumField = field as EnumFieldSchema}
          <Select
            id={controlId(field)}
            name={controlId(field)}
            value={formattedInputValue(field)}
            on:change={(event: Event) => {
              const target = event.currentTarget as HTMLSelectElement | null;
              if (!target) return;
              handlePrimitiveChange(field, target.value);
            }}
            disabled={readOnly}
          >
            {#each enumField.options ?? [] as option}
              <option value={option.value}>{option.label ?? option.value}</option>
            {/each}
          </Select>
        {:else if field.type === "boolean"}
          <Checkbox
            id={controlId(field)}
            name={controlId(field)}
            checked={Boolean(getFieldValue(field))}
            on:change={(event: Event) => {
              const target = event.currentTarget as HTMLInputElement | null;
              if (!target) return;
              handlePrimitiveChange(field, target.checked);
            }}
            disabled={readOnly}
            >{field.help ?? field.label}</Checkbox
          >
        {:else if field.type === "reference"}
          <Select
            id={controlId(field)}
            name={controlId(field)}
            value={formattedInputValue(field)}
            on:change={(event: Event) => {
              const target = event.currentTarget as HTMLSelectElement | null;
              if (!target) return;
              handlePrimitiveChange(field, target.value);
            }}
            disabled={readOnly}
          >
            <option value="">Select value</option>
            {#each resolveReferenceOptions((field as ReferenceFieldSchema).reference) as option}
              <option value={option.value}>{option.label}</option>
            {/each}
          </Select>
        {:else if field.type === "collection"}
          {@const collectionField = field as CollectionFieldSchema}
          {@const collectionValue = ensureCollectionArray(collectionField, getFieldValue(field))}
          <div
            class="space-y-3 rounded border border-slate-700 bg-slate-900/40 p-3"
            use:dndzone={{
              items: collectionValue.map((item, index) => ({ id: `${field.id}-${index}`, value: item })),
              flipDurationMs: 150
            }}
            on:consider={(event: CustomEvent<{ items: { id: string; value: unknown }[] }>) =>
              reorderCollection(collectionField, event.detail.items)
            }
            on:finalize={(event: CustomEvent<{ items: { id: string; value: unknown }[] }>) =>
              reorderCollection(collectionField, event.detail.items)
            }
          >
            {#if collectionValue.length === 0}
              <div class="rounded border border-dashed border-slate-700 p-3 text-xs text-slate-400">
                No {field.label} entries yet.
              </div>
            {/if}

            {#each collectionValue as item, index}
              <div class="rounded border border-slate-700 bg-slate-800/60 p-3 shadow-sm">
                <div class="mb-2 flex items-center justify-between text-xs uppercase tracking-wide text-slate-400">
                  <span>Item {index + 1}</span>
                  <Button
                    size="xs"
                    color="red"
                    type="button"
                    title="Remove entry"
                    aria-label="Remove entry"
                    on:click={() => removeCollectionItem(collectionField, index)}
                    disabled={readOnly}
                  >Remove</Button>
                </div>

                {#if collectionField.item.type === "entity"}
                  <svelte:self
                    entityId={(collectionField.item as EntityFieldSchema).entity}
                    value={(item as Record<string, unknown>) ?? {}}
                    on:update={(event: CustomEvent<UpdateDetail>) => {
                      const cloneCollection = [...collectionValue];
                      cloneCollection[index] = event.detail.value;
                      const next = updateAtPath(value, [...path, field.id], cloneCollection);
                      commit(next);
                    }}
                    path={[...path, field.id, String(index)]}
                    readOnly={readOnly}
                    rootValue={null}
                  />
                {:else if collectionField.item.type === "object"}
                  {@const sectionedItem = collectionField.item as ObjectFieldSchema}
                  {#if sectionedItem.sections?.length}
                    <Tabs>
                      {#each sectionedItem.sections.filter((section) => shouldRender(section)) as section (section.id)}
                        {@const nestedSection = section as ObjectFieldSchema}
                        <TabItem title={section.label}>
                          <svelte:self
                            entityId={entityId}
                            value={(item as Record<string, unknown>) ?? {}}
                            fields={nestedSection.fields ?? []}
                            on:update={(event: CustomEvent<UpdateDetail>) => {
                              const cloneCollection = [...collectionValue];
                              cloneCollection[index] = event.detail.value;
                              const next = updateAtPath(value, [...path, field.id], cloneCollection);
                              commit(next);
                            }}
                            path={[...path, field.id, String(index)]}
                            readOnly={readOnly}
                            rootValue={effectiveRoot}
                          />
                        </TabItem>
                      {/each}
                    </Tabs>
                  {:else}
                    <svelte:self
                      entityId={entityId}
                      value={(item as Record<string, unknown>) ?? {}}
                      fields={sectionedItem.fields ?? []}
                      on:update={(event: CustomEvent<UpdateDetail>) => {
                        const cloneCollection = [...collectionValue];
                        cloneCollection[index] = event.detail.value;
                        const next = updateAtPath(value, [...path, field.id], cloneCollection);
                        commit(next);
                      }}
                      path={[...path, field.id, String(index)]}
                      readOnly={readOnly}
                      rootValue={effectiveRoot}
                    />
                  {/if}
                {:else}
                  <Textarea
                    rows={2}
                    value={String(item ?? "")}
                    on:input={(event: Event) => {
                      const target = event.currentTarget as HTMLTextAreaElement | null;
                      if (!target) return;
                      const cloneCollection = [...collectionValue];
                      cloneCollection[index] = target.value;
                      const next = updateAtPath(value, [...path, field.id], cloneCollection);
                      commit(next);
                    }}
                    disabled={readOnly}
                  />
                {/if}
              </div>
            {/each}
          </div>
          <Button size="sm" type="button" on:click={() => addCollectionItem(collectionField)} disabled={readOnly}>
            Add {field.label}
          </Button>
        {:else if field.type === "object"}
          {@const objectField = field as ObjectFieldSchema}
          {@const objectValue = (getFieldValue(field) as Record<string, unknown>) ?? {}}
          {#if objectField.sections?.length}
            <Tabs>
              {#each objectField.sections.filter((section) => shouldRender(section)) as section (section.id)}
                {@const sectionField = section as ObjectFieldSchema}
                <TabItem title={section.label}>
                  <svelte:self
                    entityId={entityId}
                    value={objectValue}
                    fields={sectionField.fields ?? []}
                    on:update={(event: CustomEvent<UpdateDetail>) => {
                      const next = updateAtPath(value, [...path, field.id], event.detail.value);
                      commit(next);
                    }}
                    path={[...path, field.id]}
                    readOnly={readOnly}
                    rootValue={effectiveRoot}
                  />
                </TabItem>
              {/each}
            </Tabs>
          {:else}
            <svelte:self
              entityId={entityId}
              value={objectValue}
              fields={objectField.fields ?? []}
              on:update={(event: CustomEvent<UpdateDetail>) => {
                const next = updateAtPath(value, [...path, field.id], event.detail.value);
                commit(next);
              }}
              path={[...path, field.id]}
              readOnly={readOnly}
              rootValue={effectiveRoot}
            />
          {/if}
        {:else if field.type === "entity"}
          {@const entityField = field as EntityFieldSchema}
          <div class="rounded border border-slate-700 bg-slate-900/40 p-3">
            <svelte:self
              entityId={entityField.entity}
              value={(getFieldValue(field) as Record<string, unknown>) ?? {}}
              on:update={(event: CustomEvent<UpdateDetail>) => {
                const next = updateAtPath(value, [...path, field.id], event.detail.value);
                commit(next);
              }}
              path={[...path, field.id]}
              readOnly={readOnly}
              rootValue={null}
            />
          </div>
        {/if}

        {#if renderHint(field)}
          <p class="text-xs text-slate-400">{renderHint(field)}</p>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  :global(.dndDraggingSource) {
    opacity: 0.5;
  }
  :global(.dndPlaceholder) {
    border: 1px dashed rgba(148, 163, 184, 0.5);
    min-height: 1.5rem;
  }
</style>

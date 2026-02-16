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
  import { validateEntity } from "$lib/validators/rules";
  import type { ValidationIssue } from "$lib/validators/rules";
  import type { RootDocumentState } from "$lib/models/generated";

  type UpdateDetail = { value: Record<string, unknown> };

  export let entityId: string;
  export let value: Record<string, unknown> = {};
  export let readOnly = false;
  export let path: string[] = [];
  export let fields: FieldSchema[] | null = null;
  export let rootValue: Record<string, unknown> | null = null;
  export let validationPrefix: string | null = null;

  const dispatch = createEventDispatcher<{ update: UpdateDetail }>();
  let schema = getEntitySchema(entityId);

  let formState: Record<string, unknown> = value;

  $: if (value !== formState) {
    formState = value;
  }

  $: schema = getEntitySchema(entityId);

  $: {
    if (!schema && !fields) {
      console.warn(`Missing schema for entity ${entityId}`);
    }
  }

  let currentDocument: RootDocumentState | null = null;
  const unsubscribe = documentStore.subscribe((state) => {
    currentDocument = state;
  });

  onDestroy(() => {
    unsubscribe();
  });

  let resolvedFields: FieldSchema[] = fields ?? schema?.fields ?? [];
  $: resolvedFields = fields ?? schema?.fields ?? [];

  let effectiveRoot: Record<string, unknown> = rootValue ?? formState;
  $: effectiveRoot = (rootValue ?? formState) as Record<string, unknown>;

  let validationBase = validationPrefix ?? entityId;
  $: validationBase = validationPrefix ?? entityId;

  let collectionSnapshots: Map<string, unknown[]> = new Map();
  $: {
    formState;
    const entries: [string, unknown[]][] = [];
    resolvedFields.forEach((field) => {
      if (field.type !== "collection") return;
      const collectionField = field as CollectionFieldSchema;
      entries.push([field.id, ensureCollectionArray(collectionField, getFieldValue(field))]);
    });
    collectionSnapshots = new Map(entries);
  }

  let fieldIssues: Map<string, ValidationIssue[]> = new Map();
  $: {
    if (!schema || !currentDocument) {
      fieldIssues = new Map();
    } else {
      const rootState = currentDocument;
      const issues = validateEntity(entityId, formState, rootState, [], validationBase);
      const grouped = new Map<string, ValidationIssue[]>();
      issues.forEach((issue) => {
        const prefix = validationBase ? `${validationBase}.` : "";
        const relative = issue.path.startsWith(prefix) ? issue.path.slice(prefix.length) : issue.path;
        const key = relative.split(".")[0]?.split("[")[0] ?? relative;
        const bucket = grouped.get(key) ?? [];
        bucket.push(issue);
        grouped.set(key, bucket);
      });
      fieldIssues = grouped;
    }
  }

  let entityRecords: Map<string, Record<string, unknown>[]> = new Map();
  $: {
    if (!currentDocument) {
      entityRecords = new Map();
    } else {
      entityRecords = buildEntityIndex(currentDocument);
    }
  }

  function commit(next: Record<string, unknown>): void {
    formState = next;
    dispatch("update", { value: next });
  }

  function getFieldValue(field: FieldSchema): unknown {
    return formState?.[field.id];
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
    const next = updateAtPath(formState, [field.id], formatted);
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

  function fieldClass(hasError: boolean, hasWarning: boolean): string {
    if (hasError) {
      return "border-rose-500 focus:border-rose-500 focus:ring-rose-500";
    }
    if (hasWarning) {
      return "border-amber-300 focus:border-amber-300 focus:ring-amber-300";
    }
    return "";
  }

  function checkboxClass(hasError: boolean, hasWarning: boolean): string {
    if (hasError) return "text-rose-300";
    if (hasWarning) return "text-amber-300";
    return "";
  }

  function resolveReferenceOptions(reference: ReferenceFieldSchema["reference"]): { label: string; value: string }[] {
    if (!currentDocument) return [];
    const candidates = entityRecords.get(reference.entityId) ?? [];
    const seen = new Set<string>();

    return candidates
      .filter((entry) => {
        if (!reference.filterField) return true;
        const filterValue = (entry as Record<string, unknown>)[reference.filterField];
        if (reference.filterValue !== undefined) {
          return filterValue === reference.filterValue;
        }
        return Boolean(filterValue);
      })
      .map((entry) => {
        const raw = (entry as Record<string, unknown>)[reference.displayField];
        if (raw === undefined || raw === null) return null;
        const label = String(raw);
        if (seen.has(label)) return null;
        seen.add(label);
        return { label, value: label };
      })
      .filter((option): option is { label: string; value: string } => option !== null)
      .sort((a, b) => a.label.localeCompare(b.label, undefined, { sensitivity: "base" }));
  }

  function addCollectionItem(field: CollectionFieldSchema): void {
    if (readOnly) return;
    const items = getCollectionItems(field);
    const defaultItem = createDefaultValue(field.item);

    if (field.uniqueBy && field.item.type === "object" && defaultItem && typeof defaultItem === "object") {
      const objectField = field.item as ObjectFieldSchema;
      const uniqueField = field.uniqueBy;
      const existingValues = new Set(
        items
          .map((entry) =>
            entry && typeof entry === "object"
              ? (entry as Record<string, unknown>)[uniqueField]
              : undefined
          )
          .filter((value): value is string => typeof value === "string" && value.trim().length > 0)
      );

      const defaultsRecord = defaultItem as Record<string, unknown>;
      const currentValue = defaultsRecord[uniqueField];

      if (typeof currentValue !== "string" || existingValues.has(currentValue)) {
        const nestedFields = [...(objectField.fields ?? []), ...(objectField.sections ?? [])];
        const uniqueFieldSchema = nestedFields.find((nested) => nested.id === uniqueField);
        if (uniqueFieldSchema?.type === "enum") {
          const enumField = uniqueFieldSchema as EnumFieldSchema;
          const candidate = enumField.options.find((option) => !existingValues.has(option.value));
          if (candidate) {
            defaultsRecord[uniqueField] = candidate.value;
          }
        }
      }
    }

    const appended = [...items, defaultItem];
    const next = updateAtPath(formState, [field.id], appended);
    commit(next);
  }

  function removeCollectionItem(field: CollectionFieldSchema, index: number): void {
    if (readOnly) return;
    const items = ensureCollectionArray(field, getFieldValue(field)).filter((_, idx) => idx !== index);
    const next = updateAtPath(formState, [field.id], items);
    commit(next);
  }

  function buildEntityIndex(state: RootDocumentState): Map<string, Record<string, unknown>[]> {
    const map = new Map<string, Record<string, unknown>[]>();

    function visit(entity: string, data: unknown): void {
      if (Array.isArray(data)) {
        data.forEach((item) => visit(entity, item));
        return;
      }

      if (!data || typeof data !== "object") {
        return;
      }

      const record = data as Record<string, unknown>;
      const bucket = map.get(entity) ?? [];
      bucket.push(record);
      map.set(entity, bucket);

      const entitySchema = getEntitySchema(entity);
      if (!entitySchema) {
        return;
      }

      entitySchema.fields.forEach((field) => {
        visitField(field, record[field.id]);
      });
    }

    function visitField(field: FieldSchema, rawValue: unknown): void {
      switch (field.type) {
        case "entity": {
          const entityField = field as EntityFieldSchema;
          visit(entityField.entity, rawValue);
          break;
        }
        case "collection": {
          const collectionField = field as CollectionFieldSchema;
          const items = ensureCollectionArray(collectionField, rawValue);
          (items as unknown[]).forEach((item) => {
            if (collectionField.item.type === "entity") {
              const nestedEntity = (collectionField.item as EntityFieldSchema).entity;
              visit(nestedEntity, item);
            } else {
              visitField(collectionField.item, item as Record<string, unknown>);
            }
          });
          break;
        }
        case "object": {
          const objectField = field as ObjectFieldSchema;
          const record = (rawValue ?? {}) as Record<string, unknown>;
          const nestedFields = [...(objectField.fields ?? []), ...(objectField.sections ?? [])];
          nestedFields.forEach((nestedField) => {
            visitField(nestedField, record[nestedField.id]);
          });
          break;
        }
        default:
          break;
      }
    }

    Object.entries(state).forEach(([entity, value]) => {
      visit(entity, value);
    });

    return map;
  }

  function reorderCollection(field: CollectionFieldSchema, items: { id: string; value: unknown }[]): void {
    if (readOnly) return;
    const reordered = items.map((entry) => entry.value);
    const next = updateAtPath(formState, [field.id], reordered);
    commit(next);
  }

  function getCollectionItems(field: CollectionFieldSchema): unknown[] {
    return ensureCollectionArray(field, getFieldValue(field));
  }
</script>

{#if resolvedFields.length === 0}
  <div class="rounded border border-dashed border-slate-600 p-4 text-sm text-slate-400">
    No fields defined for {entityId}.
  </div>
{:else}
  <div class="space-y-6">
    {#each resolvedFields.filter((field) => shouldRender(field)) as field (field.id)}
      {@const issues = fieldIssues.get(field.id) ?? []}
      {@const hasError = issues.some((issue) => issue.severity === "error")}
      {@const hasWarning = !hasError && issues.some((issue) => issue.severity === "warning")}
      <div class="space-y-2">
        <div class="flex items-center justify-between gap-2">
          {#if isLabelable(field)}
            <Label
              for={controlId(field)}
              class={`text-sm font-semibold ${hasError ? "text-rose-300" : "text-slate-200"}`}
            >
              {field.label}
            </Label>
          {:else}
            <div
              class="text-sm font-semibold"
              class:text-slate-200={!hasError}
              class:text-rose-300={hasError}
            >
              {field.label}
            </div>
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
            class={fieldClass(hasError, hasWarning)}
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
            class={fieldClass(hasError, hasWarning)}
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
            class={fieldClass(hasError, hasWarning)}
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
            class={checkboxClass(hasError, hasWarning)}
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
            class={fieldClass(hasError, hasWarning)}
          >
            <option value="">Select value</option>
            {#each resolveReferenceOptions((field as ReferenceFieldSchema).reference) as option}
              <option value={option.value}>{option.label}</option>
            {/each}
          </Select>
        {:else if field.type === "collection"}
          {@const collectionField = field as CollectionFieldSchema}
          <div
            class="space-y-3 rounded border border-slate-700 bg-slate-900/40 p-3"
            use:dndzone={{
              items: (collectionSnapshots.get(field.id) ?? []).map((item, index) => ({ id: `${field.id}-${index}`, value: item })),
              flipDurationMs: 150
            }}
            on:consider={(event: CustomEvent<{ items: { id: string; value: unknown }[] }>) =>
              reorderCollection(collectionField, event.detail.items)
            }
            on:finalize={(event: CustomEvent<{ items: { id: string; value: unknown }[] }>) =>
              reorderCollection(collectionField, event.detail.items)
            }
          >
            {#if (collectionSnapshots.get(field.id) ?? []).length === 0}
              <div class="rounded border border-dashed border-slate-700 p-3 text-xs text-slate-400">
                No {field.label} entries yet.
              </div>
            {/if}

            {#each collectionSnapshots.get(field.id) ?? [] as item, index}
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
                      const items = getCollectionItems(collectionField);
                      const cloneCollection = [...items];
                      cloneCollection[index] = event.detail.value;
                      const next = updateAtPath(formState, [field.id], cloneCollection);
                      commit(next);
                    }}
                    path={[...path, field.id, String(index)]}
                    readOnly={readOnly}
                    rootValue={null}
                    validationPrefix={`${validationBase}.${field.id}[${index}]`}
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
                              const items = getCollectionItems(collectionField);
                              const cloneCollection = [...items];
                              cloneCollection[index] = event.detail.value;
                              const next = updateAtPath(formState, [field.id], cloneCollection);
                              commit(next);
                            }}
                            path={[...path, field.id, String(index)]}
                            readOnly={readOnly}
                            rootValue={effectiveRoot}
                            validationPrefix={`${validationBase}.${field.id}[${index}]`}
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
                        const items = getCollectionItems(collectionField);
                        const cloneCollection = [...items];
                        cloneCollection[index] = event.detail.value;
                        const next = updateAtPath(formState, [field.id], cloneCollection);
                        commit(next);
                      }}
                      path={[...path, field.id, String(index)]}
                      readOnly={readOnly}
                      rootValue={effectiveRoot}
                      validationPrefix={`${validationBase}.${field.id}[${index}]`}
                    />
                  {/if}
                {:else}
                  <Textarea
                    rows={2}
                    value={String(item ?? "")}
                    on:input={(event: Event) => {
                      const target = event.currentTarget as HTMLTextAreaElement | null;
                      if (!target) return;
                      const items = getCollectionItems(collectionField);
                      const cloneCollection = [...items];
                      cloneCollection[index] = target.value;
                      const next = updateAtPath(formState, [field.id], cloneCollection);
                      commit(next);
                    }}
                    disabled={readOnly}
                    class={fieldClass(hasError, hasWarning)}
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
                      const next = updateAtPath(formState, [field.id], event.detail.value);
                      commit(next);
                    }}
                    path={[...path, field.id]}
                    readOnly={readOnly}
                    rootValue={effectiveRoot}
                    validationPrefix={`${validationBase}.${field.id}`}
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
                const next = updateAtPath(formState, [field.id], event.detail.value);
                commit(next);
              }}
              path={[...path, field.id]}
              readOnly={readOnly}
              rootValue={effectiveRoot}
              validationPrefix={`${validationBase}.${field.id}`}
            />
          {/if}
        {:else if field.type === "entity"}
          {@const entityField = field as EntityFieldSchema}
          <div class="rounded border border-slate-700 bg-slate-900/40 p-3">
            <svelte:self
              entityId={entityField.entity}
              value={(getFieldValue(field) as Record<string, unknown>) ?? {}}
              on:update={(event: CustomEvent<UpdateDetail>) => {
                const next = updateAtPath(formState, [field.id], event.detail.value);
                commit(next);
              }}
              path={[...path, field.id]}
              readOnly={readOnly}
              rootValue={null}
              validationPrefix={`${validationBase}.${field.id}`}
            />
          </div>
        {/if}

        {#if issues.length}
          <ul class="space-y-1 text-xs">
            {#each issues as issue}
              <li class={issue.severity === "error" ? "text-rose-400" : "text-amber-300"}>
                {issue.message}
                {#if issue.path && issue.path !== `${validationBase}.${field.id}`}
                  <span class="ml-1 font-mono text-[0.6rem] text-slate-500">({issue.path})</span>
                {/if}
              </li>
            {/each}
          </ul>
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

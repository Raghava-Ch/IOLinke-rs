// Hierarchy store materializes schema-defined structure (data/iodd_form_schema.json + .temp.details/iodd_xsd_comprehensive.csv) for the tree view.
// Produces runtime nodes with resolved multiplicities and data-driven labels.

import { derived } from "svelte/store";
import type { RootDocumentState } from "$lib/models/generated";
import { documentStore } from "$lib/stores/documents";
import { getEntitySchema, getHierarchy } from "$lib/schema/loader";
import type { EntityFieldSchema, FieldSchema, HierarchyNode } from "$lib/schema/types";

export interface HierarchyInstanceNode {
  id: string;
  entity: string;
  label: string;
  path: string[];
  index?: number;
  multiplicity?: HierarchyNode["multiplicity"];
  count?: number;
  statePath: (string | number)[];
  children: HierarchyInstanceNode[];
}

const DISPLAY_FIELD_CANDIDATES = [
  "title",
  "name",
  "textId",
  "variantId",
  "datatypeId",
  "variableId",
  "processDataId",
  "menuId",
  "documentId"
];

function guessDisplayValue(entityId: string, data: unknown): string | null {
  if (!data || typeof data !== "object") {
    return null;
  }

  const schema = getEntitySchema(entityId);
  if (!schema) {
    return null;
  }

  const record = data as Record<string, unknown>;

  for (const candidate of DISPLAY_FIELD_CANDIDATES) {
    const raw = record[candidate];
    if (typeof raw === "string" && raw.trim().length > 0) {
      return raw;
    }
  }

  const defaultField = schema.fields.find((field) => field.type === "string" || field.type === "enum");
  if (defaultField) {
    const fallback = record[defaultField.id];
    if (typeof fallback === "string" && fallback.trim().length > 0) {
      return fallback;
    }
  }

  return null;
}

function findLinkField(parentEntity: string, childEntity: string): { field: FieldSchema; kind: "collection" | "entity" } | null {
  const schema = getEntitySchema(parentEntity);
  if (!schema) return null;

  for (const field of schema.fields) {
    if (field.type === "collection" && "item" in field) {
      const item = field.item;
      if (item.type === "entity") {
        const entityField = item as EntityFieldSchema;
        if (entityField.entity === childEntity) {
          return { field, kind: "collection" };
        }
      }
    }

    if (field.type === "entity") {
      const entityField = field as EntityFieldSchema;
      if (entityField.entity === childEntity) {
        return { field, kind: "entity" };
      }
    }
  }

  return null;
}

function createInstanceNode(
  definition: HierarchyNode,
  entityData: unknown,
  rootState: RootDocumentState,
  path: string[],
  statePath: (string | number)[],
  index?: number
): HierarchyInstanceNode {
  const displayValue = guessDisplayValue(definition.entity, entityData);
  const label = displayValue ? `${definition.label} - ${displayValue}` : definition.label;
  const children: HierarchyInstanceNode[] = [];

  if (definition.children?.length) {
    for (const child of definition.children) {
      const linkField = findLinkField(definition.entity, child.entity);
      let childData: unknown = undefined;
      let childStatePath: (string | number)[] | null = null;
      const baseStatePath = linkField ? [...statePath, linkField.field.id] : [child.entity];

      if (linkField && entityData && typeof entityData === "object") {
        childData = (entityData as Record<string, unknown>)[linkField.field.id];
      }

      if (Array.isArray(childData)) {
        childData.forEach((item, childIndex) => {
          const childPath = [...path, `${child.id}[${childIndex}]`];
          childStatePath = [...baseStatePath, childIndex];
          children.push(createInstanceNode(child, item, rootState, childPath, childStatePath, childIndex));
        });
        continue;
      }

      if (childData) {
        childStatePath = baseStatePath;
        children.push(createInstanceNode(child, childData, rootState, [...path, child.id], childStatePath));
        continue;
      }

      const fallback = rootState[child.entity as keyof RootDocumentState];
      if (fallback !== undefined) {
        if (Array.isArray(fallback)) {
          fallback.forEach((item, fallbackIndex) => {
            const childPath = [...path, `${child.id}[${fallbackIndex}]`];
            childStatePath = linkField ? [...baseStatePath, fallbackIndex] : [child.entity, fallbackIndex];
            children.push(createInstanceNode(child, item, rootState, childPath, childStatePath, fallbackIndex));
          });
        } else {
          childStatePath = baseStatePath;
          children.push(createInstanceNode(child, fallback, rootState, [...path, child.id], childStatePath));
        }
      } else {
        childStatePath = baseStatePath;
        children.push(createInstanceNode(child, undefined, rootState, [...path, child.id], childStatePath));
      }
    }
  }

  const nodeId = path.join("/");
  const count = children.length;

  return {
    id: index !== undefined ? `${nodeId}#${index}` : nodeId,
    entity: definition.entity,
    label,
    path,
    index,
    multiplicity: definition.multiplicity,
    count,
    statePath,
    children,
  };
}

function buildHierarchy(state: RootDocumentState): HierarchyInstanceNode[] {
  const base = getHierarchy();
  const nodes: HierarchyInstanceNode[] = [];

  for (const node of base) {
    const entityData = state[node.entity as keyof RootDocumentState];
    const rootPath = [node.id];
    const statePath: (string | number)[] = [node.entity];
    nodes.push(createInstanceNode(node, entityData, state, rootPath, statePath));
  }

  return nodes;
}

export const hierarchyStore = derived(documentStore, (state) => buildHierarchy(state));

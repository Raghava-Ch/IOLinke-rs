// Runtime loader for data/iodd_form_schema.json backed by .temp.details/iodd_xsd_comprehensive.csv metadata.
// Central entry point for schema consumers (stores, forms, validation).

import schemaSource from "../../../data/iodd_form_schema.json?raw";
import type {
  CrossReferencePolicy,
  EntitySchema,
  FormSchemaDocument,
  HierarchyNode
} from "./types";

let cachedSchema: FormSchemaDocument | null = null;

function parseSchema(): FormSchemaDocument {
  if (cachedSchema) {
    return cachedSchema;
  }

  try {
    const parsed = JSON.parse(schemaSource) as FormSchemaDocument;
    cachedSchema = parsed;
    return parsed;
  } catch (error) {
    console.error("Failed to parse iodd_form_schema.json", error);
    throw error;
  }
}

export function loadSchema(): FormSchemaDocument {
  return parseSchema();
}

export function getHierarchy(): HierarchyNode[] {
  return parseSchema().hierarchy;
}

export function getEntitySchema(entityId: string): EntitySchema | undefined {
  const schema = parseSchema();
  return schema.entities[entityId];
}

export function getCrossReferencePolicies(): CrossReferencePolicy[] {
  const schema = parseSchema();
  return schema.crossReferencePolicies ?? [];
}

export function listEntities(): EntitySchema[] {
  const schema = parseSchema();
  return Object.values(schema.entities);
}

export function findHierarchyNode(entityId: string): HierarchyNode | undefined {
  const nodes = getHierarchy();
  const queue: HierarchyNode[] = [...nodes];

  while (queue.length) {
    const node = queue.shift();
    if (!node) continue;
    if (node.entity === entityId) {
      return node;
    }
    if (node.children?.length) {
      queue.push(...node.children);
    }
  }

  return undefined;
}

export function flattenHierarchy(): HierarchyNode[] {
  const result: HierarchyNode[] = [];
  const queue: HierarchyNode[] = [...getHierarchy()];

  while (queue.length) {
    const node = queue.shift();
    if (!node) continue;
    result.push(node);
    if (node.children?.length) {
      queue.push(...node.children);
    }
  }

  return result;
}

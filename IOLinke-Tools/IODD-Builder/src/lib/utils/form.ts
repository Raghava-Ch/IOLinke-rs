// Form utilities mapping schema fields to default values (data/iodd_form_schema.json, .temp.details/iodd_xsd_comprehensive.csv).
// Shared helpers for EntityForm rendering and nested components.

import { getEntitySchema } from "$lib/schema/loader";
import type {
  CollectionFieldSchema,
  EntityFieldSchema,
  EnumFieldSchema,
  FieldSchema,
  ObjectFieldSchema
} from "$lib/schema/types";

export function clone<T>(value: T): T {
  if (typeof structuredClone === "function") {
    return structuredClone(value);
  }
  return JSON.parse(JSON.stringify(value)) as T;
}

export function updateAtPath<T extends Record<string, unknown>>(source: T, path: string[], newValue: unknown): T {
  const result = clone(source);
  let cursor: Record<string, unknown> = result;
  path.forEach((segment, index) => {
    if (index === path.length - 1) {
      cursor[segment] = newValue as never;
      return;
    }
    if (typeof cursor[segment] !== "object" || cursor[segment] === null) {
      cursor[segment] = {} as never;
    }
    cursor = cursor[segment] as Record<string, unknown>;
  });
  return result;
}

export function createDefaultValue(field: FieldSchema, seen = new Set<string>()): unknown {
  switch (field.type) {
    case "string":
    case "text":
      return typeof field.default === "string" ? field.default : "";
    case "integer":
    case "number":
      if (typeof field.default === "number") return field.default;
      if (field.minInclusive !== undefined) return field.minInclusive;
      return 0;
    case "boolean":
      return typeof field.default === "boolean" ? field.default : false;
    case "enum": {
      const enumField = field as EnumFieldSchema;
      return enumField.default ?? enumField.options?.[0]?.value ?? "";
    }
    case "date":
      if (typeof field.default === "string") return field.default;
      return new Date().toISOString().slice(0, 10);
    case "file":
      return field.default ?? "";
    case "reference":
      return field.default ?? "";
    case "collection":
      return [];
    case "object": {
      const objectField = field as ObjectFieldSchema;
      const target: Record<string, unknown> = {};
      const nested = [...(objectField.fields ?? []), ...(objectField.sections ?? [])];
      nested.forEach((nestedField) => {
        target[nestedField.id] = createDefaultValue(nestedField, seen);
      });
      return target;
    }
    case "entity": {
      const entityField = field as EntityFieldSchema;
      if (seen.has(entityField.entity)) {
        return {};
      }
      seen.add(entityField.entity);
      const schema = getEntitySchema(entityField.entity);
      const target: Record<string, unknown> = {};
      if (schema) {
        schema.fields.forEach((nestedField) => {
          target[nestedField.id] = createDefaultValue(nestedField, seen);
        });
      }
      seen.delete(entityField.entity);
      return target;
    }
    default:
      return null;
  }
}

export function ensureCollectionArray(field: CollectionFieldSchema, value: unknown): unknown[] {
  if (!Array.isArray(value)) return [];
  return value;
}

// Validation helpers derived from data/iodd_form_schema.json (.temp.details/iodd_xsd_comprehensive.csv) enforcing schema + checker rules.
// Provides field-level and document-level validation routines consumed by stores and UI.

import { getCrossReferencePolicies, getEntitySchema } from "$lib/schema/loader";
import type {
  CollectionFieldSchema,
  EntityFieldSchema,
  FieldSchema,
  FormSchemaDocument,
  ObjectFieldSchema,
} from "$lib/schema/types";
import type { RootDocumentState } from "$lib/models/generated";
import schemaSource from "../../../data/iodd_form_schema.json?raw";

export type ValidationSeverity = "error" | "warning";

export interface ValidationIssue {
  entity: string;
  path: string;
  message: string;
  severity: ValidationSeverity;
}

const schemaDocument = JSON.parse(schemaSource) as FormSchemaDocument;

function pushIssue(
  issues: ValidationIssue[],
  severity: ValidationSeverity,
  entity: string,
  path: string,
  message: string
): void {
  issues.push({ severity, entity, path, message });
}

function requireValue(field: FieldSchema, value: unknown): boolean {
  if (!field.required) return true;
  if (value === null || value === undefined) return false;
  if (typeof value === "string" && value.trim().length === 0) return false;
  if (Array.isArray(value) && value.length === 0) return false;
  return true;
}

function validatePrimitive(
  field: FieldSchema,
  value: unknown,
  entity: string,
  path: string,
  issues: ValidationIssue[]
): void {
  if (!requireValue(field, value)) {
    pushIssue(issues, "error", entity, path, `${field.label} is required.`);
    return;
  }

  if (value === null || value === undefined) {
    return;
  }

  if (field.type === "string" || field.type === "text" || field.type === "enum") {
    const text = String(value);
    if (field.minLength && text.length < field.minLength) {
      pushIssue(issues, "error", entity, path, `${field.label} must be at least ${field.minLength} characters.`);
    }
    if (field.maxLength && text.length > field.maxLength) {
      pushIssue(issues, "error", entity, path, `${field.label} must be at most ${field.maxLength} characters.`);
    }
    if (field.pattern) {
      const regex = new RegExp(`^${field.pattern}$`);
      if (!regex.test(text)) {
        pushIssue(issues, "error", entity, path, `${field.label} does not match required pattern ${field.pattern}.`);
      }
    }
    if (field.type === "enum" && "options" in field) {
      const validValues = field.options.map((option) => option.value);
      if (!validValues.includes(text)) {
        pushIssue(issues, "error", entity, path, `${field.label} must be one of: ${validValues.join(", " )}`);
      }
    }
  }

  if (field.type === "integer" || field.type === "number") {
    const numeric = Number(value);
    if (Number.isNaN(numeric)) {
      pushIssue(issues, "error", entity, path, `${field.label} must be numeric.`);
    }
    if (field.minInclusive !== undefined && numeric < field.minInclusive) {
      pushIssue(issues, "error", entity, path, `${field.label} must be ≥ ${field.minInclusive}.`);
    }
    if (field.maxInclusive !== undefined && numeric > field.maxInclusive) {
      pushIssue(issues, "error", entity, path, `${field.label} must be ≤ ${field.maxInclusive}.`);
    }
  }
}

function validateCollection(
  field: CollectionFieldSchema,
  value: unknown,
  entity: string,
  path: string,
  rootState: RootDocumentState,
  issues: ValidationIssue[]
): void {
  const items = Array.isArray(value) ? value : [];
  if (field.minOccurs !== undefined && items.length < field.minOccurs) {
    pushIssue(issues, "error", entity, path, `${field.label} requires at least ${field.minOccurs} entries.`);
  }
  if (field.maxOccurs !== null && field.maxOccurs !== undefined && items.length > field.maxOccurs) {
    pushIssue(issues, "error", entity, path, `${field.label} exceeds maximum of ${field.maxOccurs} entries.`);
  }

  items.forEach((item, index) => {
    const itemPath = `${path}[${index}]`;
    validateField(field.item, item, entity, itemPath, rootState, issues);
  });
}

function validateEntityField(
  field: EntityFieldSchema,
  value: unknown,
  entity: string,
  path: string,
  rootState: RootDocumentState,
  issues: ValidationIssue[]
): void {
  if (!requireValue(field, value)) {
    pushIssue(issues, "error", entity, path, `${field.label} is required.`);
    return;
  }
  if (value === null || value === undefined) return;
  validateEntity(field.entity, value, rootState, issues, path);
}

function validateField(
  field: FieldSchema,
  value: unknown,
  entity: string,
  path: string,
  rootState: RootDocumentState,
  issues: ValidationIssue[]
): void {
  switch (field.type) {
    case "collection":
      validateCollection(field as CollectionFieldSchema, value, entity, path, rootState, issues);
      break;
    case "entity":
      validateEntityField(field as EntityFieldSchema, value, entity, path, rootState, issues);
      break;
    case "object": {
      const objectField = field as ObjectFieldSchema;
      const record = (value ?? {}) as Record<string, unknown>;
      const nested = [
        ...(objectField.fields ?? []),
        ...(objectField.sections ?? []),
      ];
      nested.forEach((nestedField: FieldSchema) => {
        if (nestedField.type === "object" && nestedField.visibility) {
          const conditionValue = record[nestedField.visibility.field];
          if (
            (nestedField.visibility.equals !== undefined && conditionValue !== nestedField.visibility.equals) ||
            (nestedField.visibility.notEquals !== undefined && conditionValue === nestedField.visibility.notEquals)
          ) {
            return;
          }
        }
        validateField(
          nestedField,
          record[nestedField.id],
          entity,
          `${path}.${nestedField.id}`,
          rootState,
          issues
        );
      });
      break;
    }
    case "reference":
      validatePrimitive(field, value, entity, path, issues);
      break;
    default:
      validatePrimitive(field, value, entity, path, issues);
  }

  if (field.checkerRules?.length) {
    field.checkerRules.forEach((rule) => {
      if (rule.includes("divisible by 8") && typeof value === "number") {
        if (value % 8 !== 0) {
          pushIssue(issues, "warning", entity, path, `${field.label} should be divisible by 8.`);
        }
      }
    });
  }
}

export function validateEntity(
  entity: string,
  data: unknown,
  rootState: RootDocumentState,
  issues: ValidationIssue[] = [],
  prefix?: string
): ValidationIssue[] {
  const schema = getEntitySchema(entity);
  if (!schema) return issues;

  schema.fields.forEach((field) => {
    const value = (data as Record<string, unknown> | undefined)?.[field.id];
    const path = prefix ? `${prefix}.${field.id}` : `${entity}.${field.id}`;
    validateField(field, value, entity, path, rootState, issues);
  });

  return issues;
}

function validateCrossReferences(state: RootDocumentState, issues: ValidationIssue[]): void {
  const policies = getCrossReferencePolicies();
  policies.forEach((policy) => {
    const source = state[policy.sourceEntity as keyof RootDocumentState] as unknown;
    const target = state[policy.targetEntity as keyof RootDocumentState] as unknown;
    const targetIds = collectIds(target);

    walkReferences(policy.sourceEntity, source, policy.field, policy, targetIds, issues);
  });
}

function collectIds(target: unknown): Set<string> {
  const ids = new Set<string>();

  if (Array.isArray(target)) {
    target.forEach((item) => {
      if (item && typeof item === "object") {
        Object.keys(item).forEach((key) => {
          const candidate = (item as Record<string, unknown>)[key];
          if (typeof candidate === "string") {
            ids.add(candidate);
          }
        });
      }
    });
    return ids;
  }

  if (target && typeof target === "object") {
    Object.values(target).forEach((value) => {
      if (typeof value === "string") {
        ids.add(value);
      } else if (Array.isArray(value)) {
        value.forEach((nested) => {
          if (nested && typeof nested === "object") {
            Object.values(nested).forEach((maybe) => {
              if (typeof maybe === "string") {
                ids.add(maybe);
              }
            });
          }
        });
      }
    });
  }

  return ids;
}

function walkReferences(
  entity: string,
  source: unknown,
  fieldName: string,
  policy: ReturnType<typeof getCrossReferencePolicies>[number],
  knownIds: Set<string>,
  issues: ValidationIssue[]
): void {
  if (!source) return;

  const schema = getEntitySchema(entity);
  if (!schema) return;

  if (Array.isArray(source)) {
    source.forEach((item, index) => {
      walkReferences(entity, item, fieldName, policy, knownIds, issues);
      if (item && typeof item === "object") {
        const ref = (item as Record<string, unknown>)[fieldName];
        if (ref && typeof ref === "string" && !knownIds.has(ref)) {
          pushIssue(
            issues,
            "error",
            entity,
            `${entity}[${index}].${fieldName}`,
            `${fieldName} references missing ${policy.targetEntity} (${ref}).`
          );
        }
      }
    });
    return;
  }

  if (source && typeof source === "object") {
    const record = source as Record<string, unknown>;
    const direct = record[fieldName];
    if (typeof direct === "string" && !knownIds.has(direct)) {
      pushIssue(
        issues,
        "error",
        entity,
        `${entity}.${fieldName}`,
        `${fieldName} references missing ${policy.targetEntity} (${direct}).`
      );
    }

    schema.fields.forEach((field) => {
      if (field.type === "entity" && record[field.id]) {
        const entityField = field as EntityFieldSchema;
        walkReferences(entityField.entity, record[field.id], fieldName, policy, knownIds, issues);
      }
      if (field.type === "collection" && record[field.id]) {
        const collectionField = field as CollectionFieldSchema;
        const nestedValue = record[field.id];
        if (collectionField.item.type === "entity") {
          const nestedEntity = (collectionField.item as EntityFieldSchema).entity;
          walkReferences(nestedEntity, nestedValue, fieldName, policy, knownIds, issues);
        } else {
          walkReferences(entity, nestedValue, fieldName, policy, knownIds, issues);
        }
      }
    });
  }
}

export function validateDocument(state: RootDocumentState): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  Object.entries(state).forEach(([entity, data]) => {
    validateEntity(entity, data, state, issues);
  });
  validateCrossReferences(state, issues);
  return issues;
}

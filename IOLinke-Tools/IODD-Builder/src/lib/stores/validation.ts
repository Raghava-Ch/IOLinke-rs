// Validation state store orchestrating results produced from data/iodd_form_schema.json (.temp.details/iodd_xsd_comprehensive.csv).
// Provides imperative API to run schema + checker validation and expose summary counts.

import { derived, get, writable } from "svelte/store";
import { documentStore } from "$lib/stores/documents";
import { validateDocument } from "$lib/validators/rules";
import type { ValidationIssue } from "$lib/validators/rules";

export const validationIssues = writable<ValidationIssue[]>([]);
export const lastValidatedAt = writable<Date | null>(null);

export async function runValidation(): Promise<void> {
  const state = get(documentStore);
  const issues = validateDocument(state);
  validationIssues.set(issues);
  lastValidatedAt.set(new Date());
}

export const errorCount = derived(validationIssues, ($issues) =>
  $issues.filter((issue) => issue.severity === "error").length
);

export const warningCount = derived(validationIssues, ($issues) =>
  $issues.filter((issue) => issue.severity === "warning").length
);

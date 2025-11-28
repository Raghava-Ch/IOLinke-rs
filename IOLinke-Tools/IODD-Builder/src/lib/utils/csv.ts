// Lightweight CSV loader to inspect .temp.details/iodd_xsd_comprehensive.csv for inspector metadata.
// Supplies lookup helpers for inspector panel without bundling an extra dependency.

import csvRaw from "../../../.temp.details/iodd_xsd_comprehensive.csv?raw";

export interface CsvRow {
  source_xsd: string;
  element_name: string;
  parent_path: string;
  element_ref: string;
  type_decl: string;
  minOccurs: string;
  maxOccurs: string;
  "default/fixed": string;
  documentation: string;
  attributes: string;
  facets: string;
  mandatory: string;
  optional_reason: string;
  checker_rules: string;
  cross_reference_checks: string;
  global_limits_references: string;
}

let parsedRows: CsvRow[] | null = null;

function parseCsv(): CsvRow[] {
  if (parsedRows) return parsedRows;

  const lines = csvRaw.split(/\r?\n/).filter((line) => line.trim().length > 0);
  if (!lines.length) {
    parsedRows = [];
    return parsedRows;
  }

  const headers = parseLine(lines[0]);
  const rows: CsvRow[] = [];

  for (let i = 1; i < lines.length; i += 1) {
    const cells = parseLine(lines[i]);
    if (!cells.length) continue;
    const row = {} as CsvRow;
    headers.forEach((header, index) => {
      (row as unknown as Record<string, string>)[header] = cells[index] ?? "";
    });
    rows.push(row);
  }

  parsedRows = rows;
  return rows;
}

function parseLine(line: string): string[] {
  const cells: string[] = [];
  let current = "";
  let insideQuotes = false;

  for (let i = 0; i < line.length; i += 1) {
    const char = line[i];

    if (char === '"') {
      const next = line[i + 1];
      if (insideQuotes && next === '"') {
        current += '"';
        i += 1;
        continue;
      }
      insideQuotes = !insideQuotes;
      continue;
    }

    if (char === "," && !insideQuotes) {
      cells.push(current.trim());
      current = "";
      continue;
    }

    current += char;
  }

  if (current.length > 0) {
    cells.push(current.trim());
  } else if (line.endsWith(",")) {
    cells.push("");
  }

  return cells;
}

export function lookupElementMetadata(elementName: string): CsvRow[] {
  return parseCsv().filter((row) => row.element_name === elementName);
}

export function describeElement(elementName: string): string | null {
  const row = parseCsv().find((entry) => entry.element_name === elementName);
  if (!row) return null;
  const doc = row.documentation || row.optional_reason;
  return doc?.length ? doc : null;
}

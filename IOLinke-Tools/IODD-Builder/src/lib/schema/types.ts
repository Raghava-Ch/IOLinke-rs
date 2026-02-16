// Schema type declarations generated against data/iodd_form_schema.json and .temp.details/iodd_xsd_comprehensive.csv
// Provides strongly typed interfaces for renderer logic.

export type PrimitiveFieldType =
  | "string"
  | "text"
  | "integer"
  | "number"
  | "boolean"
  | "enum"
  | "date"
  | "file";

export type StructuredFieldType =
  | "object"
  | "collection"
  | "entity"
  | "reference";

export type FieldType = PrimitiveFieldType | StructuredFieldType;

export interface BaseFieldSchema {
  id: string;
  label: string;
  type: FieldType;
  help?: string;
  documentation?: string;
  required?: boolean;
  minLength?: number;
  maxLength?: number;
  pattern?: string;
  minInclusive?: number;
  maxInclusive?: number;
  default?: unknown;
  checkerRules?: string[];
  optionalReason?: string;
  visibility?: VisibilityCondition;
}

export interface VisibilityCondition {
  field: string;
  equals?: string | number | boolean;
  notEquals?: string | number | boolean;
}

export interface EnumOption {
  value: string;
  label?: string;
}

export interface EnumFieldSchema extends BaseFieldSchema {
  type: "enum";
  options: EnumOption[];
}

export interface ReferenceDescriptor {
  entityId: string;
  displayField: string;
  filterField?: string;
  filterValue?: string;
}

export interface ReferenceFieldSchema extends BaseFieldSchema {
  type: "reference";
  reference: ReferenceDescriptor;
}

export interface ObjectFieldSchema extends BaseFieldSchema {
  type: "object";
  fields?: FieldSchema[];
  sections?: FieldSchema[];
}

export interface CollectionFieldSchema extends BaseFieldSchema {
  type: "collection";
  item: FieldSchema;
  minOccurs?: number;
  maxOccurs?: number | null;
  uniqueBy?: string;
}

export interface EntityFieldSchema extends BaseFieldSchema {
  type: "entity";
  entity: string;
}

export type FieldSchema =
  | BaseFieldSchema
  | EnumFieldSchema
  | ReferenceFieldSchema
  | ObjectFieldSchema
  | CollectionFieldSchema
  | EntityFieldSchema;

export interface EntitySchema {
  id: string;
  label: string;
  documentation?: string;
  fields: FieldSchema[];
}

export interface HierarchyMultiplicity {
  min?: number;
  max?: number | null;
}

export interface HierarchyNode {
  id: string;
  label: string;
  entity: string;
  multiplicity?: HierarchyMultiplicity;
  children?: HierarchyNode[];
}

export interface CrossReferencePolicy {
  id: string;
  label: string;
  sourceEntity: string;
  field: string;
  targetEntity: string;
}

export interface FormSchemaDocument {
  meta: {
    title: string;
    version: string;
    source: {
      csv: string;
      notes?: string;
    };
  };
  hierarchy: HierarchyNode[];
  entities: Record<string, EntitySchema>;
  crossReferencePolicies?: CrossReferencePolicy[];
}

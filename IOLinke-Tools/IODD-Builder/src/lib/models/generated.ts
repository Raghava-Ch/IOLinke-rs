// Auto-generated interfaces derived from data/iodd_form_schema.json (.temp.details/iodd_xsd_comprehensive.csv source).
// Provides typed contracts for renderer state and XML export.

export interface RevisionEntry {
  date: string;
  author: string;
  description: string;
}

export interface DocumentInfo {
  documentId: string;
  title: string;
  subtitle?: string;
  version?: string;
  ioddRelease: "1.1" | "1.1.1" | "1.1.2";
  defaultLanguage: "en" | "de" | "fr" | "zh";
  languages: ("en" | "de" | "fr" | "zh" | "it" | "ja")[];
  revisionHistory?: RevisionEntry[];
}

export interface ProfileHeader {
  profileIdentification: string;
  profileRevision?: string;
  supportedFunctions?: ("processData" | "event" | "parameter")[];
  profileTextRef?: string | null;
}

export interface ProductReference {
  productId: string;
  orderCode?: string;
}

export interface TransportProductReference {
  productId: string;
}

export type WirePosition = "Wire1" | "Wire2" | "Wire3" | "Wire4" | "Wire5";

export type WireColor =
  | "BK"
  | "BN"
  | "RD"
  | "OG"
  | "YE"
  | "GN"
  | "BU"
  | "VT"
  | "GY"
  | "WH"
  | "PK"
  | "GD"
  | "TQ"
  | "SR";

export type WireFunction = "NC" | "L+" | "L-" | "P24" | "N24" | "Other" | "C/Q";

export interface ConnectionWire {
  position: WirePosition;
  color: WireColor;
  function: WireFunction;
}

export interface PhysicalLayerConnection {
  connectionSymbol?: string;
  description?: string | null;
  productRefs?: TransportProductReference[];
  wires?: ConnectionWire[];
}

export type PhysicalLayerBitrate = "COM1" | "COM2" | "COM3";

export interface PhysicalLayer {
  bitrate: PhysicalLayerBitrate;
  minCycleTime?: number;
  sioSupported: boolean;
  mSequenceCapability: number;
  connections?: PhysicalLayerConnection[];
}

export interface TransportLayers {
  physicalLayers: PhysicalLayer[];
}

export interface TestConfigEntry {
  index: number;
  testValue: string;
}

export interface TestEventTrigger {
  appearValue: number;
  disappearValue: number;
}

export interface TestConfig7Entry {
  index: number;
  eventTriggers: TestEventTrigger[];
}

export interface Test {
  config1?: TestConfigEntry[];
  config2?: TestConfigEntry[];
  config3?: TestConfigEntry[];
  config7?: TestConfig7Entry[];
}

export interface DeviceIdentity {
  vendorId: number;
  deviceId: number;
  deviceName: string;
  deviceFamily?: "sensor" | "actuator" | "hub" | "rfid";
  productIds: ProductReference[];
  deviceIcon?: string | null;
  deviceSymbol?: string | null;
}

export interface DeviceVariant {
  variantId: string;
  name: string;
  isDefault?: boolean;
  processDataRef?: string | null;
  supportedMenus?: string[];
}

export interface DeviceVariantCollection {
  variants: DeviceVariant[];
}

export type DatatypeBase = "Boolean" | "Integer" | "String";

export interface BooleanConstraint {
  trueTextRef?: string | null;
  falseTextRef?: string | null;
}

export interface IntegerConstraint {
  min?: number;
  max?: number;
  unitTextRef?: string | null;
}

export interface StringConstraint {
  maxLength?: number;
  pattern?: string;
}

export interface DatatypeConstraints {
  booleanOptions?: BooleanConstraint;
  integerOptions?: IntegerConstraint;
  stringOptions?: StringConstraint;
}

export interface Datatype {
  datatypeId: string;
  baseType: DatatypeBase;
  constraints?: DatatypeConstraints;
}

export interface DatatypeCollection {
  datatypes: Datatype[];
}

export interface Variable {
  variableId: string;
  nameTextRef?: string | null;
  datatypeRef: string;
  defaultValue?: string;
  accessRights?: "RO" | "WO" | "RW";
}

export interface VariableCollection {
  variables: Variable[];
}

export interface ProcessDataChannel {
  name: string;
  offset: number;
  length: number;
  variableId?: string | null;
}

export interface ProcessData {
  processDataId: string;
  name: string;
  bitLength: number;
  variableRef: string;
  channels?: ProcessDataChannel[];
}

export interface ProcessDataCollection {
  processData: ProcessData[];
}

export interface Menu {
  menuId: string;
  title: string;
  children?: Menu[];
  variableRef?: string | null;
}

export interface MenuCollection {
  menus: Menu[];
}

export interface UserInterface {
  menuCollection: MenuCollection;
}

export interface ExternalText {
  textId: string;
  language: "en" | "de" | "fr" | "zh";
  content: string;
}

export interface ExternalTextCollection {
  texts: ExternalText[];
}

export interface Stamp {
  timestamp: string;
  author: string;
  company?: string;
  comments?: string;
}

export interface RootDocumentState {
  DocumentInfo: DocumentInfo;
  TransportLayers: TransportLayers;
  Test: Test;
  ProfileHeader: ProfileHeader;
  DeviceIdentity: DeviceIdentity;
  DeviceVariantCollection: DeviceVariantCollection;
  DatatypeCollection: DatatypeCollection;
  VariableCollection: VariableCollection;
  ProcessDataCollection: ProcessDataCollection;
  UserInterface: UserInterface;
  ExternalTextCollection: ExternalTextCollection;
  Stamp: Stamp;
}

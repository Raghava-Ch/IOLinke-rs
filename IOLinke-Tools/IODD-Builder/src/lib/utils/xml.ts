// XML helpers mapping RootDocumentState to schema-aligned XML (data/iodd_form_schema.json, .temp.details/iodd_xsd_comprehensive.csv).
// Provides export + import routines used by toolbar actions and preview panel.

import type {
  DeviceVariant,
  ExternalText,
  Menu,
  ProcessData,
  RootDocumentState,
  Variable,
} from "$lib/models/generated";

type DocumentLanguage = RootDocumentState["DocumentInfo"]["languages"][number];
type SupportedFunction = NonNullable<RootDocumentState["ProfileHeader"]["supportedFunctions"]>[number];

type PartialState = Partial<RootDocumentState>;

const INDENT = "  ";

function escapeXml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

function renderElement(name: string, value: string | number, depth: number): string {
  return `${INDENT.repeat(depth)}<${name}>${escapeXml(String(value))}</${name}>`;
}

function renderDeviceVariant(variant: DeviceVariant, depth: number): string {
  const lines: string[] = [`${INDENT.repeat(depth)}<DeviceVariant>`];
  lines.push(renderElement("VariantID", variant.variantId, depth + 1));
  lines.push(renderElement("Name", variant.name, depth + 1));
  if (variant.isDefault) {
    lines.push(renderElement("IsDefault", variant.isDefault ? "true" : "false", depth + 1));
  }
  if (variant.processDataRef) {
    lines.push(renderElement("ProcessDataRef", variant.processDataRef, depth + 1));
  }
  if (variant.supportedMenus?.length) {
    lines.push(`${INDENT.repeat(depth + 1)}<SupportedMenus>`);
    variant.supportedMenus.forEach((menu) => {
      lines.push(renderElement("MenuRef", menu, depth + 2));
    });
    lines.push(`${INDENT.repeat(depth + 1)}</SupportedMenus>`);
  }
  lines.push(`${INDENT.repeat(depth)}</DeviceVariant>`);
  return lines.join("\n");
}

function renderMenu(menu: Menu, depth: number): string {
  const lines: string[] = [`${INDENT.repeat(depth)}<Menu>`];
  lines.push(renderElement("MenuID", menu.menuId, depth + 1));
  lines.push(renderElement("Title", menu.title, depth + 1));
  if (menu.variableRef) {
    lines.push(renderElement("VariableRef", menu.variableRef, depth + 1));
  }
  if (menu.children?.length) {
    lines.push(`${INDENT.repeat(depth + 1)}<Children>`);
    menu.children.forEach((child) => {
      lines.push(renderMenu(child, depth + 2));
    });
    lines.push(`${INDENT.repeat(depth + 1)}</Children>`);
  }
  lines.push(`${INDENT.repeat(depth)}</Menu>`);
  return lines.join("\n");
}

function renderProcessData(entry: ProcessData, depth: number): string {
  const lines: string[] = [`${INDENT.repeat(depth)}<ProcessData>`];
  lines.push(renderElement("ProcessDataID", entry.processDataId, depth + 1));
  lines.push(renderElement("Name", entry.name, depth + 1));
  lines.push(renderElement("BitLength", entry.bitLength, depth + 1));
  lines.push(renderElement("VariableRef", entry.variableRef, depth + 1));
  if (entry.channels?.length) {
    lines.push(`${INDENT.repeat(depth + 1)}<Channels>`);
    entry.channels.forEach((channel) => {
      lines.push(`${INDENT.repeat(depth + 2)}<Channel>`);
      lines.push(renderElement("Name", channel.name, depth + 3));
      lines.push(renderElement("Offset", channel.offset, depth + 3));
      lines.push(renderElement("Length", channel.length, depth + 3));
      if (channel.variableId) {
        lines.push(renderElement("VariableRef", channel.variableId, depth + 3));
      }
      lines.push(`${INDENT.repeat(depth + 2)}</Channel>`);
    });
    lines.push(`${INDENT.repeat(depth + 1)}</Channels>`);
  }
  lines.push(`${INDENT.repeat(depth)}</ProcessData>`);
  return lines.join("\n");
}

function renderVariable(entry: Variable, depth: number): string {
  const lines: string[] = [`${INDENT.repeat(depth)}<Variable>`];
  lines.push(renderElement("VariableID", entry.variableId, depth + 1));
  if (entry.nameTextRef) {
    lines.push(renderElement("NameTextRef", entry.nameTextRef, depth + 1));
  }
  lines.push(renderElement("DatatypeRef", entry.datatypeRef, depth + 1));
  if (entry.defaultValue) {
    lines.push(renderElement("DefaultValue", entry.defaultValue, depth + 1));
  }
  if (entry.accessRights) {
    lines.push(renderElement("AccessRights", entry.accessRights, depth + 1));
  }
  lines.push(`${INDENT.repeat(depth)}</Variable>`);
  return lines.join("\n");
}

function renderText(entry: ExternalText, depth: number): string {
  const lines: string[] = [`${INDENT.repeat(depth)}<Text>`];
  lines.push(renderElement("TextID", entry.textId, depth + 1));
  lines.push(renderElement("Language", entry.language, depth + 1));
  lines.push(renderElement("Content", entry.content, depth + 1));
  lines.push(`${INDENT.repeat(depth)}</Text>`);
  return lines.join("\n");
}

export function exportToXml(state: RootDocumentState): string {
  const lines: string[] = ['<?xml version="1.0" encoding="UTF-8"?>', '<IODD xmlns="https://www.io-link.com/IODD/2021/06">'];

  const info = state.DocumentInfo;
  lines.push(`${INDENT}<DocumentInfo>`);
  lines.push(renderElement("DocumentIdentifier", info.documentId, 2));
  lines.push(renderElement("Title", info.title, 2));
  if (info.subtitle) {
    lines.push(renderElement("Subtitle", info.subtitle, 2));
  }
  if (info.version) {
    lines.push(renderElement("Version", info.version, 2));
  }
  lines.push(renderElement("IODDVersion", info.ioddRelease, 2));
  lines.push(renderElement("DefaultLanguage", info.defaultLanguage, 2));
  if (info.languages?.length) {
    lines.push(`${INDENT.repeat(2)}<Languages>`);
    info.languages.forEach((lang) => {
      lines.push(renderElement("Language", lang, 3));
    });
    lines.push(`${INDENT.repeat(2)}</Languages>`);
  }
  if (info.revisionHistory?.length) {
    lines.push(`${INDENT.repeat(2)}<RevisionHistory>`);
    info.revisionHistory.forEach((revision) => {
      lines.push(`${INDENT.repeat(3)}<Revision>`);
      lines.push(renderElement("Date", revision.date, 4));
      lines.push(renderElement("Author", revision.author, 4));
      lines.push(renderElement("Description", revision.description, 4));
      lines.push(`${INDENT.repeat(3)}</Revision>`);
    });
    lines.push(`${INDENT.repeat(2)}</RevisionHistory>`);
  }
  lines.push(`${INDENT}</DocumentInfo>`);

  const profile = state.ProfileHeader;
  lines.push(`${INDENT}<ProfileHeader>`);
  lines.push(renderElement("ProfileIdentification", profile.profileIdentification, 2));
  if (profile.profileRevision) {
    lines.push(renderElement("ProfileRevision", profile.profileRevision, 2));
  }
  if (profile.supportedFunctions?.length) {
    lines.push(`${INDENT.repeat(2)}<SupportedFunctions>`);
    profile.supportedFunctions.forEach((func) => {
      lines.push(renderElement("Function", func, 3));
    });
    lines.push(`${INDENT.repeat(2)}</SupportedFunctions>`);
  }
  if (profile.profileTextRef) {
    lines.push(renderElement("ProfileTextRef", profile.profileTextRef, 2));
  }
  lines.push(`${INDENT}</ProfileHeader>`);

  const identity = state.DeviceIdentity;
  lines.push(`${INDENT}<DeviceIdentity>`);
  lines.push(renderElement("VendorId", identity.vendorId, 2));
  lines.push(renderElement("DeviceId", identity.deviceId, 2));
  lines.push(renderElement("DeviceName", identity.deviceName, 2));
  if (identity.deviceFamily) {
    lines.push(renderElement("DeviceFamily", identity.deviceFamily, 2));
  }
  if (identity.productIds?.length) {
    lines.push(`${INDENT.repeat(2)}<ProductRefs>`);
    identity.productIds.forEach((product) => {
      lines.push(`${INDENT.repeat(3)}<ProductRef>`);
      lines.push(renderElement("ProductId", product.productId, 4));
      if (product.orderCode) {
        lines.push(renderElement("OrderCode", product.orderCode, 4));
      }
      lines.push(`${INDENT.repeat(3)}</ProductRef>`);
    });
    lines.push(`${INDENT.repeat(2)}</ProductRefs>`);
  }
  if (identity.deviceIcon) {
    lines.push(renderElement("DeviceIcon", identity.deviceIcon, 2));
  }
  if (identity.deviceSymbol) {
    lines.push(renderElement("DeviceSymbol", identity.deviceSymbol, 2));
  }
  lines.push(`${INDENT}</DeviceIdentity>`);

  if (state.DeviceVariantCollection.variants?.length) {
    lines.push(`${INDENT}<DeviceVariantCollection>`);
    state.DeviceVariantCollection.variants.forEach((variant) => {
      lines.push(renderDeviceVariant(variant, 2));
    });
    lines.push(`${INDENT}</DeviceVariantCollection>`);
  }

  if (state.VariableCollection.variables?.length) {
    lines.push(`${INDENT}<VariableCollection>`);
    state.VariableCollection.variables.forEach((variable) => {
      lines.push(renderVariable(variable, 2));
    });
    lines.push(`${INDENT}</VariableCollection>`);
  }

  if (state.ProcessDataCollection.processData?.length) {
    lines.push(`${INDENT}<ProcessDataCollection>`);
    state.ProcessDataCollection.processData.forEach((entry) => {
      lines.push(renderProcessData(entry, 2));
    });
    lines.push(`${INDENT}</ProcessDataCollection>`);
  }

  if (state.UserInterface.menuCollection?.menus?.length) {
    lines.push(`${INDENT}<UserInterface>`);
    lines.push(`${INDENT.repeat(2)}<MenuCollection>`);
    state.UserInterface.menuCollection.menus.forEach((menu) => {
      lines.push(renderMenu(menu, 3));
    });
    lines.push(`${INDENT.repeat(2)}</MenuCollection>`);
    lines.push(`${INDENT}</UserInterface>`);
  }

  if (state.ExternalTextCollection.texts?.length) {
    lines.push(`${INDENT}<ExternalTextCollection>`);
    state.ExternalTextCollection.texts.forEach((text) => {
      lines.push(renderText(text, 2));
    });
    lines.push(`${INDENT}</ExternalTextCollection>`);
  }

  const stamp = state.Stamp;
  lines.push(`${INDENT}<Stamp>`);
  lines.push(renderElement("Timestamp", stamp.timestamp, 2));
  lines.push(renderElement("Author", stamp.author, 2));
  if (stamp.company) {
    lines.push(renderElement("Company", stamp.company, 2));
  }
  if (stamp.comments) {
    lines.push(renderElement("Comments", stamp.comments, 2));
  }
  lines.push(`${INDENT}</Stamp>`);

  lines.push('</IODD>');

  return lines.join("\n");
}

export function importFromXml(xml: string): PartialState {
  try {
    const parser = new DOMParser();
    const doc = parser.parseFromString(xml, "application/xml");
    const hasError = doc.querySelector("parsererror");
    if (hasError) {
      throw new Error("Invalid XML");
    }

    const documentInfoNode = doc.querySelector("DocumentInfo");
    const profileNode = doc.querySelector("ProfileHeader");
    const result: PartialState = {};

    if (documentInfoNode) {
      result.DocumentInfo = {
        documentId: documentInfoNode.querySelector("DocumentIdentifier")?.textContent ?? "",
        title: documentInfoNode.querySelector("Title")?.textContent ?? "",
        subtitle: documentInfoNode.querySelector("Subtitle")?.textContent ?? undefined,
        version: documentInfoNode.querySelector("Version")?.textContent ?? undefined,
        ioddRelease: (documentInfoNode.querySelector("IODDVersion")?.textContent ?? "1.1") as RootDocumentState["DocumentInfo"]["ioddRelease"],
        defaultLanguage: (documentInfoNode.querySelector("DefaultLanguage")?.textContent ?? "en") as RootDocumentState["DocumentInfo"]["defaultLanguage"],
        languages: Array.from(documentInfoNode.querySelectorAll("Languages > Language")).map((node) =>
          (node.textContent ?? "en") as DocumentLanguage
        ) as RootDocumentState["DocumentInfo"]["languages"],
        revisionHistory: Array.from(documentInfoNode.querySelectorAll("RevisionHistory > Revision")).map((revision) => ({
          date: revision.querySelector("Date")?.textContent ?? "",
          author: revision.querySelector("Author")?.textContent ?? "",
          description: revision.querySelector("Description")?.textContent ?? "",
        })),
      };
    }

    if (profileNode) {
      result.ProfileHeader = {
        profileIdentification: profileNode.querySelector("ProfileIdentification")?.textContent ?? "",
        profileRevision: profileNode.querySelector("ProfileRevision")?.textContent ?? undefined,
        supportedFunctions: Array.from(profileNode.querySelectorAll("SupportedFunctions > Function")).map((node) =>
          (node.textContent ?? "processData") as SupportedFunction
        ) as RootDocumentState["ProfileHeader"]["supportedFunctions"],
        profileTextRef: profileNode.querySelector("ProfileTextRef")?.textContent ?? null,
      };
    }

    return result;
  } catch (error) {
    console.error("Failed to import XML", error);
    return {};
  }
}

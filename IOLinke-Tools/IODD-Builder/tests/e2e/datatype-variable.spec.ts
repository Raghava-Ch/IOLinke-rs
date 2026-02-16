// Playwright coverage for datatype driven variable wiring and XML export.
// Verifies that newly created datatypes populate dependent dropdowns and surface in exports.
import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const DATATYPE_ID = "TempInt";
const VARIABLE_ID = "VarTemp";
const PROCESS_DATA_ID = "PD_TEMP1";
const BOOL_DATATYPE_ID = "DT_BOOL_OPTIONS";
const INT_DATATYPE_ID = "DT_INT_OPTIONS";
const TRUE_TEXT_ID = "TEXT_BOOL_TRUE";
const FALSE_TEXT_ID = "TEXT_BOOL_FALSE";
const UNIT_TEXT_ID = "TEXT_UNIT_CEL";
const STRING_DATATYPE_ID = "DT_STRING_LIMITS";
const NAMED_VARIABLE_ID = "VarNamed";
const NAME_TEXT_ID = "TEXT_VAR_LABEL";
const CHANNEL_DATATYPE_ID = "DT_CHANNEL";
const CHANNEL_VARIABLE_ID = "VarChannel";
const CHANNEL_PROCESS_DATA_ID = "PD_CHANNEL";

async function createExternalText(page: Page, textId: string, content: string): Promise<void> {
  await page.getByRole("treeitem", { name: /External Texts/i }).click();
  await page.getByRole("button", { name: "Add Texts" }).click();

  const textIdInput = page.getByLabel("Text ID", { exact: true }).last();
  await textIdInput.fill(textId);
  await expect(textIdInput).toHaveValue(textId);

  const languageSelect = page.getByLabel("Language", { exact: true }).last();
  await languageSelect.selectOption("en");
  await expect(languageSelect).toHaveValue("en");

  const contentTextarea = page.getByLabel("Content", { exact: true }).last();
  await contentTextarea.fill(content);
  await expect(contentTextarea).toHaveValue(content);
}

async function createDatatype(
  page: Page,
  datatypeId: string,
  baseType: "Boolean" | "Integer" | "String" = "Integer"
): Promise<void> {
  await page.getByRole("treeitem", { name: /Datatypes/i }).click();
  await page.getByRole("button", { name: "Add Datatypes" }).click();

  const idInput = page.getByLabel("Datatype ID", { exact: true }).last();
  await idInput.fill(datatypeId);

  const baseTypeSelect = page.getByLabel("Base Type", { exact: true }).last();
  await baseTypeSelect.selectOption(baseType);
}

async function createVariable(page: Page, variableId: string, datatypeId: string): Promise<void> {
  await page.getByRole("treeitem", { name: /Variables/i }).click();
  await page.getByRole("button", { name: "Add Variables" }).click();

  const variableIdInput = page.getByLabel("Variable ID", { exact: true }).last();
  await variableIdInput.fill(variableId);

  const datatypeSelect = page.getByLabel("Datatype", { exact: true }).last();
  await expect(datatypeSelect.locator("option", { hasText: datatypeId })).toHaveCount(1);
  await datatypeSelect.selectOption(datatypeId);
}

async function createProcessData(page: Page, processDataId: string, variableId: string): Promise<void> {
  await page.getByRole("treeitem", { name: /Process Data/i }).click();
  await page.getByRole("button", { name: "Add Process Data Entries" }).click();

  const processIdInput = page.getByLabel("Process Data ID", { exact: true }).last();
  await processIdInput.fill(processDataId);

  const nameInput = page.getByLabel("Name", { exact: true }).last();
  await nameInput.fill("Temperature Process");

  const bitLengthInput = page.getByLabel("Bit Length", { exact: true }).last();
  await bitLengthInput.fill("16");

  const variableSelect = page.getByLabel("Variable Reference", { exact: true }).last();
  await variableSelect.selectOption(variableId);
}

test.describe("Datatype usage", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage?.clear();
    });
    await page.goto("/");
  });

  test("variables surface newly created datatypes", async ({ page }) => {
    await createDatatype(page, DATATYPE_ID);

    await page.getByRole("treeitem", { name: /Variables/i }).click();
    await page.getByRole("button", { name: "Add Variables" }).click();

    const variableIdInput = page.getByLabel("Variable ID", { exact: true }).last();
    await variableIdInput.fill(VARIABLE_ID);

    const datatypeSelect = page.getByLabel("Datatype", { exact: true }).last();
    await expect(datatypeSelect.locator("option", { hasText: DATATYPE_ID })).toHaveCount(1);
    await datatypeSelect.selectOption(DATATYPE_ID);
    await expect(datatypeSelect).toHaveValue(DATATYPE_ID);
  });

  test("XML preview includes created variable and process data", async ({ page }) => {
    await createDatatype(page, DATATYPE_ID);
    await createVariable(page, VARIABLE_ID, DATATYPE_ID);
    await createProcessData(page, PROCESS_DATA_ID, VARIABLE_ID);

    await page.getByRole("button", { name: "Preview" }).click();

    const previewPanel = page.locator("aside").first();
    const previewTextarea = previewPanel.locator("textarea");
    await expect(previewTextarea).toBeVisible();

    const xml = await previewTextarea.inputValue();
    expect(xml).toContain(`<VariableID>${VARIABLE_ID}</VariableID>`);
    expect(xml).toContain(`<DatatypeRef>${DATATYPE_ID}</DatatypeRef>`);
    expect(xml).toContain(`<ProcessDataID>${PROCESS_DATA_ID}</ProcessDataID>`);
    expect(xml).toContain(`<VariableRef>${VARIABLE_ID}</VariableRef>`);
  });

  test("datatype constraint sections toggle by base type", async ({ page }) => {
    await createExternalText(page, TRUE_TEXT_ID, "Boolean true label");
    await createExternalText(page, FALSE_TEXT_ID, "Boolean false label");
    await createExternalText(page, UNIT_TEXT_ID, "Degrees Celsius");

    await createDatatype(page, BOOL_DATATYPE_ID, "Boolean");
    await page.getByRole("treeitem", { name: new RegExp(`Datatype - ${BOOL_DATATYPE_ID}`, "i") }).click();

    const booleanTab = page.getByRole("tab", { name: "Boolean" });
    if ((await booleanTab.count()) > 0) {
      await booleanTab.click();
    }

    const trueTextSelect = page.getByLabel("True Text", { exact: true }).last();
    await trueTextSelect.click();
    const trueOptions = await trueTextSelect.locator("option").allTextContents();
    expect(trueOptions).toContain(TRUE_TEXT_ID);
    await trueTextSelect.selectOption(TRUE_TEXT_ID);
    await expect(trueTextSelect).toHaveValue(TRUE_TEXT_ID);

    const falseTextSelect = page.getByLabel("False Text", { exact: true }).last();
    await falseTextSelect.click();
    const falseOptions = await falseTextSelect.locator("option").allTextContents();
    expect(falseOptions).toContain(FALSE_TEXT_ID);
    await falseTextSelect.selectOption(FALSE_TEXT_ID);
    await expect(falseTextSelect).toHaveValue(FALSE_TEXT_ID);

    await createDatatype(page, INT_DATATYPE_ID, "Integer");
    await page.getByRole("treeitem", { name: new RegExp(`Datatype - ${INT_DATATYPE_ID}`, "i") }).click();

    const integerTab = page.getByRole("tab", { name: "Integer" });
    if ((await integerTab.count()) > 0) {
      await integerTab.click();
    }

    const minimumInput = page.getByLabel("Minimum", { exact: true }).last();
    await minimumInput.fill("0");
    await expect(minimumInput).toHaveValue("0");

    const maximumInput = page.getByLabel("Maximum", { exact: true }).last();
    await maximumInput.fill("100");
    await expect(maximumInput).toHaveValue("100");

    const unitTextSelect = page.getByLabel("Unit Text", { exact: true }).last();
    await unitTextSelect.click();
    const unitOptions = await unitTextSelect.locator("option").allTextContents();
    expect(unitOptions).toContain(UNIT_TEXT_ID);
    await unitTextSelect.selectOption(UNIT_TEXT_ID);
    await expect(unitTextSelect).toHaveValue(UNIT_TEXT_ID);

    await createDatatype(page, STRING_DATATYPE_ID, "String");
    await page.getByRole("treeitem", { name: new RegExp(`Datatype - ${STRING_DATATYPE_ID}`, "i") }).click();

    const stringTab = page.getByRole("tab", { name: "String" });
    if ((await stringTab.count()) > 0) {
      await stringTab.click();
    }

    const maxLengthInput = page.getByLabel("Max Length", { exact: true }).last();
    await maxLengthInput.fill("24");
    await expect(maxLengthInput).toHaveValue("24");

    const patternInput = page.getByLabel("Pattern", { exact: true }).last();
    await patternInput.fill("[A-Z]+");
    await expect(patternInput).toHaveValue("[A-Z]+");
  });

  test("variable attributes capture text default and access rights", async ({ page }) => {
    await createExternalText(page, NAME_TEXT_ID, "Variable display name");
    await createDatatype(page, STRING_DATATYPE_ID, "String");
    await createVariable(page, NAMED_VARIABLE_ID, STRING_DATATYPE_ID);

    const nameTextSelect = page.getByLabel("Name Text", { exact: true }).last();
    await expect(nameTextSelect.locator("option", { hasText: NAME_TEXT_ID })).toHaveCount(1);
    await nameTextSelect.selectOption(NAME_TEXT_ID);
    await expect(nameTextSelect).toHaveValue(NAME_TEXT_ID);

    const defaultValueInput = page.getByLabel("Default Value", { exact: true }).last();
    await defaultValueInput.fill("AUTO");
    await expect(defaultValueInput).toHaveValue("AUTO");

    const accessRightsSelect = page.getByLabel("Access Rights", { exact: true }).last();
    await accessRightsSelect.selectOption("RW");
    await expect(accessRightsSelect).toHaveValue("RW");
  });

  test("process data channels capture metadata", async ({ page }) => {
    await createDatatype(page, CHANNEL_DATATYPE_ID);
    await createVariable(page, CHANNEL_VARIABLE_ID, CHANNEL_DATATYPE_ID);
    await createProcessData(page, CHANNEL_PROCESS_DATA_ID, CHANNEL_VARIABLE_ID);

    await page.getByRole("button", { name: "Add Channels" }).click();

    const channelNameInput = page.getByLabel("Name", { exact: true }).last();
    await channelNameInput.fill("Digital Output");
    await expect(channelNameInput).toHaveValue("Digital Output");

    const offsetInput = page.getByLabel("Offset", { exact: true }).last();
    await offsetInput.fill("0");
    await expect(offsetInput).toHaveValue("0");

    const lengthInput = page.getByLabel("Length", { exact: true }).last();
    await lengthInput.fill("16");
    await expect(lengthInput).toHaveValue("16");

    const channelVariableSelect = page.getByLabel("Variable", { exact: true }).last();
    await expect(channelVariableSelect.locator("option", { hasText: CHANNEL_VARIABLE_ID })).toHaveCount(1);
    await channelVariableSelect.selectOption(CHANNEL_VARIABLE_ID);
    await expect(channelVariableSelect).toHaveValue(CHANNEL_VARIABLE_ID);
  });
});

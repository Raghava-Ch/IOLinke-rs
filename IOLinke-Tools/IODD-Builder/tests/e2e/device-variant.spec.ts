// Playwright coverage for device variant configuration flows.
// Verifies variant collection interactions, default flag toggles, and cross-entity references.
import { test, expect, type Page } from "@playwright/test";

const VARIANT_ID = "VAR_DEFAULT";
const VARIANT_NAME = "Factory Default";
const VARIANT_DATATYPE_ID = "VariantDatatype";
const VARIANT_VARIABLE_ID = "VariantVariable";
const VARIANT_PROCESS_DATA_ID = "PD_VARIANT";
const VARIANT_PROCESS_NAME = "Variant Process";
const VARIANT_MENU_ID = "MENU_VARIANT";
const VARIANT_MENU_TEXT_ID = "TEXT_MENU_VARIANT";

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

async function createDatatype(page: Page, datatypeId: string, baseType: "Boolean" | "Integer" | "String"): Promise<void> {
  await page.getByRole("treeitem", { name: /Datatypes/i }).click();
  await page.getByRole("button", { name: "Add Datatypes" }).click();

  const idInput = page.getByLabel("Datatype ID", { exact: true }).last();
  await idInput.fill(datatypeId);
  await expect(idInput).toHaveValue(datatypeId);

  const baseTypeSelect = page.getByLabel("Base Type", { exact: true }).last();
  await baseTypeSelect.selectOption(baseType);
  await expect(baseTypeSelect).toHaveValue(baseType);
}

async function createVariable(page: Page, variableId: string, datatypeId: string): Promise<void> {
  await page.getByRole("treeitem", { name: /Variables/i }).click();
  await page.getByRole("button", { name: "Add Variables" }).click();

  const variableIdInput = page.getByLabel("Variable ID", { exact: true }).last();
  await variableIdInput.fill(variableId);
  await expect(variableIdInput).toHaveValue(variableId);

  const datatypeSelect = page.getByLabel("Datatype", { exact: true }).last();
  await expect(datatypeSelect.locator("option", { hasText: datatypeId })).toHaveCount(1);
  await datatypeSelect.selectOption(datatypeId);
  await expect(datatypeSelect).toHaveValue(datatypeId);
}

async function createProcessData(page: Page, processDataId: string, name: string, variableId: string): Promise<void> {
  await page.getByRole("treeitem", { name: /Process Data/i }).click();
  await page.getByRole("button", { name: "Add Process Data Entries" }).click();

  const processIdInput = page.getByLabel("Process Data ID", { exact: true }).last();
  await processIdInput.fill(processDataId);
  await expect(processIdInput).toHaveValue(processDataId);

  const nameInput = page.getByLabel("Name", { exact: true }).last();
  await nameInput.fill(name);
  await expect(nameInput).toHaveValue(name);

  const bitLengthInput = page.getByLabel("Bit Length", { exact: true }).last();
  await bitLengthInput.fill("16");
  await expect(bitLengthInput).toHaveValue("16");

  const variableSelect = page.getByLabel("Variable Reference", { exact: true }).last();
  await expect(variableSelect.locator("option", { hasText: variableId })).toHaveCount(1);
  await variableSelect.selectOption(variableId);
  await expect(variableSelect).toHaveValue(variableId);
}

async function createMenu(page: Page, menuId: string, titleTextId: string, variableId: string): Promise<void> {
  await page.getByRole("treeitem", { name: /Menu Collection/i }).click();
  await page.getByRole("button", { name: "Add Menus" }).click();

  const menuIdInput = page.getByLabel("Menu ID", { exact: true }).last();
  await menuIdInput.fill(menuId);
  await expect(menuIdInput).toHaveValue(menuId);

  const titleSelect = page.getByLabel("Title", { exact: true }).last();
  await expect(titleSelect.locator("option", { hasText: titleTextId })).toHaveCount(1);
  await titleSelect.selectOption(titleTextId);
  await expect(titleSelect).toHaveValue(titleTextId);

  const variableSelect = page.getByLabel("Variable", { exact: true }).last();
  await expect(variableSelect.locator("option", { hasText: variableId })).toHaveCount(1);
  await variableSelect.selectOption(variableId);
  await expect(variableSelect).toHaveValue(variableId);
}

function supportedMenusTextarea(page: Page) {
  return page.locator("div").filter({ has: page.getByText("Supported Menus") }).locator("textarea").last();
}

test.describe("Device variants", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage?.clear();
    });
    await page.goto("/");
  });

  test("device variants link menus and process data", async ({ page }) => {
    await createExternalText(page, VARIANT_MENU_TEXT_ID, "Variant menu title");
    await createDatatype(page, VARIANT_DATATYPE_ID, "Integer");
    await createVariable(page, VARIANT_VARIABLE_ID, VARIANT_DATATYPE_ID);
    await createProcessData(page, VARIANT_PROCESS_DATA_ID, VARIANT_PROCESS_NAME, VARIANT_VARIABLE_ID);
    await createMenu(page, VARIANT_MENU_ID, VARIANT_MENU_TEXT_ID, VARIANT_VARIABLE_ID);

    await page.getByRole("treeitem", { name: /Device Variants/i }).click();
    await page.getByRole("button", { name: "Add Variants" }).click();

    const variantIdInput = page.getByLabel("Variant ID", { exact: true }).last();
    await variantIdInput.fill(VARIANT_ID);
    await expect(variantIdInput).toHaveValue(VARIANT_ID);

    const variantNameInput = page.getByLabel("Name", { exact: true }).last();
    await variantNameInput.fill(VARIANT_NAME);
    await expect(variantNameInput).toHaveValue(VARIANT_NAME);

    const defaultCheckbox = page.getByLabel("Default Variant", { exact: true }).last();
    await defaultCheckbox.check();
    await expect(defaultCheckbox).toBeChecked();

    const processDataSelect = page.getByLabel("Process Data Reference", { exact: true }).last();
    await expect(processDataSelect.locator("option", { hasText: VARIANT_PROCESS_NAME })).toHaveCount(1);
    await processDataSelect.selectOption(VARIANT_PROCESS_NAME);
    await expect(processDataSelect).toHaveValue(VARIANT_PROCESS_NAME);

    await page.getByRole("button", { name: "Add Supported Menus" }).click();
    const supportedMenuInput = supportedMenusTextarea(page);
    await supportedMenuInput.fill(VARIANT_MENU_TEXT_ID);
    await expect(supportedMenuInput).toHaveValue(VARIANT_MENU_TEXT_ID);
  });

  test("removing final variant surfaces validation message", async ({ page }) => {
    await page.getByRole("treeitem", { name: /Device Variants/i }).click();
    await page.getByRole("button", { name: "Add Variants" }).click();

    const variantIdInput = page.getByLabel("Variant ID", { exact: true }).last();
    await variantIdInput.fill("VAR_TEMP");
    const variantNameInput = page.getByLabel("Name", { exact: true }).last();
    await variantNameInput.fill("Temporary Variant");

    await page.getByRole("button", { name: "Remove entry" }).last().click();

    const minOccursMessage = page.getByText("Variants requires at least 1 entry.");
    await expect(minOccursMessage).toBeVisible();

    await page.getByRole("button", { name: "Add Variants" }).click();
    const replacementIdInput = page.getByLabel("Variant ID", { exact: true }).last();
    await replacementIdInput.fill("VAR_RECOVERY");
    const replacementNameInput = page.getByLabel("Name", { exact: true }).last();
    await replacementNameInput.fill("Recovery Variant");

    await expect(minOccursMessage).toBeHidden();
  });
});

// Playwright regression tests for menu configuration referencing external texts and variables.
// Validates that new resources surface immediately in dependent dropdowns.
import { test, expect, type Page } from "@playwright/test";

const TEXT_ID = "TEXT_MENU_MAIN";
const DATATYPE_ID = "MenuBoolType";
const VARIABLE_ID = "MenuVarMain";
const MENU_ID = "MENU_MAIN";
const CHILD_TEXT_ID = "TEXT_MENU_CHILD";
const CHILD_DATATYPE_ID = "MenuChildType";
const CHILD_VARIABLE_ID = "MenuVarChild";
const CHILD_MENU_ID = "MENU_CHILD";

async function createExternalText(
  page: Page,
  textId: string = TEXT_ID,
  content: string = "Main menu title"
): Promise<void> {
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
  datatypeId: string = DATATYPE_ID,
  baseType: "Boolean" | "Integer" | "String" = "Boolean"
): Promise<void> {
  await page.getByRole("treeitem", { name: /Datatypes/i }).click();
  await page.getByRole("button", { name: "Add Datatypes" }).click();

  const datatypeIdInput = page.getByLabel("Datatype ID", { exact: true }).last();
  await datatypeIdInput.fill(datatypeId);
  await expect(datatypeIdInput).toHaveValue(datatypeId);

  const baseTypeSelect = page.getByLabel("Base Type", { exact: true }).last();
  await baseTypeSelect.selectOption(baseType);
  await expect(baseTypeSelect).toHaveValue(baseType);
}

async function createVariable(
  page: Page,
  variableId: string = VARIABLE_ID,
  datatypeId: string = DATATYPE_ID
): Promise<void> {
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

test.describe("User interface menus", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage?.clear();
    });
    await page.goto("/");
  });

  test("menus surface new external texts and variables", async ({ page }) => {
    await createExternalText(page);
    await createDatatype(page);
    await createVariable(page);

    await page.getByRole("treeitem", { name: /Menu Collection/i }).click();
    await page.getByRole("button", { name: "Add Menus" }).click();

    const menuIdInput = page.getByLabel("Menu ID", { exact: true }).last();
    await menuIdInput.fill(MENU_ID);
    await expect(menuIdInput).toHaveValue(MENU_ID);

    const titleSelect = page.getByLabel("Title", { exact: true }).last();
    await expect(titleSelect.locator("option", { hasText: TEXT_ID })).toHaveCount(1);
    await titleSelect.selectOption(TEXT_ID);
    await expect(titleSelect).toHaveValue(TEXT_ID);

    const variableSelect = page.getByLabel("Variable", { exact: true }).last();
    await expect(variableSelect.locator("option", { hasText: VARIABLE_ID })).toHaveCount(1);
    await variableSelect.selectOption(VARIABLE_ID);
    await expect(variableSelect).toHaveValue(VARIABLE_ID);
  });

  test("menus support nested child entries", async ({ page }) => {
    await createExternalText(page);
    await createDatatype(page);
    await createVariable(page);

    await createExternalText(page, CHILD_TEXT_ID, "Child menu title");
    await createDatatype(page, CHILD_DATATYPE_ID, "Integer");
    await createVariable(page, CHILD_VARIABLE_ID, CHILD_DATATYPE_ID);

    await page.getByRole("treeitem", { name: /Menu Collection/i }).click();
    await page.getByRole("button", { name: "Add Menus" }).click();

    const parentMenuIdInput = page.getByLabel("Menu ID", { exact: true }).last();
    await parentMenuIdInput.fill(MENU_ID);
    await expect(parentMenuIdInput).toHaveValue(MENU_ID);

    const parentTitleSelect = page.getByLabel("Title", { exact: true }).last();
    await expect(parentTitleSelect.locator("option", { hasText: TEXT_ID })).toHaveCount(1);
    await parentTitleSelect.selectOption(TEXT_ID);
    await expect(parentTitleSelect).toHaveValue(TEXT_ID);

    const parentVariableSelect = page.getByLabel("Variable", { exact: true }).last();
    await expect(parentVariableSelect.locator("option", { hasText: VARIABLE_ID })).toHaveCount(1);
    await parentVariableSelect.selectOption(VARIABLE_ID);
    await expect(parentVariableSelect).toHaveValue(VARIABLE_ID);

    await page.getByRole("button", { name: "Add Children" }).click();

    const childMenuIdInput = page.getByLabel("Menu ID", { exact: true }).last();
    await childMenuIdInput.fill(CHILD_MENU_ID);
    await expect(childMenuIdInput).toHaveValue(CHILD_MENU_ID);

    const childTitleSelect = page.getByLabel("Title", { exact: true }).last();
    await expect(childTitleSelect.locator("option", { hasText: CHILD_TEXT_ID })).toHaveCount(1);
    await childTitleSelect.selectOption(CHILD_TEXT_ID);
    await expect(childTitleSelect).toHaveValue(CHILD_TEXT_ID);

    const childVariableSelect = page.getByLabel("Variable", { exact: true }).last();
    await expect(childVariableSelect.locator("option", { hasText: CHILD_VARIABLE_ID })).toHaveCount(1);
    await childVariableSelect.selectOption(CHILD_VARIABLE_ID);
    await expect(childVariableSelect).toHaveValue(CHILD_VARIABLE_ID);
  });
});

// Playwright regression tests for menu configuration referencing external texts and variables.
// Validates that new resources surface immediately in dependent dropdowns.
import { test, expect, type Page } from "@playwright/test";

const TEXT_ID = "TEXT_MENU_MAIN";
const DATATYPE_ID = "MenuBoolType";
const VARIABLE_ID = "MenuVarMain";
const MENU_ID = "MENU_MAIN";

async function createExternalText(page: Page): Promise<void> {
  await page.getByRole("treeitem", { name: /^External Texts/i }).click();
  await page.getByRole("button", { name: "Add Texts" }).click();

  const textIdInput = page.getByLabel("Text ID", { exact: true }).last();
  await textIdInput.fill(TEXT_ID);
  await expect(textIdInput).toHaveValue(TEXT_ID);

  const languageSelect = page.getByLabel("Language", { exact: true }).last();
  await languageSelect.selectOption("en");
  await expect(languageSelect).toHaveValue("en");

  const contentTextarea = page.getByLabel("Content", { exact: true }).last();
  await contentTextarea.fill("Main menu title");
  await expect(contentTextarea).toHaveValue("Main menu title");
}

async function createDatatype(page: Page): Promise<void> {
  await page.getByRole("treeitem", { name: /^Datatypes/i }).click();
  await page.getByRole("button", { name: "Add Datatypes" }).click();

  const datatypeIdInput = page.getByLabel("Datatype ID", { exact: true }).last();
  await datatypeIdInput.fill(DATATYPE_ID);
  await expect(datatypeIdInput).toHaveValue(DATATYPE_ID);

  const baseTypeSelect = page.getByLabel("Base Type", { exact: true }).last();
  await baseTypeSelect.selectOption("Boolean");
  await expect(baseTypeSelect).toHaveValue("Boolean");
}

async function createVariable(page: Page): Promise<void> {
  await page.getByRole("treeitem", { name: /^Variables/i }).click();
  await page.getByRole("button", { name: "Add Variables" }).click();

  const variableIdInput = page.getByLabel("Variable ID", { exact: true }).last();
  await variableIdInput.fill(VARIABLE_ID);
  await expect(variableIdInput).toHaveValue(VARIABLE_ID);

  const datatypeSelect = page.getByLabel("Datatype", { exact: true }).last();
  await expect(datatypeSelect.locator("option", { hasText: DATATYPE_ID })).toHaveCount(1);
  await datatypeSelect.selectOption(DATATYPE_ID);
  await expect(datatypeSelect).toHaveValue(DATATYPE_ID);
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

    await page.getByRole("treeitem", { name: /^Menu Collection/i }).click();
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
});

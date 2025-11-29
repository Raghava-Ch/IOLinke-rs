// Playwright coverage for datatype driven variable wiring and XML export.
// Verifies that newly created datatypes populate dependent dropdowns and surface in exports.
import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const DATATYPE_ID = "TempInt";
const VARIABLE_ID = "VarTemp";
const PROCESS_DATA_ID = "PD_TEMP1";

async function createDatatype(page: Page, datatypeId: string): Promise<void> {
  await page.getByRole("treeitem", { name: /^Datatypes/i }).click();
  await page.getByRole("button", { name: "Add Datatypes" }).click();

  const idInput = page.getByLabel("Datatype ID", { exact: true }).last();
  await idInput.fill(datatypeId);

  const baseTypeSelect = page.getByLabel("Base Type", { exact: true }).last();
  await baseTypeSelect.selectOption("Integer");
}

async function createVariable(page: Page, variableId: string, datatypeId: string): Promise<void> {
  await page.getByRole("treeitem", { name: /^Variables/i }).click();
  await page.getByRole("button", { name: "Add Variables" }).click();

  const variableIdInput = page.getByLabel("Variable ID", { exact: true }).last();
  await variableIdInput.fill(variableId);

  const datatypeSelect = page.getByLabel("Datatype", { exact: true }).last();
  await expect(datatypeSelect.locator("option", { hasText: datatypeId })).toHaveCount(1);
  await datatypeSelect.selectOption(datatypeId);
}

async function createProcessData(page: Page, processDataId: string, variableId: string): Promise<void> {
  await page.getByRole("treeitem", { name: /^Process Data/i }).click();
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

    await page.getByRole("treeitem", { name: /^Variables/i }).click();
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
});

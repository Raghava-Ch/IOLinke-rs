// Playwright coverage for transport layer connections including wire metadata.
// Validates schema-driven wire positions, color/function enumerations, and XML export integration.
import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const NEW_WIRE_POSITION = "Wire3";
const NEW_WIRE_COLOR = "RD";
const NEW_WIRE_FUNCTION = "N24";

async function openTransportLayers(page: Page): Promise<void> {
  await page.getByRole("treeitem", { name: /Transport Layers/i }).click();
}

test.describe("Transport layers", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage?.clear();
    });
    await page.goto("/");
  });

  test("connections capture wire layout for export", async ({ page }) => {
    await openTransportLayers(page);

    const functionSelect = page.getByLabel("Function", { exact: true }).first();
    await functionSelect.click();
    const functionOptions = await functionSelect.locator("option").allTextContents();
    expect(functionOptions).toContain("C/Q");
    await functionSelect.selectOption("L+");

    await page.getByRole("button", { name: "Add Connections" }).click();
    const wiresMinOccurs = page.locator("li", { hasText: "Wires requires at least 1 entry." });
    await expect(wiresMinOccurs.first()).toBeVisible();

    await page.getByRole("button", { name: "Remove entry" }).last().click();
    await expect(wiresMinOccurs).toHaveCount(0);

    await page.getByRole("button", { name: "Add Wires" }).click();

    const positionSelect = page.getByLabel("Position", { exact: true }).last();
    await expect(positionSelect).toHaveValue(NEW_WIRE_POSITION);
    await positionSelect.selectOption(NEW_WIRE_POSITION);

    const colorSelect = page.getByLabel("Color", { exact: true }).last();
    await colorSelect.selectOption(NEW_WIRE_COLOR);

    const newFunctionSelect = page.getByLabel("Function", { exact: true }).last();
    await newFunctionSelect.selectOption(NEW_WIRE_FUNCTION);

    await positionSelect.selectOption("Wire1");
    const duplicateWarnings = page.locator("li", { hasText: "Position must be unique within Wires." });
    await expect(duplicateWarnings.first()).toBeVisible();
    await positionSelect.selectOption(NEW_WIRE_POSITION);
    await expect(duplicateWarnings).toHaveCount(0);

    await page.getByRole("button", { name: "Preview" }).click();

    const previewTextarea = page.locator("aside").first().locator("textarea");
    await expect(previewTextarea).toBeVisible();
    const xml = await previewTextarea.inputValue();

    expect(xml).toContain(`<${NEW_WIRE_POSITION} color="${NEW_WIRE_COLOR}" function="${NEW_WIRE_FUNCTION}"`);
    expect(xml).toContain("<Wire1");
    expect(xml).toContain("function=\"L+\"");
  });
});

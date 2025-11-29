// Playwright regression tests covering schema-driven parameter navigation.
// Validates that selecting hierarchy nodes swaps forms per data/iodd_form_schema.json.
import { test, expect } from "@playwright/test";

test.describe("Hierarchy-driven parameter editing", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage?.clear();
    });
    await page.goto("/");
  });

  test("Document Info fields render by default", async ({ page }) => {
    const documentIdField = page.getByLabel("Document Identifier", { exact: true });
    await expect(documentIdField).toBeVisible();
    await expect(documentIdField).toHaveValue("SampleDevice");

    await expect(page.getByLabel("Title", { exact: true })).toHaveValue("Sample Smart Sensor");
  });

  test("Document Info validates identifier pattern", async ({ page }) => {
    const documentIdField = page.getByLabel("Document Identifier", { exact: true });
    await documentIdField.fill("invalid id");
    await documentIdField.blur();

    await expect(
      page.getByText("Document Identifier does not match required pattern [A-Za-z0-9._-]+.")
    ).toBeVisible();
  });

  test("Selecting Profile Header swaps to profile form", async ({ page }) => {
    await page.getByRole("treeitem", { name: /Profile Header/i }).click();

    const profileIdField = page.getByLabel("Profile Identification");
    await expect(profileIdField).toBeVisible();
    await expect(profileIdField).toHaveValue("IO-Link Smart Sensor");

    await expect(page.getByLabel("Document Identifier")).toHaveCount(0);
  });

  test("Selecting Device Identity shows device-specific fields", async ({ page }) => {
    await page.getByRole("treeitem", { name: /Device Identity/i }).click();

    const vendorIdField = page.getByLabel("Vendor ID");
    await expect(vendorIdField).toBeVisible();
    await expect(vendorIdField).toHaveValue("123");

    await expect(page.getByLabel("Device Name")).toHaveValue("Smart Sensor");
  });
});

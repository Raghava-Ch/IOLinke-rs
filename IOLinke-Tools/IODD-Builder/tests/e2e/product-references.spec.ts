// Playwright coverage for Device Identity product reference interactions.
// Ensures collection add/remove flows enforce validation while remaining editable.
import { test, expect } from "@playwright/test";

test.describe("Device Identity product references", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage?.clear();
    });
    await page.goto("/");
    await page.getByRole("treeitem", { name: /Device Identity/i }).click();
  });

  test("adding a product reference enforces required product id", async ({ page }) => {
    await page.getByRole("button", { name: "Add Product References" }).click();

    const secondProductIdInput = page.locator("#productIds__1__productId");
    await expect(secondProductIdInput).toBeVisible();

    const missingSecondId = page.getByText(
      "Product ID is required. (DeviceIdentity.productIds[1].productId)"
    );
    await expect(missingSecondId).toBeVisible();

    await secondProductIdInput.fill("SS-777");
    await secondProductIdInput.blur();

    await expect(missingSecondId).toBeHidden();
    await expect(secondProductIdInput).toHaveValue("SS-777");
  });

  test("removing all product references requires at least one entry", async ({ page }) => {
    const removeButtons = page.getByRole("button", { name: "Remove" });
    await removeButtons.first().click();

    const minOccursMessage = page.getByText("Product References requires at least 1 entries.");
    await expect(minOccursMessage).toBeVisible();

    await page.getByRole("button", { name: "Add Product References" }).click();
    const replacementProductIdInput = page.locator("#productIds__0__productId");
    await replacementProductIdInput.fill("SS-321");
    await replacementProductIdInput.blur();

    await expect(minOccursMessage).toBeHidden();
  });
});

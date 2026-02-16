// Playwright regression tests covering schema-driven parameter navigation.
// Validates that selecting hierarchy nodes swaps forms per data/iodd_form_schema.json.
import { test, expect, type Page } from "@playwright/test";

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

  test("Document Info manages supported languages and revision history", async ({ page }) => {
    const subtitleInput = page.getByLabel("Subtitle", { exact: true });
    await subtitleInput.fill("Advanced Thermal Sensor");
    await expect(subtitleInput).toHaveValue("Advanced Thermal Sensor");

    const versionInput = page.getByLabel("Document Version", { exact: true });
    await versionInput.fill("2.0.0");
    await versionInput.blur();
    await expect(versionInput).toHaveValue("2.0.0");

    const releaseSelect = page.getByLabel("IODD Specification Version", { exact: true });
    await releaseSelect.selectOption("1.1");
    await expect(releaseSelect).toHaveValue("1.1");

    const defaultLanguageSelect = page.getByLabel("Default Language", { exact: true });
    await defaultLanguageSelect.selectOption("fr");
    await expect(defaultLanguageSelect).toHaveValue("fr");

    while (await page.getByRole("button", { name: "Remove entry" }).count()) {
      await page.getByRole("button", { name: "Remove entry" }).first().click();
    }

    const minLanguagesMessage = page.getByText("Supported Languages requires at least 1 entry.");
    await expect(minLanguagesMessage).toBeVisible();

    await page.getByRole("button", { name: "Add Supported Languages" }).click();

    const languagesTextarea = page.locator("textarea").first();
    await languagesTextarea.fill("fr");
    await expect(languagesTextarea).toHaveValue("fr");
    await expect(minLanguagesMessage).toBeHidden();

    await page.getByRole("button", { name: "Add Revision History" }).click();

    const revisionDateInput = page.getByLabel("Date", { exact: true }).last();
    await revisionDateInput.fill("2024-12-01");
    await expect(revisionDateInput).toHaveValue("2024-12-01");

    const revisionAuthorInput = page.getByLabel("Author", { exact: true }).last();
    await revisionAuthorInput.fill("QA Engineer");
    await expect(revisionAuthorInput).toHaveValue("QA Engineer");

    const revisionDescription = page.locator("textarea").last();
    await revisionDescription.fill("Document updated with advanced profile and localization options.");
    await expect(revisionDescription).toHaveValue(
      "Document updated with advanced profile and localization options."
    );
  });

  test("Profile Header manages revision, functions, and text reference", async ({ page }) => {
    const profileTextId = "TEXT_PROFILE_HEADER";
    await createExternalText(page, profileTextId, "Profile Header Title");

    await page.getByRole("treeitem", { name: /Profile Header/i }).click();

    const profileRevisionInput = page.getByLabel("Profile Revision", { exact: true });
    await profileRevisionInput.fill("2.1");
    await profileRevisionInput.blur();
    await expect(profileRevisionInput).toHaveValue("2.1");

    const supportedFunctionsTextarea = page.locator("textarea").first();
    await supportedFunctionsTextarea.fill("event");
    await supportedFunctionsTextarea.blur();
    await expect(supportedFunctionsTextarea).toHaveValue("event");

    await page.getByRole("button", { name: "Add Supported Functions" }).click();
    const secondFunctionTextarea = page.locator("textarea").nth(1);
    await secondFunctionTextarea.fill("parameter");
    await secondFunctionTextarea.blur();
    await expect(secondFunctionTextarea).toHaveValue("parameter");

    const profileTextSelect = page.getByLabel("Profile Text Reference", { exact: true });
    await expect(profileTextSelect.locator("option", { hasText: profileTextId })).toHaveCount(1);
    await profileTextSelect.selectOption(profileTextId);
    await expect(profileTextSelect).toHaveValue(profileTextId);
  });

  test("Transport layers manage physical layer collection", async ({ page }) => {
    const connectionTextId = "TEXT_CONN_RUNTIME";
    await createExternalText(page, connectionTextId, "Runtime connection description");

    await page.getByRole("treeitem", { name: /Transport Layers/i }).click();

    const existingBitrateSelect = page.getByLabel("Bitrate", { exact: true }).first();
    await expect(existingBitrateSelect).toHaveValue("COM3");

    await page.getByRole("button", { name: "Add Physical Layers" }).click();

    const newBitrateSelect = page.getByLabel("Bitrate", { exact: true }).last();
    await newBitrateSelect.selectOption("COM1");
    await expect(newBitrateSelect).toHaveValue("COM1");

    const minCycleInput = page.getByLabel("Minimum Cycle Time (µs)", { exact: true }).last();
    await minCycleInput.fill("2000");
    await expect(minCycleInput).toHaveValue("2000");

    const sioCheckbox = page.getByLabel("SIO Supported", { exact: true }).last();
    await sioCheckbox.check();
    await expect(sioCheckbox).toBeChecked();

    const mSequenceInput = page.getByLabel("M-Sequence Capability", { exact: true }).last();
    await mSequenceInput.fill("7");
    await expect(mSequenceInput).toHaveValue("7");

    await page.getByRole("button", { name: "Add Connections" }).last().click();

    const connectionSymbolInput = page.getByLabel("Connection Symbol", { exact: true }).last();
    await connectionSymbolInput.fill("custom-con-pic.png");
    await expect(connectionSymbolInput).toHaveValue("custom-con-pic.png");

    const descriptionSelect = page.getByLabel("Description", { exact: true }).last();
    await expect(descriptionSelect.locator("option", { hasText: connectionTextId })).toHaveCount(1);
    await descriptionSelect.selectOption(connectionTextId);
    await expect(descriptionSelect).toHaveValue(connectionTextId);

    const productIdInput = page.getByLabel("Product ID", { exact: true }).last();
    await productIdInput.fill("SS-CON-99");
    await expect(productIdInput).toHaveValue("SS-CON-99");
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

  test("Stamp metadata edits persist", async ({ page }) => {
    await page.getByRole("treeitem", { name: /Stamp/i }).click();

    const timestampInput = page.getByLabel("Timestamp", { exact: true });
    await timestampInput.fill("2025-11-30");
    await expect(timestampInput).toHaveValue("2025-11-30");

    const authorInput = page.getByLabel("Author", { exact: true });
    await authorInput.fill("QA Automation");
    await expect(authorInput).toHaveValue("QA Automation");

    const companyInput = page.getByLabel("Company", { exact: true });
    await companyInput.fill("Automation Labs");
    await expect(companyInput).toHaveValue("Automation Labs");

    const commentsTextarea = page.getByLabel("Comments", { exact: true });
    await commentsTextarea.fill("Smoke test update to metadata.");
    await expect(commentsTextarea).toHaveValue("Smoke test update to metadata.");
  });

  test("Test entity manages IO-Link test configurations", async ({ page }) => {
    await page.getByRole("treeitem", { name: /^Test/ }).click();

    const config1Index = page.getByLabel("Index", { exact: true }).first();
    await config1Index.fill("48");
    await config1Index.blur();
    await expect(config1Index).toHaveValue("48");

    const config1TestValue = page.getByLabel("Test Value", { exact: true }).first();
    await config1TestValue.fill("0xAA,0xBB");
    await config1TestValue.blur();
    await expect(config1TestValue).toHaveValue("0xAA,0xBB");

    await config1TestValue.fill("invalid");
    await config1TestValue.blur();
    const testValueWarning = page.locator("li", {
      hasText: "Test Value does not match required pattern"
    });
    await expect(testValueWarning.first()).toBeVisible();

    await config1TestValue.fill("0x0A");
    await config1TestValue.blur();
    await expect(testValueWarning).toHaveCount(0);

    const config7Index = page.getByLabel("Index", { exact: true }).nth(1);
    await config7Index.fill("64");
    await config7Index.blur();
    await expect(config7Index).toHaveValue("64");

    const appearValueInputs = page.getByLabel("Appear Value", { exact: true });
    await appearValueInputs.first().fill("5");
    await expect(appearValueInputs.first()).toHaveValue("5");

    await page.getByRole("button", { name: "Add Event Triggers" }).click();
    const updatedAppearInputs = page.getByLabel("Appear Value", { exact: true });
    await expect(updatedAppearInputs).toHaveCount(2);
    await updatedAppearInputs.nth(1).fill("10");
    await expect(updatedAppearInputs.nth(1)).toHaveValue("10");

    const disappearValueInputs = page.getByLabel("Disappear Value", { exact: true });
    await disappearValueInputs.nth(1).fill("11");
    await expect(disappearValueInputs.nth(1)).toHaveValue("11");
  });
});

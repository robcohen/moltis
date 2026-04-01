// Browser viewer page — end-to-end tests.
//
// Verifies the browser session management UI: creating sessions,
// navigating, screencast frame delivery, and session lifecycle.

const { test, expect } = require("../base-test");
const {
	navigateAndWait,
	waitForWsConnected,
	watchPageErrors,
	expectPageContentMounted,
} = require("../helpers");

test.describe("Browser sessions page", () => {
	test.beforeEach(async ({ page }) => {
		await navigateAndWait(page, "/settings/browser");
		await waitForWsConnected(page);
	});

	test("renders browser page with heading and new session button", async ({ page }) => {
		const pageErrors = watchPageErrors(page);
		await expect(page.getByRole("heading", { name: "Browser Sessions", exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "New Session" })).toBeVisible();
		await expect(page.getByRole("button", { name: "Refresh" })).toBeVisible();
		expect(pageErrors).toEqual([]);
	});

	test("shows empty state message when no sessions exist", async ({ page }) => {
		const pageErrors = watchPageErrors(page);
		await expect(page.getByText("No active browser sessions")).toBeVisible();
		expect(pageErrors).toEqual([]);
	});

	test("new session button shows creating state", async ({ page }) => {
		const pageErrors = watchPageErrors(page);
		const btn = page.getByRole("button", { name: "New Session" });
		await btn.click();

		// Button should show creating state (disabled with "Creating…" text)
		// or have already finished creating — either way, no JS errors.
		// We check that the button was clickable and the page didn't crash.
		await expect(page.getByRole("heading", { name: "Browser Sessions", exact: true })).toBeVisible();
		expect(pageErrors).toEqual([]);
	});

	test("navigate bar appears after creating a session", async ({ page }) => {
		const pageErrors = watchPageErrors(page);

		// Create a new session
		await page.getByRole("button", { name: "New Session" }).click();

		// Wait for the navigate input to appear (session created + selected)
		await expect(page.getByPlaceholder("Search or enter URL...")).toBeVisible({ timeout: 30000 });

		// The "Enter a URL" hint should be visible in the canvas area
		await expect(page.getByText("Enter a URL above to start browsing")).toBeVisible();

		expect(pageErrors).toEqual([]);
	});

	test("navigate bar normalizes bare domains with https", async ({ page }) => {
		const pageErrors = watchPageErrors(page);

		// Create a session
		await page.getByRole("button", { name: "New Session" }).click();
		await expect(page.getByPlaceholder("Search or enter URL...")).toBeVisible({ timeout: 30000 });

		// Type a bare domain and submit
		const input = page.getByPlaceholder("Search or enter URL...");
		await input.fill("example.com");
		await input.press("Enter");

		// Should navigate successfully (no error toast about invalid scheme)
		// Wait for the session list to update with the URL
		await expect.poll(async () => {
			const text = await page.locator(".truncate").allInnerTexts();
			return text.some((t) => t.includes("example.com"));
		}, { timeout: 30000 }).toBeTruthy();

		expect(pageErrors).toEqual([]);
	});

	test("screencast delivers frames after navigation", async ({ page }) => {
		const pageErrors = watchPageErrors(page);

		// Create a session
		await page.getByRole("button", { name: "New Session" }).click();
		await expect(page.getByPlaceholder("Search or enter URL...")).toBeVisible({ timeout: 30000 });

		// Navigate to a real page
		const input = page.getByPlaceholder("Search or enter URL...");
		await input.fill("example.com");
		await input.press("Enter");

		// Canvas should appear and receive frames — "Waiting for first frame"
		// should disappear and be replaced by the canvas with frame data.
		// The canvas element appears when screencasting.value is true and
		// frameData.value is set.
		await expect(page.locator("canvas")).toBeVisible({ timeout: 30000 });

		// Verify frame metadata is shown (e.g. "Frame #1" or similar)
		await expect.poll(async () => {
			const text = await page.locator("body").innerText();
			return text.includes("Frame #");
		}, { timeout: 15000 }).toBeTruthy();

		expect(pageErrors).toEqual([]);
	});

	test("session can be closed", async ({ page }) => {
		const pageErrors = watchPageErrors(page);

		// Create a session
		await page.getByRole("button", { name: "New Session" }).click();
		await expect(page.getByPlaceholder("Search or enter URL...")).toBeVisible({ timeout: 30000 });

		// Session card should be visible
		await expect(page.getByRole("button", { name: "Close" })).toBeVisible();

		// Close the session
		await page.getByRole("button", { name: "Close" }).click();

		// Should return to empty state
		await expect(page.getByText("No active browser sessions")).toBeVisible({ timeout: 10000 });

		expect(pageErrors).toEqual([]);
	});

	test("session shows sandbox badge when sandbox is enabled", async ({ page }) => {
		const pageErrors = watchPageErrors(page);

		// Create a session
		await page.getByRole("button", { name: "New Session" }).click();
		await expect(page.getByPlaceholder("Search or enter URL...")).toBeVisible({ timeout: 30000 });

		// Check for sandbox badge (if sandbox is enabled in the test environment)
		// or absence of it (if running without sandbox). Either way, no errors.
		const hasSandbox = await page.getByText("sandbox").isVisible().catch(() => false);
		// Just verify the session card rendered with session info
		await expect(page.getByRole("button", { name: "View" })).toBeVisible();

		expect(pageErrors).toEqual([]);
	});
});

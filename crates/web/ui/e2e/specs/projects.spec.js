const { expect, test } = require("../base-test");
const { navigateAndWait, watchPageErrors } = require("../helpers");

test.describe("Projects page", () => {
	test("projects page loads", async ({ page }) => {
		const pageErrors = watchPageErrors(page);
		await navigateAndWait(page, "/projects");

		await expect(page.getByRole("heading", { name: "Repositories", exact: true })).toBeVisible();
		expect(pageErrors).toEqual([]);
	});

	test("add project input present", async ({ page }) => {
		await navigateAndWait(page, "/projects");

		await expect(page.getByText("Directory", { exact: true })).toBeVisible();
		await expect(page.getByPlaceholder("/path/to/project")).toBeVisible();
		await expect(page.getByRole("button", { name: "Add", exact: true })).toBeVisible();
	});

	test("auto-detect button present", async ({ page }) => {
		await navigateAndWait(page, "/projects");

		await expect(page.getByRole("button", { name: "Auto-detect", exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "Clear All", exact: true })).toBeVisible();
		await expect(page.getByText(/does not delete anything from disk/i)).toBeVisible();
		await expect(page.getByText(/scans common directories/i)).toBeVisible();
	});

	test("work board loads with operator controls", async ({ page }) => {
		await navigateAndWait(page, "/projects");

		await expect(page.getByRole("heading", { name: "Work board", exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "Refresh", exact: true })).toBeVisible();
		await expect(page.getByText("Portfolio", { exact: true })).toBeVisible();
		await expect(page.getByText("Cross-project blockers", { exact: true })).toBeVisible();
		await expect(page.getByText("Pending approvals", { exact: true })).toBeVisible();
		await expect(page.getByText("Goal planner", { exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "Plan goal", exact: true })).toBeVisible();
		await expect(page.getByText("Template library", { exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "Create template", exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "Instantiate template", exact: true })).toBeVisible();
		await expect(page.getByText("Recurring work", { exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "Create recurring job", exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "Run recurring now", exact: true })).toBeVisible();
		await expect(page.getByText("Recent materializations", { exact: true })).toBeVisible();
		await expect(page.getByText("Tracker sync", { exact: true })).toBeVisible();
		await expect(page.getByText("Fetch from MCP", { exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "Fetch into form", exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "Import tracker item", exact: true })).toBeVisible();
		await expect(page.getByText("Recent links", { exact: true })).toBeVisible();
		await expect(page.getByText("Project budget", { exact: true })).toBeVisible();
		await expect(page.getByText("Work package", { exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "Export package", exact: true })).toBeVisible();
		await expect(page.getByRole("button", { name: "Import package", exact: true })).toBeVisible();
		await expect(page.getByText("Active runs", { exact: true })).toBeVisible();
	});

	test("projects route is hidden from nav", async ({ page }) => {
		await navigateAndWait(page, "/projects");
		await expect(page.locator('a.nav-link[href="/projects"]')).toHaveCount(0);
	});

	test("projects accessible from settings sidebar", async ({ page }) => {
		const pageErrors = watchPageErrors(page);
		await navigateAndWait(page, "/settings/projects");

		await expect(page.getByRole("heading", { name: "Repositories", exact: true })).toBeVisible();
		expect(pageErrors).toEqual([]);
	});

	test("page has no JS errors", async ({ page }) => {
		const pageErrors = watchPageErrors(page);
		await navigateAndWait(page, "/projects");
		expect(pageErrors).toEqual([]);
	});
});

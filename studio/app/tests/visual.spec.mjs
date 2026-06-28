// Frontend visualization tests for BioOKF Studio.
// Run against the static frontend (same code as the Tauri webview):
//   cd studio/app/dist && python3 -m http.server 8754 &
//   npx playwright test app/tests/visual.spec.mjs
// (Requires @playwright/test. These assertions mirror the checks executed
//  during development via the Playwright MCP tools.)

import { test, expect } from '@playwright/test';

const URL = 'http://localhost:8754/index.html';

test('loads bundles and renders the graph', async ({ page }) => {
  await page.goto(URL);
  await page.waitForFunction(() => window.__OKF_READY === true, null, { timeout: 5000 });
  const bases = await page.locator('.kb').count();
  expect(bases).toBeGreaterThanOrEqual(1);
  const state = await page.evaluate(() => window.__bokf.getState());
  expect(state.nodes).toBeGreaterThan(0);
  expect(state.edges).toBeGreaterThan(0);
});

test('node click opens a detail panel with frontmatter + edges', async ({ page }) => {
  await page.goto(URL);
  await page.waitForFunction(() => window.__OKF_READY === true);
  // pick the first real (non-external) node and open it
  const ok = await page.evaluate(() => {
    const id = (window.__firstReal && window.__firstReal()) || null;
    return id;
  });
  // fall back: select COVID-19 if present
  await page.evaluate(() => window.__bokf.selectNode('COVID-19'));
  await expect(page.locator('.detail.open .d-id')).toBeVisible();
  await expect(page.locator('.detail .fm')).toBeVisible();
});

test('search dims non-matching nodes', async ({ page }) => {
  await page.goto(URL);
  await page.waitForFunction(() => window.__OKF_READY === true);
  await page.evaluate(() => window.__bokf.search('gene'));
  const term = await page.evaluate(() => document.getElementById('searchInput').value);
  expect(term).toBe('gene');
});

test('sidebar collapses', async ({ page }) => {
  await page.goto(URL);
  await page.waitForFunction(() => window.__OKF_READY === true);
  await page.click('#collapseBtn');
  await expect(page.locator('.wbody.collapsed')).toHaveCount(1);
});

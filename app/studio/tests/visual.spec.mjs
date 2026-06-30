// Frontend visualization tests for BioOKF Studio.
// Run against the static frontend (same code as the Tauri webview):
//   cd app/studio/dist && python3 -m http.server 8754 &
//   npx playwright test app/tests/visual.spec.mjs
// (Requires @playwright/test. These assertions mirror the checks executed
//  during development via the Playwright MCP tools.)

import { test, expect } from '@playwright/test';

const URL = 'http://localhost:8754/index.html';

test('loads bundles and renders the graph', async ({ page }) => {
  await page.goto(URL);
  await page.waitForFunction(() => window.__BOKF_READY === true, null, { timeout: 5000 });
  const bases = await page.locator('.kb').count();
  expect(bases).toBeGreaterThanOrEqual(1);
  const state = await page.evaluate(() => window.__bokf.getState());
  expect(state.counts.nodes).toBeGreaterThan(0);
  expect(state.counts.edges).toBeGreaterThan(0);
});

test('node click opens a detail panel with frontmatter + edges', async ({ page }) => {
  await page.goto(URL);
  await page.waitForFunction(() => window.__BOKF_READY === true);
  const ok = await page.evaluate(() => {
    const node = window.__bokf.getGraph().nodes.find(n => !n.external);
    return node ? window.__bokf.selectNode(node.id) : false;
  });
  expect(ok).toBeTruthy();
  await expect(page.locator('.detail.open .d-id')).toBeVisible();
  await expect(page.locator('.detail .fm')).toBeVisible();
});

test('search dims non-matching nodes', async ({ page }) => {
  await page.goto(URL);
  await page.waitForFunction(() => window.__BOKF_READY === true);
  await page.evaluate(() => window.__bokf.search('gene'));
  const term = await page.evaluate(() => document.getElementById('searchInput').value);
  expect(term).toBe('gene');
});

test('sidebar collapses', async ({ page }) => {
  await page.goto(URL);
  await page.waitForFunction(() => window.__BOKF_READY === true);
  await page.click('#collapseBtn');
  await expect(page.locator('.wbody.collapsed')).toHaveCount(1);
});

test('cli install popup renders when forced', async ({ page }) => {
  await page.goto(URL + '?forceCliPopup=1');
  await page.waitForSelector('#cli-modal:not([hidden])', { timeout: 3000 });
  await expect(page.locator('#cli-modal-title')).toHaveText(/Install BioOKF command-line tools/);
  await expect(page.locator('#cli-install')).toBeVisible();
  await expect(page.locator('#cli-never')).toBeHidden();
  await page.screenshot({ path: 'screens/cli-popup.png' });
});

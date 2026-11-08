import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';

const API = 'http://127.0.0.1:18080/api/v1';
const token = 'canopy-relay-field-lead-token';
const auth = { Authorization: `Bearer ${token}` };
const evidence = process.env.DREAM_MACHINE_EVIDENCE_DIR;

async function openWorkspace(page: Page) {
  await page.goto('/');
  await page.getByLabel('Session token').fill(token);
  await page.getByRole('button', { name: 'Load live habitat' }).click();
  await expect(page.getByRole('heading', { name: 'Habitat pulse' })).toBeVisible();
}

async function auditCount(page: Page) {
  const response = await page.request.get(`${API}/audit`, { headers: auth });
  expect(response.ok()).toBeTruthy();
  return ((await response.json()) as { data: unknown[] }).data.length;
}

test('[journey:primary] signed field evidence becomes a rendered habitat anomaly', async ({ page }, info) => {
  await openWorkspace(page);
  await page.getByRole('button', { name: /Anomaly Workbench/ }).click();
  await page.getByLabel('Signal kind').selectOption('canopy-moisture');
  await page.getByLabel('Signal value').fill('0.18');
  await page.getByRole('button', { name: 'Analyze live reading' }).click();
  await expect(page.getByText('Verified.')).toBeVisible();
  await expect(page.locator('#observation-id')).not.toBeEmpty();
  await expect(page.getByText(/RuCelium|Midstream|OpenDAL/i).first()).toBeVisible();

  if (evidence && info.project.name === 'desktop') {
    await page.screenshot({ path: path.join(evidence, 'desktop.png'), fullPage: true, scale: 'css' });
  }
  if (evidence && info.project.name === 'mobile') {
    await page.screenshot({ path: path.join(evidence, 'mobile.png'), fullPage: true, scale: 'css' });
  }
});

test('[journey:denial-recovery] forbidden habitat effect leaves no durable side effect and safe recovery succeeds', async ({ page }) => {
  await openWorkspace(page);
  const before = await auditCount(page);
  await page.getByRole('button', { name: /Intervention Planner/ }).click();
  await page.getByLabel('Intervention action').selectOption('open-firebreak');
  await page.getByLabel('Risk authority').selectOption('restricted');
  await page.getByLabel('Intervention rationale').fill('Attempt a protected habitat effect without the required independent ecological approver.');
  await page.getByRole('button', { name: 'Request intervention' }).click();
  await expect(page.getByRole('alert')).toContainText(/policy|denied|approver/i);
  expect(await auditCount(page)).toBe(before);

  await page.getByLabel('Intervention action').selectOption('deploy-mesh-relay');
  await page.getByLabel('Risk authority').selectOption('low');
  await page.getByLabel('Intervention rationale').fill('Restore signed field observability through a reversible relay with no habitat mutation.');
  await page.getByRole('button', { name: 'Request intervention' }).click();
  await expect(page.locator('#intervention-id')).not.toBeEmpty();
  expect(await auditCount(page)).toBeGreaterThan(before);
});

test('[journey:durable-retrieval] authorized intervention survives browser refresh and backend restart', async ({ page }) => {
  await openWorkspace(page);
  await page.getByRole('button', { name: /Intervention Planner/ }).click();
  await page.getByLabel('Intervention action').selectOption('deploy-mesh-relay');
  await page.getByLabel('Risk authority').selectOption('low');
  await page.getByLabel('Intervention rationale').fill('Persist this exact relay authorization so operators can recover it after a service restart.');
  await page.getByRole('button', { name: 'Request intervention' }).click();
  const interventionId = (await page.locator('#intervention-id').textContent())?.trim();
  expect(interventionId).toBeTruthy();

  const restart = process.env.CANOPY_RESTART_COMMAND ?? '../scripts/restart-journey.sh';
  execFileSync(restart, { cwd: process.cwd(), env: process.env, stdio: 'inherit' });
  await expect.poll(async () => (await page.request.get('http://127.0.0.1:18080/readyz')).ok()).toBeTruthy();

  await page.reload();
  await page.getByLabel('Session token').fill(token);
  await page.getByRole('button', { name: 'Load live habitat' }).click();
  await page.getByRole('button', { name: /Intervention Planner/ }).click();
  await expect(page.getByText(interventionId!, { exact: true })).toBeVisible();
  await page.getByRole('button', { name: /Audit Vault/ }).click();
  await expect(page.getByRole('heading', { name: 'Evidence ledger' })).toBeVisible();

  const saved = await page.request.get(`${API}/interventions`, { headers: auth });
  expect(saved.ok()).toBeTruthy();
  expect(((await saved.json()) as { data: { id: string }[] }).data).toEqual(expect.arrayContaining([expect.objectContaining({ id: interventionId })]));
});

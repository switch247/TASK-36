const assert = require('node:assert/strict');
const { test, expect } = require('@playwright/test');
const {
  FRONTEND_URL,
  creds,
  expectToast,
  login,
  logout,
  api,
} = require('./helpers');

async function createCandidateFlow(page, unique) {
  await page.getByRole('link', { name: 'Candidates' }).click();
  await expect(page.getByRole('heading', { name: 'Candidates' })).toBeVisible({ timeout: 30000 });
  await page.getByPlaceholder('DOB (MM/DD/YYYY)').fill('03/27/2001');
  await page.getByPlaceholder('National ID').fill(`ID-${unique}`);
  await page.getByPlaceholder('Barcode').fill(`BAR-${unique}`);
  await page.getByPlaceholder('Candidate Name').fill(`Candidate ${unique}`);
  await page.locator('select').first().selectOption({ index: 1 });
  await page.getByRole('button', { name: 'Create' }).click();
  await expectToast(page, 'Candidate created');
  await expect(page.getByText(`BAR-${unique}`, { exact: false })).toBeVisible({ timeout: 30000 });
}

async function createSessionAndAssignmentFlow(page) {
  await page.getByRole('link', { name: 'Exams' }).click();
  await expect(page.getByRole('heading', { name: 'Exams' })).toBeVisible({ timeout: 30000 });
  await page.getByPlaceholder('Template Name').fill('base-template');
  await page.getByPlaceholder('Duration Minutes').fill('75');
  await page.getByPlaceholder('Starts (MM/DD/YYYY hh:mm AM/PM)').fill('04/10/2026 09:00 AM');
  await page.getByPlaceholder('Ends (MM/DD/YYYY hh:mm AM/PM)').fill('04/10/2026 10:15 AM');
  await page.getByRole('button', { name: 'Create Exam Session' }).click();
  await expectToast(page, 'Exam session created');

  const sessionsResp = await api(page, '/sessions?page=1&limit=200');
  assert.equal(sessionsResp.status(), 200);
  const sessions = await sessionsResp.json();
  const createdSession = sessions.find((row) => row.duration_minutes === 75 && row.template_name === 'base-template');
  assert.ok(createdSession, 'created exam session should be discoverable via API');

  const usersResp = await api(page, '/users');
  assert.equal(usersResp.status(), 200);
  const users = await usersResp.json();
  const proctor = users.find((row) => row.username === creds.proctor.username);
  assert.ok(proctor, 'expected seeded proctor user');

  const assignResp = await api(page, `/sessions/${createdSession.id}/assignments`, {
    method: 'POST',
    data: { user_id: proctor.id },
  });
  assert.equal(assignResp.status(), 201);

  await logout(page);
  await login(page, creds.proctor);
  await page.goto(`${FRONTEND_URL}/sessions`, { waitUntil: 'networkidle', timeout: 120000 });
  await expect(page.getByRole('heading', { name: 'Sessions' })).toBeVisible({ timeout: 30000 });
  await expect(page.getByRole('cell', { name: createdSession.id, exact: true })).toBeVisible({ timeout: 30000 });

  await logout(page);
  await login(page, creds.admin);
  return createdSession.id;
}

async function outputsAndExportFlow(page, sessionId) {
  await page.goto(`${FRONTEND_URL}/outputs`, { waitUntil: 'networkidle', timeout: 120000 });
  await expect(page.getByRole('heading', { name: 'Outputs' })).toBeVisible({ timeout: 30000 });
  await page.selectOption('select', { label: sessionId });
  const selects = page.locator('select');
  await selects.nth(1).selectOption('AdmitCard');
  await selects.nth(2).selectOption('TestPrint');
  await page.getByRole('button', { name: 'Generate Output' }).click();
  await expectToast(page, 'Output generated');
  await expect(page.getByRole('cell', { name: sessionId, exact: true })).toBeVisible({ timeout: 30000 });

  await page.getByRole('link', { name: 'Reports' }).click();
  await expect(page.getByRole('heading', { name: 'Reports' })).toBeVisible({ timeout: 30000 });
  await page.getByRole('button', { name: 'Export Incident CSV' }).click();
  await expectToast(page, 'CSV ready:');
}

async function dashboardLoadFlow(page) {
  await page.goto(`${FRONTEND_URL}/dashboard`, { waitUntil: 'networkidle', timeout: 120000 });
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible({ timeout: 30000 });
  await expect(page.getByText('Upcoming Sessions', { exact: false })).toBeVisible({ timeout: 30000 });
  await expect(page.getByText('Recent Outputs', { exact: false })).toBeVisible({ timeout: 30000 });
  await expect(page.getByText('Seat Utilization Trend', { exact: false })).toBeVisible({ timeout: 30000 });
}

async function roleRestrictedFlow(page) {
  await logout(page);
  await login(page, creds.auditor);
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible({ timeout: 30000 });
  await expect(page.getByRole('link', { name: 'Outputs' })).toHaveCount(0, { timeout: 10000 });
  await page.goto(`${FRONTEND_URL}/admin`, { waitUntil: 'networkidle', timeout: 120000 });
  await expectToast(page, 'HTTP 403');
}

test.describe('Fullstack E2E', () => {
  test('admin can load dashboard', async ({ page }) => {
    await login(page, creds.admin);
    await dashboardLoadFlow(page);
  });

  test('admin can create a candidate', async ({ page }) => {
    const unique = `${Date.now()}`;
    await login(page, creds.admin);
    await createCandidateFlow(page, unique);
  });

  test('admin can create and assign an exam session to proctor', async ({ page }) => {
    const unique = `${Date.now()}`;
    await login(page, creds.admin);
    await createCandidateFlow(page, unique);
    const sessionId = await createSessionAndAssignmentFlow(page);
    assert.ok(sessionId, 'expected created session id');
  });

  test('admin can generate outputs and export reports', async ({ page }) => {
    const unique = `${Date.now()}`;
    await login(page, creds.admin);
    await createCandidateFlow(page, unique);
    const sessionId = await createSessionAndAssignmentFlow(page);
    await outputsAndExportFlow(page, sessionId);
  });

  test('auditor is restricted from admin surface', async ({ page }) => {
    await login(page, creds.auditor);
    await roleRestrictedFlow(page);
  });
});

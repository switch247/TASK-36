const { test, expect } = require('@playwright/test');
const {
  FRONTEND_URL,
  creds,
  expectToast,
  login,
  logout,
  expectNav,
} = require('./helpers');

async function verifyAdminSurface(page) {
  await page.goto(`${FRONTEND_URL}/dashboard`, { waitUntil: 'networkidle', timeout: 120000 });
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible({ timeout: 30000 });
  await expectNav(page, 'Admin', true);
  await expectNav(page, 'Templates', true);
  await expectNav(page, 'Outputs', true);

  await page.goto(`${FRONTEND_URL}/admin`, { waitUntil: 'networkidle', timeout: 120000 });
  await expect(page.getByRole('heading', { name: 'Admin' })).toBeVisible({ timeout: 30000 });
  await page.getByPlaceholder('Username').fill(`admin_e2e_${Date.now()}`);
  await page.getByPlaceholder('Password').fill('AdminPass#2026!');
  await page.locator('select').first().selectOption('Auditor');
  await page.getByRole('button', { name: 'Create User' }).click();
  await expectToast(page, 'User created successfully');
}

async function verifyCoordinatorSurface(page) {
  await page.goto(`${FRONTEND_URL}/dashboard`, { waitUntil: 'networkidle', timeout: 120000 });
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible({ timeout: 30000 });
  await expectNav(page, 'Admin', false);
  await expectNav(page, 'Templates', true);
  await expectNav(page, 'Outputs', true);

  await page.goto(`${FRONTEND_URL}/templates`, { waitUntil: 'networkidle', timeout: 120000 });
  await expect(page.getByRole('heading', { name: 'Templates' })).toBeVisible({ timeout: 30000 });
  await page.getByPlaceholder('Template ID').fill(`coord-template-${Date.now()}`);
  await page.getByPlaceholder('Version').fill('1');
  await page.getByPlaceholder('Snapshot JSON').fill('{"rules":{"id":["Required"]}}');
  await page.getByRole('button', { name: 'Save' }).click();
  await expectToast(page, 'Template saved');
}

async function verifyProctorSurface(page) {
  await page.goto(`${FRONTEND_URL}/dashboard`, { waitUntil: 'networkidle', timeout: 120000 });
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible({ timeout: 30000 });
  await expectNav(page, 'Templates', false);
  await expectNav(page, 'Outputs', true);
  await expectNav(page, 'Reports', true);

  await page.goto(`${FRONTEND_URL}/outputs`, { waitUntil: 'networkidle', timeout: 120000 });
  await expect(page.getByRole('heading', { name: 'Outputs' })).toBeVisible({ timeout: 30000 });
  await page.getByPlaceholder('Record ID').fill('cand-e2e-record');
  await page.getByPlaceholder('File name').fill('evidence');
  await page.getByPlaceholder('Ext (pdf/jpg...)').fill('pdf');
  await page.getByPlaceholder('Paste file bytes as base64').fill('UERG');
  await page.getByRole('button', { name: 'Upload Attachment' }).click();
  await expectToast(page, 'HTTP 403');
}

async function verifyAuditorSurface(page) {
  await page.goto(`${FRONTEND_URL}/dashboard`, { waitUntil: 'networkidle', timeout: 120000 });
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible({ timeout: 30000 });
  await expectNav(page, 'Candidates', false);
  await expectNav(page, 'Rooms', false);
  await expectNav(page, 'Outputs', false);
  await expectNav(page, 'Reports', true);

  await page.goto(`${FRONTEND_URL}/reports`, { waitUntil: 'networkidle', timeout: 120000 });
  await expect(page.getByRole('heading', { name: 'Reports' })).toBeVisible({ timeout: 30000 });
  await page.goto(`${FRONTEND_URL}/admin`, { waitUntil: 'networkidle', timeout: 120000 });
  await expectToast(page, 'HTTP 403');
}

test.describe('Role Matrix E2E', () => {
  test('admin sees admin surfaces and can create users', async ({ page }) => {
    await login(page, creds.admin);
    await verifyAdminSurface(page);
  });

  test('coordinator sees coordinator surfaces and can save templates', async ({ page }) => {
    await login(page, creds.coordinator);
    await verifyCoordinatorSurface(page);
  });

  test('proctor sees proctor surfaces and is blocked from attachment upload', async ({ page }) => {
    await login(page, creds.proctor);
    await verifyProctorSurface(page);
  });

  test('auditor sees read-only surfaces and is blocked from admin', async ({ page }) => {
    await login(page, creds.auditor);
    await verifyAuditorSurface(page);
  });
});

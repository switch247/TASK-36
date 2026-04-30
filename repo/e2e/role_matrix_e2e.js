const assert = require('node:assert/strict');
const { chromium } = require('playwright');

const FRONTEND_URL = process.env.FRONTEND_URL || 'http://frontend:8080';

const creds = {
  admin: { username: process.env.ADMIN_USERNAME || 'admin_local', password: process.env.ADMIN_PASSWORD || 'AdminPass#2026!' },
  coordinator: { username: process.env.COORD_USERNAME || 'coord_local', password: process.env.COORD_PASSWORD || 'CoordPass#2026!' },
  proctor: { username: process.env.PROCTOR_USERNAME || 'proctor_local', password: process.env.PROCTOR_PASSWORD || 'ProctorPass#2026!' },
  auditor: { username: process.env.AUDITOR_USERNAME || 'auditor_local', password: process.env.AUDITOR_PASSWORD || 'AuditorPass#2026!' },
};

async function expectToast(page, text) {
  await page.getByText(text, { exact: false }).waitFor({ timeout: 30000 });
}

async function login(page, { username, password }) {
  await page.goto(FRONTEND_URL, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByPlaceholder('Username').fill(username);
  await page.getByPlaceholder('Password').fill(password);
  await page.getByRole('button', { name: 'Sign In' }).click();
  await page.waitForFunction(() => !!window.localStorage.getItem('proctorops_auth_session'), { timeout: 30000 });
  await page.getByRole('button', { name: 'Logout' }).waitFor({ timeout: 30000 });
  await page.getByRole('heading', { name: 'Login' }).waitFor({ state: 'detached', timeout: 30000 }).catch(() => {});
}

async function logout(page) {
  await page.getByRole('button', { name: 'Logout' }).click();
  await expectToast(page, 'Logged out');
  await page.getByRole('heading', { name: 'Login' }).waitFor({ timeout: 30000 });
}

async function expectNav(page, label, visible) {
  const locator = page.getByRole('link', { name: label });
  if (visible) {
    await locator.waitFor({ timeout: 30000 });
    assert.equal(await locator.count(), 1, `expected ${label} nav`);
  } else {
    await locator.waitFor({ state: 'detached', timeout: 10000 }).catch(() => {});
    assert.equal(await locator.count(), 0, `did not expect ${label} nav`);
  }
}

async function verifyAdminSurface(page) {
  await page.goto(`${FRONTEND_URL}/dashboard`, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByRole('heading', { name: 'Dashboard' }).waitFor({ timeout: 30000 });
  await expectNav(page, 'Admin', true);
  await expectNav(page, 'Templates', true);
  await expectNav(page, 'Outputs', true);

  await page.goto(`${FRONTEND_URL}/admin`, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByRole('heading', { name: 'Admin' }).waitFor({ timeout: 30000 });
  await page.getByPlaceholder('Username').fill(`admin_e2e_${Date.now()}`);
  await page.getByPlaceholder('Password').fill('AdminPass#2026!');
  await page.locator('select').first().selectOption('Auditor');
  await page.getByRole('button', { name: 'Create User' }).click();
  await expectToast(page, 'User created successfully');
}

async function verifyCoordinatorSurface(page) {
  await page.goto(`${FRONTEND_URL}/dashboard`, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByRole('heading', { name: 'Dashboard' }).waitFor({ timeout: 30000 });
  await expectNav(page, 'Admin', false);
  await expectNav(page, 'Templates', true);
  await expectNav(page, 'Outputs', true);

  await page.goto(`${FRONTEND_URL}/templates`, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByRole('heading', { name: 'Templates' }).waitFor({ timeout: 30000 });
  await page.getByPlaceholder('Template ID').fill(`coord-template-${Date.now()}`);
  await page.getByPlaceholder('Version').fill('1');
  await page.getByPlaceholder('Snapshot JSON').fill('{"rules":{"id":["Required"]}}');
  await page.getByRole('button', { name: 'Save' }).click();
  await expectToast(page, 'Template saved');
}

async function verifyProctorSurface(page) {
  await page.goto(`${FRONTEND_URL}/dashboard`, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByRole('heading', { name: 'Dashboard' }).waitFor({ timeout: 30000 });
  await expectNav(page, 'Templates', false);
  await expectNav(page, 'Outputs', true);
  await expectNav(page, 'Reports', true);

  await page.goto(`${FRONTEND_URL}/outputs`, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByRole('heading', { name: 'Outputs' }).waitFor({ timeout: 30000 });
  await page.getByPlaceholder('Record ID').fill('cand-e2e-record');
  await page.getByPlaceholder('File name').fill('evidence');
  await page.getByPlaceholder('Ext (pdf/jpg...)').fill('pdf');
  await page.getByPlaceholder('Paste file bytes as base64').fill('UERG');
  await page.getByRole('button', { name: 'Upload Attachment' }).click();
  await expectToast(page, 'HTTP 403');
}

async function verifyAuditorSurface(page) {
  await page.goto(`${FRONTEND_URL}/dashboard`, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByRole('heading', { name: 'Dashboard' }).waitFor({ timeout: 30000 });
  await expectNav(page, 'Candidates', false);
  await expectNav(page, 'Rooms', false);
  await expectNav(page, 'Outputs', false);
  await expectNav(page, 'Reports', true);

  await page.goto(`${FRONTEND_URL}/reports`, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByRole('heading', { name: 'Reports' }).waitFor({ timeout: 30000 });
  await page.goto(`${FRONTEND_URL}/admin`, { waitUntil: 'networkidle', timeout: 120000 });
  await expectToast(page, 'HTTP 403');
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  try {
    await login(page, creds.admin);
    await verifyAdminSurface(page);
    await logout(page);

    await login(page, creds.coordinator);
    await verifyCoordinatorSurface(page);
    await logout(page);

    await login(page, creds.proctor);
    await verifyProctorSurface(page);
    await logout(page);

    await login(page, creds.auditor);
    await verifyAuditorSurface(page);
  } finally {
    await browser.close();
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

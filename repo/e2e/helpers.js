const assert = require('node:assert/strict');
const { expect } = require('@playwright/test');

const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:8080';
const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8001/api/v1';

const creds = {
  admin: { username: process.env.ADMIN_USERNAME || 'admin_local', password: process.env.ADMIN_PASSWORD || 'AdminPass#2026!' },
  coordinator: { username: process.env.COORD_USERNAME || 'coord_local', password: process.env.COORD_PASSWORD || 'CoordPass#2026!' },
  proctor: { username: process.env.PROCTOR_USERNAME || 'proctor_local', password: process.env.PROCTOR_PASSWORD || 'ProctorPass#2026!' },
  auditor: { username: process.env.AUDITOR_USERNAME || 'auditor_local', password: process.env.AUDITOR_PASSWORD || 'AuditorPass#2026!' },
};

async function expectToast(page, text) {
  await expect(page.getByText(text, { exact: false })).toBeVisible({ timeout: 30000 });
}

async function login(page, { username, password }) {
  await page.goto(FRONTEND_URL, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByPlaceholder('Username').fill(username);
  await page.getByPlaceholder('Password').fill(password);
  await page.getByRole('button', { name: 'Sign In' }).click();
  await page.waitForFunction(() => !!window.localStorage.getItem('proctorops_auth_session'), { timeout: 30000 });
  await expect(page.getByRole('button', { name: 'Logout' })).toBeVisible({ timeout: 30000 });
  await expect(page.getByRole('heading', { name: 'Login' })).toHaveCount(0, { timeout: 30000 });
}

async function logout(page) {
  await page.getByRole('button', { name: 'Logout' }).click();
  await expectToast(page, 'Logged out');
  await expect(page.getByRole('heading', { name: 'Login' })).toBeVisible({ timeout: 30000 });
}

async function getStoredSession(page) {
  const raw = await page.evaluate(() => window.localStorage.getItem('proctorops_auth_session'));
  assert.ok(raw, 'expected stored auth session');
  return JSON.parse(raw);
}

async function api(page, path, options = {}) {
  const session = await getStoredSession(page);
  const headers = {
    'Content-Type': 'application/json',
    Authorization: `Bearer ${session.jwt}`,
    'x-session-id': session.session_id,
    ...(options.headers || {}),
  };
  return page.request.fetch(`${BACKEND_URL}${path}`, {
    method: options.method || 'GET',
    headers,
    data: options.data,
  });
}

async function expectNav(page, label, visible) {
  const locator = page.getByRole('link', { name: label });
  if (visible) {
    await expect(locator).toBeVisible({ timeout: 30000 });
    await expect(locator).toHaveCount(1);
  } else {
    await expect(locator).toHaveCount(0, { timeout: 10000 });
  }
}

module.exports = {
  FRONTEND_URL,
  BACKEND_URL,
  creds,
  expectToast,
  login,
  logout,
  getStoredSession,
  api,
  expectNav,
};

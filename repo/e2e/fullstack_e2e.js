const assert = require('node:assert/strict');
const { chromium } = require('playwright');

const FRONTEND_URL = process.env.FRONTEND_URL || 'http://frontend:8080';
const BACKEND_URL = process.env.BACKEND_URL || 'http://app:8001/api/v1';

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
  await expectToast(page, 'Login successful');
}

async function logout(page) {
  await page.getByRole('button', { name: 'Logout' }).click();
  await expectToast(page, 'Logged out');
  await page.getByRole('heading', { name: 'Login' }).waitFor({ timeout: 30000 });
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

async function createCandidateFlow(page, unique) {
  await page.getByRole('link', { name: 'Candidates' }).click();
  await page.getByRole('heading', { name: 'Candidates' }).waitFor({ timeout: 30000 });
  await page.getByPlaceholder('DOB (MM/DD/YYYY)').fill('03/27/2001');
  await page.getByPlaceholder('National ID').fill(`ID-${unique}`);
  await page.getByPlaceholder('Barcode').fill(`BAR-${unique}`);
  await page.getByPlaceholder('Candidate Name').fill(`Candidate ${unique}`);
  await page.locator('select').first().selectOption({ index: 1 });
  await page.getByRole('button', { name: 'Create' }).click();
  await expectToast(page, 'Candidate created');
  await page.getByText(`BAR-${unique}`, { exact: false }).waitFor({ timeout: 30000 });
}

async function createSessionAndAssignmentFlow(page, unique) {
  await page.getByRole('link', { name: 'Exams' }).click();
  await page.getByRole('heading', { name: 'Exams' }).waitFor({ timeout: 30000 });
  await page.getByPlaceholder('Template Name').fill('Template A');
  await page.getByPlaceholder('Duration Minutes').fill('75');
  await page.getByPlaceholder('Starts (MM/DD/YYYY hh:mm AM/PM)').fill('04/10/2026 09:00 AM');
  await page.getByPlaceholder('Ends (MM/DD/YYYY hh:mm AM/PM)').fill('04/10/2026 10:15 AM');
  await page.getByRole('button', { name: 'Create Exam Session' }).click();
  await expectToast(page, 'Exam session created');

  const sessionsResp = await api(page, '/sessions?page=1&limit=200');
  assert.equal(sessionsResp.status(), 200);
  const sessions = await sessionsResp.json();
  const createdSession = sessions.find((row) => row.duration_minutes === 75 && row.template_name === 'Template A');
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
  await page.getByRole('heading', { name: 'Sessions' }).waitFor({ timeout: 30000 });
  await page.getByText(createdSession.id, { exact: false }).waitFor({ timeout: 30000 });

  await logout(page);
  await login(page, creds.admin);
  return createdSession.id;
}

async function outputsAndExportFlow(page, sessionId) {
  await page.goto(`${FRONTEND_URL}/outputs`, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByRole('heading', { name: 'Outputs' }).waitFor({ timeout: 30000 });
  await page.selectOption('select', { label: sessionId });
  const selects = page.locator('select');
  await selects.nth(1).selectOption('AdmitCard');
  await selects.nth(2).selectOption('TestPrint');
  await page.getByRole('button', { name: 'Generate Output' }).click();
  await expectToast(page, 'Output generated');
  await page.getByText(sessionId, { exact: false }).waitFor({ timeout: 30000 });

  await page.getByRole('link', { name: 'Reports' }).click();
  await page.getByRole('heading', { name: 'Reports' }).waitFor({ timeout: 30000 });
  await page.getByRole('button', { name: 'Export Incident CSV' }).click();
  await expectToast(page, 'CSV ready:');
}

async function dashboardLoadFlow(page) {
  await page.goto(`${FRONTEND_URL}/dashboard`, { waitUntil: 'networkidle', timeout: 120000 });
  await page.getByRole('heading', { name: 'Dashboard' }).waitFor({ timeout: 30000 });
  await page.getByText('Upcoming Sessions', { exact: false }).waitFor({ timeout: 30000 });
  await page.getByText('Recent Outputs', { exact: false }).waitFor({ timeout: 30000 });
  await page.getByText('Seat Utilization Trend', { exact: false }).waitFor({ timeout: 30000 });
}

async function roleRestrictedFlow(page) {
  await logout(page);
  await login(page, creds.auditor);
  await page.getByRole('heading', { name: 'Dashboard' }).waitFor({ timeout: 30000 });
  await page.getByRole('link', { name: 'Outputs' }).waitFor({ state: 'detached', timeout: 10000 }).catch(() => {});
  assert.equal(await page.getByRole('link', { name: 'Outputs' }).count(), 0, 'auditor must not see Outputs nav');
  await page.goto(`${FRONTEND_URL}/admin`, { waitUntil: 'networkidle', timeout: 120000 });
  await expectToast(page, 'HTTP 403');
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  try {
    const unique = `${Date.now()}`;
    await login(page, creds.admin);
    await dashboardLoadFlow(page);
    await createCandidateFlow(page, unique);
    const sessionId = await createSessionAndAssignmentFlow(page, unique);
    await outputsAndExportFlow(page, sessionId);
    await roleRestrictedFlow(page);
  } finally {
    await browser.close();
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

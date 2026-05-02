const { spawn } = require('node:child_process');
const path = require('node:path');

process.env.FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:8080';
process.env.BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8001/api/v1';
process.env.ADMIN_USERNAME = process.env.ADMIN_USERNAME || 'admin_local';
process.env.ADMIN_PASSWORD = process.env.ADMIN_PASSWORD || 'AdminPass#2026!';
process.env.COORD_USERNAME = process.env.COORD_USERNAME || 'coord_local';
process.env.COORD_PASSWORD = process.env.COORD_PASSWORD || 'CoordPass#2026!';
process.env.PROCTOR_USERNAME = process.env.PROCTOR_USERNAME || 'proctor_local';
process.env.PROCTOR_PASSWORD = process.env.PROCTOR_PASSWORD || 'ProctorPass#2026!';
process.env.AUDITOR_USERNAME = process.env.AUDITOR_USERNAME || 'auditor_local';
process.env.AUDITOR_PASSWORD = process.env.AUDITOR_PASSWORD || 'AuditorPass#2026!';

const cli = require.resolve('@playwright/test/cli', { paths: [__dirname, process.cwd()] });
const child = spawn(process.execPath, [
  cli,
  'test',
  '--config',
  path.join(__dirname, 'playwright.config.js'),
], {
  stdio: 'inherit',
  env: process.env,
});

child.on('exit', (code) => process.exit(code ?? 1));
child.on('error', (err) => {
  console.error(err);
  process.exit(1);
});

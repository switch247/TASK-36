const { spawn } = require('node:child_process');
const path = require('node:path');

const cli = require.resolve('@playwright/test/cli', {
  paths: [
    __dirname,
    process.cwd(),
    '/opt/test-deps/node_modules',
  ],
});

const args = [
  cli,
  'test',
  '--config',
  path.join(__dirname, 'playwright.config.js'),
];

const child = spawn(process.execPath, args, {
  stdio: 'inherit',
  env: process.env,
});

child.on('exit', (code) => process.exit(code ?? 1));
child.on('error', (err) => {
  console.error(err);
  process.exit(1);
});

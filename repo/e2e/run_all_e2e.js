const { spawn } = require('node:child_process');
const path = require('node:path');

function resolvePlaywrightCli() {
  const explicit = process.env.PLAYWRIGHT_TEST_CLI;
  if (explicit) {
    return explicit;
  }

  const candidates = [
    ['@playwright/test/cli', [__dirname, process.cwd(), '/opt/test-deps/node_modules']],
    ['@playwright/test/package.json', [__dirname, process.cwd(), '/opt/test-deps/node_modules']],
  ];

  for (const [request, paths] of candidates) {
    try {
      const resolved = require.resolve(request, { paths });
      if (request.endsWith('package.json')) {
        return path.join(path.dirname(resolved), 'cli.js');
      }
      return resolved;
    } catch (_) {
      // Try the next resolution strategy.
    }
  }

  return '/opt/test-deps/node_modules/@playwright/test/cli.js';
}

const cli = resolvePlaywrightCli();

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

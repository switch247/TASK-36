const { spawn } = require('node:child_process');
const path = require('node:path');

const scripts = [
  'fullstack_e2e.js',
  'role_matrix_e2e.js',
];

async function run(script) {
  await new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [path.join(__dirname, script)], {
      stdio: 'inherit',
      env: process.env,
    });
    child.on('exit', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${script} failed with exit code ${code}`));
      }
    });
    child.on('error', reject);
  });
}

async function main() {
  for (const script of scripts) {
    await run(script);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

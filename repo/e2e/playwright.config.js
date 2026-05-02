const { defineConfig } = require('@playwright/test');

module.exports = defineConfig({
  testDir: __dirname,
  testMatch: /.*_e2e\.js$/,
  testIgnore: [
    /helpers\.js$/,
    /run_all_e2e\.js$/,
    /run_local_e2e\.js$/,
  ],
  reporter: 'line',
  workers: 1,
  timeout: 120000,
  use: {
    headless: true,
  },
});

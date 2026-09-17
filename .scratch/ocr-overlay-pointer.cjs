const { chromium } = require(process.argv[2]);
const assert = require('node:assert/strict');
(async () => {
  const browser = await chromium.launch({ channel: 'chrome', headless: true });
  try {
    for (const scale of [1, 1.25, 1.5, 2]) {
      const page = await browser.newPage({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: scale, reducedMotion: 'no-preference' });
      await page.addInitScript(() => {
        window.__TAURI_INTERNALS__ = {
          transformCallback: (callback) => { window.ocrStart = callback; return 1; },
          invoke: async (command) => command === 'plugin:event|listen' ? 1 : undefined,
        };
      });
      await page.goto('http://127.0.0.1:5173/?window=ocr-overlay');
      await page.waitForFunction(() => typeof window.ocrStart === 'function');
      await page.evaluate(() => window.ocrStart({ payload: { requestId: 'pointer-test' } }));
      await page.mouse.move(100, 150);
      await page.mouse.down();
      await page.mouse.move(400, 300);
      await page.waitForTimeout(180);
      const rect = await page.locator('.selection').boundingBox();
      console.log(JSON.stringify({ scale, expected: { x: 100, y: 150, width: 300, height: 150 }, actual: rect }));
      for (const [key, value] of Object.entries({ x: 100, y: 150, width: 300, height: 150 })) {
        assert.ok(Math.abs(rect[key] - value) < 0.5, `Held drag ${key}: expected ${value}, got ${rect[key]}`);
      }
      await page.close();
    }
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });

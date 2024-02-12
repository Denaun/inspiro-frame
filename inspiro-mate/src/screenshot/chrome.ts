import puppeteer from "npm:puppeteer@22";

const browser = await puppeteer.launch({
  headless: true,
  ignoreHTTPSErrors: true,
});

export async function takeScreenshot(
  url: string,
  width: number,
  height: number,
): Promise<Uint8Array> {
  const ctx = await browser.createBrowserContext();
  try {
    const page = await ctx.newPage();
    await Promise.all([
      await page.setViewport({ width, height }),
      await page.goto(url, { waitUntil: "networkidle0" }),
    ]);
    return await page.screenshot();
  } finally {
    await ctx.close();
  }
}

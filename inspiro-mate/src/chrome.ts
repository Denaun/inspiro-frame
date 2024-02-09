import { type Buffer } from "https://deno.land/std@0.212.0/streams/mod.ts";
import {
  type Context,
  type Next,
  type RouterMiddleware,
  Status,
} from "https://deno.land/x/oak@v13.0.0/mod.ts";
import puppeteer from "npm:puppeteer@22";

export async function screenshotMiddleware<
  R extends string,
>(): Promise<RouterMiddleware<R>> {
  const browser = await puppeteer.launch({
    headless: true,
    ignoreHTTPSErrors: true,
  });

  return async (context: Context, _next: Next): Promise<void> => {
    const url = context.request.url.searchParams.get("url");
    context.assert(url, Status.BadRequest, "Missing URL");
    const width = Number(context.request.url.searchParams.get("width") ?? 0);
    const height = Number(context.request.url.searchParams.get("height") ?? 0);

    const ctx = await browser.createBrowserContext();
    let data: Buffer;
    try {
      const page = await ctx.newPage();
      await Promise.all([
        await page.setViewport({ width, height }),
        await page.goto(decodeURIComponent(url), { waitUntil: "networkidle0" }),
      ]);
      data = await page.screenshot();
    } finally {
      await ctx.close();
    }

    context.response.type = "png";
    context.response.body = data;
  };
}

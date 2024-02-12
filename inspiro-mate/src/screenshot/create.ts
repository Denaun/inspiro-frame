import {
  type Context,
  type Next,
  type RouterMiddleware,
  Status,
} from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { takeScreenshot } from "./chrome.ts";
import { insertScreenshot } from "./db.ts";

export function createScreenshotMiddleware<
  R extends string,
>(): RouterMiddleware<R> {
  return async (context: Context, _next: Next) => {
    const { url, width, height } = await context.request.body.json();
    context.assert(url, Status.BadRequest, "Missing URL");
    const data = await takeScreenshot(
      url,
      Number(width) || 0,
      Number(height) || 0,
    );
    const id = await insertScreenshot(data);
    context.response.body = id;
  };
}

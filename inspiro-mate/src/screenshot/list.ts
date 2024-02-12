import {
  type Context,
  type Next,
  type RouterMiddleware,
} from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { getAllScreenshots } from "./db.ts";

export function listScreenshotsMiddleware<
  R extends string,
>(): RouterMiddleware<R> {
  return async (context: Context, _next: Next) => {
    context.response.body = await getAllScreenshots();
  };
}

import {
  type Context,
  type Next,
  type RouterMiddleware,
  Status,
} from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { getQuery } from "https://deno.land/x/oak@v13.0.0/helpers.ts";
import { deleteScreenshotById } from "./db.ts";

export function deleteScreenshotMiddleware<
  R extends string,
>(): RouterMiddleware<R> {
  return async (context: Context, _next: Next) => {
    const { id } = getQuery(context, { mergeParams: true });
    await deleteScreenshotById(id);
    context.response.status = Status.OK;
  };
}

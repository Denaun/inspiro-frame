import {
  type Context,
  type Next,
  type RouterMiddleware,
  Status,
} from "https://deno.land/x/oak@v13.0.0/mod.ts";

const DEFAULT_ENDPOINT = "https://inspirobot.me";

export function inspiroBotMiddleware<R extends string>(
  endpoint = DEFAULT_ENDPOINT,
): RouterMiddleware<R> {
  return async (context: Context, _next: Next): Promise<void> => {
    const response = await fetch(`${endpoint}/api?generate=true`);
    if (!response.ok) {
      context.throw(
        Status.InternalServerError,
        `generate failed, got ${response}`,
      );
    }
    context.response.redirect(new URL(await response.text()));
  };
}

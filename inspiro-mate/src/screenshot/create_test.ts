import {
  assertEquals,
  assertExists,
  assertRejects,
} from "https://deno.land/std@0.213.0/assert/mod.ts";
import {
  HttpError,
  Request,
  Status,
  testing,
} from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { createScreenshotMiddleware } from "./create.ts";
import { getScreenshotById } from "./db.ts";
import { readableStreamFromIterable } from "https://deno.land/std@0.129.0/streams/conversion.ts";

Deno.test(
  "createScreenshotMiddleware",
  // TODO: denoland/deno#15425 - Remove.
  { sanitizeOps: false, sanitizeResources: false },
  async (t) => {
    const middleware = createScreenshotMiddleware();

    await t.step("stores the screenshot", async () => {
      const context = createMockPostContext({
        url: "chrome://about",
        width: 6,
        height: 4,
      });

      await middleware(context, testing.createMockNext());

      assertEquals(context.response.status, Status.OK);
      assertExists(context.response.body);
      assertEquals(typeof context.response.body, "string");
      assertExists(getScreenshotById(context.response.body as string));
    });

    await t.step("requires the URL", () => {
      const context = createMockPostContext({});

      assertRejects(
        async () => await middleware(context, testing.createMockNext()),
        HttpError,
      );
    });
  },
);

function createMockPostContext(body: object) {
  const context = testing.createMockContext({ method: "POST" });
  context.request = new Request({
    headers: context.request.headers,
    method: context.request.method,
    remoteAddr: context.request.ip,
    url: context.request.url.toString(),
    error: (_) => {},
    respond: (_) => Promise.resolve(),
    getBody: () =>
      readableStreamFromIterable([
        new TextEncoder().encode(JSON.stringify(body)),
      ]),
  });
  return context;
}

import { assertEquals } from "https://deno.land/std@0.213.0/assert/mod.ts";
import { decode } from "https://deno.land/x/imagescript@1.2.17/mod.ts";
import { Status, testing } from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { screenshotMiddleware } from "./chrome.ts";

Deno.test(
  "screenshotMiddleware",
  // TODO: denoland/deno#15425 - Remove.
  { sanitizeOps: false, sanitizeResources: false },
  async (t) => {
    const middleware = await screenshotMiddleware();

    await t.step("renders pages", async () => {
      const context = testing.createMockContext({
        path: "?width=200&height=250&url=chrome://about",
      });

      await middleware(context, testing.createMockNext());

      assertEquals(context.response.status, Status.OK);
      assertEquals(context.response.type, "png");
      assertEquals(
        await decode(context.response.body),
        await decode(Deno.readFileSync("src/goldens/chrome_about.png")),
      );
    });
  },
);

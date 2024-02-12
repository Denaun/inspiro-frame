import {
  assertEquals,
  assertRejects,
} from "https://deno.land/std@0.213.0/assert/mod.ts";
import { decode } from "https://deno.land/x/imagescript@1.2.17/mod.ts";
import {
  HttpError,
  Status,
  testing,
} from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { insertScreenshot } from "./db.ts";
import { getScreenshotMiddleware } from "./get.ts";

Deno.test("getScreenshotMiddleware", async (t) => {
  const middleware = getScreenshotMiddleware();

  await t.step("gets the screenshot", async () => {
    const data = Deno.readFileSync("src/screenshot/goldens/chrome_about.png");
    const id = await insertScreenshot(data);
    const context = testing.createMockContext({
      params: { id },
    });

    await middleware(context, testing.createMockNext());

    assertEquals(context.response.status, Status.OK);
    assertEquals(
      await decode(context.response.body),
      await decode(data),
    );
  });

  await t.step("crops screenshots", async () => {
    const data = Deno.readFileSync("src/screenshot/goldens/chrome_about.png");
    const id = await insertScreenshot(data);
    const context = testing.createMockContext({
      params: { id },
      path: "?x=25&y=50&width=20&height=30",
    });

    await middleware(context, testing.createMockNext());

    assertEquals(context.response.status, Status.OK);
    assertEquals(
      await decode(context.response.body),
      await decode(
        Deno.readFileSync("src/screenshot/goldens/chrome_about_crop.png"),
      ),
    );
  });

  await t.step({
    name: "converts to BWR",
    // TODO: denoland/deno#21686 - Enable.
    ignore: true,
    fn: async () => {
      const data = Deno.readFileSync("src/screenshot/goldens/chrome_about.png");
      const id = await insertScreenshot(data);
      const context = testing.createMockContext({
        params: { id, format: "bwr" },
      });

      await middleware(context, testing.createMockNext());

      assertEquals(context.response.status, Status.OK);
      assertEquals(
        await decode(context.response.body),
        await decode(
          Deno.readFileSync("src/screenshot/goldens/chrome_about_bwr.png"),
        ),
      );
    },
  });

  await t.step("rejects invalid IDs", () => {
    const id = "my-id";
    const context = testing.createMockContext({
      params: { id },
    });

    assertRejects(
      async () => await middleware(context, testing.createMockNext()),
      HttpError,
    );

    assertEquals(context.response.status, Status.NotFound);
  });
});

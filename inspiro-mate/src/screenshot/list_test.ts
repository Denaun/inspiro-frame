import { assertEquals } from "https://deno.land/std@0.213.0/assert/mod.ts";
import { Status, testing } from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { listScreenshotsMiddleware } from "./list.ts";
import { clear, insertScreenshot } from "./db.ts";

Deno.test("listScreenshotMiddleware", async (t) => {
  await clear();
  const middleware = listScreenshotsMiddleware();

  await t.step("lists the screenshots", async () => {
    const id1 = await insertScreenshot(new Uint8Array());
    const id2 = await insertScreenshot(new Uint8Array());
    const context = testing.createMockContext();

    await middleware(context, testing.createMockNext());

    assertEquals(context.response.status, Status.OK);
    assertEquals(context.response.body, [id1, id2]);
  });
});

import {
  assertEquals,
  assertFalse,
} from "https://deno.land/std@0.213.0/assert/mod.ts";
import { Status, testing } from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { deleteScreenshotMiddleware } from "./delete.ts";
import { getAllScreenshots, insertScreenshot } from "./db.ts";

Deno.test("deleteScreenshotMiddleware", async (t) => {
  const middleware = deleteScreenshotMiddleware();

  await t.step("deletes the screenshot", async () => {
    const id = await insertScreenshot(new TextEncoder().encode("test"));
    const context = testing.createMockContext({
      method: "DELETE",
      params: { id },
    });

    await middleware(context, testing.createMockNext());

    assertEquals(context.response.status, Status.OK);
    assertFalse((await getAllScreenshots()).includes(id));
  });

  await t.step("ignores invalid IDs", async () => {
    const id = "my-id";
    const context = testing.createMockContext({
      method: "DELETE",
      params: { id },
    });

    await middleware(context, testing.createMockNext());

    assertEquals(context.response.status, Status.OK);
  });
});

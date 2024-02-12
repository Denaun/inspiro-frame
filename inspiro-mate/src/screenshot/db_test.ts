import { assertEquals } from "https://deno.land/std@0.213.0/assert/mod.ts";
import { getScreenshotById, insertScreenshot } from "./db.ts";

Deno.test("db", async (t) => {
  await t.step("can store and retrieve large screenshots", async () => {
    const data = randomBytes(1_00_000);

    const id = await insertScreenshot(data);
    const actual = await getScreenshotById(id);

    assertEquals(actual, data);
  });
});

function randomBytes(length: number): Uint8Array {
  const data = new Uint8Array(length);
  for (let i = 0; i < length; i++) {
    data[i] = Math.floor(Math.random() * 256);
  }
  return data;
}

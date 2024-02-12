import { assertEquals } from "https://deno.land/std@0.213.0/assert/mod.ts";
import { decode } from "https://deno.land/x/imagescript@1.2.17/mod.ts";
import { takeScreenshot } from "./chrome.ts";

Deno.test(
  "takeScreenshot",
  // TODO: denoland/deno#15425 - Remove.
  { sanitizeOps: false, sanitizeResources: false },
  async (t) => {
    await t.step("renders pages", async () => {
      const data = await takeScreenshot("chrome://about", 200, 250);

      assertEquals(
        await decode(data),
        await decode(
          Deno.readFileSync("src/screenshot/goldens/chrome_about.png"),
        ),
      );
    });
  },
);

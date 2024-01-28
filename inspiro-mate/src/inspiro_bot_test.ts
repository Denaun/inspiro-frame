import {
  assertEquals,
  assertRejects,
} from "https://deno.land/std@0.213.0/assert/mod.ts";
import { MockFetch } from "https://deno.land/x/deno_mock_fetch@1.0.1/mod.ts";
import {
  HttpError,
  Status,
  testing,
} from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { inspiroBotMiddleware } from "./inspiro_bot.ts";

const mockFetch = new MockFetch();
const ENDPOINT = "https://my.endpoi.nt";

Deno.test("InspiroBotMiddleware", async (t) => {
  const middleware = inspiroBotMiddleware(ENDPOINT);

  await t.step("redirects to the generated image", async () => {
    mockFetch
      .intercept(`${ENDPOINT}/api?generate=true`, { method: "GET" })
      .response(`${ENDPOINT}/some-image.jpg`, { status: Status.OK });

    const context = testing.createMockContext();
    await middleware(context, testing.createMockNext());

    assertEquals(context.response.status, Status.Found);
    assertEquals(
      context.response.headers.get("Location"),
      `${ENDPOINT}/some-image.jpg`,
    );
  });

  await t.step("wraps errors", () => {
    mockFetch
      .intercept(`${ENDPOINT}/api?generate=true`, { method: "GET" })
      .response(`${ENDPOINT}/some-image.jpg`, { status: Status.Forbidden });

    assertRejects(
      async () =>
        await middleware(testing.createMockContext(), testing.createMockNext()),
      HttpError,
    );
  });

  await t.step("rejects invalid URLs", () => {
    mockFetch
      .intercept(`${ENDPOINT}/api?generate=true`, { method: "GET" })
      .response("not-a-url", { status: Status.OK });

    assertRejects(async () =>
      await middleware(testing.createMockContext(), testing.createMockNext())
    );
  });
});

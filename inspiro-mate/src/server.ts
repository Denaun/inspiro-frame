import {
  Application,
  type Context,
  type Next,
  Router,
  Status,
} from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { inspiroBotMiddleware } from "./inspiro_bot.ts";
import {
  createScreenshotMiddleware,
  deleteScreenshotMiddleware,
  getScreenshotMiddleware,
  listScreenshotsMiddleware,
  takeScreenshot,
} from "./screenshot/mod.ts";
import { getQuery } from "https://deno.land/x/oak@v13.0.0/helpers.ts";

const router = new Router();
router.get("/inspiration", inspiroBotMiddleware());
router.get("/screenshot", async (context: Context, _next: Next) => {
  const { url, width, height } = getQuery(context, {
    mergeParams: true,
  });
  context.assert(url, Status.BadRequest, "Missing URL");
  context.response.body = await takeScreenshot(
    decodeURIComponent(url),
    Number(width) || 0,
    Number(height) || 0,
  );
});
router.get("/screenshots", listScreenshotsMiddleware());
router.post("/screenshots", createScreenshotMiddleware());
router.get("/screenshots/:id", getScreenshotMiddleware());
router.delete("/screenshots/:id", deleteScreenshotMiddleware());

const app = new Application({ state: { screenshots: new Map() } });
app.use(router.routes());
app.use(router.allowedMethods());

await app.listen({ port: 8000 });

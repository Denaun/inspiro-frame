import { Application, Router } from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { screenshotMiddleware } from "./chrome.ts";
import { inspiroBotMiddleware } from "./inspiro_bot.ts";

const router = new Router();
router.get("/inspiration", inspiroBotMiddleware());
router.get("/screenshot", await screenshotMiddleware());

const app = new Application();
app.use(router.routes());
app.use(router.allowedMethods());

await app.listen({ port: 8000 });

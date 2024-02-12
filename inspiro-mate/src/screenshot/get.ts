import {
  type Context,
  type Next,
  type RouterMiddleware,
  Status,
} from "https://deno.land/x/oak@v13.0.0/mod.ts";
import { ditherInplace, packBuffer } from "./image.ts";
import { getQuery } from "https://deno.land/x/oak@v13.0.0/helpers.ts";
import { getScreenshotById } from "./db.ts";
import sharp from "npm:sharp@0.33.2";
import { Buffer } from "https://deno.land/std@0.215.0/io/buffer.ts";

export function getScreenshotMiddleware<
  R extends string,
>(): RouterMiddleware<R> {
  return async (context: Context, _next: Next) => {
    const { id, x, y, width, height, format } = getQuery(context, {
      mergeParams: true,
    });
    const screenshot = await getScreenshotById(id);
    context.assert(screenshot, Status.NotFound);
    let image = sharp(screenshot);
    if (format == "bwr" || format == "bwr-raw") {
      const { data, info } = await image
        .toColorspace("srgb").raw()
        .toBuffer({ resolveWithObject: true });
      ditherInplace(data, info, [black, white, red]);
      image = sharp(data, { raw: info });
    }
    if (x || y || width || height) {
      try {
        image.extract({
          left: Number(x) || 0,
          top: Number(y) || 0,
          width: Number(width) || 0,
          height: Number(height) || 0,
        });
      } catch (error) {
        context.throw(Status.BadRequest, error);
      }
    }
    if (format == "bwr-raw") {
      const { data, info } = await image.raw()
        .toBuffer({ resolveWithObject: true });
      const body = new Buffer();
      body.write(packBuffer(data, info, white));
      body.write(packBuffer(data, info, red));
      context.response.body = body.bytes({ copy: false });
    } else {
      context.response.body = await image.png().toBuffer();
    }
  };
}

const black = { r: 0, g: 0, b: 0 };
const red = { r: 255, g: 0, b: 0 };
const white = { r: 255, g: 255, b: 255 };

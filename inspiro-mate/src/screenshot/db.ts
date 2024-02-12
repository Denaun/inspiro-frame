import { Buffer } from "https://deno.land/std@0.215.0/io/buffer.ts";
import { gunzip, gzip } from "https://deno.land/x/compress@v0.4.5/mod.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";

const kv = await Deno.openKv();

const kvValueSize = 65_536;
const expireIn = Temporal.Duration.from({ minutes: 5 }).total("millisecond");

export async function getAllScreenshots(): Promise<
  string[]
> {
  const ids = new Set<string>();
  for await (
    const chunk of kv.list<Uint8Array>({ prefix: ["screenshots"] })
  ) {
    ids.add(chunk.key[1] as string);
  }
  return Array.from(ids);
}

export async function getScreenshotById(
  id: string,
): Promise<Uint8Array | null> {
  const data: Buffer = new Buffer();
  for await (
    const chunk of kv.list<Uint8Array>({ prefix: ["screenshots", id] })
  ) {
    data.writeSync(chunk.value);
  }
  return data.empty() ? null : gunzip(data.bytes());
}

export async function insertScreenshot(data: Uint8Array): Promise<string> {
  const id = ulid();
  await Promise.all(
    chunked(gzip(data), kvValueSize).map(async (chunk, index) => {
      const key = ["screenshots", id, index];
      await kv.set(key, chunk, { expireIn });
    }),
  );
  return id;
}

export async function deleteScreenshotById(id: string) {
  const operation = kv.atomic();
  for await (
    const chunk of kv.list<Uint8Array>({ prefix: ["screenshots", id] })
  ) {
    operation.delete(chunk.key);
  }
  await operation.commit();
}

export async function clear() {
  for await (const chunk of kv.list<Uint8Array>({ prefix: [] })) {
    await kv.delete(chunk.key);
  }
}

function chunked(data: Uint8Array, chunkSize: number): Uint8Array[] {
  const chunks = [data.slice(0, chunkSize)];
  let offset = chunkSize;
  while (offset < data.length) {
    chunks.push(data.slice(offset, offset + chunkSize));
    offset += chunkSize;
  }
  return chunks;
}

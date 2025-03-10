FROM denoland/deno:bin-1.42.4 AS deno
FROM ghcr.io/puppeteer/puppeteer:22
COPY --from=deno /deno /usr/local/bin/deno

EXPOSE 8000

WORKDIR /app
COPY src .
RUN deno cache server.ts

CMD ["deno", "run", "-A", "--unstable-kv", "--unstable-temporal", "server.ts"]


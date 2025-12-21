import { Hono } from "hono";
import { auth } from "./lib/auth.js"; // path to your auth file
import { serve } from "@hono/node-server";
import { cors } from "hono/cors";
import { etag } from "hono/etag";
import { logger } from "hono/logger";

const app = new Hono();
app.use(etag(), logger());

app.get("/api/health", (c) => {
  return c.json({
    success: true,
    uptime: process.uptime(),
    timestamp: new Date().toISOString(),
  });
});

// app.on(["POST", "GET"], "/api/auth/*", (c) => auth.handler(c.req.raw));

let server = serve({ fetch: app.fetch, port: 3000 });

process.on("SIGINT", () => {
  server.close();
  process.exit(0);
});

process.on("SIGTERM", () => {
  server.close((err) => {
    if (err) {
      console.error(err);
      process.exit(1);
    }
    process.exit(0);
  });
});

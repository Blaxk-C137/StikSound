import { createServer } from "node:http";
import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const BUILD_DIRECTORY = fileURLToPath(new URL("../apps/web/dist/", import.meta.url));
const MIME_TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".webmanifest": "application/manifest+json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  ".ico": "image/x-icon",
  ".wasm": "application/wasm",
  ".woff": "font/woff",
  ".woff2": "font/woff2"
};

export function createStaticServer(directory = BUILD_DIRECTORY) {
  const root = path.resolve(directory);
  return createServer(async (request, response) => {
    response.setHeader("X-Content-Type-Options", "nosniff");
    const fail = (status, message) => {
      response.writeHead(status, {
        "Content-Type": "text/plain; charset=utf-8",
        "Content-Length": Buffer.byteLength(message)
      });
      response.end(message);
    };
    if (request.method !== "GET" && request.method !== "HEAD") {
      response.setHeader("Allow", "GET, HEAD");
      fail(405, "Method not allowed");
      return;
    }

    let pathname;
    try {
      pathname = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
      if (pathname.includes("\0") || pathname.includes("\\")) throw new Error("Invalid path");
    } catch {
      fail(400, "Invalid URL");
      return;
    }
    let file = path.resolve(root, `.${pathname}`);
    if (file !== root && !file.startsWith(root + path.sep)) {
      fail(403, "Forbidden");
      return;
    }

    try {
      let info = await stat(file).catch(() => null);
      if (!info?.isFile()) {
        if (path.extname(pathname) || pathname.startsWith("/assets/") || pathname.startsWith("/icons/")) {
          fail(404, "File not found");
          return;
        }
        file = path.join(root, "index.html");
        info = await stat(file);
      }
      response.writeHead(200, {
        "Content-Type": MIME_TYPES[path.extname(file)] ?? "application/octet-stream",
        "Content-Length": info.size,
        "Cache-Control": pathname.startsWith("/assets/")
          ? "public, max-age=31536000, immutable"
          : "no-cache"
      });
      if (request.method === "HEAD") {
        response.end();
        return;
      }
      const stream = createReadStream(file);
      stream.on("error", () => response.destroy());
      response.on("close", () => stream.destroy());
      stream.pipe(response);
    } catch {
      fail(500, "Unable to serve the app. Run npm run build first.");
    }
  });
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const port = Number(process.env.PORT ?? 3000);
  if (!Number.isInteger(port) || port < 0 || port > 65535) {
    console.error("PORT must be a valid port number.");
    process.exitCode = 1;
  } else if (!(await stat(path.join(BUILD_DIRECTORY, "index.html")).catch(() => null))?.isFile()) {
    console.error("Build files not found. Run npm run build before npm start.");
    process.exitCode = 1;
  } else {
    const server = createStaticServer();
    server.on("error", (error) => {
      console.error(`Unable to start blaxkstik: ${error.message}`);
      process.exitCode = 1;
    });
    server.listen(port, "0.0.0.0", () => {
      console.log(`blaxkstik listening on port ${server.address().port}`);
    });
  }
}

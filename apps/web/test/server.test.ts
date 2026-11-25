import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { IncomingMessage, ServerResponse, type Server } from "node:http";
import { PassThrough } from "node:stream";
import type { Socket } from "node:net";
import { createStaticServer } from "../../../tools/server.mjs";

describe("production web server", () => {
  let directory: string;
  let server: Server;

  async function request(route: string, method = "GET"): Promise<Response> {
    // Exercise Node's real HTTP request/response objects without requiring a listening port.
    const socket = new PassThrough();
    const incoming = new IncomingMessage(socket as unknown as Socket);
    incoming.url = route;
    incoming.method = method;
    incoming.httpVersionMajor = 1;
    incoming.httpVersionMinor = 1;
    const outgoing = new ServerResponse(incoming);
    outgoing.assignSocket(socket as unknown as Socket);
    const chunks: Buffer[] = [];
    socket.on("data", (chunk) => chunks.push(Buffer.from(chunk)));
    await new Promise<void>((resolve, reject) => {
      outgoing.once("finish", resolve);
      outgoing.once("error", reject);
      server.emit("request", incoming, outgoing);
    });
    const raw = Buffer.concat(chunks);
    const boundary = raw.indexOf("\r\n\r\n");
    const lines = raw.subarray(0, boundary).toString().split("\r\n");
    const status = Number(lines.shift()!.split(" ")[1]);
    const headers = new Headers();
    for (const line of lines) {
      const colon = line.indexOf(":");
      headers.set(line.slice(0, colon), line.slice(colon + 1).trim());
    }
    socket.destroy();
    return new Response(method === "HEAD" ? null : raw.subarray(boundary + 4), { status, headers });
  }

  beforeAll(async () => {
    directory = await mkdtemp(path.join(tmpdir(), "blaxkstik-server-"));
    await mkdir(path.join(directory, "assets"));
    await writeFile(path.join(directory, "index.html"), "<h1>blaxkstik</h1>");
    await writeFile(path.join(directory, "assets/app-123.js"), "console.log('blaxkstik');");
    await writeFile(path.join(directory, "sw.js"), "self.addEventListener('fetch', () => {});");
    server = createStaticServer(directory);
  });

  afterAll(async () => {
    if (directory) await rm(directory, { recursive: true, force: true });
  });

  it("serves the app and HTML navigation routes", async () => {
    for (const route of ["/", "/editor/example?tab=settings"]) {
      const response = await request(route);
      expect(response.status).toBe(200);
      expect(response.headers.get("content-type")).toBe("text/html; charset=utf-8");
      expect(await response.text()).toBe("<h1>blaxkstik</h1>");
    }
  });

  it("serves JavaScript with a valid MIME type and keeps deployment files fresh", async () => {
    const asset = await request("/assets/app-123.js");
    expect(asset.headers.get("content-type")).toBe("text/javascript; charset=utf-8");
    expect(asset.headers.get("cache-control")).toBe("public, max-age=31536000, immutable");
    expect(asset.headers.get("x-content-type-options")).toBe("nosniff");
    expect(await asset.text()).toBe("console.log('blaxkstik');");
    const worker = await request("/sw.js");
    expect(worker.headers.get("cache-control")).toBe("no-cache");
  });

  it("returns 404 for missing assets instead of the app HTML", async () => {
    const response = await request("/assets/missing.js");
    expect(response.status).toBe(404);
    expect(await response.text()).not.toContain("<h1>");
  });

  it("handles HEAD without a body and rejects write requests", async () => {
    const head = await request("/", "HEAD");
    expect(head.status).toBe(200);
    expect(await head.text()).toBe("");
    const post = await request("/", "POST");
    expect(post.status).toBe(405);
    expect(post.headers.get("allow")).toBe("GET, HEAD");
  });

  it("rejects encoded path traversal and malformed URLs", async () => {
    expect((await request("/%2e%2e%2fprivate.txt")).status).toBe(403);
    expect((await request("/%E0%A4%A")).status).toBe(400);
  });
});

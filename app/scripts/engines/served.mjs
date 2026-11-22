import { spawn } from "node:child_process";
import { dirname, join, relative, resolve } from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import { createServer, normalizePath } from "vite";

const here = dirname(fileURLToPath(import.meta.url));
const app = resolve(here, "..", "..");
const repository = dirname(app);

const BRIDGE = ["run", "-q", "-p", "tisty-markdown", "--example", "placing"];

const core = () => {
  const bridge = spawn("cargo", BRIDGE, { cwd: repository, stdio: ["pipe", "pipe", "inherit"] });
  const waiting = [];
  let stopping = false;
  let broken = null;
  const ended = (why) => {
    if (stopping || broken) return;
    broken = new Error(`the Markdown core (cargo ${BRIDGE.join(" ")}) ${why}`);
    for (const one of waiting.splice(0)) one.failed(broken);
  };
  bridge.on("error", (error) => ended(`could not start: ${error.message}`));
  bridge.on("close", (code, signal) => ended(`stopped with ${signal ?? `exit code ${code}`}`));
  bridge.stdin.on("error", (error) => ended(`stopped listening: ${error.message}`));
  createInterface({ input: bridge.stdout }).on("line", (line) => waiting.shift()?.done(line));
  let queue = Promise.resolve();
  const asked = (question) => {
    const answer = queue.then(
      () =>
        new Promise((done, failed) => {
          if (broken) return failed(broken);
          waiting.push({ done, failed });
          bridge.stdin.write(`${question.replace(/\n/g, "")}\n`);
        }),
    );
    queue = answer.catch(() => {});
    return answer;
  };
  const stop = () => {
    stopping = true;
    bridge.stdin.end();
    bridge.kill();
  };
  return { asked, stop };
};

export const questionOf = async (request) => {
  const parts = [];
  for await (const part of request) parts.push(part);
  return Buffer.concat(parts).toString("utf8");
};

const shell = (entry) =>
  `<!doctype html><html><head><meta charset="utf-8"></head><body style="margin:0;background:#fff"><div id="root"></div><script type="module" src="/${normalizePath(relative(app, join(here, entry)))}"></script></body></html>`;

export const served = async (entry = "page.ts") => {
  const bridge = core();
  const answering = {
    name: "engines",
    configureServer(server) {
      server.middlewares.use(async (request, response, next) => {
        if (request.url === "/bridge" && request.method === "POST") {
          const question = await questionOf(request);
          const said = await bridge.asked(question).catch((error) => JSON.stringify({ refused: error.message }));
          response.writeHead(200, { "content-type": "application/json" }).end(said);
          return;
        }
        if (request.url === "/engines.html") {
          const html = await server.transformIndexHtml(request.url, shell(entry));
          response.writeHead(200, { "content-type": "text/html" }).end(html);
          return;
        }
        next();
      });
    },
  };
  const server = await createServer({
    configFile: join(app, "vite.config.ts"),
    root: app,
    plugins: [answering],
    server: { port: 0, strictPort: false, hmr: false, fs: { allow: [repository] } },
    logLevel: "silent",
  });
  await server.listen();
  const { port } = server.httpServer.address();
  return {
    url: `http://localhost:${port}/engines.html`,
    close: async () => {
      bridge.stop();
      await server.close();
    },
  };
};

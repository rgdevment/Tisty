import { spawn } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const here = dirname(fileURLToPath(import.meta.url));
const app = resolve(here, "..", "..");
const repository = dirname(app);

const core = () => {
  const bridge = spawn("cargo", ["run", "-q", "-p", "tisty-markdown", "--example", "placing"], {
    cwd: repository,
    stdio: ["pipe", "pipe", "inherit"],
  });
  const waiting = [];
  createInterface({ input: bridge.stdout }).on("line", (line) => waiting.shift()?.(line));
  let queue = Promise.resolve();
  const asked = (question) => {
    const answer = queue.then(
      () =>
        new Promise((done) => {
          waiting.splice(waiting.length, 0, done);
          bridge.stdin.write(`${question.replace(/\n/g, "")}\n`);
        }),
    );
    queue = answer.catch(() => {});
    return answer;
  };
  return { asked, stop: () => bridge.kill() };
};

const shell = `<!doctype html><html><head><meta charset="utf-8"></head><body style="margin:0;background:#fff"><div id="root"></div><script type="module" src="/@fs${here}/page.ts"></script></body></html>`;

export const served = async () => {
  const bridge = core();
  const answering = {
    name: "engines",
    configureServer(server) {
      server.middlewares.use(async (request, response, next) => {
        if (request.url === "/bridge" && request.method === "POST") {
          let question = "";
          for await (const chunk of request) question += chunk;
          const said = await bridge.asked(question);
          response.writeHead(200, { "content-type": "application/json" }).end(said);
          return;
        }
        if (request.url === "/engines.html") {
          const html = await server.transformIndexHtml(request.url, shell);
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

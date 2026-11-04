import { spawn } from "node:child_process";
import { once } from "node:events";
import { existsSync, mkdtempSync, readFileSync, statSync } from "node:fs";
import { rm } from "node:fs/promises";
import { createServer as serving } from "node:http";
import { tmpdir } from "node:os";
import { extname, join } from "node:path";
import { text } from "node:stream/consumers";

const CHROMES = [
  process.env.CHROME_BIN,
  "/usr/bin/google-chrome",
  "/usr/bin/chromium",
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "C:/Program Files/Google/Chrome/Application/chrome.exe",
  "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
];
const TYPES = { ".js": "text/javascript", ".woff2": "font/woff2" };

const stopped = async (browser) => {
  if (browser.pid === undefined || browser.exitCode !== null || browser.signalCode !== null) return;
  const gone = once(browser, "exit");
  browser.kill();
  await gone;
};

// What only a real browser lays out is asked of one: it opens the page, reads /sources and answers at /drawn, or says why not at /failed.
export const drawnInChrome = async (sources, files, page, what, within = 120_000) => {
  const chrome = CHROMES.find((one) => one && existsSync(one));
  if (!chrome) throw new Error(`${what} are drawn in Chrome: none found, name one in CHROME_BIN`);
  let drawn;
  let failed;
  const answer = new Promise((resolve, reject) => {
    drawn = resolve;
    failed = reject;
  });
  const served = async (request, response) => {
    const { pathname } = new URL(request.url, "http://localhost");
    const file = join(files, pathname);
    if (pathname === "/") response.writeHead(200, { "content-type": "text/html" }).end(page);
    else if (pathname === "/sources") response.end(JSON.stringify(sources));
    else if (pathname === "/drawn") {
      const said = await text(request);
      response.writeHead(204).end();
      drawn(JSON.parse(said));
    } else if (pathname === "/failed") {
      const said = await text(request);
      response.writeHead(204).end();
      failed(new Error(`Chrome could not draw ${what}: ${said}`));
    } else if (existsSync(file) && statSync(file).isFile())
      response.writeHead(200, { "content-type": TYPES[extname(file)] ?? "application/octet-stream" }).end(readFileSync(file));
    else response.writeHead(404).end();
  };
  const server = serving((request, response) => served(request, response).catch(failed));
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const profile = mkdtempSync(join(tmpdir(), "tisty-guide-"));
  // The page is this script's own, served from this machine, and a runner may not allow Chrome its sandbox.
  const flags = ["--headless=new", "--no-sandbox", "--disable-gpu", "--no-first-run", "--window-size=800,600", `--user-data-dir=${profile}`];
  let browser = null;
  const clock = setTimeout(() => failed(new Error(`Chrome did not finish ${what}`)), within);
  try {
    browser = spawn(chrome, [...flags, `http://127.0.0.1:${server.address().port}/`], { stdio: "ignore" });
    browser.once("error", failed);
    browser.once("exit", () => failed(new Error(`Chrome closed before drawing ${what}`)));
    return await answer;
  } finally {
    clearTimeout(clock);
    server.closeAllConnections();
    server.close();
    if (browser) await stopped(browser);
    // Chrome's helpers can outlive it, and what they leave in the temp folder is no reason to fail a build.
    await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 }).catch(() => {});
  }
};

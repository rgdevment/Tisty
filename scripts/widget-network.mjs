#!/usr/bin/env node
import { spawn, spawnSync } from "node:child_process";
import dgram from "node:dgram";
import fs from "node:fs";
import http from "node:http";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const UDP = { stun: 41001, "turn-udp": 41002, "ice-direct-udp": 41022, "stun-in-a-frame": 41024 };
const TCP = {
  "turn-tcp": 41003,
  turns: 41004,
  preconnect: 41005,
  "ice-direct-tcp": 41023,
  "preconnect-in-a-frame": 41025,
  "turn-tcp-in-a-frame": 41026,
  "preconnect-in-markup": 41027,
};
const TCP_OVER_IPV6 = { "preconnect-ipv6": 41020, "turn-tcp-ipv6": 41021 };
const HTTP = 41010;
const FIRST_DEBUG_PORT = 9471;
const WIDGET_TAKES = 12000;

const repo = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const BY_HAND = process.argv[2] === "--by-hand";
const appExe = process.argv[2] ?? path.join(repo, "target", "debug", "tisty-gui.exe");

const lanAddress = () =>
  Object.values(os.networkInterfaces())
    .flat()
    .find((one) => one.family === "IPv4" && !one.internal)?.address;

const sleep = (ms) => new Promise((done) => setTimeout(done, ms));

const hostile = (canary, token) => `<p>hostile</p>
<link rel="preconnect" href="http://${canary}:${TCP["preconnect-in-markup"]}">
<link rel="dns-prefetch" href="http://dns-in-markup-${token}.invalid">
<script>
(() => {
  const C = ${JSON.stringify(canary)};
  const seen = {};
  const note = (k, v) => { seen[k] = v; };
  const url = (name) => "http://" + C + ":${HTTP}/" + name;
  const ice = (label, server) => {
    const pc = new RTCPeerConnection({ iceServers: [server] });
    pc.onicecandidate = (e) => { if (!e.candidate) note(label, "gathered"); };
    pc.createDataChannel("x");
    pc.createOffer().then((o) => pc.setLocalDescription(o));
  };
  const relay = (urls) => ({ urls, username: "u", credential: "p" });
  const head = (rel, href) => {
    const link = document.createElement("link");
    link.rel = rel;
    link.href = href;
    document.head.append(link);
  };
  note("rtc", typeof RTCPeerConnection);
  ice("stun", { urls: "stun:" + C + ":${UDP.stun}" });
  ice("turn-udp", relay("turn:" + C + ":${UDP["turn-udp"]}?transport=udp"));
  ice("turn-tcp", relay("turn:" + C + ":${TCP["turn-tcp"]}?transport=tcp"));
  ice("turns", relay("turns:" + C + ":${TCP.turns}?transport=tcp"));
  ice("turn-tcp-ipv6", relay("turn:[::1]:${TCP_OVER_IPV6["turn-tcp-ipv6"]}?transport=tcp"));
  head("preconnect", "http://" + C + ":${TCP.preconnect}");
  head("preconnect", "http://[::1]:${TCP_OVER_IPV6["preconnect-ipv6"]}");
  head("preconnect", "http://localhost:${TCP_OVER_IPV6["preconnect-ipv6"]}");
  head("dns-prefetch", "http://dns-in-script-${token}.invalid");
  (async () => {
    const pc = new RTCPeerConnection();
    pc.createDataChannel("direct");
    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);
    const candidates = "a=candidate:1 1 udp 2122260223 " + C + " ${UDP["ice-direct-udp"]} typ host\\r\\n" +
      "a=candidate:2 1 tcp 1518280447 " + C + " ${TCP["ice-direct-tcp"]} typ host tcptype passive\\r\\n";
    await pc.setRemoteDescription({ type: "answer", sdp: offer.sdp.replace("a=setup:actpass", "a=setup:active") + candidates });
  })().catch((why) => note("direct", String(why)));
  const inner = "<!doctype html><script>" +
    "const a = new RTCPeerConnection({ iceServers: [{ urls: 'stun:" + C + ":${UDP["stun-in-a-frame"]}' }] });" +
    "const b = new RTCPeerConnection({ iceServers: [{ urls: 'turn:" + C + ":${TCP["turn-tcp-in-a-frame"]}?transport=tcp', username: 'u', credential: 'p' }] });" +
    "for (const pc of [a, b]) { pc.createDataChannel('x'); pc.createOffer().then((o) => pc.setLocalDescription(o)); }" +
    "const l = document.createElement('link'); l.rel = 'preconnect'; l.href = 'http://" + C + ":${TCP["preconnect-in-a-frame"]}'; document.head.append(l);" +
    "</scr" + "ipt>";
  const frame = document.createElement("iframe");
  frame.src = URL.createObjectURL(new Blob([inner], { type: "text/html" }));
  document.body.append(frame);
  for (const send of [
    () => fetch(url("fetch"), { mode: "no-cors" }).catch(() => {}),
    () => navigator.sendBeacon(url("beacon"), "x"),
    () => new WebSocket("ws://" + C + ":${HTTP}/websocket"),
    () => { new Image().src = url("img"); },
    () => new Worker(url("worker")),
    () => head("stylesheet", url("stylesheet")),
    () => head("prefetch", url("prefetch")),
  ]) { try { send(); } catch {} }
  setTimeout(() => parent.postMessage({ probe: true, seen }, "*"), 8000);
  setTimeout(() => { location.href = url("navigated"); }, 9000);
})();
</script>`;

const listening = (reached) => {
  const heard = (vector) => () => {
    reached[vector] = (reached[vector] ?? 0) + 1;
  };
  const sockets = [];
  for (const [vector, port] of Object.entries(UDP)) {
    const socket = dgram.createSocket("udp4");
    socket.on("message", heard(vector));
    socket.bind(port, lan);
    sockets.push(socket);
  }
  const overTcp = (ports, at) => {
    for (const [vector, port] of Object.entries(ports)) {
      const server = net.createServer((socket) => {
        heard(vector)();
        socket.on("error", () => {});
        socket.on("data", () => {});
      });
      server.listen(port, at);
      sockets.push(server);
    }
  };
  overTcp(TCP, lan);
  overTcp(TCP_OVER_IPV6, "::1");
  const web = http.createServer((request, answer) => {
    heard(`http ${request.url}`)();
    answer.end();
  });
  web.on("connection", heard("any connection to the web port"));
  web.listen(HTTP, lan);
  sockets.push(web);
  return () => {
    for (const one of sockets) one.close();
  };
};

const debugged = async (target) => {
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((open, failed) => {
    socket.onopen = open;
    socket.onerror = failed;
  });
  let last = 0;
  const waiting = new Map();
  socket.onmessage = (message) => {
    const said = JSON.parse(message.data);
    waiting.get(said.id)?.(said);
    waiting.delete(said.id);
  };
  const send = (method, params) =>
    new Promise((answered) => {
      last += 1;
      waiting.set(last, answered);
      socket.send(JSON.stringify({ id: last, method, params }));
    });
  return {
    run: async (expression) => {
      const said = await send("Runtime.evaluate", {
        expression,
        awaitPromise: true,
        returnByValue: true,
      });
      if (said.result?.exceptionDetails) throw new Error(JSON.stringify(said.result.exceptionDetails));
      return said.result?.result?.value;
    },
    close: () => socket.close(),
  };
};

const pages = async (port) => {
  try {
    const all = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
    return all.filter((one) => one.type === "page" && /localhost:1420|tauri\.localhost/.test(one.url));
  } catch {
    return [];
  }
};

const until = async (found, tries = 240) => {
  for (let at = 0; at < tries; at += 1) {
    const one = await found();
    if (one) return one;
    await sleep(500);
  }
  throw new Error("the window never came up");
};

const shownInItsOwnWindow = async (port, opener, lent) => {
  const before = new Set((await pages(port)).map((one) => one.id));
  await opener.run(`window.__TAURI_INTERNALS__.invoke("view_widget", { lent: ${JSON.stringify(lent)} })`);
  const view = await until(async () => (await pages(port)).find((one) => !before.has(one.id)));
  const page = await debugged(view);
  await page.run(`window.__heard = []; addEventListener("message", (e) => { if (e.data?.probe) window.__heard.push(e.data.seen); }); 0`);
  return page;
};

const netlogSaw = (file) => {
  const text = fs.readFileSync(file, "utf8").trimEnd().replace(/,\s*$/, "");
  const log = JSON.parse(text.endsWith("]}") ? text : `${text}]}`);
  const named = Object.fromEntries(Object.entries(log.constants.logEventTypes).map(([name, id]) => [id, name]));
  const of = (name, field) => [
    ...new Set(log.events.filter((one) => named[one.type] === name && one.params?.[field]).map((one) => one.params[field])),
  ];
  return {
    dns: of("DNS_TRANSACTION_QUERY", "qname"),
    tcp: of("TCP_CONNECT", "remote_address"),
    udp: of("UDP_BYTES_SENT", "address"),
  };
};

const trial = async (name, port, extraArgs) => {
  const room = fs.mkdtempSync(path.join(os.tmpdir(), `tisty-widget-network-${name}-`));
  const netlog = path.join(room, "net.json");
  const pageFile = path.join(room, "hostile.html");
  const reached = {};
  const stopListening = listening(reached);
  const token = Date.now().toString(36);
  const body = hostile(lan, token);
  fs.writeFileSync(pageFile, `<!doctype html><html><head></head><body>${body}</body></html>`);
  const running = spawn(appExe, [], {
    stdio: "ignore",
    env: {
      ...process.env,
      TISTY_PROFILE: "widget-network",
      TISTY_DATA: path.join(room, "data"),
      TISTY_CONFIG: path.join(room, "config"),
      TISTY_CACHE: path.join(room, "cache"),
      WEBVIEW2_USER_DATA_FOLDER: path.join(room, "webview"),
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: [`--remote-debugging-port=${port}`, `--log-net-log=${netlog}`, ...extraArgs].join(" "),
    },
  });
  try {
    const first = await until(async () => (await pages(port))[0]);
    const opener = await debugged(first);
    await until(() => opener.run(`document.readyState === "complete" && Boolean(window.__TAURI_INTERNALS__)`));
    const written = await opener.run(`window.__TAURI_INTERNALS__.invoke("attach", { path: ${JSON.stringify(pageFile)} })`);
    const reference = written.slice(written.lastIndexOf("(") + 1, written.lastIndexOf(")"));
    const page = await shownInItsOwnWindow(port, opener, { reference });
    await sleep(WIDGET_TAKES);
    const said = await page.run("window.__heard");
    for (const one of [page, opener]) one.close();
    return { reached, said };
  } finally {
    spawnSync("taskkill", ["/F", "/T", "/PID", String(running.pid)], { stdio: "ignore" });
    stopListening();
    await sleep(1000);
    if (fs.existsSync(netlog)) reachedByTheEngine.set(name, netlogSaw(netlog));
    fs.rmSync(room, { recursive: true, force: true, maxRetries: 5, retryDelay: 500 });
  }
};

const openedByHand = async () => {
  const token = Date.now().toString(36);
  const page = path.join(os.tmpdir(), "tisty-hostile-page.html");
  fs.writeFileSync(page, `<!doctype html><html><head></head><body>${hostile(lan, token)}</body></html>`);
  const reached = {};
  const stopListening = listening(reached);
  console.log(`Attach ${page} to a document in Tisty, draw it live, and wait 15 seconds.`);
  console.log(`What the listeners cannot see goes to the DNS server: sudo tcpdump -n -l -i any port 53 | grep ${token}`);
  console.log("Then press Enter.");
  await new Promise((pressed) => process.stdin.once("data", pressed));
  stopListening();
  console.log("reached:", reached);
  process.exit(Object.keys(reached).length ? 1 : 0);
};

const reachedByTheEngine = new Map();
const lan = lanAddress();
if (!lan) {
  console.error("This machine has no network address for the page to aim at.");
  process.exit(2);
}
if (BY_HAND) await openedByHand();
if (process.platform !== "win32") {
  console.error("Only WebView2 can be driven from here: on macOS run this with --by-hand.");
  process.exit(2);
}
if (!fs.existsSync(appExe)) {
  console.error(`No build at ${appExe}. Run: npm --prefix app run build && cargo build -p tisty-gui --features tauri/custom-protocol`);
  process.exit(2);
}

const ran = (trialResult) => trialResult.said?.[0]?.rtc === "function";

const closed = await trial("closed", FIRST_DEBUG_PORT, []);
const reopened = await trial("reopened", FIRST_DEBUG_PORT + 1, ["--webrtc-ip-handling-policy=default", '--host-resolver-rules=""']);
const engine = reachedByTheEngine.get("closed");
const silent = engine && !engine.dns.length && !engine.tcp.length && !engine.udp.length;

console.log("as the app starts it, reached:", closed.reached, "and its engine saw:", engine);
console.log("with the arguments undone, reached:", reopened.reached);

const failures = [
  !ran(closed) && "the hostile code never ran, so nothing was proved",
  Object.keys(closed.reached).length > 0 && "a page reached the network",
  !silent && "the engine opened a connection of its own",
  !Object.keys(reopened.reached).length && "with the arguments undone nothing got out either, so this test proves nothing",
].filter(Boolean);
for (const one of failures) console.error(`x  ${one}`);
process.exit(failures.length ? 1 : 0);

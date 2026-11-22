const QUIET_MS = 40;
const PATIENCE_MS = 4000;
const CALM_PATIENCE_MS = 10_000;

const watchedIn = ([quiet, patience, calmPatience]) => {
  const tauri = window.__TAURI_INTERNALS__;
  if (!tauri.paced) {
    const inner = tauri.invoke;
    tauri.paced = true;
    tauri.invoke = (cmd, args, options) => {
      const own = window.paced?.asking(cmd, args);
      return inner(cmd, args, options).finally(() => own?.());
    };
  }
  const frame = document.querySelector("#root iframe");
  const sheet = frame.contentDocument.querySelector(".page");
  const view = frame.contentWindow;
  const bell = new MessageChannel();
  const bytes = new TextEncoder();
  const WHOLE = new Set(["doc_placed", "doc_drawn"]);
  let current = null;
  let asking = 0;
  let ringing = false;
  let stirred = performance.now();

  const freshKey = (key) => ({
    key,
    at: performance.now(),
    last: 0,
    changes: 0,
    painted: 0,
    paintedAt: 0,
    waits: [],
    whole: 0,
    sent: 0,
    blocks: 0,
    nodes: 0,
    attributes: 0,
  });

  const ring = () => {
    if (ringing || !current) return;
    ringing = true;
    view.requestAnimationFrame(() =>
      bell.port2.postMessage({ key: current?.at, changes: current?.changes }),
    );
  };
  bell.port1.onmessage = ({ data }) => {
    ringing = false;
    if (!current) return;
    if (data.key === current.at) {
      current.painted = data.changes;
      current.paintedAt = performance.now();
    }
    if (current.painted < current.changes) ring();
  };

  view.addEventListener(
    "keydown",
    (event) => {
      current = freshKey(event.key);
    },
    true,
  );

  const counted = new MutationObserver((records) => {
    stirred = performance.now();
    if (!current) return;
    for (const one of records) {
      if (one.type === "attributes") current.attributes += 1;
      if (one.type !== "childList") continue;
      const moved = one.addedNodes.length + one.removedNodes.length;
      current.nodes += moved;
      if (one.target === sheet) current.blocks += moved;
    }
    current.changes += 1;
    sheet.getBoundingClientRect();
    current.last = performance.now();
    ring();
  });
  counted.observe(frame.contentDocument.documentElement, {
    childList: true,
    subtree: true,
    attributes: true,
    characterData: true,
  });

  const waited = (waits, from, to) => {
    let total = 0;
    let reached = from;
    for (const [began, ended] of [...waits].sort((one, other) => one[0] - other[0])) {
      const start = Math.max(began, reached);
      const end = Math.min(ended, to);
      if (end > start) total += end - start;
      reached = Math.max(reached, end);
    }
    return total;
  };

  const settled = () =>
    new Promise((done) => {
      const began = performance.now();
      const look = () => {
        const one = current;
        const now = performance.now();
        const still = one && one.changes > 0 && asking === 0 && one.painted === one.changes;
        if (still && now - one.last >= quiet) {
          current = null;
          const { key, whole, sent, blocks, nodes, attributes } = one;
          const ms = one.paintedAt - one.at;
          const core = waited(one.waits, one.at, one.last);
          const work = one.last - one.at - core;
          return done({ key, ms, work, core, whole, sent, blocks, nodes, attributes });
        }
        if (now - began > patience) {
          current = null;
          return done({ key: one?.key ?? null, unsettled: true });
        }
        setTimeout(look, 5);
      };
      look();
    });

  const calm = (still) =>
    new Promise((done) => {
      const began = performance.now();
      const look = () => {
        const now = performance.now();
        if (now - stirred >= still) return done(true);
        if (now - began > calmPatience) return done(false);
        setTimeout(look, 20);
      };
      look();
    });

  window.paced = {
    settled,
    calm,
    asking: (cmd, args) => {
      const one = current;
      const wait = [performance.now(), Number.POSITIVE_INFINITY];
      asking += 1;
      if (one) {
        one.waits.push(wait);
        one.sent += bytes.encode(JSON.stringify(args ?? null)).length;
        if (WHOLE.has(cmd)) one.whole += 1;
      }
      return () => {
        asking -= 1;
        wait[1] = performance.now();
      };
    },
  };
};

export const watched = (page) => page.evaluate(watchedIn, [QUIET_MS, PATIENCE_MS, CALM_PATIENCE_MS]);

export const calmed = (page, still) => page.evaluate((ms) => window.paced.calm(ms), still);

export const keyed = async (page, key) => {
  if (key.length === 1) await page.keyboard.type(key);
  else await page.keyboard.press(key);
  return page.evaluate(() => window.paced.settled());
};

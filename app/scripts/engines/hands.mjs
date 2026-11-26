export const KEY_MS = 80;
const BUSY_MS = 250;
const HELD_PATIENCE_MS = 10_000;

export const changeOf = (was, now) => {
  let from = 0;
  while (from < was.length && was[from] === now[from]) from++;
  let back = 0;
  while (back < was.length - from && was[was.length - 1 - back] === now[now.length - 1 - back]) back++;
  return { was: was.slice(from, was.length - back), now: now.slice(from, now.length - back) };
};

export const macIn = (page) => page.evaluate(() => navigator.userAgent.includes("Macintosh"));

const OPTION_WORD_KEYS = new Set(["Backspace", "Delete", "ArrowLeft", "ArrowRight"]);
const CLIPBOARD_KEY = /(?:^|\+)(?:Meta|Control)(?:\+(?:Alt|Shift))*\+[cvx]$|^Shift\+Delete$|^(?:Control|Shift)\+Insert$/i;

const macOnlyKey = (key) => {
  const parts = key.split("+");
  return parts.includes("Meta") || (parts.includes("Alt") && OPTION_WORD_KEYS.has(parts.at(-1)));
};

const busy = (page, held) =>
  page.evaluate(([on, ms]) => {
    window.unbusied ??= window.setTimeout;
    window.setTimeout = on ? (job, wait, ...rest) => window.unbusied(job, wait || ms, ...rest) : window.unbusied;
  }, [held, BUSY_MS]);

export const handsOn = (page, frame) => {
  let mac;
  const onMac = async () => (mac ??= await macIn(page));
  const clickAt = async (words, before = false) => {
    await page.waitForFunction(() => window.engines.drawn());
    const spot = await frame.evaluate(
      ([said, early]) => {
        const walk = document.createTreeWalker(document.querySelector(".page"), NodeFilter.SHOW_TEXT);
        for (let node = walk.nextNode(); node; node = walk.nextNode()) {
          const at = node.data.indexOf(said);
          if (at < 0) continue;
          const shown = node.parentElement.getBoundingClientRect();
          if (shown.top < 0 || shown.bottom > innerHeight) node.parentElement.scrollIntoView({ block: "center" });
          const range = document.createRange();
          const end = early ? at : at + said.length;
          range.setStart(node, early ? end : end - 1);
          range.setEnd(node, early ? end + 1 : end);
          const box = range.getBoundingClientRect();
          return { x: early ? box.left + 0.5 : box.right - 0.5, y: box.top + box.height / 2 };
        }
        return null;
      },
      [words, before],
    );
    if (!spot) throw new Error(`no "${words}" on the page`);
    const sheet = await (await frame.frameElement()).boundingBox();
    await page.mouse.click(sheet.x + spot.x, sheet.y + spot.y);
    await page.waitForTimeout(150);
  };
  const typed = async (said) => {
    for (const letter of said) {
      await page.keyboard.type(letter);
      await page.waitForTimeout(KEY_MS);
    }
  };
  const pressable = async (key) => {
    if (CLIPBOARD_KEY.test(key))
      throw new Error(`${key} would reach the system's clipboard: a case calls cut() or copied(), and paste is tried in vitest`);
    if (macOnlyKey(key) && !(await onMac()))
      throw new Error(`${key} is a Mac key: off a Mac a case presses command() or wordBack(), or is macOnly`);
  };
  const press = async (key, many = 1) => {
    await pressable(key);
    for (let time = 0; time < many; time++) {
      await page.keyboard.press(key);
      await page.waitForTimeout(KEY_MS);
    }
  };
  const busyTyped = async (keys, said) => {
    for (const key of keys) await pressable(key);
    await busy(page, true);
    let lastKey = Date.now();
    try {
      for (const key of keys) await page.keyboard.press(key);
      for (const letter of said) await page.keyboard.type(letter);
      lastKey = Date.now();
    } finally {
      await busy(page, false);
    }
    await page.waitForTimeout(Math.max(KEY_MS, BUSY_MS - (Date.now() - lastKey)));
  };
  const drawingsHeld = async (run) => {
    await page.evaluate(() => window.engines.hold());
    try {
      await run();
    } finally {
      await page.evaluate(() => window.engines.letGo());
    }
    await page.waitForFunction(() => window.engines.drawn());
  };
  const oneDrawingLet = async () => {
    const before = await page.evaluate(() => {
      if (window.engines.holding() === 0) return null;
      const told = window.engines.answered();
      window.engines.letOne();
      return told;
    });
    if (before === null) throw new Error("no drawing of the core was held to let through");
    const patience = { timeout: HELD_PATIENCE_MS };
    await page.waitForFunction((told) => window.engines.answered() > told, before, patience);
    await page.waitForFunction(
      () => window.engines.holding() > 0 || window.engines.drawn(),
      null,
      patience,
    );
  };
  const command = async (key) => press(`${(await onMac()) ? "Meta" : "Control"}+${key}`);
  const clipped = async (kind) => {
    const carried = await frame.evaluate((type) => {
      const held = new DataTransfer();
      const sheet = document.querySelector(".page");
      sheet.dispatchEvent(new ClipboardEvent(type, { clipboardData: held, bubbles: true, cancelable: true }));
      // The page carries its own Markdown under its own type, and readable words as plain text.
      return held.getData("text/x-tisty-markdown") || held.getData("text/plain");
    }, kind);
    await page.waitForTimeout(KEY_MS);
    return carried;
  };
  const cut = () => clipped("cut");
  const copied = () => clipped("copy");
  const wordBack = async () => press((await onMac()) ? "Alt+Backspace" : "Control+Backspace");
  const wordForward = async () => press((await onMac()) ? "Alt+Delete" : "Control+Delete");
  return {
    clickAt,
    typed,
    press,
    busyTyped,
    drawingsHeld,
    oneDrawingLet,
    command,
    cut,
    copied,
    wordBack,
    wordForward,
    onMac,
    page,
    frame,
  };
};

export const opened = async (page, url) => {
  // replaces the entry instead of adding one: the app window has no page behind it for a Back to walk to
  await Promise.all([page.waitForEvent("load"), page.evaluate((to) => location.replace(to), url)]);
  await page.waitForFunction(() => window.enginesReady === true, null, { timeout: 120_000 });
  if ((await page.evaluate(() => history.length)) !== 1) throw new Error("the page has a page behind it, which the app window does not");
};

// With `native` the page lets the browser write plain words itself, as the switch does in the app.
export const launched = async (type, url, launch = {}, native = false) => {
  const browser = await type.launch({ headless: true, ...launch });
  try {
    const page = await browser.newPage({ viewport: { width: 1000, height: 1400 } });
    if (native) await page.addInitScript(() => localStorage.setItem("tisty.nativeTyping", "everywhere"));
    await opened(page, url);
    return { browser, page };
  } catch (error) {
    await browser.close();
    throw error;
  }
};

export const sheetOf = async (page, doc) => {
  await page.evaluate((one) => window.engines.open(one), doc);
  return page.frames().find((other) => other !== page.mainFrame() && !other.isDetached());
};

export const KEY_MS = 80;

export const changeOf = (was, now) => {
  let from = 0;
  while (from < was.length && was[from] === now[from]) from++;
  let back = 0;
  while (back < was.length - from && was[was.length - 1 - back] === now[now.length - 1 - back]) back++;
  return { was: was.slice(from, was.length - back), now: now.slice(from, now.length - back) };
};

export const macIn = (page) => page.evaluate(() => navigator.userAgent.includes("Macintosh"));

const OPTION_WORD_KEYS = new Set(["Backspace", "Delete", "ArrowLeft", "ArrowRight"]);

const macOnlyKey = (key) => {
  const parts = key.split("+");
  return parts.includes("Meta") || (parts.includes("Alt") && OPTION_WORD_KEYS.has(parts.at(-1)));
};

export const handsOn = (page, frame) => {
  let mac;
  const onMac = async () => (mac ??= await macIn(page));
  const clickAt = async (words, before = false) => {
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
  const press = async (key, many = 1) => {
    if (macOnlyKey(key) && !(await onMac()))
      throw new Error(`${key} is a Mac key: off a Mac a case presses command() or wordBack(), or is macOnly`);
    for (let time = 0; time < many; time++) {
      await page.keyboard.press(key);
      await page.waitForTimeout(KEY_MS);
    }
  };
  const command = async (key) => press(`${(await onMac()) ? "Meta" : "Control"}+${key}`);
  const wordBack = async () => press((await onMac()) ? "Alt+Backspace" : "Control+Backspace");
  return { clickAt, typed, press, command, wordBack, page, frame };
};

export const opened = async (page, url) => {
  await page.goto(url);
  await page.waitForFunction(() => window.enginesReady === true, null, { timeout: 120_000 });
};

export const launched = async (type, url, launch = {}) => {
  const browser = await type.launch({ headless: true, ...launch });
  try {
    const page = await browser.newPage({ viewport: { width: 1000, height: 1400 } });
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

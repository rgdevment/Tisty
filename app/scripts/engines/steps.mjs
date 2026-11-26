export const randomOf = (seed) => {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let mixed = Math.imul(state ^ (state >>> 15), state | 1);
    mixed ^= mixed + Math.imul(mixed ^ (mixed >>> 7), mixed | 61);
    return ((mixed ^ (mixed >>> 14)) >>> 0) / 4294967296;
  };
};

const pick = (random, from) => from[Math.floor(random() * from.length)];

const LETTERS = [..."abcdefghijklmnopqrstuvwxyz"];
const ACCENTED = [..."áéíóúñüÁÉç"];
const WIDE = [..."漢字日本語中文"];
const SIGNS = [..."*_`[]#-"];
const MARKS = ["# ", "## ", "- ", "* ", "> ", "1. ", "[ ] ", "```", "`"];
const ARROWS = ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"];
const WORD_ARROWS = ["Alt+ArrowLeft", "Alt+ArrowRight", "Meta+ArrowLeft", "Meta+ArrowRight"];
const CHOOSING = ["Shift+ArrowLeft", "Shift+ArrowRight", "Shift+ArrowUp", "Shift+ArrowDown", "Alt+Shift+ArrowLeft"];
const DELETES = new Set(["Backspace", "Delete", "Alt+Backspace", "Meta+Backspace", "Meta+z", "Meta+Shift+z"]);

const clickOf = (random) => ["click", Number(random().toFixed(3)), Number(random().toFixed(3))];
const UNDO = ["press", "Meta+z"];
const REDO = ["press", "Meta+Shift+z"];
const timesOf = (random, most, step) => Array.from({ length: 1 + Math.floor(random() * most) }, () => step);
const one = (made) => (random) => [made(random)];

const KINDS = [
  [30, one((random) => ["type", pick(random, LETTERS)])],
  [7, one(() => ["type", " "])],
  [4, one((random) => ["type", pick(random, ACCENTED)])],
  [3, one((random) => ["type", pick(random, WIDE)])],
  [3, one((random) => ["type", pick(random, SIGNS)])],
  [6, one(() => ["press", "Enter"])],
  [2, one(() => ["press", "Shift+Enter"])],
  [7, one(() => ["press", "Backspace"])],
  [3, one(() => ["press", "Delete"])],
  [2, one(() => ["press", "Alt+Backspace"])],
  [1, one(() => ["press", "Meta+Backspace"])],
  [10, one((random) => ["press", pick(random, ARROWS)])],
  [4, one((random) => ["press", pick(random, WORD_ARROWS)])],
  [2, one((random) => ["press", pick(random, ["Home", "End"])])],
  [
    4,
    one((random) => {
      const how = pick(random, CHOOSING);
      const many = 1 + Math.floor(random() * 5);
      return random() < 0.5
        ? ["choose", how, many, "type", pick(random, LETTERS)]
        : ["choose", how, many, "press", pick(random, ["Backspace", "Delete"])];
    }),
  ],
  [5, one((random) => ["mark", pick(random, MARKS)])],
  [3, one((random) => ["press", random() < 0.7 ? "Tab" : "Shift+Tab"])],
  [3, (random) => [...timesOf(random, 3, UNDO), ...(random() < 0.5 ? timesOf(random, 2, REDO) : [])]],
  [1, one(() => REDO)],
  [3, one(clickOf)],
  [2, one((random) => ["beside", Number(random().toFixed(3))])],
];

const WEIGHT = KINDS.reduce((sum, [weight]) => sum + weight, 0);

const stepsOf = (random) => {
  let left = random() * WEIGHT;
  for (const [weight, made] of KINDS) {
    left -= weight;
    if (left < 0) return made(random);
  }
  return KINDS[0][1](random);
};

export const stepsFor = (seed, start, count) => {
  const random = randomOf(seed);
  const steps = start.doc ? [clickOf(random)] : [];
  while (steps.length < count) steps.push(...stepsOf(random));
  return steps.slice(0, count);
};

export const deletes = ([kind, key]) => kind === "choose" || (kind === "press" && DELETES.has(key));

const spotOf = ([across, along, beside]) => {
  const sheet = document.querySelector(".page");
  if (!sheet) return null;
  const shown = (node) => node.data.trim() && node.parentElement.checkVisibility?.() !== false;
  const chips = () =>
    [...sheet.querySelectorAll("span.tag[data-src], :not(pre) > code[data-src]")]
      .map((one) => one.firstChild)
      .filter((node) => node?.nodeType === Node.TEXT_NODE && shown(node));
  const words = () => {
    const found = [];
    const walk = document.createTreeWalker(sheet, NodeFilter.SHOW_TEXT);
    for (let node = walk.nextNode(); node; node = walk.nextNode()) {
      const holder = node.parentElement;
      if (!holder?.closest("[data-src]") || holder.closest("[contenteditable='false'], button, .table")) continue;
      if (shown(node)) found.push(node);
    }
    return found;
  };
  const all = beside ? chips() : words();
  if (!all.length) return null;
  const node = all[Math.min(all.length - 1, Math.floor(across * all.length))];
  const at = beside ? 0 : Math.min(node.data.length - 1, Math.floor(along * node.data.length));
  const range = document.createRange();
  range.setStart(node, at);
  range.setEnd(node, at + 1);
  const seen = range.getBoundingClientRect();
  if (seen.top < 0 || seen.bottom > innerHeight) node.parentElement.scrollIntoView({ block: "center" });
  const box = range.getBoundingClientRect();
  if (!box.width || !box.height) return null;
  return { x: beside ? box.left + 0.5 : box.left + box.width / 2, y: box.top + box.height / 2 };
};

const clickedAt = async (page, frame, where) => {
  const spot = await frame.evaluate(spotOf, where);
  if (!spot) return;
  const sheet = await (await frame.frameElement()).boundingBox();
  await page.mouse.click(sheet.x + spot.x, sheet.y + spot.y);
  await page.waitForTimeout(150);
};

// Off a Mac the same moves have other keys, and a delete to the line's start is a choice to it and a Backspace.
const OFF_MAC = {
  "Meta+z": ["Control+z"],
  "Meta+Shift+z": ["Control+y"],
  "Meta+ArrowLeft": ["Home"],
  "Meta+ArrowRight": ["End"],
  "Alt+ArrowLeft": ["Control+ArrowLeft"],
  "Alt+ArrowRight": ["Control+ArrowRight"],
  "Alt+Shift+ArrowLeft": ["Control+Shift+ArrowLeft"],
  "Alt+Backspace": ["Control+Backspace"],
  "Meta+Backspace": ["Shift+Home", "Backspace"],
};

const pressed = async ({ press, onMac }, key, many = 1) => {
  const keys = (await onMac()) ? [key] : (OFF_MAC[key] ?? [key]);
  for (let time = 0; time < many; time++) for (const one of keys) await press(one);
};

const TAKEN = {
  type: ({ typed }, [said]) => typed(said),
  press: (hands, [key]) => pressed(hands, key),
  mark: async (hands, [said]) => {
    await pressed(hands, "Meta+ArrowLeft");
    await hands.typed(said);
  },
  choose: async (hands, [how, many, then, what]) => {
    await pressed(hands, how, many);
    await (then === "type" ? hands.typed(what) : pressed(hands, what));
  },
  click: ({ page, frame }, [across, along]) => clickedAt(page, frame, [across, along, false]),
  beside: ({ page, frame }, [across]) => clickedAt(page, frame, [across, 0, true]),
};

export const taken = (hands, [kind, ...rest]) => TAKEN[kind](hands, rest);

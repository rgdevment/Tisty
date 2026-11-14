import { afterEach, describe, expect, it } from "vitest";
import { type Hands, type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY = "# Hola\n\nUno **dos** tres\n\n- primero\n- segundo\n";
const PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-24"><span data-src="8-12">Uno </span><strong><span data-src="14-17">dos</span></strong><span data-src="19-24"> tres</span></p>' +
  '<ul data-at="26-46"><li data-at="26-35"><span data-src="28-35">primero</span></li>' +
  '<li data-at="36-45"><span data-src="38-45">segundo</span></li></ul>';
const PIECES = [
  { said: "Hola", from: 2 },
  { said: "Uno ", from: 8 },
  { said: "dos", from: 14 },
  { said: " tres", from: 19 },
  { said: "primero", from: 28 },
  { said: "segundo", from: 38 },
];

type Piece = { readonly said: string; readonly from: number };
type Place = { readonly said: string; readonly at: number };
type Setup = {
  readonly body: string;
  readonly html: string;
  readonly pieces: readonly Piece[];
  readonly place: Place;
};

const ROWS = {
  h1: (said: string, at: string, src: string) =>
    `<h1 data-at="${at}"><span data-src="${src}">${said}</span></h1>`,
  p: (said: string, at: string, src: string) =>
    `<p data-at="${at}"><span data-src="${src}">${said}</span></p>`,
  li: (said: string, at: string, src: string) =>
    `<ul data-at="${at}"><li data-at="${at}"><span data-src="${src}">${said}</span></li></ul>`,
  quote: (said: string, at: string, src: string) =>
    `<blockquote data-at="${at}"><p><span data-src="${src}">${said}</span></p></blockquote>`,
  code: (said: string, at: string, src: string) =>
    `<figure class="code" data-at="${at}"><pre><code><span data-src="${src}">${said}</span></code></pre></figure>`,
};

const layout = (body: string, rows: readonly (readonly [keyof typeof ROWS, string])[]): Setup => {
  let cursor = 0;
  const pieces: Piece[] = [];
  const html = rows
    .map(([kind, said]) => {
      const from = body.indexOf(said, cursor);
      cursor = from + said.length;
      pieces.push({ said, from });
      const src = `${from}-${cursor}`;
      return ROWS[kind](said, src, src);
    })
    .join("");
  return { body, html, pieces, place: { said: pieces[0]?.said ?? "", at: 0 } };
};
type Trace = {
  readonly start: string;
  readonly texts: readonly string[];
  readonly errors: readonly string[];
  readonly caret: number;
};

let writing: PageWriting | null = null;
const errors: string[] = [];
const noted = (event: ErrorEvent) => {
  errors.push(String(event.message));
  event.preventDefault();
};

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
  window.removeEventListener("error", noted);
});

const mulberry32 = (seed: number) => {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let mixed = Math.imul(state ^ (state >>> 15), state | 1);
    mixed ^= mixed + Math.imul(mixed ^ (mixed >>> 7), mixed | 61);
    return ((mixed ^ (mixed >>> 14)) >>> 0) / 4294967296;
  };
};

const pick = <T>(random: () => number, from: readonly T[]): T =>
  from[Math.floor(random() * from.length)] as T;

const LETTERS = [..."abcdefgh xyz0129é"];
const KEYS = ["ctrl:b", "ctrl:i", "ctrl:u", "ctrl:X", "ctrl:C"];

const walk = (random: () => number, size: number, kinds: readonly string[]) =>
  Array.from({ length: size }, () => {
    const kind = pick(random, kinds);
    if (kind === "type") return `type:${pick(random, LETTERS)}`;
    if (kind === "key") return pick(random, KEYS);
    return kind;
  });

const ALL = [
  "type",
  "type",
  "type",
  "type",
  "key",
  "key",
  "bs",
  "del",
  "enter",
  "br",
  "undo",
  "redo",
];
const ONLY_WRITING = ["type", "type", "type", "type", "key", "key", "enter", "br"];

const asked = (inputType: string, data: string | null = null) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(
      new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
    );

const pressed = (name: string) => {
  const shift = name === name.toUpperCase();
  document
    .querySelector(".page")
    ?.dispatchEvent(
      new KeyboardEvent("keydown", { key: name, ctrlKey: true, shiftKey: shift, bubbles: true }),
    );
};

const INPUTS: Record<string, string> = {
  bs: "deleteContentBackward",
  del: "deleteContentForward",
  enter: "insertParagraph",
  br: "insertLineBreak",
  undo: "historyUndo",
  redo: "historyRedo",
};

const act = (action: string) => {
  if (action.startsWith("type:")) return asked("insertText", action.slice(5));
  if (action.startsWith("ctrl:")) return pressed(action.slice(5));
  asked(INPUTS[action] as string);
};

const standard: Setup = {
  body: BODY,
  html: PLACED,
  pieces: PIECES,
  place: { said: "Hola", at: 4 },
};

const placeAt = (random: () => number, pieces: readonly Piece[] = PIECES): Place => {
  const one = pick(random, pieces);
  return { said: one.said, at: Math.floor(random() * (one.said.length + 1)) };
};

const caretOf = (setup: Setup) => {
  const one = setup.pieces.find((each) => each.said === setup.place.said);
  return (one?.from ?? 0) + setup.place.at;
};

const handsOf = (read: () => string, write: (next: string) => void): Hands => ({
  text: read,
  write,
  shown: () => {},
  redraw: () => {},
  near: () => Promise.reject(new Error("no core here")),
  dressed: () => {},
  translated: (_, plain) => Promise.resolve(plain),
  ask: () => {},
  blocks: () => {},
  acts: () => {},
});

const run = (setup: Setup, actions: readonly string[]): Trace => {
  let text = setup.body;
  errors.length = 0;
  window.addEventListener("error", noted);
  document.body.innerHTML = `<article class="page">${setup.html}</article>`;
  writing = writtenOn(
    document,
    handsOf(
      () => text,
      (next) => {
        text = next;
      },
    ),
  );
  const piece = Array.from(document.querySelectorAll("[data-src]")).find(
    (one) => one.textContent === setup.place.said,
  )?.firstChild;
  if (piece) document.getSelection()?.collapse(piece, setup.place.at);
  const texts: string[] = [];
  for (const action of actions) {
    try {
      act(action);
    } catch (thrown) {
      errors.push(String(thrown));
    }
    texts.push(text);
  }
  const result = { start: setup.body, texts, errors: [...errors], caret: caretOf(setup) };
  writing?.stop();
  writing = null;
  window.removeEventListener("error", noted);
  return result;
};

const MARKS = ["**", "++", "~~", "=="];
const emptyIn = (text: string) => {
  if (text.includes("[]()")) return "[]()";
  for (const mark of MARKS) {
    const parts = text.split(mark);
    for (let at = 1; at < parts.length - 1; at += 2) if (parts[at] === "") return mark + mark;
  }
  return null;
};

const prefixLength = (one: string, other: string) => {
  let at = 0;
  while (at < one.length && at < other.length && one[at] === other[at]) at++;
  return at;
};

const subsequence = (said: string, text: string) => {
  let from = 0;
  for (const one of said) {
    from = text.indexOf(one, from);
    if (from < 0) return false;
    from++;
  }
  return true;
};

type Check = (
  trace: Trace,
  actions: readonly string[],
  again: (list: readonly string[]) => Trace,
) => string | null;

const times = (size: number, one: string) => Array<string>(size).fill(one);

const noThrow: Check = (trace) => trace.errors[0] ?? null;

const noEmptyFormat: Check = (trace) => {
  for (const [index, text] of trace.texts.entries()) {
    const found = emptyIn(text);
    if (found) return `after action ${index}: ${found} in ${JSON.stringify(text)}`;
  }
  return null;
};

const undoesToStart: Check = (trace, actions, again) => {
  const undone = again([...actions, ...times(actions.length + 2, "undo")]);
  const bottom = undone.texts[undone.texts.length - 1];
  return bottom === trace.start ? null : `undo ended on ${JSON.stringify(bottom)}`;
};

const redoesToLast: Check = (trace, actions, again) => {
  const last = trace.texts[trace.texts.length - 1] ?? trace.start;
  const there = [...times(actions.length + 2, "undo"), ...times(actions.length + 2, "redo")];
  const redone = again([...actions, ...there]);
  const top = redone.texts[redone.texts.length - 1];
  return top === last ? null : `redo ended on ${JSON.stringify(top)}, not ${JSON.stringify(last)}`;
};

const staysAfterCaret: Check = (trace, actions) => {
  let back = 0;
  for (const [index, action] of actions.entries()) {
    if (action === "bs") back++;
    const text = trace.texts[index] ?? "";
    const kept = prefixLength(text, trace.start);
    const marks =
      /[*_+~=`]*$/.exec(trace.start.slice(0, Math.max(0, trace.caret - back)))?.[0].length ?? 0;
    const floor = Math.max(0, trace.caret - back - marks);
    if (kept < Math.min(floor, trace.start.length))
      return `after action ${index}: the text changed at ${kept}, before ${floor}`;
  }
  return null;
};

const keepsTypedWords: Check = (trace, actions) => {
  const typed = actions
    .filter((one) => one.startsWith("type:"))
    .map((one) => one.slice(5))
    .join("")
    .replace(/\s/g, "");
  const text = trace.texts[trace.texts.length - 1] ?? "";
  return subsequence(typed, text)
    ? null
    : `typed ${JSON.stringify(typed)} is not in ${JSON.stringify(text)}`;
};

const trouble = (setup: Setup, actions: readonly string[], check: Check) =>
  check(run(setup, actions), actions, (list) => run(setup, list));

const shrunk = (setup: Setup, actions: readonly string[], check: Check) => {
  let small = [...actions];
  for (let size = Math.ceil(small.length / 2); size >= 1; size = Math.floor(size / 2)) {
    for (let at = 0; at + size <= small.length; ) {
      const without = [...small.slice(0, at), ...small.slice(at + size)];
      if (trouble(setup, without, check)) small = without;
      else at += size;
    }
  }
  return small;
};

const SEEDS = 300;

const fuzz = (
  kinds: readonly string[],
  check: Check,
  setup?: (random: () => number) => Setup,
  seeds = SEEDS,
) => {
  for (let seed = 1; seed <= seeds; seed++) {
    const random = mulberry32(seed);
    const one = setup ? setup(random) : { ...standard, place: placeAt(random) };
    const actions = walk(random, 20 + Math.floor(random() * 21), kinds);
    if (trouble(one, actions, check)) {
      const small = shrunk(one, actions, check);
      throw new Error(
        `seed ${seed}, caret ${JSON.stringify(one.place)}, actions ${JSON.stringify(small)}: ${trouble(one, small, check)}`,
      );
    }
  }
};

const EDGES: Record<string, Setup> = {
  crlf: layout("# Hola\r\n\r\nUno dos\r\n\r\n- primero\r\n- segundo\r\n", [
    ["h1", "Hola"],
    ["p", "Uno dos"],
    ["li", "primero"],
    ["li", "segundo"],
  ]),
  "no final newline": layout("# Hola\n\nUno dos", [
    ["h1", "Hola"],
    ["p", "Uno dos"],
  ]),
  "fence, quote and list": layout(
    "# T\n\n> quote text\n> more\n\n```\ncode line\n```\n\n- a\n- b\n\n1. one\n2. two\n",
    [
      ["h1", "T"],
      ["quote", "quote text"],
      ["quote", "more"],
      ["code", "code line"],
      ["li", "a"],
      ["li", "b"],
      ["li", "one"],
      ["li", "two"],
    ],
  ),
  "very long words": layout(`# T\n\n${"w".repeat(4000)}\n\n- ${"v".repeat(3000)}\n`, [
    ["h1", "T"],
    ["p", "w".repeat(4000)],
    ["li", "v".repeat(3000)],
  ]),
};

const onEdge = (name: string) => (random: () => number) => {
  const setup = EDGES[name] as Setup;
  return { ...setup, place: placeAt(random, setup.pieces) };
};

describe("random walks over the page written in place", () => {
  it("never throws on any key of a long walk", () => {
    fuzz(ALL, noThrow);
  });

  it("never leaves an empty format while only writing outside a format already open", () => {
    // Inside bold a second bold switches it off, so its marks legitimately sit side by side.
    const outside = PIECES.filter((one) => one.said !== "dos");
    const there = (random: () => number) => ({ ...standard, place: placeAt(random, outside) });
    fuzz(ONLY_WRITING, noEmptyFormat, there);
  });

  it("goes back to the exact first text with undo", () => {
    fuzz(ALL, undoesToStart);
  });

  it("comes forward to the last text with redo after a full undo", () => {
    fuzz(["type", "type", "key", "bs", "del", "enter", "br"], redoesToLast);
  });

  it("only changes the text where the caret goes", () => {
    fuzz(ALL, staysAfterCaret);
  });

  it("keeps every typed letter, in order, while only writing", () => {
    fuzz(ONLY_WRITING, keepsTypedWords);
  });
});

const EDGE_SEEDS = 100;
const LONG_SEEDS = 10;

const together =
  (...checks: readonly Check[]): Check =>
  (trace, actions, again) => {
    for (const one of checks) {
      const found = one(trace, actions, again);
      if (found) return found;
    }
    return null;
  };

describe("random walks over the edges of a body", () => {
  for (const name of Object.keys(EDGES)) {
    const seeds = name === "very long words" ? LONG_SEEDS : EDGE_SEEDS;

    it(`throws nowhere, undoes to the start and stays after the caret on a body with ${name}`, () => {
      fuzz(ALL, together(noThrow, undoesToStart, staysAfterCaret), onEdge(name), seeds);
    });

    it(`keeps its typed words on a body with ${name}`, () => {
      fuzz(ONLY_WRITING, together(noThrow, keepsTypedWords), onEdge(name), seeds);
    });
  }
});

const lastOf = (setup: Setup, said: string, at: number, actions: readonly string[]) =>
  run({ ...setup, place: { said, at } }, actions).texts.slice(-1)[0];

const FORMATS: Record<string, readonly [string, string]> = {
  "ctrl:b": ["**", "**"],
  "ctrl:i": ["*", "*"],
  "ctrl:u": ["++", "++"],
  "ctrl:X": ["~~", "~~"],
  "ctrl:C": ["`", "`"],
};

describe("a format pressed with nothing chosen", () => {
  it("goes on inside the same format closed just before the caret", () => {
    expect(lastOf(standard, " tres", 0, ["ctrl:b", "type:e", "type:f"])).toBe(
      BODY.replace("**dos**", "**dosef**"),
    );
  });

  it("wraps exactly the words typed next in a balanced pair", () => {
    for (let seed = 1; seed <= SEEDS; seed++) {
      const random = mulberry32(seed);
      const place = placeAt(random);
      const key = pick(random, Object.keys(FORMATS));
      const [open, close] = FORMATS[key] as readonly [string, string];
      const word = Array.from({ length: 1 + Math.floor(random() * 6) }, () =>
        pick(random, [..."abcdefgh0129é"]),
      );
      const setup = { ...standard, place };
      const caret = caretOf(setup);
      const steps =
        BODY.startsWith(close, caret) && !(close === "*" && BODY.startsWith("**", caret));
      const rejoins =
        BODY.slice(0, caret).endsWith(close) && /\S/.test(BODY[caret - close.length - 1] ?? "");
      if (steps || rejoins) continue;
      const got = lastOf(setup, place.said, place.at, [key, ...word.map((one) => `type:${one}`)]);
      const want = `${BODY.slice(0, caret)}${open}${word.join("")}${close}${BODY.slice(caret)}`;
      expect(got, `seed ${seed}, ${JSON.stringify(place)}, ${key}`).toBe(want);
    }
  });

  it("steps past the closing marks instead of opening a second pair when pressed right before them", () => {
    expect(lastOf(standard, "dos", 3, ["ctrl:b", "type:x"])).toBe(
      BODY.replace("** tres", "**x tres"),
    );
  });

  it("lets the format go when pressed twice", () => {
    expect(lastOf(standard, "Hola", 4, ["ctrl:b", "ctrl:b", "type:x"])).toBe(
      BODY.replace("Hola", "Holax"),
    );
  });

  it("keeps a space typed first outside the pair", () => {
    expect(lastOf(standard, "Hola", 4, ["ctrl:b", "type: ", "type:x"])).toBe(
      BODY.replace("Hola", "Hola **x**"),
    );
  });
});

describe("the edges of a body", () => {
  const fence = EDGES["fence, quote and list"] as Setup;
  const crlf = EDGES.crlf as Setup;
  const long = EDGES["very long words"] as Setup;
  const bare = EDGES["no final newline"] as Setup;
  const word = "w".repeat(4000);

  it("writes into a code fence as plain words, with a bare line break for Enter and no format", () => {
    const got = lastOf(fence, "code line", 4, ["ctrl:b", "type:x", "enter", "type:y"]);
    expect(got).toBe(fence.body.replace("code line", "codex\ny line"));
  });

  it("starts a paragraph inside a quote with its mark kept", () => {
    const got = lastOf(fence, "more", 4, ["enter", "type:x"]);
    expect(got).toBe(fence.body.replace("> more", "> more\n>\n> x"));
  });

  it("starts the next item of a list, and the next number of a numbered list", () => {
    expect(lastOf(fence, "b", 1, ["enter", "type:x"])).toBe(fence.body.replace("- b", "- b\n- x"));
    expect(lastOf(fence, "two", 3, ["enter", "type:x"])).toBe(
      fence.body.replace("2. two", "2. two\n3. x"),
    );
  });

  it("writes at the end of a body with no final line break, and takes it back", () => {
    const got = lastOf(bare, "Uno dos", 7, ["enter", "type:x", "ctrl:b", "type:y"]);
    expect(got).toBe(`${bare.body}\n\nx**y**`);
    const undone = lastOf(bare, "Uno dos", 7, ["enter", "type:x", "undo", "undo"]);
    expect(undone).toBe(bare.body);
  });

  it("writes into a CRLF body without losing a character", () => {
    const got = lastOf(crlf, "primero", 7, ["type:x", "type:y"]);
    expect(got).toBe(crlf.body.replace("primero", "primeroxy"));
  });

  it("writes into the middle of a very long word, and undoes it", () => {
    const got = lastOf(long, word, 2000, ["type:a", "type:b"]);
    expect(got).toBe(long.body.replace(word, `${"w".repeat(2000)}ab${"w".repeat(2000)}`));
    expect(lastOf(long, word, 2000, ["type:a", "type:b", "undo", "undo"])).toBe(long.body);
  });

  it("ends a very long word's paragraph with Enter in the same place as a short one", () => {
    const got = lastOf(long, word, 4000, ["enter", "type:x"]);
    expect(got).toBe(long.body.replace(`${word}\n`, `${word}\n\nx\n`));
  });

  it("writes nothing, and throws nothing, when the page has no words to put a caret in", () => {
    const trace = run(layout("", []), ["type:a", "enter", "bs", "ctrl:b", "undo", "redo"]);
    expect(trace.errors).toEqual([]);
    expect(trace.texts.slice(-1)[0]).toBe("");
  });

  it("writes the first words of an empty body into the line begun for them", () => {
    let text = "";
    document.body.innerHTML = '<article class="page"></article>';
    const sheet = document.querySelector(".page") as Element;
    const hands = handsOf(
      () => text,
      (next) => {
        text = next;
      },
    );
    writing = writtenOn(document, hands);
    writing.begin(0, "", sheet);
    asked("insertText", "h");
    asked("insertText", "i");
    expect(text).toBe("hi");
  });
});

describe("what random walks once caught on the page written in place", () => {
  it("takes the marks away with the last character deleted from a format", () => {
    expect(lastOf(standard, "Hola", 4, ["ctrl:b", "type:x", "bs"])).toBe(BODY);
  });

  it("takes the marks of an italic away with its last character deleted", () => {
    expect(lastOf(standard, "Hola", 4, ["ctrl:i", "type:x", "bs"])).toBe(BODY);
  });

  it("keeps both formats when two are pressed before the first key", () => {
    const got = lastOf(standard, "Hola", 4, ["ctrl:b", "ctrl:i", "type:a"]) ?? "";
    expect(got.split("\n")[0]?.match(/\*/g)?.length).toBe(6);
  });

  it("keeps the CRLF line ends of a body when Enter starts a new item", () => {
    const crlf = EDGES.crlf as Setup;
    expect(lastOf(crlf, "primero", 7, ["enter", "type:x"])).not.toMatch(/(?<!\r)\n/);
  });

  it("keeps the CRLF line ends of a body when Shift and Enter break a line", () => {
    const crlf = EDGES.crlf as Setup;
    expect(lastOf(crlf, "Hola", 2, ["br", "type:x"])).not.toMatch(/(?<!\r)\n/);
  });

  it("deletes a whole CRLF line end with one backspace", () => {
    const crlf = EDGES.crlf as Setup;
    expect(lastOf(crlf, "Uno dos", 0, ["type:x", "bs", "bs"])).not.toMatch(/\r(?!\n)/);
  });
});

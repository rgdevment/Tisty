const errors: string[] = [];
const noted = (where: string) => (event: ErrorEvent | PromiseRejectionEvent) => {
  const why = "reason" in event ? event.reason : (event.error ?? event.message);
  errors.splice(errors.length, 0, `${where}: ${why?.stack ?? why}`);
};
window.addEventListener("error", noted("window"));
window.addEventListener("unhandledrejection", noted("window"));

const DRAWN = new Set(["doc_placed", "doc_drawn"]);
const PICTURE =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAgCAIAAADbtmxLAAAAMklEQVR42u3OQQ0AAAgEoOsfwBCGMJstnA82ApDqeSVCQkJCQkJCQkJCQkJCQkJCQpcWu0j0ppZpK1kAAAAASUVORK5CYII=";

Object.assign(window, {
  __TAURI_INTERNALS__: {
    invoke: async (cmd: string, args: unknown) => {
      if (cmd === "served") {
        if (String((args as { reference?: string }).reference).includes("missing"))
          throw new Error("the harness keeps no picture by that name");
        return PICTURE;
      }
      const answer = await fetch("/bridge", { method: "POST", body: JSON.stringify({ cmd, args }) });
      const said = await answer.json();
      if (said && typeof said === "object" && "refused" in said)
        throw new Error(`the core refused ${cmd}: ${said.refused}`);
      return DRAWN.has(cmd) ? new TextEncoder().encode(said).buffer : said;
    },
    transformCallback: () => 0,
    convertFileSrc: (path: string) => path,
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
  },
});

const { framed, dressedAsWindow } = await import("/src/ui/framed.ts");
const { writtenOn } = await import("/src/ui/pageWriting.ts");
const { docPlaced, docPlacedNear } = await import("/src/placing.ts");
const { wired, worn, fitted } = await import("/src/ui/dressing.ts");
const { diagrams, sheeting } = await import("tisty-editor/read");
const { unnavigated } = await import("/src/unnavigated.ts");
const { sourceAt } = await import("/src/ui/pageSpots.ts");
const { colonPairs } = await import("/src/ui/pageColons.ts");
const { charsAt } = await import("tisty-editor/read");
window.addEventListener("keydown", unnavigated);

const freshen = (sheet: Element, html: string) => {
  for (const name of sheet.getAttributeNames()) if (name !== "class") sheet.removeAttribute(name);
  sheet.className = "page";
  sheet.innerHTML = html;
};

let text = "";
let writer: ReturnType<typeof writtenOn> | null = null;
let frame: HTMLIFrameElement | null = null;
const heard: unknown[] = [];
let listening = false;

const named = (node: Node | null) => {
  if (!node) return "nothing";
  if (node.nodeType === Node.TEXT_NODE) return `text ${JSON.stringify((node as Text).data.slice(0, 40))}`;
  const element = node as Element;
  const marks = ["class", "data-src", "data-at", "contenteditable", "hidden"]
    .filter((name) => element.hasAttribute?.(name))
    .map((name) => `${name}=${JSON.stringify(element.getAttribute(name))}`);
  return [element.nodeName.toLowerCase(), ...marks].join(" ");
};

const pointSaid = (node: Node | null, offset: number) => ({
  node: named(node),
  offset,
  source: node ? sourceAt(text, node, offset) : null,
});

const rangeSaid = (range: AbstractRange) => ({
  from: pointSaid(range.startContainer, range.startOffset),
  to: pointSaid(range.endContainer, range.endOffset),
});

const hearing = (page: Document) => (event: InputEvent) => {
  const held = page.getSelection();
  heard.push({
    inputType: event.inputType,
    data: event.data,
    targets: (event.getTargetRanges?.() ?? []).map(rangeSaid),
    chosen: held?.rangeCount ? rangeSaid(held.getRangeAt(0)) : null,
    caretAt: writer?.caretAt() ?? null,
  });
};

// What was written shows only once the core has drawn it, so the hands ask before they aim a click or set the caret.
let awaited = 0;
let answered = 0;
const drawing = <T>(asked: Promise<T>): Promise<T> => {
  awaited += 1;
  return asked.finally(() => {
    awaited -= 1;
    answered += 1;
  });
};

let answersHeld: (() => void)[] | null = null;
const heldBack = <T>(answer: Promise<T>): Promise<T> => {
  const waiting = answersHeld;
  if (!waiting) return answer;
  answer.catch(() => {});
  return new Promise<T>((done, failed) => {
    waiting.push(() => answer.then(done, failed));
  });
};
const answersLet = () => {
  const waiting = answersHeld ?? [];
  answersHeld = null;
  for (const one of waiting) one();
};

const reaching = () => ({
  body: text,
  onDoc: () => {},
  onBegin: (at: number, seed: string, near: Element) => writer?.begin(at, seed, near),
  onHintWords: (at: [number, number], hint: HTMLElement) => writer?.hintWords(at, hint),
});

const dressedUp = (page: Document) => {
  wired(page, reaching);
  dressedAsWindow(page);
  fitted(page);
  worn(page, () => true, reaching());
  sheeting(page, () => undefined);
  void diagrams(page, false, () => true).catch(noted("diagrams"));
};

const writtenIn = (page: Document) => {
  writer = writtenOn(page, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: (now) => {
      void drawing(
        docPlaced(now).then((drawn) => {
          const sheet = page.querySelector(".page");
          if (sheet) freshen(sheet, drawn);
          worn(page, () => true, reaching());
          writer?.drawn();
        }),
      );
    },
    near: (...asked) => drawing(heldBack(docPlacedNear(...asked))),
    dressed: () => {},
    translated: (_html, plain) => Promise.resolve(plain),
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  writer.drawn();
  const slot = page.querySelector<HTMLElement>(".page > h1[data-title]");
  if (slot && !text.trim()) {
    page.defaultView?.focus();
    writer.begin(0, "# ", slot);
  }
};

const open = async (body: string) => {
  writer?.stop();
  answersLet();
  frame?.remove();
  errors.length = 0;
  heard.length = 0;
  text = body;
  const held = document.createElement("iframe");
  held.style.cssText = "display:block;width:900px;height:760px;border:0";
  document.getElementById("root")?.append(held);
  frame = held;
  const html = await docPlaced(text);
  await new Promise<void>((ready) => {
    held.addEventListener("load", function loaded() {
      const page = held.contentDocument;
      const sheet = page?.querySelector(".page");
      if (!page || !sheet) return;
      held.removeEventListener("load", loaded);
      page.defaultView?.addEventListener("error", noted("page"));
      page.defaultView?.addEventListener("unhandledrejection", noted("page"));
      sheet.removeAttribute("inert");
      page.addEventListener("keydown", unnavigated);
      if (listening) page.addEventListener("beforeinput", hearing(page), true);
      dressedUp(page);
      writtenIn(page);
      ready();
    });
    held.srcdoc = framed(html);
  });
};

const places = (holder: ParentNode) =>
  [...holder.children]
    .filter((one) => one.hasAttribute("data-at"))
    .map((one) => one.getAttribute("data-at"))
    .join(" ");

const consistent = async () => {
  const page = frame?.contentDocument;
  const sheet = page?.querySelector(".page");
  if (!page || !sheet) return "no page";
  const fresh = page.createElement("template");
  fresh.innerHTML = await docPlaced(text);
  const shown = places(sheet);
  const drawn = places(fresh.content);
  return shown === drawn ? "ok" : `the page shows [${shown}] but the text draws [${drawn}]`;
};

const CONTAINERS = "details[data-at], blockquote[data-callout][data-at]";

const colonsDrawn = () => {
  const sheet = frame?.contentDocument?.querySelector(".page");
  const coloned = [...(sheet?.querySelectorAll(CONTAINERS) ?? [])].flatMap((one) => {
    const [from = 0, to = 0] = (one.getAttribute("data-at") ?? "").split("-").map(Number);
    const start = text.lastIndexOf("\n", charsAt(text, from) - 1) + 1;
    const line = text.slice(start, text.indexOf("\n", start) < 0 ? text.length : text.indexOf("\n", start));
    const end = charsAt(text, to);
    return /^[ \t>]*:::/.test(line) ? [`${start}-${text.slice(0, end).replace(/\s+$/, "").length}`] : [];
  });
  const paired = colonPairs(text).map(({ from, to }: { from: number; to: number }) => `${from}-${to}`);
  const said = (spans: string[]) => [...spans].sort().join(" ");
  return said(coloned) === said(paired)
    ? "ok"
    : `the core draws [${said(coloned)}] but the editor pairs [${said(paired)}]`;
};

Object.assign(window, {
  engines: {
    open,
    consistent,
    colonsDrawn,
    drawn: () => awaited === 0,
    text: () => text,
    errors: () => [...errors],
    heard: () => [...heard],
    hear: () => {
      listening = true;
    },
    hold: () => {
      answersHeld ??= [];
    },
    letOne: () => answersHeld?.shift()?.(),
    letGo: answersLet,
    holding: () => answersHeld?.length ?? 0,
    answered: () => answered,
  },
  enginesReady: true,
});

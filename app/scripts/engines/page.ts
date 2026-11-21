const errors: string[] = [];
const noted = (where: string) => (event: ErrorEvent | PromiseRejectionEvent) => {
  const why = "reason" in event ? event.reason : (event.error ?? event.message);
  errors.splice(errors.length, 0, `${where}: ${why?.stack ?? why}`);
};
window.addEventListener("error", noted("window"));
window.addEventListener("unhandledrejection", noted("window"));

const DRAWN = new Set(["doc_placed", "doc_drawn"]);

Object.assign(window, {
  __TAURI_INTERNALS__: {
    invoke: async (cmd: string, args: unknown) => {
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
window.addEventListener("keydown", unnavigated);

const freshen = (sheet: Element, html: string) => {
  for (const name of sheet.getAttributeNames()) if (name !== "class") sheet.removeAttribute(name);
  sheet.className = "page";
  sheet.innerHTML = html;
};

let text = "";
let writer: ReturnType<typeof writtenOn> | null = null;
let frame: HTMLIFrameElement | null = null;

const dressedUp = (page: Document) => {
  const reach = () => ({
    body: text,
    onDoc: () => {},
    onBegin: (at: number, seed: string, near: Element) => writer?.begin(at, seed, near),
    onHintWords: (at: [number, number], hint: HTMLElement) => writer?.hintWords(at, hint),
  });
  wired(page, reach);
  dressedAsWindow(page);
  fitted(page);
  worn(page, () => true, reach());
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
      void docPlaced(now).then((drawn) => {
        const sheet = page.querySelector(".page");
        if (sheet) freshen(sheet, drawn);
        writer?.drawn();
      });
    },
    near: docPlacedNear,
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
  frame?.remove();
  errors.length = 0;
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

Object.assign(window, {
  engines: {
    open,
    consistent,
    text: () => text,
    errors: () => [...errors],
  },
  enginesReady: true,
});

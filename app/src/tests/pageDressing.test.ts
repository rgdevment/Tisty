import { afterEach, describe, expect, it, vi } from "vitest";
import type { Reaching } from "../ui/dressing";
import { wornLaid } from "../ui/pageDressing";

const path = vi.hoisted(() => ({ inPlace: true }));

vi.mock("../ui/pagePatching", async (original) => {
  const real = await original<typeof import("../ui/pagePatching")>();
  return { ...real, keptInPlace: (one: Element) => path.inPlace && real.keptInPlace(one) };
});

vi.mock("../inUse", async (original) => ({
  ...(await original<typeof import("../inUse")>()),
  serve: (reference: string) => Promise.resolve(`C:/kept/${reference}`),
}));

vi.mock("@tauri-apps/api/core", async (original) => ({
  ...(await original<typeof import("@tauri-apps/api/core")>()),
  convertFileSrc: (path: string) => `asset://${path}`,
}));

const REACH = {
  body: "",
  onDoc: () => {},
  onTable: () => {},
  onWiden: () => {},
  stepDates: { segundo: "Vence mañana" },
} as unknown as Reaching;

const TABLE =
  '<div class="table" data-at="40-70"><table><thead><tr><th data-type="text">a</th></tr></thead><tbody><tr><td>1</td></tr></tbody></table></div>';

afterEach(() => {
  document.body.innerHTML = "";
  path.inPlace = true;
  vi.restoreAllMocks();
});

const pageOf = (inner: string) => {
  document.body.innerHTML = `<article class="page">${inner}</article>`;
  return document.querySelector(".page") as Element;
};

describe("the blocks the core draws again on the page written in place", () => {
  it("are read as the whole page is, and stay where they were laid", () => {
    const sheet = pageOf(
      '<p data-at="0-5">Antes</p><blockquote class="callout" data-callout="tip" data-at="7-20"><p>Ojo</p></blockquote><p data-at="22-28">Después</p>',
    );
    const laid = [sheet.children[1] as Element];
    wornLaid(document, laid, () => true, REACH);
    const children = Array.from(sheet.children);
    expect(children[1]).toBe(laid[0]);
    expect(children.map((one) => one.textContent)).toEqual(["Antes", "Ojo", "Después"]);
    expect((laid[0] as HTMLElement).dataset.said).toBeTruthy();
  });

  it("dress a table laid at the top of the page, the block itself", () => {
    const sheet = pageOf(TABLE);
    wornLaid(document, [sheet.children[0] as Element], () => true, REACH);
    expect(sheet.querySelector(".table")?.classList.contains("dressed")).toBe(true);
    expect(sheet.querySelector(".table")?.classList.contains("celled")).toBe(true);
    expect(sheet.querySelector(".col-grip")).not.toBeNull();
  });

  it("count the stages again across the page, and give a step its date", () => {
    const sheet = pageOf(
      '<ul data-at="0-30"><li class="task" id="stage--1" data-at="0-10">primero</li></ul>' +
        '<ul data-at="32-50"><li class="task" data-at="32-50">segundo</li></ul>',
    );
    const laid = [sheet.children[1] as Element];
    wornLaid(document, laid, () => true, REACH);
    const stages = Array.from(sheet.querySelectorAll("li.task"), (one) => one.id);
    expect(stages).toEqual(["stage--1", "stage--2"]);
    expect(sheet.querySelectorAll("li.task")[1]?.getAttribute("title")).toBe("Vence mañana");
  });

  it("give a formula drawn again its way to its code", () => {
    const sheet = pageOf('<div class="math" data-at="0-20"><math></math></div>');
    wornLaid(document, [sheet.children[0] as Element], () => true, {
      ...REACH,
      onPiece: () => {},
    });
    expect(sheet.querySelector('.math-bar [data-asked="code"]')).not.toBeNull();
  });
});

const WORDS = [
  '<p data-at="0-9"><a class="card" data-doc="ab12-0001">A</a></p>',
  '<p data-at="0-9">mira <a class="card" data-doc="ab12-0001">A</a> ahí</p>',
  '<p data-at="0-9"><a href="https://x.y/z">x</a></p>',
  '<p data-at="0-9">mira <a href="https://x.y/z">x</a> ahí</p>',
  '<p data-at="0-9"><a href="tisty:doc/ab12-0002">B</a></p>',
  '<p data-at="0-9"><a href="attachments/a.pdf">x</a></p>',
  '<p data-at="0-9"><span class="tag" data-tag="work">#work</span></p>',
  '<p data-at="0-20">Sea <math data-src="4-9" contenteditable="false"><mi>x</mi></math> y <code class="math" data-src="12-17" contenteditable="false">\fr</code></p>',
  '<p data-at="0-9"><img src="attachments/a.png" alt="A"><img src="https://x.y/b.png" alt="B"></p>',
  '<p data-at="0-9"><img src="attachments/walk-0006bf09.mp4" alt="A walk"></p>',
  '<p data-at="0-9"><img src="attachments/bark-0006bf10.mp3" alt="Bark"></p>',
  '<p data-at="0-9"><img src="walks-0006bf09.html" alt="Walks"></p>',
  '<p data-at="0-9">plain <code>x</code> words</p>',
  '<h2 id="b" data-at="0-9">B <span class="tag" data-tag="x">#x</span></h2>',
  '<p data-at="0-9"><span class="media is-audio" data-file="attachments/bark-0006bf10.mp3" data-name="Bark" data-placed="0-9"><span class="media-name">Bark</span><audio controls=""></audio></span></p>',
  '<p data-at="0-9"><span class="media is-audio" data-file="attachments/bark-0006bf10.mp3" data-name="Bark" data-placed="0-5"><span class="media-name">Bark</span><audio controls=""></audio></span> <a href="https://x.y/z">x</a></p>',
];

const READING = {
  ...REACH,
  card: () => ({ kind: "doc", title: "Notas" }),
  blurb: () => Promise.resolve("Hola."),
  onMore: () => {},
  onPiece: () => {},
  onLanguage: () => {},
} as unknown as Reaching;

const settled = () => new Promise((resolve) => setTimeout(resolve, 20));

const dressedAs = async (inPlace: boolean, block: string) => {
  path.inPlace = inPlace;
  const sheet = pageOf(`<p data-at="100-105">Antes</p>${block}<p data-at="200-206">Después</p>`);
  const one = sheet.children[1] as Element;
  const heard = vi.spyOn(EventTarget.prototype, "addEventListener");
  const watch = new MutationObserver(() => {});
  watch.observe(sheet, { childList: true });
  wornLaid(document, [one], () => true, READING);
  const moved = watch.takeRecords().length;
  watch.disconnect();
  const now = one.outerHTML;
  await settled();
  const lost = heard.mock.contexts.filter(
    (target) => target instanceof Element && !target.isConnected && target.matches("p, h2"),
  ).length;
  heard.mockRestore();
  return { now, later: one.outerHTML, kept: sheet.children[1] === one, moved, lost };
};

describe("a block of words the core draws again, dressed where it stands", () => {
  it.each(WORDS)("comes out as it would taken away to be dressed: %s", async (block) => {
    const away = await dressedAs(false, block);
    const kept = await dressedAs(true, block);
    expect(kept.now).toBe(away.now);
    expect(kept.later).toBe(away.later);
    expect(kept.kept).toBe(true);
    expect(kept.moved).toBe(0);
    expect(away.moved).toBeGreaterThan(0);
    expect(kept.lost).toBe(0);
  });
});

describe("a player among the words dressed again", () => {
  it("never goes to the page the words are dressed on, so what it plays goes on", () => {
    const sheet = pageOf(
      '<p data-at="0-30"><span data-src="0-6">Antes </span><span class="media is-audio" data-file="a.mp3" data-name="Ladrido" data-placed="6-26"><audio></audio></span></p>',
    );
    const player = sheet.querySelector(".media") as Element;
    const carried: Node[] = [];
    const append = Element.prototype.append;
    vi.spyOn(Element.prototype, "append").mockImplementation(function (
      this: Element,
      ...nodes: (Node | string)[]
    ) {
      if (this.ownerDocument !== document)
        carried.push(...nodes.filter((one): one is Node => typeof one !== "string"));
      return append.apply(this, nodes);
    });
    wornLaid(document, [sheet.children[0] as Element], () => true, REACH);
    expect(carried).not.toContain(player);
    expect(sheet.querySelector(".media")).toBe(player);
  });
});

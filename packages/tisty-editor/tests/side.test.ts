import { describe, expect, it } from "vitest";
import { type Panel, sided, wordsIn } from "../src/side";
import type { ReadWord } from "../src/words";

const say = (word: ReadWord, ...filled: string[]) => [word, ...filled].join(" ");

const host = () => {
  const page = document.implementation.createHTMLDocument("");
  const one = page.createElement("aside");
  page.body.append(one);
  return one;
};

const text = (side: HTMLElement, selector: string) =>
  Array.from(side.querySelectorAll(selector), (one) => one.textContent);

describe("the panels beside a sheet", () => {
  it("shows only the panels it is given, in their cards", () => {
    const side = host();
    sided(side, say, [{ kind: "title", title: "receta.md" }]);
    expect(text(side, ".side-title")).toEqual(["receta.md"]);
    expect(side.querySelector(".side-under")).toBeNull();
    expect(side.querySelector(".side-more")).toBeNull();
    sided(side, say, [{ kind: "index", heads: [] }]);
    expect(text(side, ".side-none")).toEqual(["outlineNone"]);
    expect(side.querySelector(".side-top .side-title")).toBeNull();
  });

  it("goes to a heading, picks a paper and turns the sheets, the landscape and the Markdown", () => {
    const side = host();
    const asked: string[] = [];
    const turn = (name: string, on: boolean) => ({
      on,
      onTurn: (now: boolean) => asked.push(`${name} ${now}`),
    });
    sided(side, say, [
      { kind: "title", title: "Guía", under: "D:\\Descargas", onShut: () => asked.push("shut") },
      {
        kind: "paper",
        leaf: "tabloid",
        onLeaf: (leaf) => asked.push(leaf),
        sheets: turn("sheets", true),
        landscape: turn("across", false),
        raw: turn("raw", true),
      },
      {
        kind: "index",
        heads: [
          { key: "f", level: 0, text: "", go: () => asked.push("front") },
          { key: "a", level: 2, text: "Pasos", go: () => asked.push("pasos") },
        ],
      },
    ]);
    expect(text(side, ".side-leaf")).toEqual(["leafA4", "leafLetter", "leafFolio", "leafTabloid"]);
    const go = side.querySelectorAll<HTMLButtonElement>(".side-go");
    expect(go).toHaveLength(1);
    expect(go[0].style.getPropertyValue("--depth")).toBe("1");
    go[0].click();
    side.querySelector<HTMLButtonElement>(".side-leaf")?.click();
    for (const tool of side.querySelectorAll<HTMLButtonElement>(".side-tool")) tool.click();
    side.querySelector<HTMLButtonElement>(".side-shut")?.click();
    expect(asked).toEqual(["pasos", "a4", "sheets false", "across true", "raw false", "shut"]);
  });

  it("tells the details it is given, the front matter's fields among them", () => {
    const side = host();
    sided(side, say, [
      {
        kind: "details",
        facts: {
          body: "# Hola\n\nUna dos tres",
          author: "Mario",
          weighs: "1 kB",
          fields: [{ key: "version", values: ["2"] }],
        },
      },
    ]);
    expect(text(side, ".side-gist")).toEqual(["paperWords 4 · 1 kB"]);
    expect(text(side, ".side-fact dt")).toEqual([
      "paperAuthor",
      "paperLong",
      "paperWeighs",
      "version",
    ]);
    sided(side, say, [{ kind: "details", facts: { body: "" } }]);
    expect(text(side, ".side-gist")).toEqual(["paperEmpty"]);
  });

  it("folds the signature, opens it with its print, and has nothing to open when nobody signed", () => {
    const side = host();
    const signed = (seal: Extract<Panel, { kind: "seal" }>["seal"]): Panel[] => [
      { kind: "seal", seal },
    ];
    sided(
      side,
      say,
      signed({ verdict: "holds", who: "rgdevment", fingerprint: "AB:CD", sender: "otro" }),
    );
    const row = side.querySelector<HTMLDetailsElement>("details[data-row=seal]");
    expect(row?.open).toBe(false);
    expect(text(side, ".side-gist.is-good")).toEqual(["rgdevment · signedHolds"]);
    expect(text(side, ".side-said")).toEqual(["signedHoldsSaid", "signedSentBy otro"]);
    expect(text(side, ".side-print")).toEqual(["AB:CD"]);
    row?.querySelector("summary")?.click();
    if (row) row.open = true;
    sided(side, say, signed({ verdict: "revoked", who: "rgdevment" }));
    expect(side.querySelector<HTMLDetailsElement>("details[data-row=seal]")?.open).toBe(true);
    expect(text(side, ".side-gist.is-warn")).toEqual(["rgdevment · signedRevoked"]);
    sided(side, say, signed({ verdict: "unreadable" }));
    expect(text(side, ".side-gist.is-bad")).toEqual(["signedUnreadable"]);
    sided(side, say, signed({ verdict: "unsigned" }));
    expect(side.querySelector("details")).toBeNull();
    expect(text(side, ".side-gist")).toEqual(["signedNone"]);
  });

  it("says how it is shared, and acts by link or by press", () => {
    const side = host();
    let pressed = 0;
    sided(side, say, [
      { kind: "shared", gist: "Pronto", said: "Podrás compartirlo con un link." },
      { kind: "shared", gist: "Sin compartir" },
      {
        kind: "acts",
        acts: [
          { key: "md", label: "Descargar Markdown", href: "blob:x", download: "receta.md" },
          { key: "open", label: "Abrir en Tisty", main: true, onPick: () => pressed++ },
        ],
      },
    ]);
    expect(text(side, "details .side-said")).toEqual(["Podrás compartirlo con un link."]);
    expect(side.querySelectorAll("div.side-row")).toHaveLength(1);
    expect(side.querySelector<HTMLAnchorElement>("a.side-act")?.download).toBe("receta.md");
    side.querySelector<HTMLButtonElement>("button.side-act.is-main")?.click();
    expect(pressed).toBe(1);
  });

  it("walks the folders it sits in, opens apart, exports, and lists the book a page belongs to", () => {
    const side = host();
    const asked: string[] = [];
    sided(side, say, [
      {
        kind: "title",
        title: "Capítulo 2",
        trail: [
          { key: "all", label: "Todos", onPick: () => asked.push("all") },
          { key: "f1", label: "Guía", onPick: () => asked.push("f1") },
        ],
        onApart: () => asked.push("apart"),
      },
      {
        kind: "paper",
        leaf: "a4",
        onLeaf: () => undefined,
        exporting: true,
        onExport: () => asked.push("export"),
      },
      {
        kind: "index",
        heads: [],
        book: {
          up: { title: "", onPick: () => asked.push("up") },
          said: "2 páginas",
          pages: [
            { key: "1", title: "Uno", current: true, onPick: () => asked.push("uno") },
            { key: "2", title: "Dos", away: true, archived: true, onPick: () => undefined },
            { key: "3", title: "Tres", flagged: true, onPick: () => undefined },
          ],
        },
      },
    ]);
    expect(text(side, ".side-under")).toEqual(["Todos / Guía"]);
    expect(side.querySelector(".side-none")).toBeNull();
    for (const step of side.querySelectorAll<HTMLButtonElement>(".side-step")) step.click();
    side.querySelector<HTMLButtonElement>("[aria-label=viewDocApart]")?.click();
    const out = side.querySelector<HTMLButtonElement>("[aria-haspopup=menu]");
    expect(out?.getAttribute("aria-label")).toBe("makingPdf");
    out?.click();
    const [up, one, two, three] = side.querySelectorAll<HTMLButtonElement>(".side-go.is-page");
    expect(up.textContent).toBe("untitledDoc");
    up.click();
    one.click();
    expect(one.getAttribute("aria-current")).toBe("true");
    expect(two.getAttribute("aria-label")).toBe("Dos — isArchived");
    expect(three.getAttribute("aria-label")).toBe("Tres — docFlagged");
    expect(three.querySelector(".side-flag")).not.toBeNull();
    expect(asked).toEqual(["all", "f1", "apart", "export", "up", "uno"]);
  });

  it("shows the properties and tags of the details, and keeps one row open at a time", () => {
    const side = host();
    let fronted = 0;
    const panels: Panel[] = [
      {
        kind: "details",
        open: true,
        facts: {
          body: "Hola",
          fields: [{ key: "estado", values: ["listo"] }],
          overgrown: true,
          onFront: () => fronted++,
          tags: [
            { name: "casa", times: 2 },
            { name: "nueva", times: 0 },
          ],
        },
      },
      { kind: "shared", gist: "Pronto", said: "Con un link." },
    ];
    sided(side, say, panels);
    const [details, shared] = side.querySelectorAll<HTMLDetailsElement>("details[data-row]");
    expect(details.open).toBe(true);
    expect(text(side, ".side-part .side-label")).toEqual(["frontNamed", "tags"]);
    expect(text(side, ".side-part .side-none")).toEqual(["frontOvergrown"]);
    side.querySelector<HTMLButtonElement>(".side-link")?.click();
    expect(fronted).toBe(1);
    const [casa, nueva] = side.querySelectorAll<HTMLElement>(".side-tag");
    expect(casa.title).toBe("tagTimes 2");
    expect(nueva.querySelector(".side-times")).toBeNull();
    shared.querySelector("summary")?.click();
    shared.open = true;
    shared.dispatchEvent(new Event("toggle"));
    expect(details.open).toBe(false);
    sided(side, say, panels);
    const [again, sharedAgain] = side.querySelectorAll<HTMLDetailsElement>("details[data-row]");
    expect(sharedAgain.open).toBe(true);
    expect(again.open).toBe(false);
  });

  it("keeps the outline where it was scrolled, and the keys where they were, when drawn again", () => {
    const side = document.createElement("aside");
    document.body.append(side);
    const heads = Array.from({ length: 40 }, (_, at) => ({
      key: `h${at}`,
      level: 1,
      text: `Sección ${at}`,
      go: () => undefined,
    }));
    const panels: Panel[] = [
      { kind: "paper", leaf: "a4", onLeaf: () => undefined },
      { kind: "index", heads },
    ];
    sided(side, say, panels);
    const index = side.querySelector<HTMLElement>(".side-index") as HTMLElement;
    index.scrollTop = 120;
    side.querySelectorAll<HTMLButtonElement>(".side-leaf")[1].focus();
    sided(side, say, panels);
    expect(side.querySelector<HTMLElement>(".side-index")?.scrollTop).toBe(120);
    expect(document.activeElement?.textContent).toBe("leafLetter");
    side.remove();
  });

  it("counts the words of a body, without its code or its marks", () => {
    expect(wordsIn("# Compras\n\n- leche\n- pan")).toBe(3);
    expect(wordsIn("hola\n\n```\nlet x = 1\n```\n\nadios")).toBe(2);
  });
});

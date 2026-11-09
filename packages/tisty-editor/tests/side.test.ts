import type { ReadWord } from "tisty-sheet";
import { describe, expect, it } from "vitest";
import { type Panel, sided, wordsIn } from "../src/side";

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

  describe("the paper and the sheets", () => {
    const turned = (on: boolean) => ({ on, onTurn: () => undefined });
    const leaves = (side: HTMLElement) =>
      Array.from(side.querySelectorAll<HTMLButtonElement>(".side-leaf"));

    it("puts the sheets first, the papers after them and the turn of the page last", () => {
      const side = host();
      sided(side, say, [
        {
          kind: "paper",
          leaf: "a4",
          onLeaf: () => undefined,
          sheets: turned(true),
          landscape: turned(false),
        },
      ]);

      const order = Array.from(side.querySelector(".side-paper")?.children ?? []);
      expect(order.map((one) => one.getAttribute("aria-label") ?? one.className)).toEqual([
        "sheetsShown",
        "leafIs",
        "sheetsAcross",
      ]);
    });

    it("keeps the papers from being pressed until the sheets are on, and shows which it is", () => {
      const side = host();
      const asked: string[] = [];
      sided(side, say, [
        { kind: "paper", leaf: "letter", onLeaf: (one) => asked.push(one), sheets: turned(false) },
      ]);

      expect(leaves(side).every((one) => one.disabled)).toBe(true);
      expect(side.querySelector<HTMLElement>(".side-leaves")?.title).toBe("leafNeedsSheets");
      expect(leaves(side)[1].getAttribute("aria-pressed")).toBe("true");
      leaves(side)[0].click();
      expect(asked).toEqual([]);
    });

    it("lets a paper be pressed once the sheets are on", () => {
      const side = host();
      const asked: string[] = [];
      sided(side, say, [
        { kind: "paper", leaf: "a4", onLeaf: (one) => asked.push(one), sheets: turned(true) },
      ]);

      expect(leaves(side).some((one) => one.disabled)).toBe(false);
      expect(side.querySelector<HTMLElement>(".side-leaves")?.title).toBe("");
      leaves(side)[1].click();
      expect(asked).toEqual(["letter"]);
    });

    it("never locks the paper where sheets cannot be turned on", () => {
      const side = host();
      sided(side, say, [{ kind: "paper", leaf: "a4", onLeaf: () => undefined }]);

      expect(leaves(side).some((one) => one.disabled)).toBe(false);
    });
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
    sided(side, say, [{ kind: "details", facts: { author: "Mario" } }]);
    expect(text(side, ".side-fact dt")).toEqual(["paperAuthor"]);
    expect(side.textContent).not.toContain("paperEmpty");
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

  it("shows the properties and tags of the details, and keeps the row as the reader left it", () => {
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
    ];
    sided(side, say, panels);
    const details = side.querySelector<HTMLDetailsElement>(
      "details[data-row]",
    ) as HTMLDetailsElement;
    expect(details.open).toBe(true);
    expect(text(side, ".side-part .side-label")).toEqual(["frontNamed", "tags"]);
    expect(text(side, ".side-part .side-none")).toEqual(["frontOvergrown"]);
    side.querySelector<HTMLButtonElement>(".side-link")?.click();
    expect(fronted).toBe(1);
    const [casa, nueva] = side.querySelectorAll<HTMLElement>(".side-tag");
    expect(casa.title).toBe("tagTimes 2");
    expect(nueva.querySelector(".side-times")).toBeNull();
    details.querySelector("summary")?.click();
    details.open = false;
    sided(side, say, panels);
    expect(side.querySelector<HTMLDetailsElement>("details[data-row]")?.open).toBe(false);
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

  it("finds the control that had the keys by what it says, wherever the outline moved it", () => {
    const side = document.createElement("aside");
    document.body.append(side);
    const heads = (many: number) =>
      Array.from({ length: many }, (_, at) => ({
        key: `h${at}`,
        level: 1,
        text: `Sección ${at}`,
        go: () => undefined,
      }));
    const panels = (many: number): Panel[] => [
      { kind: "index", heads: heads(many) },
      { kind: "details", facts: { body: "Hola", fields: [{ key: "title", values: ["Plan"] }] } },
    ];
    sided(side, say, panels(1));
    side.querySelector<HTMLElement>("summary")?.focus();
    sided(side, say, panels(4));
    expect(document.activeElement?.tagName).toBe("SUMMARY");
    expect(side.querySelector(".side-part")).toBeNull();
    side.remove();
  });

  it("draws a held row with what it says, its links and what can be pressed", () => {
    const side = host();
    const asked: string[] = [];
    sided(side, say, [
      {
        kind: "held",
        key: "shared",
        label: "Compartido",
        gist: "Solo leer",
        tone: "warn",
        open: true,
        said: ["Caduca el lunes", "3 lecturas"],
        links: [
          {
            key: "read",
            label: "Leer",
            url: "https://tisty.cl/a",
            copy: "Copiar",
            onCopy: () => asked.push("copy read"),
          },
          {
            key: "edit",
            label: "Editar",
            url: "https://tisty.cl/b",
            note: "Solo en Tisty",
            copy: "Copiar",
            onCopy: () => asked.push("copy edit"),
          },
        ],
        acts: [
          { key: "sync", label: "Sincronizar", main: true, onPick: () => asked.push("sync") },
          {
            key: "stop",
            label: "Dejar",
            danger: true,
            onPick: (at) => asked.push(`at ${at.textContent}`),
          },
          { key: "wait", label: "Espera", off: true, onPick: () => asked.push("wait") },
          { key: "still", label: "Sin nada" },
        ],
      },
    ]);

    const row = side.querySelector<HTMLDetailsElement>("details[data-row]") as HTMLDetailsElement;
    expect(row.dataset.row).toBe("shared");
    expect(row.open).toBe(true);
    expect(text(side, ".side-row-label")).toEqual(["Compartido"]);
    expect(text(side, ".side-gist")).toEqual(["Solo leer"]);
    expect(side.querySelector(".side-gist")?.className).toBe("side-gist is-warn");
    expect(text(side, "p.side-said")).toEqual(["Caduca el lunes", "3 lecturas"]);
    expect(text(side, ".side-held .side-held-label")).toEqual(["Leer", "Editar"]);
    expect(text(side, ".side-held .side-held-url")).toEqual([
      "https://tisty.cl/a",
      "https://tisty.cl/b",
    ]);
    expect(text(side, ".side-held .side-held-note")).toEqual(["Solo en Tisty"]);

    const copies = side.querySelectorAll<HTMLButtonElement>(".side-held .side-link");
    expect(Array.from(copies, (one) => one.textContent)).toEqual(["Copiar", "Copiar"]);
    for (const copy of copies) copy.click();

    const acts = Array.from(
      side.querySelectorAll<HTMLButtonElement>(".side-acts.is-inside > button"),
    );
    expect(acts.map((one) => [one.textContent, one.className, one.disabled])).toEqual([
      ["Sincronizar", "side-act is-main", false],
      ["Dejar", "side-act is-danger", false],
      ["Espera", "side-act", true],
      ["Sin nada", "side-act", false],
    ]);
    for (const act of acts) act.click();

    expect(asked).toEqual(["copy read", "copy edit", "sync", "at Dejar"]);
  });

  it("leaves out of a held row what it was not given, and starts it shut", () => {
    const side = host();
    sided(side, say, [
      { kind: "held", key: "shared", label: "Compartido", gist: "Sin compartir", acts: [] },
    ]);

    expect(side.querySelector<HTMLDetailsElement>("details[data-row]")?.open).toBe(false);
    expect(side.querySelector(".side-gist")?.className).toBe("side-gist");
    expect(side.querySelector(".side-said")).toBeNull();
    expect(side.querySelector(".side-held")).toBeNull();
    expect(side.querySelector(".side-acts")).toBeNull();
  });

  it("puts a held row's mark beside its name, and none when it has none", () => {
    const side = host();
    sided(side, say, [
      { kind: "held", key: "shared", label: "Compartido", mark: "beta", gist: "Sin compartir" },
    ]);

    const head = side.querySelector(".side-row-head");
    expect(head?.children[1]?.className).toBe("side-row-mark");
    expect(head?.querySelector(".side-row-mark")?.textContent).toBe("beta");

    const plain = host();
    sided(plain, say, [{ kind: "held", key: "s", label: "L", gist: "G" }]);
    expect(plain.querySelector(".side-row-mark")).toBeNull();
  });

  it("tells the bad tone of a held row apart from the warning", () => {
    const side = host();
    sided(side, say, [{ kind: "held", key: "s", label: "L", gist: "G", tone: "bad" }]);

    expect(side.querySelector(".side-gist")?.className).toBe("side-gist is-bad");
  });

  it("keeps a held row as the reader left it when the panels are drawn again", () => {
    const side = host();
    const panels: Panel[] = [
      { kind: "held", key: "shared", label: "Compartido", gist: "G", open: true },
    ];
    sided(side, say, panels);
    const row = side.querySelector<HTMLDetailsElement>("details[data-row]") as HTMLDetailsElement;
    row.querySelector("summary")?.click();
    row.open = false;

    sided(side, say, panels);

    expect(side.querySelector<HTMLDetailsElement>("details[data-row]")?.open).toBe(false);
  });

  it("draws a row of acts out in the open, not inside a held row", () => {
    const side = host();
    const asked: string[] = [];
    sided(side, say, [
      { kind: "acts", acts: [{ key: "a", label: "Abrir", onPick: () => asked.push("a") }] },
    ]);

    expect(side.querySelector(".side-acts")?.className).toBe("side-acts");
    side.querySelector<HTMLButtonElement>(".side-act")?.click();
    expect(asked).toEqual(["a"]);
  });

  it("counts the words of a body, without its code or its marks", () => {
    expect(wordsIn("# Compras\n\n- leche\n- pan")).toBe(3);
    expect(wordsIn("hola\n\n```\nlet x = 1\n```\n\nadios")).toBe(2);
  });
});

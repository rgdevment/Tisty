import type { Linkable } from "tisty-editor/edit";
import { afterEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { linkMatch, type TagMatch, tagList, tagMatch } from "../ui/pageTags";
import { type Hands, type PageWriting, writtenOn } from "../ui/pageWriting";

const TAGS = ["perro", "pera", "pina", "casa", "rina", "roger"];
const at = (text: string, tags: readonly string[] = TAGS) => tagMatch(text, text.length, tags);

describe("where a tag is offered", () => {
  it("offers the tags that start with the letters after a hash", () => {
    expect(at("hola #p")?.shown).toEqual(["perro", "pera", "pina"]);
    expect(at("#c")?.shown).toEqual(["casa"]);
  });

  it("replaces from the hash to the caret", () => {
    expect(at("hola #pe")).toMatchObject({ from: 5, to: 8 });
  });

  it("reaches through the rest of the word after the caret", () => {
    expect(tagMatch("hola #pe y más", 8, TAGS)).toMatchObject({ from: 5, to: 8 });
    expect(tagMatch("hola #pexx y más", 8, TAGS)).toMatchObject({ from: 5, to: 10 });
  });

  it("stops the rest of the word at the end of the line", () => {
    expect(tagMatch("#pe\nresto", 3, TAGS)).toMatchObject({ from: 0, to: 3 });
  });

  it("works on the line of the caret and not on the ones before", () => {
    expect(at("uno #p\ndos #c")?.shown).toEqual(["casa"]);
  });

  it("offers nothing for a hash alone", () => {
    expect(at("hola #")).toBeNull();
  });

  it("offers nothing for a hash followed by a digit", () => {
    expect(at("hola #1", ["1a"])).toBeNull();
  });

  it("offers nothing for a heading mark", () => {
    expect(at("# ")).toBeNull();
    expect(at("## p")).toBeNull();
    expect(at("# p")).toBeNull();
  });

  it("offers nothing for a hash in the middle of a word", () => {
    expect(at("a#p")).toBeNull();
    expect(at("hola a#p")).toBeNull();
  });

  it.each(["/", ".", ":", "-", "_"])("offers nothing for a hash right after %s", (mark) => {
    expect(at(`hola${mark}#p`)).toBeNull();
    expect(at(`${mark}#p`)).toBeNull();
  });

  it("offers nothing for a second hash in a row", () => {
    expect(at("hola ##p")).toBeNull();
  });

  it("offers after an opening bracket, a quote or a comma", () => {
    expect(at("(#p")?.shown).toHaveLength(3);
    expect(at('"#p')?.shown).toHaveLength(3);
    expect(at("a,#p")?.shown).toHaveLength(3);
  });

  it("offers at the start of a later line", () => {
    expect(at("uno\n#p")?.shown).toHaveLength(3);
  });

  it("offers nothing inside inline code", () => {
    expect(at("`code #p")).toBeNull();
    expect(at("`a` y `b #p")).toBeNull();
  });

  it("offers again once the code is closed", () => {
    expect(at("`code` #p")?.shown).toHaveLength(3);
  });

  it("counts only the backticks of the caret's own line", () => {
    expect(at("`abierto\n#p")?.shown).toHaveLength(3);
  });

  it("offers nothing when the word is already a tag", () => {
    expect(at("#perro")).toBeNull();
    expect(at("#casa")).toBeNull();
  });

  it("still offers a tag that others are longer than", () => {
    expect(at("#per", ["per2", "perro"])?.shown).toEqual(["per2", "perro"]);
  });

  it("folds the accents of what was typed", () => {
    expect(at("#Pérr")?.shown).toEqual(["perro"]);
    expect(at("#pín")?.shown).toEqual(["pina"]);
  });

  it("folds an accent typed as a letter and a separate mark", () => {
    expect(at("#Pérr")?.shown).toEqual(["perro"]);
  });

  it("folds the case of what was typed", () => {
    expect(at("#PER")?.shown).toEqual(["perro", "pera"]);
  });

  it("takes a word typed in capitals as the tag already in use", () => {
    expect(at("#CASA")).toBeNull();
  });

  it("shows at most eight", () => {
    const many = Array.from({ length: 12 }, (_, index) => `tag${index}`);
    expect(at("#t", many)?.shown).toEqual(many.slice(0, 8));
  });

  it("offers nothing when no tag starts with it", () => {
    expect(at("#zzz")).toBeNull();
    expect(at("#erro")).toBeNull();
  });

  it("offers nothing when no tag is in use", () => {
    expect(at("#p", [])).toBeNull();
  });

  it("matches the letters of other alphabets", () => {
    expect(at("#ñan", ["nandu"])?.shown).toEqual(["nandu"]);
    expect(at("#Ñ", ["nandu"])?.shown).toEqual(["nandu"]);
    expect(at("#ΓΕ", ["γεια"])?.shown).toEqual(["γεια"]);
    expect(at("#γειά", ["γεια"])).toBeNull();
    expect(at("#über", ["uber"])).toBeNull();
    expect(at("#üb", ["uber", "ubung"])?.shown).toEqual(["uber", "ubung"]);
  });

  it("offers nothing right after a letter with a combining accent", () => {
    expect(at("é#p")).toBeNull();
  });

  it("carries on through hyphens, underscores and digits of the word", () => {
    expect(at("#mi-ca", ["mi-casa"])).toMatchObject({ from: 0, to: 6 });
    expect(at("#mi_c", ["mi_casa"])?.shown).toEqual(["mi_casa"]);
    expect(at("#a1", ["a1b"])?.shown).toEqual(["a1b"]);
  });

  it("offers nothing at the very start of the text", () => {
    expect(tagMatch("#p", 0, TAGS)).toBeNull();
    expect(tagMatch("", 0, TAGS)).toBeNull();
  });
});

const keyed = (name: string, init: KeyboardEventInit = {}, target: EventTarget = document.body) => {
  const event = new KeyboardEvent("keydown", {
    key: name,
    bubbles: true,
    cancelable: true,
    ...init,
  });
  target.dispatchEvent(event);
  return event;
};

const matched = (shown = ["rina", "roger", "ramon"]): TagMatch => ({ from: 3, to: 5, shown });

describe("the list of tags", () => {
  let list: ReturnType<typeof tagList> | null = null;
  const picks: [TagMatch, string][] = [];

  const opened = (match: TagMatch | null = matched()) => {
    picks.length = 0;
    document.body.append(document.createTextNode("hola #r"));
    document.getSelection()?.collapse(document.body.lastChild, 7);
    list = tagList(document, (found, tag) => picks.push([found, tag]));
    list.offer(match);
    return document.querySelector<HTMLElement>(".tag-list") as HTMLElement;
  };
  const lit = () =>
    Array.from(document.querySelectorAll("[role=option]")).findIndex(
      (one) => one.getAttribute("aria-selected") === "true",
    );

  afterEach(() => {
    list?.stop();
    list = null;
    document.body.innerHTML = "";
  });

  it("is a hidden listbox until it has something to offer", () => {
    const shown = opened(null);
    expect(shown.getAttribute("role")).toBe("listbox");
    expect(shown.hidden).toBe(true);
  });

  it("shows one option for each tag, written with its hash", () => {
    const shown = opened();
    expect(shown.hidden).toBe(false);
    expect(
      Array.from(shown.querySelectorAll("[role=option]")).map((one) => one.textContent),
    ).toEqual(["#rina", "#roger", "#ramon"]);
    expect(lit()).toBe(-1);
  });

  it("lights the first option with ArrowDown and goes on one by one, then back to the first", () => {
    opened();
    const seen = [1, 2, 3, 4].map(() => {
      keyed("ArrowDown");
      return lit();
    });
    expect(seen).toEqual([0, 1, 2, 0]);
  });

  it("lights the first option with ArrowUp from none, and cycles upward", () => {
    opened();
    keyed("ArrowUp");
    expect(lit()).toBe(0);
    keyed("ArrowUp");
    expect(lit()).toBe(2);
    keyed("ArrowUp");
    expect(lit()).toBe(1);
  });

  it("lights only one option at a time", () => {
    opened();
    keyed("ArrowDown");
    keyed("ArrowDown");
    expect(document.querySelectorAll('[aria-selected="true"]')).toHaveLength(1);
  });

  it("keeps an arrow from reaching anything else", () => {
    opened();
    const heard = vi.fn();
    document.addEventListener("keydown", heard);
    const event = keyed("ArrowDown");
    document.removeEventListener("keydown", heard);
    expect(event.defaultPrevented).toBe(true);
    expect(heard).not.toHaveBeenCalled();
  });

  it("picks the first option with Tab when none is lit", () => {
    opened();
    const event = keyed("Tab");
    expect(event.defaultPrevented).toBe(true);
    expect(picks).toEqual([[matched(), "rina"]]);
  });

  it("picks the lit option with Tab", () => {
    opened();
    keyed("ArrowDown");
    keyed("ArrowDown");
    keyed("Tab");
    expect(picks.map(([, tag]) => tag)).toEqual(["roger"]);
  });

  it("picks the lit option with Enter", () => {
    opened();
    keyed("ArrowUp");
    const event = keyed("Enter");
    expect(event.defaultPrevented).toBe(true);
    expect(picks.map(([, tag]) => tag)).toEqual(["rina"]);
  });

  it("lets Enter pass untouched while no option is lit", () => {
    opened();
    const heard = vi.fn();
    document.addEventListener("keydown", heard);
    const event = keyed("Enter");
    document.removeEventListener("keydown", heard);
    expect(event.defaultPrevented).toBe(false);
    expect(heard).toHaveBeenCalledOnce();
    expect(picks).toEqual([]);
  });

  it("hides after a pick", () => {
    const shown = opened();
    keyed("Tab");
    expect(shown.hidden).toBe(true);
  });

  it("picks once, however many times Tab is pressed", () => {
    opened();
    keyed("Tab");
    keyed("Tab");
    expect(picks).toHaveLength(1);
  });

  it("hides on Escape and keeps it from every other listener", () => {
    const shown = opened();
    const heard = vi.fn();
    document.addEventListener("keydown", heard);
    const event = keyed("Escape");
    document.removeEventListener("keydown", heard);
    expect(shown.hidden).toBe(true);
    expect(event.defaultPrevented).toBe(true);
    expect(heard).not.toHaveBeenCalled();
  });

  it("lets Escape through once it is hidden", () => {
    const shown = opened();
    keyed("Escape");
    const heard = vi.fn();
    document.addEventListener("keydown", heard);
    keyed("Escape");
    document.removeEventListener("keydown", heard);
    expect(shown.hidden).toBe(true);
    expect(heard).toHaveBeenCalledOnce();
  });

  it("lets every key through while it is hidden", () => {
    opened(null);
    for (const name of ["ArrowDown", "ArrowUp", "Tab", "Enter"]) {
      expect(keyed(name).defaultPrevented).toBe(false);
    }
    expect(picks).toEqual([]);
  });

  it("lets other keys through while it is open", () => {
    opened();
    for (const name of ["a", "ArrowLeft", "Backspace", " "]) {
      expect(keyed(name).defaultPrevented).toBe(false);
    }
  });

  it("picks the option pressed with the mouse, without taking the page's focus", () => {
    const shown = opened();
    const press = new MouseEvent("mousedown", { bubbles: true, cancelable: true });
    shown.querySelectorAll("[role=option]")[1]?.dispatchEvent(press);
    expect(press.defaultPrevented).toBe(true);
    expect(picks).toEqual([[matched(), "roger"]]);
    expect(shown.hidden).toBe(true);
  });

  it.each(["ctrlKey", "metaKey", "altKey"])("ignores a key held with %s", (modifier) => {
    opened();
    for (const name of ["ArrowDown", "ArrowUp", "Tab", "Enter", "Escape"]) {
      expect(keyed(name, { [modifier]: true }).defaultPrevented).toBe(false);
    }
    expect(lit()).toBe(-1);
    expect(picks).toEqual([]);
  });

  it("is hidden by offering nothing", () => {
    const shown = opened();
    keyed("ArrowDown");
    list?.offer(null);
    expect(shown.hidden).toBe(true);
    expect(keyed("Tab").defaultPrevented).toBe(false);
    expect(picks).toEqual([]);
  });

  it("starts with nothing lit again when it is offered anew", () => {
    opened();
    keyed("ArrowDown");
    list?.offer(matched(["uno", "dos"]));
    expect(lit()).toBe(-1);
    expect(document.querySelectorAll("[role=option]")).toHaveLength(2);
    keyed("ArrowUp");
    keyed("Tab");
    expect(picks.map(([, tag]) => tag)).toEqual(["uno"]);
  });

  it("is hidden by hide, and picks nothing afterwards", () => {
    const shown = opened();
    list?.hide();
    expect(shown.hidden).toBe(true);
    keyed("Tab");
    expect(picks).toEqual([]);
  });

  it("cycles through a single option", () => {
    opened(matched(["solo"]));
    keyed("ArrowDown");
    keyed("ArrowDown");
    expect(lit()).toBe(0);
    keyed("ArrowUp");
    expect(lit()).toBe(0);
  });

  it("leaves no list and no listener behind once stopped", () => {
    opened();
    list?.stop();
    expect(document.querySelector(".tag-list")).toBeNull();
    expect(keyed("Tab").defaultPrevented).toBe(false);
    expect(keyed("Escape").defaultPrevented).toBe(false);
    expect(picks).toEqual([]);
  });

  it("stops without trouble when it was never offered anything", () => {
    opened(null);
    expect(() => list?.stop()).not.toThrow();
  });
});

const BODY = "# Hola\n\nUno **dos** tres\n\n- primero\n- segundo\n";
const PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-24"><span data-src="8-12">Uno </span><strong><span data-src="14-17">dos</span></strong><span data-src="19-24"> tres</span></p>' +
  '<ul data-at="26-46"><li data-at="26-35"><span data-src="28-35">primero</span></li>' +
  '<li data-at="36-45"><span data-src="38-45">segundo</span></li></ul>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const pageOf = (html = PLACED) => {
  document.body.innerHTML = `<article class="page">${html}</article>`;
  return document;
};

const piece = (said: string) =>
  Array.from(document.querySelectorAll("[data-src]")).find((one) => one.textContent === said)
    ?.firstChild as Text;

const caretIn = (said: string, place: number) =>
  document.getSelection()?.collapse(piece(said), place);

const asked = (inputType: string, data: string | null = null) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(
      new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
    );

const writer = (body = BODY, more: Partial<Hands> = {}) => {
  const said: string[] = [];
  let text = body;
  writing = writtenOn(pageOf(), {
    text: () => text,
    write: (next) => {
      text = next;
      said.push(next);
    },
    shown: () => {},
    redraw: () => {},
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translator: { fromHtml: async () => "", fromText: async () => null },
    ask: () => {},
    blocks: () => {},
    acts: () => {},
    ...more,
  });
  return said;
};

const last = (said: string[]) => said[said.length - 1] ?? "";
const listed = () => document.querySelector<HTMLElement>(".tag-list");
const options = () =>
  Array.from(document.querySelectorAll(".tag-list [role=option]")).map((one) => one.textContent);
const pressed = (name: string, init: KeyboardEventInit = {}) =>
  keyed(name, init, document.querySelector(".page") as Element);

const typed = (words: string, tags = ["rina", "roger", "casa"]) => {
  const said = writer(BODY, { tags: () => tags });
  caretIn(" tres", 5);
  asked("insertText", words);
  writing?.drawn();
  return said;
};

describe("the tags offered while writing on the page", () => {
  it("offers the tags after a space and a hash once the page is drawn", () => {
    typed(" #r");
    expect(listed()?.hidden).toBe(false);
    expect(options()).toEqual(["#rina", "#roger"]);
  });

  it("offers nothing before the page is drawn again", () => {
    writer(BODY, { tags: () => ["rina"] });
    caretIn(" tres", 5);
    asked("insertText", " #r");
    expect(listed()?.hidden).toBe(true);
  });

  it("writes the tag in place of the word typed", () => {
    const said = typed(" #ro");
    pressed("Tab");
    expect(last(said)).toBe("# Hola\n\nUno **dos** tres #roger \n\n- primero\n- segundo\n");
  });

  it("writes the tag lit with the arrows and Enter", () => {
    const said = typed(" #r");
    pressed("ArrowDown");
    pressed("ArrowDown");
    const enter = pressed("Enter");
    expect(enter.defaultPrevented).toBe(true);
    expect(last(said)).toContain("tres #roger \n");
  });

  it("writes the tag chosen with the mouse", () => {
    const said = typed(" #r");
    listed()
      ?.querySelectorAll("[role=option]")[0]
      ?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, cancelable: true }));
    expect(last(said)).toContain("tres #rina \n");
    expect(listed()?.hidden).toBe(true);
  });

  it("never opens for the hash of a heading", () => {
    writer(BODY, { tags: () => ["rina"] });
    caretIn("Hola", 4);
    asked("insertText", "x");
    writing?.drawn();
    expect(listed()?.hidden).toBe(true);
  });

  it("never opens for a heading hash typed at the start of a paragraph", () => {
    writer(BODY, { tags: () => ["rina", "hola"] });
    caretIn(" tres", 5);
    asked("insertParagraph");
    asked("insertText", "# ");
    writing?.drawn();
    expect(listed()?.hidden).toBe(true);
  });

  it("offers nothing for a hash the caret was only placed after", () => {
    writer(BODY, { tags: () => ["rina"] });
    caretIn(" tres", 5);
    asked("insertText", " #r");
    pressed("ArrowRight");
    writing?.drawn();
    expect(listed()?.hidden).toBe(true);
  });

  it("offers nothing when the writer is given no tags", () => {
    writer();
    caretIn(" tres", 5);
    asked("insertText", " #r");
    writing?.drawn();
    expect(listed()?.hidden).toBe(true);
  });

  it("still ends the line with Enter while no tag is lit", () => {
    const said = typed(" #r");
    const enter = pressed("Enter");
    expect(enter.defaultPrevented).toBe(false);
    asked("insertParagraph");
    expect(last(said)).toContain("tres #r\n\n");
  });

  it("puts the list away on Escape without leaving the page", () => {
    typed(" #r");
    const outside = vi.fn();
    document.addEventListener("keydown", outside);
    pressed("Escape");
    document.removeEventListener("keydown", outside);
    expect(listed()?.hidden).toBe(true);
    expect(outside).not.toHaveBeenCalled();
  });

  it("puts the list away when the caret moves with an arrow key", () => {
    typed(" #r");
    pressed("ArrowLeft");
    expect(listed()?.hidden).toBe(true);
  });

  it("puts the list away when the caret moves with End", () => {
    typed(" #r");
    pressed("End");
    expect(listed()?.hidden).toBe(true);
  });

  it("puts the list away on a press of the mouse in the page", () => {
    typed(" #r");
    document
      .querySelector(".page")
      ?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, cancelable: true }));
    expect(listed()?.hidden).toBe(true);
  });

  it("keeps the list when a key is pressed on the list itself", () => {
    typed(" #r");
    keyed("ArrowLeft", {}, listed() as Element);
    expect(listed()?.hidden).toBe(false);
  });

  it("closes the list when the word stops being a tag in the making", () => {
    typed(" #r");
    asked("insertText", " ");
    writing?.drawn();
    expect(listed()?.hidden).toBe(true);
  });

  it("closes the list when the tag is complete", () => {
    typed(" #casa", ["casa", "casita"]);
    expect(listed()?.hidden).toBe(true);
  });

  it("offers nothing inside inline code", () => {
    typed(" `#r");
    expect(listed()?.hidden).toBe(true);
  });

  it("leaves no list in the document once stopped", () => {
    typed(" #r");
    writing?.stop();
    writing = null;
    expect(listed()).toBeNull();
  });
});

const fileOf = (name = "captura.png") => new File(["x"], name, { type: "image/png" });

const pasteOf = (data: unknown, target: Element | null = document.querySelector(".page")) => {
  const event = new Event("paste", { bubbles: true, cancelable: true });
  Object.defineProperty(event, "clipboardData", { value: data });
  target?.dispatchEvent(event);
  return event;
};

const pictured = (files = [fileOf()]) => ({ files, getData: () => "" });
const settled = async () => {
  for (let turn = 0; turn < 6; turn += 1) await Promise.resolve();
};

describe("the files pasted on the page", () => {
  it("attaches a picture after the block the caret stands in, and holds the paste", async () => {
    const said = writer(BODY, { pasted: () => Promise.resolve("![captura](captura.png)") });
    caretIn("dos", 1);
    const event = pasteOf(pictured());
    expect(event.defaultPrevented).toBe(true);
    await settled();
    expect(last(said)).toContain("Uno **dos** tres\n\n![captura](captura.png)");
    expect(last(said).startsWith("# Hola\n\nUno **dos** tres")).toBe(true);
    expect(last(said)).toContain("- primero");
  });

  it("hands every file on the clipboard over, in order", async () => {
    const given: File[] = [];
    writer(BODY, {
      pasted: (files) => {
        given.push(...files);
        return Promise.resolve("");
      },
    });
    caretIn("dos", 1);
    const [one, two] = [fileOf("a.png"), fileOf("b.png")];
    pasteOf(pictured([one as File, two as File]));
    await settled();
    expect(given).toEqual([one, two]);
  });

  it("attaches past the last block when the caret is in it", async () => {
    const said = writer(BODY, { pasted: () => Promise.resolve("![x](x.png)") });
    caretIn("segundo", 3);
    pasteOf(pictured());
    await settled();
    expect(last(said).indexOf("![x](x.png)")).toBeGreaterThan(last(said).indexOf("- segundo"));
  });

  it("attaches after the title when the caret stands in it", async () => {
    const said = writer(BODY, { pasted: () => Promise.resolve("![x](x.png)") });
    caretIn("Hola", 2);
    pasteOf(pictured());
    await settled();
    expect(last(said).startsWith("# Hola\n\n![x](x.png)")).toBe(true);
  });

  it("does nothing here when words come with the files", async () => {
    const said = writer(BODY, { pasted: () => Promise.resolve("![x](x.png)") });
    caretIn("dos", 1);
    const event = pasteOf({ files: [fileOf()], getData: () => "palabras" });
    await settled();
    expect(event.defaultPrevented).toBe(false);
    expect(said).toEqual([]);
  });

  it("does nothing when no file is on the clipboard", async () => {
    const pasted = vi.fn(() => Promise.resolve("x"));
    const said = writer(BODY, { pasted });
    caretIn("dos", 1);
    const event = pasteOf({ files: [], getData: () => "" });
    await settled();
    expect(event.defaultPrevented).toBe(false);
    expect(pasted).not.toHaveBeenCalled();
    expect(said).toEqual([]);
  });

  it("does nothing when the clipboard is missing", () => {
    const pasted = vi.fn(() => Promise.resolve("x"));
    writer(BODY, { pasted });
    caretIn("dos", 1);
    const event = pasteOf(null);
    expect(event.defaultPrevented).toBe(false);
    expect(pasted).not.toHaveBeenCalled();
  });

  it("leaves the paste alone when the writer cannot keep files", async () => {
    const said = writer();
    caretIn("dos", 1);
    const event = pasteOf(pictured());
    await settled();
    expect(event.defaultPrevented).toBe(false);
    expect(said).toEqual([]);
  });

  it("leaves a paste into a field inside the page alone", async () => {
    const pasted = vi.fn(() => Promise.resolve("![x](x.png)"));
    const said = writer(BODY, { pasted });
    const field = document.createElement("input");
    document.querySelector("ul")?.append(field);
    caretIn("dos", 1);
    const event = pasteOf(pictured(), field);
    await settled();
    expect(event.defaultPrevented).toBe(false);
    expect(pasted).not.toHaveBeenCalled();
    expect(said).toEqual([]);
  });

  it("writes nothing when the files give back no words", async () => {
    const said = writer(BODY, { pasted: () => Promise.resolve("") });
    caretIn("dos", 1);
    const event = pasteOf(pictured());
    await settled();
    expect(event.defaultPrevented).toBe(true);
    expect(said).toEqual([]);
  });

  it("breaks nothing when the files are refused", async () => {
    const said = writer(BODY, { pasted: () => Promise.reject(new Error("too big")) });
    caretIn("dos", 1);
    const lost: unknown[] = [];
    const heard = (event: PromiseRejectionEvent) => lost.push(event.reason);
    window.addEventListener("unhandledrejection", heard);
    const event = pasteOf(pictured());
    await settled();
    window.removeEventListener("unhandledrejection", heard);
    expect(event.defaultPrevented).toBe(true);
    expect(said).toEqual([]);
    expect(lost).toEqual([]);
  });

  it("holds the paste even when no caret is in the page", async () => {
    const pasted = vi.fn(() => Promise.resolve("![x](x.png)"));
    const said = writer(BODY, { pasted });
    document.getSelection()?.removeAllRanges();
    const event = pasteOf(pictured());
    await settled();
    expect(event.defaultPrevented).toBe(true);
    expect(said).toEqual([]);
  });

  it("stops hearing pastes once stopped", () => {
    const pasted = vi.fn(() => Promise.resolve("![x](x.png)"));
    writer(BODY, { pasted });
    caretIn("dos", 1);
    const target = document.querySelector(".page");
    writing?.stop();
    writing = null;
    const event = pasteOf(pictured(), target);
    expect(event.defaultPrevented).toBe(false);
    expect(pasted).not.toHaveBeenCalled();
  });
});

const TRIPS: Linkable = {
  key: "a",
  label: "Viajes",
  detail: "Libro",
  link: "[Viajes](tisty:doc/a)",
};
const linkable = (asked: string) => (asked.toLowerCase().startsWith("vi") ? [TRIPS] : []);
const linksShown = () =>
  document.querySelector<HTMLElement>(`.tag-list[aria-label="${t("linkDocs")}"]`);

describe("documents offered after [[ while writing on the page", () => {
  it("reads [[ and the words after it on the caret's line, and leaves [[# to the steps", () => {
    const ready = () => {};
    expect(linkMatch("uno [[Vi", 8, { linkable }, ready)).toEqual({
      from: 4,
      to: 8,
      shown: [TRIPS],
    });
    expect(linkMatch("uno [[Vi]] y", 8, { linkable }, ready)?.to).toBe(10);
    expect(linkMatch("uno [[#2", 8, { linkable }, ready)).toBeNull();
    expect(linkMatch("uno `[[Vi", 9, { linkable }, ready)).toBeNull();
    expect(linkMatch("uno [[zz", 8, { linkable }, ready)).toBeNull();
  });

  it("lights the first document and writes its link with Enter in place of what was typed", () => {
    const said = writer(BODY, { linkable });
    caretIn(" tres", 5);
    asked("insertText", " [[vi");
    writing?.drawn();

    expect(linksShown()?.hidden).toBe(false);
    expect(linksShown()?.querySelector("[aria-selected=true]")?.textContent).toBe("ViajesLibro");
    const enter = pressed("Enter");

    expect(enter.defaultPrevented).toBe(true);
    expect(last(said)).toContain("tres [Viajes](tisty:doc/a)\n");
  });
});

describe("one list at the caret", () => {
  it("offers the headings after [[Title # and keeps the tags list shut", () => {
    const heads: Linkable = {
      key: "a#casa",
      label: "Casa",
      link: "[Viajes › Casa](tisty:doc/a#casa)",
    };
    writer(BODY, { tags: () => ["casa"], linkable, headsOf: () => [heads] });
    caretIn(" tres", 5);
    asked("insertText", " [[Viajes #ca");
    writing?.drawn();

    expect(linksShown()?.hidden).toBe(false);
    expect(listed()?.hidden).toBe(true);
  });
});

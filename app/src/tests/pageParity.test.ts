import { afterEach, describe, expect, it, vi } from "vitest";
import type { PageWriting } from "../ui/pageWriting";

const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)";
const WINDOWS = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";

const on = (agent: string) => {
  vi.resetModules();
  vi.spyOn(navigator, "userAgent", "get").mockReturnValue(agent);
};

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  vi.restoreAllMocks();
  localStorage.removeItem("tisty.nativeTyping");
  document.body.innerHTML = "";
});

const BODY = "# Hola\n\nUno dos tres\n\nTras\n";
const PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-21"><span data-src="8-20">Uno dos tres</span></p>' +
  '<p data-at="22-26"><span data-src="22-26">Tras</span></p>';

const written = async (agent: string, body = BODY, placed = PLACED) => {
  on(agent);
  const { writtenOn } = await import("../ui/pageWriting");
  document.body.innerHTML = `<article class="page">${placed}</article>`;
  const said: string[] = [];
  let text = body;
  writing = writtenOn(document, {
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
  });
  return said;
};

const piece = (said: string) =>
  Array.from(document.querySelectorAll("[data-src]")).find((one) => one.textContent === said)
    ?.firstChild as Text;

const sheet = () => document.querySelector(".page") as HTMLElement;

const keyed = (key: string, init: KeyboardEventInit & { keyCode?: number } = {}) => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init });
  sheet().dispatchEvent(event);
  return event;
};

describe("the menu click and the paste shortcut a Mac names", () => {
  it("counts Control with a click as the right button on a Mac only", async () => {
    on(MAC);
    const mac = await import("../ui/macKeys");
    expect(mac.menuClick({ button: 0, ctrlKey: true })).toBe(true);
    expect(mac.menuClick({ button: 2, ctrlKey: false })).toBe(true);
    expect(mac.menuClick({ button: 0, ctrlKey: false })).toBe(false);

    on(WINDOWS);
    const windows = await import("../ui/macKeys");
    expect(windows.menuClick({ button: 0, ctrlKey: true })).toBe(false);
    expect(windows.menuClick({ button: 2, ctrlKey: false })).toBe(true);
  });

  it("names paste without format with Option on a Mac and Shift alone elsewhere", async () => {
    on(MAC);
    expect((await import("../ui/macKeys")).PASTE_PLAIN).toBe("⌥ Shift V");
    on(WINDOWS);
    expect((await import("../ui/macKeys")).PASTE_PLAIN).toBe("Shift V");
  });

  it("shows the menu's paste rows with the keys of the machine", async () => {
    const rows = async (agent: string) => {
      on(agent);
      const { pageChoices } = await import("../ui/pageMenu");
      const asked = {
        words: null,
        cuttable: false,
        said: null,
        spelled: null,
        formattable: false,
        limited: false,
        block: false,
        pasting: true,
        pasteByKeys: false,
      };
      const acts = {
        guessed: vi.fn(),
        learned: vi.fn(),
        cut: vi.fn(),
        copy: vi.fn(),
        paste: vi.fn(),
        format: vi.fn(),
        insert: vi.fn(),
        blockActed: vi.fn(),
      };
      return pageChoices(asked, acts).map((one) => one.keys);
    };

    expect(await rows(MAC)).toEqual(["⌘ V", "⌘ ⌥ Shift V"]);
    expect(await rows(WINDOWS)).toEqual(["Ctrl V", "Ctrl Shift V"]);
  });
});

describe("words carried by the pointer on a Mac", () => {
  const choose = () => {
    const range = document.createRange();
    range.setStart(piece("Uno dos tres"), 0);
    range.setEnd(piece("Uno dos tres"), 7);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
  };
  const pointing = () =>
    Object.defineProperty(document, "caretRangeFromPoint", {
      configurable: true,
      value: () => {
        const range = document.createRange();
        range.setStart(piece("Uno dos tres"), 2);
        return range;
      },
    });
  const mouse = (kind: string, init: MouseEventInit = {}) => {
    const event = new MouseEvent(kind, {
      bubbles: true,
      cancelable: true,
      button: 0,
      clientX: 10,
      detail: 1,
      ...init,
    });
    (kind === "mousedown" ? piece("Uno dos tres").parentElement : document)?.dispatchEvent(event);
    return event;
  };

  afterEach(() => Reflect.deleteProperty(document, "caretRangeFromPoint"));

  it("does not start a drag nor collapse the words chosen on a click with Control held", async () => {
    await written(MAC);
    choose();
    pointing();

    expect(mouse("mousedown", { ctrlKey: true }).defaultPrevented).toBe(false);
    mouse("mouseup", { ctrlKey: true });

    expect(document.getSelection()?.isCollapsed).toBe(false);
    expect(document.getSelection()?.toString()).toBe("Uno dos");
  });

  it("still takes a plain click on the words chosen for a drag", async () => {
    await written(MAC);
    choose();
    pointing();

    expect(mouse("mousedown").defaultPrevented).toBe(true);
    mouse("mouseup");

    expect(document.getSelection()?.isCollapsed).toBe(true);
  });

  it("takes a click with Control held for a drag where Control is no menu", async () => {
    await written(WINDOWS);
    choose();
    pointing();

    expect(mouse("mousedown", { ctrlKey: true }).defaultPrevented).toBe(true);
  });
});

const standing = async (agent: string) => {
  on(agent);
  const { keysOn } = await import("../ui/pageKeys");
  document.body.innerHTML = `<article class="page">${PLACED}</article>`;
  const line = piece("Uno dos tres");
  document.getSelection()?.collapse(line, 3);
  const kept = { node: line, offset: 3, at: 11 };
  const writer = {
    page: document,
    hands: { text: () => BODY },
    kept: kept as object | null,
    pending: {} as object | null,
    carried: {} as object | null,
    typing: true,
    leftFrom: undefined as number | undefined,
  };
  const caret = {
    chosen: () => document.getSelection()?.getRangeAt(0) ?? null,
    blockStart: () => 8,
    caretSpan: () => ({ from: 11, to: 11 }),
    solidsLeft: vi.fn(),
  };
  const choose = vi.fn();
  const keys = keysOn(
    writer as never,
    caret as never,
    {} as never,
    {} as never,
    {} as never,
    {} as never,
    { hide: vi.fn() },
    { choose } as never,
  );
  return { writer, kept, caret, choose, keys, line };
};

describe("the line being written when its menu is asked for", () => {
  const clicked = async (agent: string, init: MouseEventInit) => {
    const { writer, kept, keys, line } = await standing(agent);
    const block = line.parentElement as Element;
    block.addEventListener("mousedown", keys.moved);
    block.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, button: 0, ...init }));
    return { writer, kept };
  };

  it("keeps the place kept for it on a click with Control held on a Mac", async () => {
    const { writer, kept } = await clicked(MAC, { ctrlKey: true });
    expect(writer.kept).toBe(kept);
  });

  it("keeps it for the right button, on a Mac and elsewhere", async () => {
    const mac = await clicked(MAC, { button: 2 });
    expect(mac.writer.kept).toBe(mac.kept);
    const windows = await clicked(WINDOWS, { button: 2 });
    expect(windows.writer.kept).toBe(windows.kept);
  });

  it("lets go of it on a plain click, and on a click with Control held elsewhere", async () => {
    expect((await clicked(MAC, {})).writer.kept).toBeNull();
    expect((await clicked(WINDOWS, { ctrlKey: true })).writer.kept).toBeNull();
  });
});

describe("the key that ends a composition on WebKit", () => {
  it("is left to the composition by Escape, which comes with keyCode 229 and no isComposing", async () => {
    const { choose, keys, line } = await standing(MAC);
    const press = (init: KeyboardEventInit & { keyCode?: number }) =>
      keys.keyed(
        Object.defineProperty(
          new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true, ...init }),
          "target",
          { value: line.parentElement },
        ),
      );

    press({ keyCode: 229 });
    expect(choose).not.toHaveBeenCalled();
    press({});
    expect(choose).toHaveBeenCalledTimes(1);
  });

  it("is not taken as a Backspace that leaves the solids behind", async () => {
    const { caret, keys, line } = await standing(MAC);
    const press = (init: KeyboardEventInit & { keyCode?: number }) =>
      keys.outOfSolids(
        Object.defineProperty(
          new KeyboardEvent("keydown", { key: "Backspace", bubbles: true, ...init }),
          "target",
          { value: line.parentElement },
        ),
      );

    press({ keyCode: 229 });
    expect(caret.solidsLeft).not.toHaveBeenCalled();
    press({});
    expect(caret.solidsLeft).toHaveBeenCalledTimes(1);
  });
});

describe("what a composition writes on WebKit", () => {
  const asked = (inputType: string, data: string | null = null) =>
    sheet().dispatchEvent(
      new InputEvent("beforeinput", {
        inputType,
        data,
        isComposing: false,
        bubbles: true,
        cancelable: true,
      }),
    );

  it.each([
    "insertCompositionText",
    "deleteCompositionText",
    "insertFromComposition",
    "deleteByComposition",
  ])("leaves %s to the composition, though no isComposing comes with it", async (inputType) => {
    const said = await written(MAC);
    document.getSelection()?.collapse(piece("Uno dos tres"), 3);

    expect(asked(inputType, "é")).toBe(true);
    expect(said).toEqual([]);
  });

  it("still takes the words typed plainly", async () => {
    const said = await written(MAC);
    document.getSelection()?.collapse(piece("Uno dos tres"), 3);

    asked("deleteContentBackward");
    expect(said.length).toBeGreaterThan(0);
  });

  it("does not hold a Delete that comes with keyCode 229 for a deletion of its own", async () => {
    const TWO = "# T\n\nHola mundo bonito\n\nSegundo párrafo\n";
    const DRAWN =
      '<h1 id="t" data-at="0-4"><span data-src="2-3">T</span></h1>' +
      '<p data-at="5-23"><span data-src="5-22">Hola mundo bonito</span></p>' +
      '<p data-at="24-41"><span data-src="24-40">Segundo párrafo</span></p>';
    localStorage.setItem("tisty.nativeTyping", "off");
    const typedX = async () => {
      const said = await written(MAC, TWO, DRAWN);
      document.getSelection()?.collapse(piece("Hola mundo bonito"), 17);
      keyed("X");
      asked("insertText", "X");
      return said;
    };

    await typedX();
    expect(keyed("Delete").defaultPrevented).toBe(true);

    await typedX();
    expect(keyed("Delete", { keyCode: 229 }).defaultPrevented).toBe(false);
  });
});

describe("Enter on a collapsible's name", () => {
  const FOLD = ":::details Nombre\n\nx\n\n:::\n";
  const DRAWN =
    '<details data-at="0-24"><summary>Nombre</summary>' +
    '<p data-at="19-20"><span data-src="19-20">x</span></p></details>';

  it("opens no line under the name when it ends a composition with keyCode 229", async () => {
    await written(MAC, FOLD, DRAWN);
    document.getSelection()?.collapse(document.querySelector("summary")?.firstChild as Text, 3);

    expect(keyed("Enter", { keyCode: 229 }).defaultPrevented).toBe(false);
    expect(keyed("Enter", { keyCode: 229, shiftKey: true }).defaultPrevented).toBe(false);
    expect(keyed("Enter").defaultPrevented).toBe(true);
  });
});

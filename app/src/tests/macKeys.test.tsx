import { act, fireEvent, render, renderHook, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import type { PageWriting } from "../ui/pageWriting";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  convertFileSrc: (path: string) => path,
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ label: "main", onFocusChanged: () => Promise.resolve(() => {}) }),
}));

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
  document.body.innerHTML = "";
});

const keyOn = (target: EventTarget, key: string, init: KeyboardEventInit = {}) => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init });
  act(() => {
    target.dispatchEvent(event);
  });
  return event;
};

const fieldIn = () => {
  const field = document.createElement("input");
  document.body.append(field);
  return field;
};

const BODY = "Uno dos\n\n- primero\n- segundo\n";
const PLACED =
  '<p data-at="0-7"><span data-src="0-7">Uno dos</span></p>' +
  '<ul data-at="9-29"><li data-at="9-18"><span data-src="11-18">primero</span></li>' +
  '<li data-at="19-28"><span data-src="21-28">segundo</span></li></ul>';

const written = async (body = BODY, placed = PLACED) => {
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
    translated: (_, plain) => Promise.resolve(plain),
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  return said;
};

const piece = (said: string) =>
  Array.from(document.querySelectorAll("[data-src]")).find((one) => one.textContent === said)
    ?.firstChild as Text;

const caretIn = (said: string, at: number) => document.getSelection()?.collapse(piece(said), at);

const typed = (data: string) =>
  document.querySelector(".page")?.dispatchEvent(
    new InputEvent("beforeinput", {
      inputType: "insertText",
      data,
      bubbles: true,
      cancelable: true,
    }),
  );

const sheet = () => document.querySelector(".page") as HTMLElement;

const EMPTY_STEP = "- [ ] uno\n- [ ] \n\nFin\n";
const EMPTY_STEP_PLACED =
  '<ul data-at="0-17"><li class="task" data-at="0-10"><span data-src="6-9">uno</span></li>' +
  '<li class="task" data-at="10-17"></li></ul><p data-at="18-21"><span data-src="18-21">Fin</span></p>';

describe("the chord a shortcut is pressed with", () => {
  it("is Cmd alone on a Mac, where Control moves through the text", async () => {
    on(MAC);
    const { modKey } = await import("../ui/macKeys");
    expect(modKey({ ctrlKey: true, metaKey: false })).toBe(false);
    expect(modKey({ ctrlKey: false, metaKey: true })).toBe(true);
  });

  it("is Ctrl or the Windows key elsewhere, as it always was", async () => {
    on(WINDOWS);
    const { modKey } = await import("../ui/macKeys");
    expect(modKey({ ctrlKey: true, metaKey: false })).toBe(true);
    expect(modKey({ ctrlKey: false, metaKey: true })).toBe(true);
    expect(modKey({ ctrlKey: false, metaKey: false })).toBe(false);
  });
});

describe("the window's keys on a Mac", () => {
  it("leaves Ctrl+E to the end of the line and switches the document with Cmd+E", async () => {
    on(MAC);
    const { useWholeKeys } = await import("../ui/wholeKeys");
    const keys = renderHook(() => useWholeKeys(true, true));
    expect(keyOn(document.body, "e", { ctrlKey: true }).defaultPrevented).toBe(false);
    expect(keys.result.current.wholeAsked).toBe(0);
    expect(keyOn(document.body, "e", { metaKey: true }).defaultPrevented).toBe(true);
    expect(keys.result.current.wholeAsked).toBe(1);
  });

  it("keeps Cmd+E from the menu in a field of the window, or held down, and switches nothing", async () => {
    on(MAC);
    const { useWholeKeys } = await import("../ui/wholeKeys");
    const keys = renderHook(() => useWholeKeys(true, true));
    expect(keyOn(fieldIn(), "e", { metaKey: true }).defaultPrevented).toBe(true);
    expect(keyOn(document.body, "e", { metaKey: true, repeat: true }).defaultPrevented).toBe(true);
    expect(keys.result.current.wholeAsked).toBe(0);
  });

  it("leaves Ctrl+E in a field as it was on Windows", async () => {
    on(WINDOWS);
    const { useWholeKeys } = await import("../ui/wholeKeys");
    const keys = renderHook(() => useWholeKeys(true, true));
    expect(keyOn(fieldIn(), "e", { ctrlKey: true }).defaultPrevented).toBe(false);
    expect(keyOn(document.body, "e", { ctrlKey: true, repeat: true }).defaultPrevented).toBe(false);
    expect(keys.result.current.wholeAsked).toBe(0);
  });

  it("leaves Ctrl+P, Ctrl+B and Ctrl+K to the text, and opens, folds and finds with Cmd", async () => {
    on(MAC);
    const { usePaletteKey } = await import("../ui/Palette");
    const { useFoldKey } = await import("../ui/SideEdge");
    const { useFindKey } = await import("../ui/finding");
    const opened = vi.fn();
    const found = vi.fn();
    renderHook(() => {
      usePaletteKey(false, opened);
      useFoldKey(true);
      useFindKey(true, found);
    });
    for (const key of ["p", "b", "k"])
      expect(keyOn(document.body, key, { ctrlKey: true }).defaultPrevented, key).toBe(false);
    expect(opened).not.toHaveBeenCalled();
    expect(found).not.toHaveBeenCalled();
    for (const key of ["p", "b", "k"])
      expect(keyOn(document.body, key, { metaKey: true }).defaultPrevented, key).toBe(true);
    expect(opened).toHaveBeenCalledTimes(1);
    expect(found).toHaveBeenCalledTimes(1);
  });
});

describe("the page written in place on a Mac", () => {
  it("leaves Ctrl+B, Ctrl+K, Ctrl+Y and Ctrl+Z to the text, and formats with Cmd", async () => {
    on(MAC);
    const said = await written();
    caretIn("Uno dos", 3);
    for (const key of ["b", "k", "y", "z", "i", "u"])
      expect(keyOn(sheet(), key, { ctrlKey: true }).defaultPrevented, key).toBe(false);
    expect(said).toEqual([]);
    expect(keyOn(sheet(), "b", { metaKey: true }).defaultPrevented).toBe(true);
    typed("x");
    expect(said[said.length - 1]?.startsWith("Uno**x** dos")).toBe(true);
  });

  it("keeps Ctrl's letters in a block being written, and passes Cmd's to the window", async () => {
    on(MAC);
    const { wired } = await import("../ui/dressing");
    const page = new DOMParser().parseFromString(
      '<body><div class="editing"><p>x</p></div></body>',
      "text/html",
    );
    wired(page, () => ({ body: "", onDoc: () => {} }));
    const heard: string[] = [];
    const listen = (event: KeyboardEvent) =>
      heard.push(`${event.metaKey ? "cmd+" : ""}${event.key}`);
    window.addEventListener("keydown", listen);
    const inside = page.querySelector("p") as HTMLElement;
    keyOn(inside, "e", { ctrlKey: true });
    keyOn(inside, "p", { ctrlKey: true });
    keyOn(inside, "e", { metaKey: true });
    window.removeEventListener("keydown", listen);
    expect(heard).toEqual(["cmd+e"]);
  });

  it("leaves Ctrl+E in a table's cell to the cell, which Cmd+E closes", async () => {
    on(MAC);
    const { typing } = await import("../ui/tabled");
    const cell = document.createElement("td");
    document.body.append(cell);
    const done = vi.fn();
    const input = typing(cell, "Spotify", "Cell", done);
    keyOn(input, "e", { ctrlKey: true });
    expect(done).not.toHaveBeenCalled();
    keyOn(input, "e", { metaKey: true });
    expect(done).toHaveBeenCalledWith("Spotify", "blur");
  });
});

describe("Control moving the caret on a Mac", () => {
  it("takes an empty step away when Control and N leave it, as the down arrow does", async () => {
    on(MAC);
    const said = await written(EMPTY_STEP, EMPTY_STEP_PLACED);
    writing?.drawn();
    document.getSelection()?.collapse(document.querySelectorAll("li")[1] as Element, 0);
    expect(keyOn(sheet(), "n", { ctrlKey: true }).defaultPrevented).toBe(false);
    caretIn("Fin", 1);
    document.dispatchEvent(new Event("selectionchange"));
    expect(said[said.length - 1]).toBe("- [ ] uno\n\nFin\n");
  });

  it("is no move off a Mac, where Control and a letter stay a shortcut", async () => {
    on(WINDOWS);
    const { caretWalk } = await import("../ui/macKeys");
    expect(
      caretWalk({
        key: "n",
        code: "KeyN",
        ctrlKey: true,
        metaKey: false,
        altKey: false,
        shiftKey: false,
      }),
    ).toBeNull();
  });

  it("is told by the letter written, or by the key pressed when the layout writes no Latin letter", async () => {
    on(MAC);
    const { caretWalk } = await import("../ui/macKeys");
    const held = { ctrlKey: true, metaKey: false, altKey: false, shiftKey: false };
    expect(caretWalk({ ...held, key: "т", code: "KeyN" })).toBe("line");
    expect(caretWalk({ ...held, key: "у", code: "KeyE" })).toBe("end");
    expect(caretWalk({ ...held, key: "a", code: "KeyQ" })).toBe("start");
    expect(caretWalk({ ...held, key: "1", code: "Digit1" })).toBeNull();
    expect(caretWalk({ ...held, key: ".", code: "KeyE" })).toBeNull();
    expect(caretWalk({ ...held, key: "Process", code: "KeyA" })).toBeNull();
    expect(caretWalk({ ...held, key: "ğ", code: "KeyE" })).toBeNull();
  });

  it("takes an empty step away when Control and N leave it on a Russian layout", async () => {
    on(MAC);
    const said = await written(EMPTY_STEP, EMPTY_STEP_PLACED);
    writing?.drawn();
    document.getSelection()?.collapse(document.querySelectorAll("li")[1] as Element, 0);
    keyOn(sheet(), "т", { ctrlKey: true, code: "KeyN" });
    caretIn("Fin", 1);
    document.dispatchEvent(new Event("selectionchange"));
    expect(said[said.length - 1]).toBe("- [ ] uno\n\nFin\n");
  });

  it("is told by the key pressed when the layout writes a vowel sign, as Devanagari and Thai do", async () => {
    on(MAC);
    const { caretWalk } = await import("../ui/macKeys");
    const held = { ctrlKey: true, metaKey: false, altKey: false, shiftKey: false };
    expect(caretWalk({ ...held, key: "ा", code: "KeyE" })).toBe("end");
    expect(caretWalk({ ...held, key: "ो", code: "KeyA" })).toBe("start");
    expect(caretWalk({ ...held, key: "ि", code: "KeyF" })).toBe("letter");
    expect(caretWalk({ ...held, key: "ื", code: "KeyN" })).toBe("line");
    expect(caretWalk({ ...held, key: "ิ", code: "KeyB" })).toBe("letter");
    expect(caretWalk({ ...held, key: "้", code: "KeyH" })).toBeNull();
  });
});

describe("letter shortcuts on a layout that writes no Latin letters", () => {
  it("formats and undoes by the key pressed, and by the letter written on a Latin layout", async () => {
    on(WINDOWS);
    const said = await written();
    caretIn("Uno dos", 3);
    expect(keyOn(sheet(), "ิ", { ctrlKey: true, code: "KeyB" }).defaultPrevented).toBe(true);
    typed("x");
    expect(said[said.length - 1]?.startsWith("Uno**x** dos")).toBe(true);
    expect(keyOn(sheet(), "w", { ctrlKey: true, code: "KeyZ" }).defaultPrevented).toBe(false);
    expect(keyOn(sheet(), "я", { ctrlKey: true, code: "KeyZ" }).defaultPrevented).toBe(true);
  });

  it("keeps chosen blocks through a copy pressed on a Russian layout", async () => {
    on(WINDOWS);
    await written();
    caretIn("Uno dos", 1);
    keyOn(sheet(), "Escape");
    keyOn(sheet(), "с", { ctrlKey: true, code: "KeyC" });
    expect(document.querySelectorAll(".chosen-block")).toHaveLength(1);
  });

  it("switches the whole document and chooses every word", async () => {
    on(WINDOWS);
    const { useWholeKeys } = await import("../ui/wholeKeys");
    const keys = renderHook(() => useWholeKeys(true, true));
    expect(keyOn(document.body, "у", { ctrlKey: true, code: "KeyE" }).defaultPrevented).toBe(true);
    expect(keyOn(document.body, "ф", { ctrlKey: true, code: "KeyA" }).defaultPrevented).toBe(true);
    expect(keys.result.current.wholeAsked).toBe(1);
    expect(keys.result.current.allAsked).toBe(1);
  });

  it("closes a table's cell", async () => {
    on(WINDOWS);
    const { typing } = await import("../ui/tabled");
    const cell = document.createElement("td");
    document.body.append(cell);
    const done = vi.fn();
    const input = typing(cell, "Spotify", "Cell", done);
    keyOn(input, "у", { ctrlKey: true, code: "KeyE" });
    expect(done).toHaveBeenCalledWith("Spotify", "blur");
  });

  it("opens the palette, folds the side and finds", async () => {
    on(WINDOWS);
    const { usePaletteKey } = await import("../ui/Palette");
    const { useFoldKey } = await import("../ui/SideEdge");
    const { useFindKey } = await import("../ui/finding");
    const opened = vi.fn();
    const found = vi.fn();
    renderHook(() => {
      usePaletteKey(false, opened);
      useFoldKey(true);
      useFindKey(true, found);
    });
    for (const [key, code] of [
      ["з", "KeyP"],
      ["и", "KeyB"],
      ["л", "KeyK"],
    ])
      expect(keyOn(document.body, key, { ctrlKey: true, code }).defaultPrevented, key).toBe(true);
    expect(opened).toHaveBeenCalledTimes(1);
    expect(found).toHaveBeenCalledTimes(1);
  });
});

describe("Control with A and E on a Mac", () => {
  const drawn = async (body: string, placed: string) => {
    await written(body, placed);
    writing?.drawn();
  };
  const walked = async (letter: string, from: [Node, number], more: KeyboardEventInit = {}) => {
    document.getSelection()?.collapse(from[0], from[1]);
    keyOn(sheet(), letter, { ctrlKey: true, code: `Key${letter.toUpperCase()}`, ...more });
    await new Promise((done) => setTimeout(done, 0));
    const held = document.getSelection();
    return [held?.anchorNode, held?.anchorOffset];
  };

  it("ends the line past a tag that closes it and the space before it, as End does", async () => {
    on(MAC);
    await drawn(
      "Uno #eti\n",
      '<p data-at="0-8"><span data-src="0-4">Uno </span><span class="tag" data-src="4-8" contenteditable="false">#eti</span></p>',
    );
    const line = document.querySelector("p") as Element;
    expect(await walked("e", [piece("Uno "), 4])).toEqual([line, 2]);
    expect(await walked("e", [piece("Uno "), 3])).toEqual([line, 2]);
    expect(await walked("e", [piece("Uno "), 1])).toEqual([piece("Uno "), 1]);
    expect(await walked("e", [piece("Uno "), 4], { isComposing: true })).toEqual([
      piece("Uno "),
      4,
    ]);
  });

  it("ends the line at its break, never past it", async () => {
    on(MAC);
    await drawn(
      "uno #eti\\\ndos\n",
      '<p data-at="0-14"><span data-src="0-4">uno </span><span class="tag" data-src="4-8" contenteditable="false">#eti</span>' +
        '<br data-src="8-10" contenteditable="false"><span data-src="10-13">dos</span></p>',
    );
    expect(await walked("e", [piece("uno "), 4])).toEqual([document.querySelector("p"), 2]);
  });

  it("leaves a line of code where it ends, never past the code's last line break", async () => {
    on(MAC);
    await drawn(
      "```\nfoo()\n```\n",
      '<figure class="code" data-at="0-14"><pre><code data-src="4-10">foo()\n</code></pre></figure>',
    );
    const code = document.querySelector("code")?.firstChild as Text;
    expect(await walked("e", [code, 5])).toEqual([code, 5]);
  });

  it("starts the line before a tag that opens it, in a step too, past the step's box", async () => {
    on(MAC);
    await drawn(
      "#eti uno\n\n- [ ] #eti dos\n",
      '<p data-at="0-8"><span class="tag" data-src="0-4" contenteditable="false">#eti</span><span data-src="4-8"> uno</span></p>' +
        '<ul data-at="10-25"><li class="task" data-at="10-25"><span class="check" contenteditable="false"></span>' +
        '<span class="tag" data-src="16-20" contenteditable="false">#eti</span><span data-src="20-24"> dos</span></li></ul>',
    );
    expect(await walked("a", [piece(" uno"), 0])).toEqual([document.querySelector("p"), 0]);
    const step = document.querySelector("li") as Element;
    expect(await walked("a", [piece(" dos"), 0])).toEqual([step, 2]);
    expect(step.childNodes[1]?.nodeName).toBe("WBR");
  });
});

describe("a key to a line's edge on a Mac, after the system has moved the caret", () => {
  const pressed = async (
    body: string,
    placed: string,
    key: KeyboardEventInit & { key: string },
    from: () => [Node, number],
    system: () => [Node, number],
  ) => {
    await written(body, placed);
    writing?.drawn();
    const [node, offset] = from();
    document.getSelection()?.collapse(node, offset);
    keyOn(sheet(), key.key, key);
    document.getSelection()?.collapse(...system());
    await new Promise((done) => setTimeout(done, 0));
    const held = document.getSelection();
    return [held?.anchorNode, held?.anchorOffset];
  };
  const controlE = { key: "e", code: "KeyE", ctrlKey: true };
  const controlA = { key: "a", code: "KeyA", ctrlKey: true };
  const cmdLeft = { key: "ArrowLeft", metaKey: true };
  const cmdRight = { key: "ArrowRight", metaKey: true };
  const BROKEN =
    '<p data-at="0-19"><span data-src="0-4">uno </span><span class="tag" data-src="4-8" contenteditable="false">#eti</span>' +
    '<br data-src="8-10" contenteditable="false"><span data-src="10-18">dos tres</span></p>';
  const STEPS =
    '<ul data-at="0-21"><li class="task" data-at="0-10"><span class="check" contenteditable="false"></span><span data-src="6-9">uno</span></li>' +
    '<li class="task" data-at="10-21"><span class="check" contenteditable="false"></span>' +
    '<span class="tag" data-src="16-20" contenteditable="false">#eti</span><span data-src="20-24"> dos</span></li></ul>';
  const step = () => document.querySelectorAll("li")[1] as Element;
  const beforeTag = () => {
    const tag = step().querySelector(".tag");
    return [step(), Array.prototype.indexOf.call(step().childNodes, tag)];
  };

  it("brings Control+E back to the end of its own line when the move ran past the line's break", async () => {
    on(MAC);
    const spot = await pressed(
      "uno #eti\\\ndos tres\n",
      BROKEN,
      controlE,
      () => [piece("uno "), 1],
      () => [piece("dos tres"), 8],
    );
    expect(spot).toEqual([document.querySelector("p"), 2]);
  });

  it("brings Control+A back to the start of its own line, after the break, not the block's first line", async () => {
    on(MAC);
    const spot = await pressed(
      "uno #eti\\\n#dos tres\n",
      '<p data-at="0-19"><span data-src="0-4">uno </span><span class="tag" data-src="4-8" contenteditable="false">#eti</span>' +
        '<br data-src="8-10" contenteditable="false"><span class="tag" data-src="10-14" contenteditable="false">#dos</span>' +
        '<span data-src="14-19"> tres</span></p>',
      controlA,
      () => [piece(" tres"), 2],
      () => [piece("uno "), 0],
    );
    const line = document.querySelector("p") as Element;
    expect(spot).toEqual([
      line,
      Array.from(line.childNodes).indexOf(line.querySelector("br") as Element) + 1,
    ]);
  });

  it("keeps Control+A in its block when the move went to the block above", async () => {
    on(MAC);
    const spot = await pressed(
      "Uno.\n\n#eti dos\n",
      '<p data-at="0-4"><span data-src="0-4">Uno.</span></p>' +
        '<p data-at="6-14"><span class="tag" data-src="6-10" contenteditable="false">#eti</span><span data-src="10-14"> dos</span></p>',
      controlA,
      () => [piece(" dos"), 2],
      () => [piece("Uno."), 4],
    );
    expect(spot).toEqual([document.querySelectorAll("p")[1], 0]);
  });

  it("ends the line past a tag with Cmd+Right when the move went on to the next paragraph", async () => {
    on(MAC);
    const spot = await pressed(
      "Antes #eti\n\nOtra.\n",
      '<p data-at="0-10"><span data-src="0-6">Antes </span><span class="tag" data-src="6-10" contenteditable="false">#eti</span></p>' +
        '<p data-at="12-17"><span data-src="12-17">Otra.</span></p>',
      cmdRight,
      () => [piece("Antes "), 1],
      () => [piece("Otra."), 0],
    );
    expect(spot).toEqual([document.querySelector("p"), 2]);
  });

  it("keeps Cmd+Left in a step opened by a tag, before the tag, when the move went to the step above", async () => {
    on(MAC);
    const spot = await pressed(
      "- [ ] uno\n- [ ] #eti dos\n",
      STEPS,
      cmdLeft,
      () => [piece(" dos"), 2],
      () => [piece("uno"), 3],
    );
    expect(spot).toEqual(beforeTag());
  });

  it("takes Cmd+Left on from beside a step's box to just before its tag", async () => {
    on(MAC);
    const spot = await pressed(
      "- [ ] uno\n- [ ] #eti dos\n",
      STEPS,
      cmdLeft,
      () => [piece(" dos"), 2],
      () => [step(), 1],
    );
    expect(spot).toEqual(beforeTag());
  });

  it("keeps a second Cmd+Right at the line's end past its tag when the move went to the next paragraph", async () => {
    on(MAC);
    const spot = await pressed(
      "Antes #eti\n\nOtra.\n",
      '<p data-at="0-10"><span data-src="0-6">Antes </span><span class="tag" data-src="6-10" contenteditable="false">#eti</span></p>' +
        '<p data-at="12-17"><span data-src="12-17">Otra.</span></p>',
      cmdRight,
      () => [document.querySelector("p") as Element, 2],
      () => [piece("Otra."), 0],
    );
    expect(spot).toEqual([document.querySelector("p"), 2]);
  });

  it("ends the line of the chosen words' end with Cmd+Right, though they begin in the block above", async () => {
    on(MAC);
    await written(
      "Uno.\n\nAntes #eti\n\nOtra.\n",
      '<p data-at="0-4"><span data-src="0-4">Uno.</span></p>' +
        '<p data-at="6-16"><span data-src="6-12">Antes </span><span class="tag" data-src="12-16" contenteditable="false">#eti</span></p>' +
        '<p data-at="18-23"><span data-src="18-23">Otra.</span></p>',
    );
    writing?.drawn();
    document.getSelection()?.setBaseAndExtent(piece("Uno."), 1, piece("Antes "), 2);
    keyOn(sheet(), "ArrowRight", { metaKey: true });
    document.getSelection()?.collapse(piece("Otra."), 0);
    await new Promise((done) => setTimeout(done, 0));
    const held = document.getSelection();
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([document.querySelectorAll("p")[1], 2]);
  });

  it("ends a parent step's line before the list nested in it", async () => {
    on(MAC);
    const spot = await pressed(
      "- [ ] uno #eti\n  - [ ] hijo\n",
      '<ul data-at="0-27"><li class="task" data-at="0-27"><span class="check" contenteditable="false"></span>' +
        '<span data-src="6-10">uno </span><span class="tag" data-src="10-14" contenteditable="false">#eti</span>' +
        '<ul data-at="15-27"><li class="task" data-at="15-27"><span class="check" contenteditable="false"></span>' +
        '<span data-src="23-27">hijo</span></li></ul></li></ul>',
      controlE,
      () => [piece("uno "), 1],
      () => [document.querySelector("li") as Element, 4],
    );
    expect(spot).toEqual([document.querySelector("li"), 3]);
  });

  it("keeps an empty step when Cmd+Left first lands in the step above", async () => {
    on(MAC);
    const said = await written(EMPTY_STEP, EMPTY_STEP_PLACED);
    writing?.drawn();
    const step = document.querySelectorAll("li")[1] as Element;
    document.getSelection()?.collapse(step, 0);
    keyOn(sheet(), "ArrowLeft", { metaKey: true });
    caretIn("uno", 3);
    document.dispatchEvent(new Event("selectionchange"));
    await new Promise((done) => setTimeout(done, 0));
    const held = document.getSelection();
    expect(said).toEqual([]);
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([step, 0]);
  });

  it("keeps Cmd+Right on its own wrapped line when the next one holds only a tag, where Control+E goes past it", async () => {
    on(MAC);
    await written(
      "Uno dos #eti\n\nOtra.\n",
      '<p data-at="0-12"><span data-src="0-8">Uno dos </span><span class="tag" data-src="8-12" contenteditable="false">#eti</span></p>' +
        '<p data-at="14-19"><span data-src="14-19">Otra.</span></p>',
    );
    writing?.drawn();
    const line = { top: 0, bottom: 20, height: 20 } as DOMRect;
    const below = { top: 30, bottom: 50, height: 20 } as DOMRect;
    Object.assign(Range.prototype, { getBoundingClientRect: () => line });
    vi.spyOn(document.querySelector(".tag") as Element, "getBoundingClientRect").mockReturnValue(
      below,
    );
    const pressedTo = async (key: KeyboardEventInit & { key: string }) => {
      caretIn("Uno dos ", 1);
      keyOn(sheet(), key.key, key);
      caretIn("Uno dos ", 8);
      await new Promise((done) => setTimeout(done, 0));
      const held = document.getSelection();
      return [held?.anchorNode, held?.anchorOffset];
    };
    const wrapped = await pressedTo({ key: "ArrowRight", metaKey: true });
    const paragraph = await pressedTo({ key: "e", code: "KeyE", ctrlKey: true });
    delete (Range.prototype as { getBoundingClientRect?: unknown }).getBoundingClientRect;
    expect(wrapped).toEqual([piece("Uno dos "), 8]);
    expect(paragraph).toEqual([document.querySelector("p"), 2]);
  });

  it("ends the line past its tag with Cmd+Right though the page was drawn again before the move settled", async () => {
    on(MAC);
    const placed =
      '<p data-at="0-10"><span data-src="0-6">Antes </span><span class="tag" data-src="6-10" contenteditable="false">#eti</span></p>' +
      '<p data-at="12-17"><span data-src="12-17">Otra.</span></p>';
    await written("Antes #eti\n\nOtra.\n", placed);
    writing?.drawn();
    caretIn("Antes ", 1);
    keyOn(sheet(), "ArrowRight", { metaKey: true });
    sheet().innerHTML = placed;
    caretIn("Otra.", 0);
    await new Promise((done) => setTimeout(done, 0));
    const held = document.getSelection();
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([document.querySelector("p"), 2]);
  });

  it("takes an empty step away when Cmd+Right left it and its line can no longer be found", async () => {
    on(MAC);
    const said = await written(EMPTY_STEP, EMPTY_STEP_PLACED);
    writing?.drawn();
    document.getSelection()?.collapse(document.querySelectorAll("li")[1] as Element, 0);
    keyOn(sheet(), "ArrowRight", { metaKey: true });
    sheet().innerHTML = '<p data-at="18-21"><span data-src="18-21">Fin</span></p>';
    caretIn("Fin", 0);
    document.dispatchEvent(new Event("selectionchange"));
    await new Promise((done) => setTimeout(done, 0));
    expect(said[said.length - 1]).toBe("- [ ] uno\n\nFin\n");
  });

  it("lets a line's edge go when its words changed before the system's move settled", async () => {
    on(MAC);
    await written();
    caretIn("Uno dos", 6);
    vi.useFakeTimers();
    keyOn(sheet(), "e", { ctrlKey: true, code: "KeyE" });
    piece("Uno dos").data = "Uno";
    expect(() => vi.runOnlyPendingTimers()).not.toThrow();
    vi.useRealTimers();
  });

  const unsettledPastTag = async () => {
    const said = await written(
      "Antes #eti\n\nOtra.\n",
      '<p data-at="0-10"><span data-src="0-6">Antes </span><span class="tag" data-src="6-10" contenteditable="false">#eti</span></p>' +
        '<p data-at="12-17"><span data-src="12-17">Otra.</span></p>',
    );
    writing?.drawn();
    const pressedNow = () => {
      caretIn("Antes ", 1);
      keyOn(sheet(), "ArrowRight", { metaKey: true });
      caretIn("Otra.", 0);
    };
    const held = () => {
      const now = document.getSelection();
      return [now?.anchorNode, now?.anchorOffset];
    };
    return { said, pressedNow, held, lineEnd: [document.querySelector("p"), 2] };
  };

  it("types past the tag that ends the line when the letter comes before Cmd+Right's move settled", async () => {
    on(MAC);
    const { said, pressedNow } = await unsettledPastTag();
    pressedNow();
    typed("X");
    expect(said[said.length - 1]).toBe("Antes #etiX\n\nOtra.\n");
  });

  it("lays Cmd+Right's line end before the next key, a click or a composition reads the caret", async () => {
    on(MAC);
    const { pressedNow, held, lineEnd } = await unsettledPastTag();
    const next = [
      new KeyboardEvent("keydown", { key: "Backspace", bubbles: true, cancelable: true }),
      new MouseEvent("mousedown", { bubbles: true, cancelable: true }),
      new CompositionEvent("compositionstart", { bubbles: true }),
    ];
    for (const one of next) {
      pressedNow();
      sheet().dispatchEvent(one);
      expect(held(), one.type).toEqual(lineEnd);
    }
  });

  it("leaves the caret where a click put it right after Cmd+Right", async () => {
    on(MAC);
    const { pressedNow, held } = await unsettledPastTag();
    pressedNow();
    sheet().dispatchEvent(new MouseEvent("mousedown", { bubbles: true, cancelable: true }));
    caretIn("Otra.", 3);
    await new Promise((done) => setTimeout(done, 0));
    expect(held()).toEqual([piece("Otra."), 3]);
  });

  it("takes an empty step away when a click comes right after Cmd+Right left it", async () => {
    on(MAC);
    const said = await written(EMPTY_STEP, EMPTY_STEP_PLACED);
    writing?.drawn();
    document.getSelection()?.collapse(document.querySelectorAll("li")[1] as Element, 0);
    keyOn(sheet(), "ArrowRight", { metaKey: true });
    caretIn("Fin", 0);
    sheet().dispatchEvent(new MouseEvent("mousedown", { bubbles: true, cancelable: true }));
    caretIn("Fin", 2);
    document.dispatchEvent(new Event("selectionchange"));
    expect(said[said.length - 1]).toBe("- [ ] uno\n\nFin\n");
  });

  it("lets a line's edge still waiting go when the page stops", async () => {
    on(MAC);
    const { pressedNow, held } = await unsettledPastTag();
    pressedNow();
    writing?.stop();
    writing = null;
    await new Promise((done) => setTimeout(done, 0));
    expect(held()).toEqual([piece("Otra."), 0]);
  });

  it("reads Cmd with the side arrows as the line's edges, and nothing with Shift, a composition or off a Mac", async () => {
    on(MAC);
    const { edgeWalk } = await import("../ui/macKeys");
    const held = {
      ctrlKey: false,
      metaKey: true,
      altKey: false,
      shiftKey: false,
      isComposing: false,
    };
    expect(edgeWalk({ ...held, key: "ArrowLeft", code: "ArrowLeft" })).toEqual({
      toStart: true,
      visual: true,
    });
    expect(edgeWalk({ ...held, key: "ArrowRight", code: "ArrowRight" })).toEqual({
      toStart: false,
      visual: true,
    });
    expect(edgeWalk({ ...held, key: "ArrowRight", code: "ArrowRight", shiftKey: true })).toBeNull();
    expect(edgeWalk({ ...held, key: "ArrowUp", code: "ArrowUp" })).toBeNull();
    expect(edgeWalk({ ...held, metaKey: false, ctrlKey: true, key: "e", code: "KeyE" })).toEqual({
      toStart: false,
      visual: false,
    });
    expect(
      edgeWalk({
        ...held,
        metaKey: false,
        ctrlKey: true,
        key: "e",
        code: "KeyE",
        isComposing: true,
      }),
    ).toBeNull();
    on(WINDOWS);
    const windows = await import("../ui/macKeys");
    expect(windows.edgeWalk({ ...held, key: "ArrowLeft", code: "ArrowLeft" })).toBeNull();
  });

  it("reads Cmd with a side arrow as a line's edge once per key", async () => {
    on(MAC);
    const edges = vi.fn();
    vi.doMock("../ui/macKeys", async (real: () => Promise<typeof import("../ui/macKeys")>) => {
      const keys = await real();
      return {
        ...keys,
        edgeWalk: (event: Parameters<typeof keys.edgeWalk>[0]) => {
          edges();
          return keys.edgeWalk(event);
        },
      };
    });
    await written();
    caretIn("Uno dos", 3);
    keyOn(sheet(), "ArrowLeft", { metaKey: true });
    vi.doUnmock("../ui/macKeys");
    expect(edges).toHaveBeenCalledOnce();
  });

  it("keeps a format waiting for the next letter through a Control key the input method takes", async () => {
    on(MAC);
    const said = await written();
    caretIn("Uno dos", 3);
    keyOn(sheet(), "b", { metaKey: true });
    keyOn(sheet(), "a", { ctrlKey: true, code: "KeyA", isComposing: true });
    typed("x");
    expect(said[said.length - 1]?.startsWith("Uno**x** dos")).toBe(true);
  });
});

describe("Cmd as the command key on a Mac, where Control is the text's", () => {
  it("keeps today's words with Cmd and Enter, not with Control and Enter", async () => {
    on(MAC);
    const { default: DiaryComposer } = await import("../ui/DiaryComposer");
    const before = vi.fn(() => Promise.resolve());
    render(<DiaryComposer onBefore={before} onWritten={() => {}} onError={() => {}} />);
    const box = screen.getByRole("textbox");
    fireEvent.change(box, { target: { value: "Paseo" } });
    keyOn(box, "Enter", { ctrlKey: true });
    expect(before).not.toHaveBeenCalled();
    keyOn(box, "Enter", { metaKey: true });
    expect(before).toHaveBeenCalledOnce();
  });

  it("completes the focused task with Cmd and Enter, and says so", async () => {
    on(MAC);
    const { default: TaskList } = await import("../ui/TaskList");
    const completed = vi.fn();
    const one = {
      id: "1",
      title: "pagar la luz",
      status: "open",
      priority: "unset",
      order: "a0",
      tags: [],
      reminders: [],
    } as unknown as Task;
    render(
      <TaskList tasks={[one]} lists={[]} title="Open" onSelect={() => {}} onComplete={completed} />,
    );
    const row = screen.getByRole("listitem");
    expect(row.getAttribute("aria-keyshortcuts")).toBe("Meta+Enter");
    keyOn(row, "Enter", { ctrlKey: true });
    expect(completed).not.toHaveBeenCalled();
    keyOn(row, "Enter", { metaKey: true });
    expect(completed).toHaveBeenCalledWith("1", "pagar la luz");
  });

  it("keeps a day being written with Cmd and Enter, not with Control and Enter", async () => {
    on(MAC);
    const { boxKey } = await import("../ui/diaryPaging");
    document.body.innerHTML = '<div class="diary-box"><textarea></textarea></div>';
    const box = document.querySelector("textarea") as HTMLTextAreaElement;
    const went: unknown[] = [];
    box.addEventListener("keydown", (event) => went.push(boxKey(event)));
    keyOn(box, "Enter", { ctrlKey: true });
    keyOn(box, "Enter", { metaKey: true });
    expect(went).toEqual([null, { kind: "dayKeep" }]);
  });

  it("takes Cmd with End to the end of the last block, leaving Control with End alone", async () => {
    on(MAC);
    await written();
    caretIn("Uno dos", 0);
    expect(keyOn(sheet(), "End", { ctrlKey: true }).defaultPrevented).toBe(false);
    caretIn("segundo", 2);
    await new Promise((done) => setTimeout(done, 0));
    expect(document.getSelection()?.anchorNode?.textContent).toBe("segundo");
    expect(document.getSelection()?.anchorOffset).toBe(2);
    expect(keyOn(sheet(), "End", { metaKey: true }).defaultPrevented).toBe(true);
    const held = document.getSelection();
    expect(held?.anchorNode?.textContent).toBe("segundo");
    expect(held?.anchorOffset).toBe(7);
  });

  it("brings End back to an empty step when the system moved the caret out of it", async () => {
    on(WINDOWS);
    await written(EMPTY_STEP, EMPTY_STEP_PLACED);
    writing?.drawn();
    const step = document.querySelectorAll("li")[1] as Element;
    step.innerHTML = '<span class="check" contenteditable="false"></span>';
    document.getSelection()?.collapse(step, 1);
    keyOn(sheet(), "End");
    caretIn("Fin", 0);
    await new Promise((done) => setTimeout(done, 0));
    const held = document.getSelection();
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([step, 1]);
  });

  it("brings End back into its block of code when the system moved the caret out of it", async () => {
    on(WINDOWS);
    await written(
      "```\nfoo()\nbar()\n```\n\nFin\n",
      '<figure class="code" data-at="0-20"><pre><code data-src="4-16">foo()\nbar()\n</code></pre></figure>' +
        '<p data-at="21-24"><span data-src="21-24">Fin</span></p>',
    );
    writing?.drawn();
    const code = document.querySelector("code")?.firstChild as Text;
    document.getSelection()?.collapse(code, 2);
    keyOn(sheet(), "End");
    caretIn("Fin", 0);
    await new Promise((done) => setTimeout(done, 0));
    const held = document.getSelection();
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([code, 12]);
  });

  it("chooses past a table that ends the page with Ctrl, Shift and End", async () => {
    on(WINDOWS);
    await written(
      "Uno dos\n\n| a |\n|---|\n| 1 |\n",
      '<p data-at="0-7"><span data-src="0-7">Uno dos</span></p>' +
        '<div class="table" data-at="9-26" contenteditable="false"><table><tr><td>1</td></tr></table></div>',
    );
    caretIn("Uno dos", 1);
    keyOn(sheet(), "End", { ctrlKey: true, shiftKey: true });
    const held = document.getSelection();
    expect([held?.focusNode, held?.focusOffset]).toEqual([sheet(), 2]);
  });

  it("chooses up to the end of the last block with Ctrl, Shift and End off a Mac", async () => {
    on(WINDOWS);
    await written();
    caretIn("Uno dos", 1);
    expect(keyOn(sheet(), "End", { ctrlKey: true, shiftKey: true }).defaultPrevented).toBe(true);
    const held = document.getSelection();
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([piece("Uno dos"), 1]);
    expect([held?.focusNode, held?.focusOffset]).toEqual([piece("segundo"), 7]);
  });
});

describe("Home and End on a Mac", () => {
  const held = {
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    shiftKey: false,
    isComposing: false,
  };

  const walked = () => {
    const walks: string[][] = [];
    Object.defineProperty(Selection.prototype, "modify", {
      value: (...said: string[]) => walks.push(said),
      configurable: true,
    });
    return walks;
  };

  afterEach(() => {
    Reflect.deleteProperty(Selection.prototype, "modify");
  });

  it("reads Home and End as a line's edges on a Mac, Shift reaching there", async () => {
    on(MAC);
    const { lineKey } = await import("../ui/macKeys");
    expect(lineKey({ ...held, key: "Home", code: "Home" })).toEqual({
      toStart: true,
      reaching: false,
    });
    expect(lineKey({ ...held, key: "End", code: "End", shiftKey: true })).toEqual({
      toStart: false,
      reaching: true,
    });
    expect(lineKey({ ...held, key: "End", code: "End", metaKey: true })).toBeNull();
    expect(lineKey({ ...held, key: "Home", code: "Home", isComposing: true })).toBeNull();
    expect(lineKey({ ...held, key: "ArrowLeft", code: "ArrowLeft" })).toBeNull();
    on(WINDOWS);
    const windows = await import("../ui/macKeys");
    expect(windows.lineKey({ ...held, key: "Home", code: "Home" })).toBeNull();
  });

  it("walks the caret to the line's edge with Home and End on a Mac, where the system only scrolls", async () => {
    on(MAC);
    const walks = walked();
    await written();
    caretIn("Uno dos", 3);
    expect(keyOn(sheet(), "Home").defaultPrevented).toBe(true);
    expect(keyOn(sheet(), "End", { shiftKey: true }).defaultPrevented).toBe(true);
    expect(walks).toEqual([
      ["move", "backward", "lineboundary"],
      ["extend", "forward", "lineboundary"],
    ]);
  });
});

describe("the whole page chosen with Cmd or Ctrl and A", () => {
  it("reaches past a table that ends the page, and Backspace takes it all", async () => {
    on(WINDOWS);
    const said = await written(
      "Uno dos\n\n| a |\n|---|\n| 1 |\n",
      '<p data-at="0-7"><span data-src="0-7">Uno dos</span></p>' +
        '<div class="table" data-at="9-26" contenteditable="false"><table><tr><td>1</td></tr></table></div>',
    );
    caretIn("Uno dos", 3);
    expect(keyOn(sheet(), "a", { ctrlKey: true, code: "KeyA" }).defaultPrevented).toBe(true);
    const held = document.getSelection();
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([piece("Uno dos"), 0]);
    expect([held?.focusNode, held?.focusOffset]).toEqual([sheet(), 2]);
    expect(keyOn(sheet(), "Backspace").defaultPrevented).toBe(true);
    expect(said[said.length - 1]).toBe("\n");
  });

  it("reaches from before a table that opens the page, leaving the properties out", async () => {
    on(MAC);
    const said = await written(
      "---\ntitle: X\n---\n\n| a |\n|---|\n| 1 |\n\nUno dos\n",
      '<dl class="front" data-at="0-17"><dt>title</dt><dd>X</dd></dl>' +
        '<div class="table" data-at="18-36" contenteditable="false"><table><tr><td>1</td></tr></table></div>' +
        '<p data-at="37-44"><span data-src="37-44">Uno dos</span></p>',
    );
    caretIn("Uno dos", 3);
    expect(keyOn(sheet(), "a", { metaKey: true, code: "KeyA" }).defaultPrevented).toBe(true);
    const held = document.getSelection();
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([sheet(), 1]);
    expect([held?.focusNode, held?.focusOffset]).toEqual([piece("Uno dos"), 7]);
    expect(keyOn(sheet(), "Delete").defaultPrevented).toBe(true);
    expect(said[said.length - 1]).toBe("---\ntitle: X\n---\n\n");
  });

  it("leaves Shift with Delete, the cut of Windows, to the system on a page that opens with a table", async () => {
    on(WINDOWS);
    const said = await written(
      "| a |\n|---|\n| 1 |\n\nUno dos\n",
      '<div class="table" data-at="0-18" contenteditable="false"><table><tr><td>1</td></tr></table></div>' +
        '<p data-at="19-26"><span data-src="19-26">Uno dos</span></p>',
    );
    caretIn("Uno dos", 3);
    keyOn(sheet(), "a", { ctrlKey: true, code: "KeyA" });
    const before = said.length;
    expect(keyOn(sheet(), "Delete", { shiftKey: true }).defaultPrevented).toBe(false);
    expect(said).toHaveLength(before);
  });

  it("starts at the first words of a page that opens with steps, after the first box", async () => {
    on(WINDOWS);
    await written(
      "- [ ] uno\n\nFin\n",
      '<ul data-at="0-10"><li class="task" data-at="0-10"><span class="check"></span><span data-src="6-9">uno</span></li></ul>' +
        '<p data-at="11-14"><span data-src="11-14">Fin</span></p>',
    );
    caretIn("Fin", 1);
    keyOn(sheet(), "a", { ctrlKey: true, code: "KeyA" });
    const held = document.getSelection();
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([piece("uno"), 0]);
  });

  it("reaches past a table before a task's last label, leaving the label out", async () => {
    on(WINDOWS);
    document.documentElement.classList.add("untitled");
    try {
      await written(
        "Uno dos\n\n| a |\n|---|\n| 1 |\n\n## Notas\n",
        '<p data-at="0-7"><span data-src="0-7">Uno dos</span></p>' +
          '<div class="table" data-at="9-26" contenteditable="false"><table><tr><td>1</td></tr></table></div>' +
          '<h2 data-at="28-37"><span data-src="31-36">Notas</span></h2>',
      );
      caretIn("Uno dos", 3);
      keyOn(sheet(), "a", { ctrlKey: true, code: "KeyA" });
      expect([document.getSelection()?.focusNode, document.getSelection()?.focusOffset]).toEqual([
        sheet(),
        2,
      ]);
      caretIn("Uno dos", 3);
      keyOn(sheet(), "End", { ctrlKey: true, shiftKey: true });
      expect([document.getSelection()?.focusNode, document.getSelection()?.focusOffset]).toEqual([
        sheet(),
        2,
      ]);
    } finally {
      document.documentElement.classList.remove("untitled");
    }
  });
});

describe("the chord that moves a block", () => {
  it("is Control and Option on a Mac, leaving Option and Shift with Option to the text", async () => {
    on(MAC);
    const { moveKey, MOVE_KEYS, MOVE_SHORTCUTS } = await import("../ui/macKeys");
    const held = { ctrlKey: false, metaKey: false, altKey: false, shiftKey: false };
    expect(moveKey({ ...held, altKey: true })).toBe(false);
    expect(moveKey({ ...held, altKey: true, shiftKey: true })).toBe(false);
    expect(moveKey({ ...held, altKey: true, ctrlKey: true })).toBe(true);
    expect(moveKey({ ...held, altKey: true, ctrlKey: true, shiftKey: true })).toBe(false);
    expect(moveKey({ ...held, altKey: true, ctrlKey: true, metaKey: true })).toBe(false);
    expect(MOVE_KEYS).toBe("⌃⌥");
    expect(MOVE_SHORTCUTS).toBe("Control+Alt+ArrowUp Control+Alt+ArrowDown");
  });

  it("is Alt elsewhere, with or without the rest, as it always was", async () => {
    on(WINDOWS);
    const { moveKey, MOVE_KEYS, MOVE_SHORTCUTS } = await import("../ui/macKeys");
    const held = { ctrlKey: false, metaKey: false, altKey: true, shiftKey: false };
    expect(moveKey(held)).toBe(true);
    expect(moveKey({ ...held, shiftKey: true })).toBe(true);
    expect(moveKey({ ...held, ctrlKey: true })).toBe(true);
    expect(moveKey({ ...held, altKey: false })).toBe(false);
    expect(MOVE_KEYS).toBe("Alt");
    expect(MOVE_SHORTCUTS).toBe("Alt+ArrowUp Alt+ArrowDown");
  });
});

describe("moving blocks on a Mac", () => {
  it("leaves Option with an arrow to the paragraph, and moves the item with Control and Option", async () => {
    on(MAC);
    const said = await written();
    caretIn("segundo", 2);
    expect(keyOn(sheet(), "ArrowUp", { altKey: true }).defaultPrevented).toBe(false);
    expect(keyOn(sheet(), "ArrowUp", { altKey: true, shiftKey: true }).defaultPrevented).toBe(
      false,
    );
    expect(said).toEqual([]);
    expect(keyOn(sheet(), "ArrowUp", { altKey: true, ctrlKey: true }).defaultPrevented).toBe(true);
    expect(said[said.length - 1]?.endsWith("- segundo\n- primero\n")).toBe(true);
  });

  it("lets chosen blocks go on Option with an arrow, and moves them with Control and Option", async () => {
    on(MAC);
    const said = await written();
    caretIn("Uno dos", 1);
    keyOn(sheet(), "Escape");
    expect(document.querySelectorAll(".chosen-block")).toHaveLength(1);
    keyOn(sheet(), "ArrowDown", { altKey: true });
    expect(document.querySelectorAll(".chosen-block")).toHaveLength(0);
    expect(said).toEqual([]);
    caretIn("Uno dos", 1);
    keyOn(sheet(), "Escape");
    keyOn(sheet(), "ArrowDown", { altKey: true, ctrlKey: true });
    expect(said[said.length - 1]?.startsWith("- primero\n- segundo\n\nUno dos")).toBe(true);
  });

  it("moves a sticky note on the page with Control and Option, never with Option alone", async () => {
    on(MAC);
    const { sticking } = await import("../ui/sticking");
    const page = document.implementation.createHTMLDocument();
    page.body.innerHTML = `<article class="page">
      <p data-at="8-12">Uno.</p>
      <blockquote class="callout" data-callout="idea" data-at="14-60">Idea</blockquote>
      <p data-at="62-66">Dos.</p>
    </article>`;
    const note = page.querySelector("blockquote") as HTMLElement;
    const moved = vi.fn();
    sticking(page, () => moved);
    expect(keyOn(note, "ArrowDown", { altKey: true }).defaultPrevented).toBe(false);
    expect(moved).not.toHaveBeenCalled();
    keyOn(note, "ArrowDown", { altKey: true, ctrlKey: true });
    expect(moved).toHaveBeenCalledWith([14, 60], 66);
  });

  it("leaves Option with an arrow to a table's cell, and moves its row with Control and Option", async () => {
    on(MAC);
    const { typing } = await import("../ui/tabled");
    const cell = document.createElement("td");
    document.body.append(cell);
    const done = vi.fn();
    const input = typing(cell, "Spotify", "Cell", done);
    keyOn(input, "ArrowUp", { altKey: true });
    expect(done).not.toHaveBeenCalled();
    keyOn(input, "ArrowUp", { altKey: true, ctrlKey: true });
    expect(done).toHaveBeenCalledWith("Spotify", "up");
  });

  it("leaves Option with an arrow to an idea's words, and moves the idea with Control and Option", async () => {
    on(MAC);
    const { default: Notes } = await import("../ui/Notes");
    const onWrite = vi.fn();
    const body =
      "---\nview: notes\n---\n\n# Ideas\n\n> [!IDEA] 2026-11-10 18:40\n> primera\n\n> [!IDEA] 2026-11-11 09:12\n> segunda\n";
    render(<Notes title="Ideas" body={body} reading={false} onWrite={onWrite} />);
    const words = screen.getAllByRole("textbox")[0];
    expect(words.getAttribute("aria-keyshortcuts")).toBe(
      "Control+Alt+ArrowUp Control+Alt+ArrowDown",
    );
    for (const key of ["ArrowRight", "ArrowLeft", "ArrowDown"])
      expect(keyOn(words, key, { altKey: true }).defaultPrevented, key).toBe(false);
    expect(onWrite).not.toHaveBeenCalled();
    keyOn(words, "ArrowRight", { altKey: true, ctrlKey: true });
    const moved: string = onWrite.mock.calls[0][0];
    expect(moved.indexOf("segunda")).toBeLessThan(moved.indexOf("primera"));
  });
});

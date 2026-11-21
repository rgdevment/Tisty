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
    const { writtenOn } = await import("../ui/pageWriting");
    document.body.innerHTML =
      '<article class="page"><ul data-at="0-17"><li class="task" data-at="0-10"><span data-src="6-9">uno</span></li>' +
      '<li class="task" data-at="10-17"></li></ul><p data-at="18-21"><span data-src="18-21">Fin</span></p></article>';
    const said: string[] = [];
    let text = "- [ ] uno\n- [ ] \n\nFin\n";
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
    writing.drawn();
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
    const said = await written(
      "- [ ] uno\n- [ ] \n\nFin\n",
      '<ul data-at="0-17"><li class="task" data-at="0-10"><span data-src="6-9">uno</span></li>' +
        '<li class="task" data-at="10-17"></li></ul><p data-at="18-21"><span data-src="18-21">Fin</span></p>',
    );
    writing?.drawn();
    document.getSelection()?.collapse(document.querySelectorAll("li")[1] as Element, 0);
    keyOn(sheet(), "т", { ctrlKey: true, code: "KeyN" });
    caretIn("Fin", 1);
    document.dispatchEvent(new Event("selectionchange"));
    expect(said[said.length - 1]).toBe("- [ ] uno\n\nFin\n");
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

  it("starts the line before a tag that opens it, but never between a step's box and its tag", async () => {
    on(MAC);
    await drawn(
      "#eti uno\n\n- [ ] #eti dos\n",
      '<p data-at="0-8"><span class="tag" data-src="0-4" contenteditable="false">#eti</span><span data-src="4-8"> uno</span></p>' +
        '<ul data-at="10-25"><li class="task" data-at="10-25"><span class="check" contenteditable="false"></span>' +
        '<span class="tag" data-src="16-20" contenteditable="false">#eti</span><span data-src="20-24"> dos</span></li></ul>',
    );
    expect(await walked("a", [piece(" uno"), 0])).toEqual([document.querySelector("p"), 0]);
    expect(await walked("a", [piece(" dos"), 0])).toEqual([piece(" dos"), 0]);
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

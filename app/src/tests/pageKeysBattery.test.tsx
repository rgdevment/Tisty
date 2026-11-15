import { act, fireEvent, render, renderHook, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { answered, type Reaching, wired } from "../ui/dressing";
import { PageMore, useFrameScroll } from "../ui/liveSheet";

const pageOf = (html: string): Document =>
  new DOMParser().parseFromString(`<body>${html}</body>`, "text/html");

const reach: Reaching = { body: "", onDoc: () => {} };

const heardKeys = () => {
  const heard: string[] = [];
  const listen = (event: KeyboardEvent) => {
    heard.push(`${event.ctrlKey || event.metaKey ? "mod+" : ""}${event.key}`);
  };
  window.addEventListener("keydown", listen);
  return { heard, off: () => window.removeEventListener("keydown", listen) };
};

const pressedIn = (target: Element, key: string, init: KeyboardEventInit = {}) => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
};

const EDITABLE = '<div class="page" contenteditable="true"><p>x</p></div>';
const READ_ONLY = '<div class="page"><p>x</p></div>';

describe("the keys the page keeps and the ones it passes to the window", () => {
  let off = () => {};
  afterEach(() => off());

  const listening = (html: string) => {
    const drawn = pageOf(html);
    wired(drawn, () => reach);
    const keys = heardKeys();
    off = keys.off;
    return { inside: drawn.querySelector("p") as HTMLElement, heard: keys.heard, drawn };
  };

  it.each(["b", "i", "u", "k"])("keeps Ctrl+%s to the page written in place", (key) => {
    const { inside, heard } = listening(EDITABLE);
    pressedIn(inside, key, { ctrlKey: true });
    expect(heard).toEqual([]);
  });

  it.each(["b", "i", "u", "k"])("keeps Cmd+%s to the page written in place", (key) => {
    const { inside, heard } = listening(EDITABLE);
    pressedIn(inside, key, { metaKey: true });
    expect(heard).toEqual([]);
  });

  it("keeps the capital of those letters as well", () => {
    const { inside, heard } = listening(EDITABLE);
    for (const key of ["B", "I", "U", "K"]) pressedIn(inside, key, { ctrlKey: true });
    expect(heard).toEqual([]);
  });

  it.each(["e", "f", "p", "n"])("passes Ctrl+%s from the page written in place", (key) => {
    const { inside, heard } = listening(EDITABLE);
    pressedIn(inside, key, { ctrlKey: true });
    expect(heard).toEqual([`mod+${key}`]);
  });

  it("passes F1 from the page written in place", () => {
    const { inside, heard } = listening(EDITABLE);
    pressedIn(inside, "F1");
    expect(heard).toEqual(["F1"]);
  });

  it("passes Escape from the page written in place", () => {
    const { inside, heard } = listening(EDITABLE);
    pressedIn(inside, "Escape");
    expect(heard).toEqual(["Escape"]);
  });

  it("passes Ctrl+b from a page that is not written in place", () => {
    const { inside, heard } = listening(READ_ONLY);
    pressedIn(inside, "b", { ctrlKey: true });
    expect(heard).toEqual(["mod+b"]);
  });

  it("passes Ctrl+b from a page whose contenteditable is off", () => {
    const { inside, heard } = listening('<div class="page" contenteditable="false"><p>x</p></div>');
    pressedIn(inside, "b", { ctrlKey: true });
    expect(heard).toEqual(["mod+b"]);
  });

  it("passes Ctrl+b from an editable element that is not the page", () => {
    const { inside, heard } = listening('<div contenteditable="true"><p>x</p></div>');
    pressedIn(inside, "b", { ctrlKey: true });
    expect(heard).toEqual(["mod+b"]);
  });

  it("passes a plain b, since a letter alone is not the window's to hold back", () => {
    const { inside, heard } = listening(EDITABLE);
    pressedIn(inside, "b");
    expect(heard).toEqual(["b"]);
  });

  it("passes a key pressed on the page itself, and holds what the window took", () => {
    const { drawn } = listening(READ_ONLY);
    const took = (event: KeyboardEvent) => event.preventDefault();
    window.addEventListener("keydown", took);
    const event = pressedIn(drawn.body, "f", { ctrlKey: true });
    window.removeEventListener("keydown", took);
    expect(event.defaultPrevented).toBe(true);
  });

  it("keeps Ctrl+Shift+b to the page as it keeps Ctrl+b", () => {
    const { inside, heard } = listening(EDITABLE);
    pressedIn(inside, "B", { ctrlKey: true, shiftKey: true });
    expect(heard).toEqual([]);
  });

  it("passes the window's shortcuts typed inside a block being written", () => {
    const { inside, heard } = listening('<div class="editing"><p>x</p></div>');
    pressedIn(inside, "w", { ctrlKey: true });
    pressedIn(inside, "F1");
    pressedIn(inside, "a");
    expect(heard).toEqual(["mod+w", "F1"]);
  });
});

describe("the properties kept out of the page", () => {
  const sources: number[] = [];
  const BODY = "---\ntags: [a]\n---\n\nPan.\n";
  const asking = (body: string): Reaching => ({
    body,
    onDoc: () => {},
    onPiece: (held, place) => {
      if (held === "front" && place) sources.push(place.at);
    },
  });
  const written = () =>
    pageOf(
      '<div class="page"><dl class="front" data-at="0-14"></dl><p data-at="15-20">Pan.</p><h2 data-at="21-30">Otro</h2></div>',
    );

  afterEach(() => {
    sources.length = 0;
  });

  it("opens the properties in their own window when they are asked for", () => {
    answered(written(), { kind: "block", at: [0, 14] }, asking(BODY));
    expect(sources).toEqual([0]);
  });

  it("opens nothing for any other block", () => {
    const drawn = written();
    answered(drawn, { kind: "block", at: [15, 20] }, asking(BODY));
    answered(drawn, { kind: "block", at: [21, 30] }, asking(BODY));
    expect(sources).toEqual([]);
  });

  it("opens nothing for a block that only has the same start", () => {
    answered(written(), { kind: "block", at: [0, 15] }, asking(BODY));
    expect(sources).toEqual([]);
  });

  it("opens nothing when the page has no properties", () => {
    const drawn = pageOf('<div class="page"><p data-at="0-5">Pan.</p></div>');
    answered(drawn, { kind: "block", at: [0, 5] }, asking("Pan.\n"));
    expect(sources).toEqual([]);
  });

  it("opens nothing when the window cannot open a piece", () => {
    expect(() =>
      answered(written(), { kind: "block", at: [0, 14] }, { body: "x", onDoc: () => {} }),
    ).not.toThrow();
  });

  it("names the properties in bytes, as the core counts them, when the words before are wider", () => {
    const drawn = pageOf(
      '<div class="page"><p data-at="0-4">ñé</p><dl class="front" data-at="6-20"></dl></div>',
    );
    answered(drawn, { kind: "block", at: [6, 20] }, asking("ñé\n\n---\na: b\n---\n"));
    expect(sources).toEqual([6]);
  });

  it("opens nothing for a front that is not a direct child of the page", () => {
    const drawn = pageOf(
      '<div class="page"><blockquote><dl class="front" data-at="0-14"></dl></blockquote></div>',
    );
    answered(drawn, { kind: "block", at: [0, 14] }, asking("---\na: b\n---\n"));
    expect(sources).toEqual([]);
  });

  it("does nothing when nothing was asked", () => {
    answered(written(), null, asking("x"));
    expect(sources).toEqual([]);
  });
});

const opened = (markdown: boolean, onTurn = vi.fn()) => {
  render(<PageMore markdown={markdown} onTurn={onTurn} />);
  const button = screen.getByRole("button", { name: t("pageMore") });
  return { button, onTurn };
};
const rich = () => screen.getByRole("menuitem", { name: new RegExp(t("viewRich")) });
const written = () => screen.getByRole("menuitem", { name: new RegExp(t("viewMarkdown")) });

describe("the page's own choices, reached with the keyboard", () => {
  it("opens with a click that comes from the keyboard", () => {
    const { button } = opened(false);
    fireEvent.click(button, { detail: 0 });
    expect(screen.getByRole("menu")).not.toBeNull();
    expect(button.getAttribute("aria-expanded")).toBe("true");
  });

  it("opens with Enter on the focused button", async () => {
    const { button } = opened(false);
    button.focus();
    await userEvent.keyboard("{Enter}");
    expect(screen.getByRole("menu")).not.toBeNull();
  });

  it("opens with Space on the focused button", async () => {
    const { button } = opened(false);
    button.focus();
    await userEvent.keyboard(" ");
    expect(screen.getByRole("menu")).not.toBeNull();
  });

  it("closes with a second click from the keyboard", () => {
    const { button } = opened(false);
    fireEvent.click(button, { detail: 0 });
    fireEvent.click(button, { detail: 0 });
    expect(screen.queryByRole("menu")).toBeNull();
    expect(button.getAttribute("aria-expanded")).toBe("false");
  });

  it("does not open twice for a press of the mouse and the click that follows it", () => {
    const { button } = opened(false);
    fireEvent.mouseDown(button);
    fireEvent.click(button, { detail: 1 });
    expect(screen.getByRole("menu")).not.toBeNull();
  });

  it("opens with a press of the mouse alone, and holds the press from taking the focus", () => {
    const { button } = opened(false);
    expect(fireEvent.mouseDown(button)).toBe(false);
    expect(screen.getByRole("menu")).not.toBeNull();
  });

  it("closes with a second press of the mouse", () => {
    const { button } = opened(false);
    fireEvent.mouseDown(button);
    fireEvent.mouseDown(button);
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("tells whoever reads the button that a menu hangs from it", () => {
    const { button } = opened(false);
    expect(button.getAttribute("aria-haspopup")).toBe("menu");
    expect(button.getAttribute("aria-expanded")).toBe("false");
  });

  it("marks only the way in use", () => {
    const { button } = opened(true);
    fireEvent.click(button, { detail: 0 });
    expect(written().className).toContain("font-semibold");
    expect(rich().className).not.toContain("font-semibold");
  });

  it("closes the menu after picking the other way, and tells it once", async () => {
    const { button, onTurn } = opened(false);
    fireEvent.click(button, { detail: 0 });
    await userEvent.click(written());
    expect(onTurn).toHaveBeenCalledTimes(1);
    expect(onTurn).toHaveBeenCalledWith(true);
    expect(screen.queryByRole("menu")).toBeNull();
    expect(button.getAttribute("aria-expanded")).toBe("false");
  });

  it("closes the menu after picking the way already in use, and turns nothing", async () => {
    const { button, onTurn } = opened(false);
    fireEvent.click(button, { detail: 0 });
    await userEvent.click(rich());
    expect(onTurn).not.toHaveBeenCalled();
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("closes with Escape", async () => {
    const { button } = opened(false);
    fireEvent.click(button, { detail: 0 });
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("closes with a press outside the menu", async () => {
    const { button } = opened(false);
    fireEvent.click(button, { detail: 0 });
    await act(() => new Promise((later) => setTimeout(later)));
    fireEvent.mouseDown(document.body);
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("opens again after it closed", async () => {
    const { button } = opened(false);
    fireEvent.click(button, { detail: 0 });
    await userEvent.click(written());
    fireEvent.click(button, { detail: 0 });
    expect(screen.getByRole("menu")).not.toBeNull();
  });
});

const framed = (page: string) => {
  const frame = document.createElement("iframe");
  document.body.append(frame);
  const inner = frame.contentWindow as Window;
  inner.document.body.innerHTML = `<div class="page">${page}</div>`;
  Object.defineProperty(inner.document, "scrollingElement", {
    value: inner.document.documentElement,
  });
  return inner;
};

const framing = (inner: Window, onTitleGone = vi.fn(), onStuck = vi.fn(), lively = true) =>
  renderHook(() =>
    useFrameScroll({
      lively,
      frame: () => inner,
      putting: { current: 0 },
      onStuck,
      onTitleGone,
      onSeen: vi.fn(),
      ready: true,
      turned: 0,
    }),
  );

const watched = (inner: Window, lively = true) => {
  const onTitleGone = vi.fn();
  const onStuck = vi.fn();
  framing(inner, onTitleGone, onStuck, lively);
  return { onTitleGone, onStuck };
};

const rolledTo = (inner: Window, top: number) => {
  (inner.document.scrollingElement as Element).scrollTop = top;
  inner.dispatchEvent(new Event("scroll"));
};

const titleAt = (inner: Window, selector: string, bottom: number) =>
  vi
    .spyOn(inner.document.querySelector(selector) as HTMLElement, "getBoundingClientRect")
    .mockReturnValue({ bottom } as DOMRect);

describe("the ribbon naming the page, in more cases", () => {
  afterEach(() => {
    document.body.innerHTML = "";
  });

  it("waits for a title that has only just touched the top", () => {
    const inner = framed("<h1>Recetas</h1><p>Pan</p>");
    const { onTitleGone } = watched(inner);
    titleAt(inner, "h1", 0);
    rolledTo(inner, 100);
    expect(onTitleGone).toHaveBeenLastCalledWith(false);
  });

  it("names the page once the title is a pixel above the top", () => {
    const inner = framed("<h1>Recetas</h1><p>Pan</p>");
    const { onTitleGone } = watched(inner);
    titleAt(inner, "h1", -1);
    rolledTo(inner, 100);
    expect(onTitleGone).toHaveBeenLastCalledWith(true);
  });

  it("follows the title back into view", () => {
    const inner = framed("<h1>Recetas</h1><p>Pan</p>");
    const { onTitleGone } = watched(inner);
    const spy = titleAt(inner, "h1", -20);
    rolledTo(inner, 100);
    spy.mockReturnValue({ bottom: 30 } as DOMRect);
    rolledTo(inner, 0);
    expect(onTitleGone.mock.calls.map(([gone]) => gone)).toEqual([true, false]);
  });

  it("takes the title behind a properties block as the page's own", () => {
    const inner = framed('<div class="front"></div><h1>Recetas</h1><p>Pan</p>');
    const { onTitleGone } = watched(inner);
    titleAt(inner, "h1", -5);
    rolledTo(inner, 100);
    expect(onTitleGone).toHaveBeenLastCalledWith(true);
  });

  it("does not take a heading of another level for the title", () => {
    const inner = framed("<h2>Sección</h2><p>Pan</p>");
    const { onTitleGone } = watched(inner);
    rolledTo(inner, 4);
    expect(onTitleGone).toHaveBeenLastCalledWith(false);
    rolledTo(inner, 20);
    expect(onTitleGone).toHaveBeenLastCalledWith(true);
  });

  it("does not take a title that comes after other words for the page's own", () => {
    const inner = framed("<p>Pan</p><h1>Recetas</h1>");
    const { onTitleGone } = watched(inner);
    titleAt(inner, "h1", 500);
    rolledTo(inner, 20);
    expect(onTitleGone).toHaveBeenLastCalledWith(true);
  });

  it("names a page that is only its properties as soon as it moves", () => {
    const inner = framed('<dl class="front"></dl>');
    const { onTitleGone } = watched(inner);
    rolledTo(inner, 20);
    expect(onTitleGone).toHaveBeenLastCalledWith(true);
  });

  it("tells that the page is stuck only past eight pixels", () => {
    const inner = framed("<h1>Recetas</h1>");
    const { onStuck } = watched(inner);
    titleAt(inner, "h1", 40);
    rolledTo(inner, 8);
    expect(onStuck).toHaveBeenLastCalledWith(false);
    rolledTo(inner, 9);
    expect(onStuck).toHaveBeenLastCalledWith(true);
  });

  it("says nothing while the sheet is not the lively one", () => {
    const inner = framed("<h1>Recetas</h1>");
    const { onTitleGone, onStuck } = watched(inner, false);
    rolledTo(inner, 100);
    expect(onTitleGone).not.toHaveBeenCalled();
    expect(onStuck).not.toHaveBeenCalled();
  });

  it("stops listening once the sheet is gone", () => {
    const inner = framed("<h1>Recetas</h1>");
    const onTitleGone = vi.fn();
    framing(inner, onTitleGone).unmount();
    rolledTo(inner, 100);
    expect(onTitleGone).not.toHaveBeenCalled();
  });
});

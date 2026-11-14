import { render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { handTo } from "../dropped";
import { closed, laid, parting } from "../ui/parting";
import Sheet from "../ui/Sheet";

const ipc = vi.hoisted(() => ({
  drawn: "",
  opened: [] as {
    said: string;
    change: (text: string) => void;
    leave: (text: string) => void;
    asking?: () => unknown;
    caret: number;
  }[],
  left: 0,
  kept: [] as boolean[],
  copied: [] as string[],
  picked: { from: 0, to: 0 },
  cut: [] as [number, number][],
  typed: [] as string[],
  mends: [] as {
    from: number;
    to: number;
    fixed: string;
    pieces: { said: string; fixed: string }[];
  }[],
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: (text: string) => {
    ipc.copied.push(text);
    return Promise.resolve();
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "doc_drawn" || cmd === "doc_placed")
      return Promise.resolve(new TextEncoder().encode(ipc.drawn).buffer);
    if (cmd === "mends_in") return Promise.resolve(ipc.mends);
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
}));

vi.mock("../ui/hosting", async (original) => {
  const standIn = (
    block: HTMLElement,
    said: string,
    change: (text: string) => void,
    leave: (text: string) => void,
    asking?: () => unknown,
  ) => {
    const host = block.ownerDocument.createElement("div");
    host.className = "editing";
    block.replaceWith(host);
    const opened = { said, change, leave, asking, caret: -1 };
    ipc.opened.push(opened);
    return {
      view: {
        state: {
          sliceDoc: (from?: number, to?: number) =>
            from === undefined ? said : said.slice(from, to),
          selection: {
            get main() {
              return ipc.picked;
            },
          },
          doc: said,
          field: () => false,
          lineBreak: "\n",
          replaceSelection: (insert: string) => ({ changes: { from: 0, to: 0, insert } }),
        },
        dispatch: (spec: {
          selection?: { anchor: number };
          changes?: { from: number; to: number; insert?: string };
        }) => {
          if (spec.selection) opened.caret = spec.selection.anchor;
          if (spec.changes) ipc.cut.push([spec.changes.from, spec.changes.to]);
          if (spec.changes?.insert) ipc.typed.push(spec.changes.insert);
        },
        focus: () => {},
      },
      leave: () => {
        ipc.left += 1;
        leave(said);
      },
      kept: (yes: boolean) => ipc.kept.push(yes),
    };
  };
  return {
    ...(await original<typeof import("../ui/hosting")>()),
    edited: standIn,
    editedLive: (_drawn: unknown, ...rest: Parameters<typeof standIn>) => standIn(...rest),
  };
});

const BODY = "# Compras\n\nleche\n\npan\n";
const DRAWN =
  '<h1 data-at="0-10">Compras</h1><p data-at="11-17">leche</p><p data-at="18-22">pan</p>';

const loaded = async (frame: HTMLIFrameElement) => {
  await waitFor(() => expect(frame.srcdoc).toContain("<article"));
  const page = frame.contentDocument as Document;
  page.open();
  page.write(frame.srcdoc);
  page.close();
  frame.dispatchEvent(new Event("load"));
  return page;
};

const shown = async (container: HTMLElement, said?: string) =>
  waitFor(() => {
    const found = Array.from(container.querySelectorAll("iframe")).find(
      (one) =>
        !said ||
        one.srcdoc.includes(said) ||
        (one.contentDocument?.querySelector(".page")?.innerHTML.includes(said) ?? false),
    );
    expect(found).toBeTruthy();
    return found as HTMLIFrameElement;
  });

beforeEach(() => {
  ipc.drawn = DRAWN;
  ipc.opened = [];
  ipc.left = 0;
  ipc.kept = [];
  ipc.copied = [];
  ipc.picked = { from: 0, to: 0 };
  ipc.cut = [];
  ipc.typed = [];
  ipc.mends = [];
});

// A choice's name, without the mark or the keys beside it.
const named = (one: Element) => one.querySelector(".truncate")?.textContent?.trim();

const offered = () => Array.from(document.querySelectorAll('[role="menuitem"]')).map(named);

const chosen = (label: string) =>
  Array.from(document.querySelectorAll<HTMLElement>('[role="menuitem"]'))
    .find((one) => named(one) === label)
    ?.click();

describe("the sheet in the main window", () => {
  it("mends a block outside the Tisty language from the line under it, without opening it", async () => {
    const writes: string[] = [];
    const body = "# Compras\n\nleche <b>fresca</b>\n";
    ipc.drawn = '<h1 data-at="0-10">Compras</h1><p data-at="11-31">leche <b>fresca</b></p>';
    ipc.mends = [
      {
        from: 11,
        to: 30,
        fixed: "leche **fresca**",
        pieces: [{ said: "<b>fresca</b>", fixed: "**fresca**" }],
      },
    ];
    const view = render(
      <Sheet value={body} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    const page = await loaded(await shown(view.container));

    await waitFor(() => expect(page.querySelector(".mend-said")).toBeTruthy());
    page.querySelector<HTMLElement>(".mend-said button")?.click();

    expect(writes).toEqual(["# Compras\n\nleche **fresca**\n"]);
    expect(ipc.opened).toHaveLength(0);
  });

  it("tells nothing to mend while it is only being read", async () => {
    ipc.mends = [{ from: 11, to: 16, fixed: "leche", pieces: [] }];
    const view = render(<Sheet value={BODY} label="Compras" reading onWrite={() => {}} />);
    const page = await loaded(await shown(view.container));
    expect(page.querySelector(".mend-said")).toBeNull();
  });

  it("opens nothing to write while it is only being read", async () => {
    const view = render(<Sheet value={BODY} label="Compras" reading onWrite={() => {}} />);
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(ipc.opened).toHaveLength(0);
  });

  it("puts a page's card at the end and moves it, writing the text", async () => {
    const writes: string[] = [];
    const got: {
      put?: ((file: string, title: string) => boolean) | null;
      move?: ((file: string, before: string | null) => string) | null;
    } = {};
    render(
      <Sheet
        value={BODY}
        label="Compras"
        onWrite={(text) => writes.push(text)}
        onInsert={(one) => {
          got.put = one;
        }}
        onOrder={(one) => {
          got.move = one;
        }}
      />,
    );
    await waitFor(() => expect(got.put).toBeTruthy());
    expect(got.put?.("a3f1-0002", "Fruta")).toBe(true);
    expect(writes[writes.length - 1]).toBe(
      "# Compras\n\nleche\n\npan\n\n![Fruta](tisty:doc/a3f1-0002)\n",
    );
    expect(got.move?.("a3f1-0009", null)).toBe("unseen");
  });

  it("hands no way to put or move cards while it is only being read", async () => {
    const puts: unknown[] = [];
    render(
      <Sheet
        value={BODY}
        label="Compras"
        reading
        onWrite={() => {}}
        onInsert={(one) => puts.push(one)}
      />,
    );
    await waitFor(() => expect(puts.length).toBeGreaterThan(0));
    expect(puts.every((one) => one === null)).toBe(true);
  });

  it("opens a document as the core's own page to write on, and its key turns it to its Markdown", async () => {
    const sheet = (n: number) => (
      <Sheet value={BODY} label="Compras" onWrite={() => {}} wholeable wholeAsked={n} />
    );
    const view = render(sheet(0));
    const page = await loaded(await shown(view.container));
    await waitFor(() =>
      expect(page.querySelector(".page")?.getAttribute("contenteditable")).toBe("true"),
    );
    expect(ipc.opened).toHaveLength(0);
    view.rerender(sheet(1));
    await waitFor(() => expect(ipc.opened.map((one) => one.said)).toEqual([BODY]));
    view.rerender(sheet(2));
    await waitFor(() => expect(ipc.left).toBe(1));
    expect(page.querySelector(".page")?.getAttribute("contenteditable")).toBe("true");
  });

  it("chooses every word of the page as read when select-all is handed to it", async () => {
    const sheet = (n: number) => (
      <Sheet value={BODY} label="Compras" onWrite={() => {}} allAsked={n} />
    );
    const view = render(sheet(0));
    const page = await loaded(await shown(view.container));
    view.rerender(sheet(1));
    expect(page.getSelection()?.toString()).toContain("leche");
    expect(ipc.opened).toHaveLength(0);
  });

  it("takes words written from outside into the Markdown view without leaving it", async () => {
    const sheet = (value: string) => (
      <Sheet value={value} label="Compras" onWrite={() => {}} wholeable wholeAsked={1} />
    );
    const view = render(sheet(BODY));
    await loaded(await shown(view.container));
    await waitFor(() => expect(ipc.opened.map((one) => one.said)).toEqual([BODY]));
    view.rerender(sheet(`${BODY}\nfruta\n`));
    expect(ipc.left).toBe(0);
    expect(ipc.cut).toEqual([[BODY.length, BODY.length]]);
  });

  it("takes words from outside into the Markdown view with Windows line ends too", async () => {
    const sheet = (value: string) => (
      <Sheet value={value} label="Compras" onWrite={() => {}} wholeable wholeAsked={1} />
    );
    const view = render(sheet(BODY));
    await loaded(await shown(view.container));
    await waitFor(() => expect(ipc.opened.map((one) => one.said)).toEqual([BODY]));
    view.rerender(sheet("# Compras\r\n\r\nleche\r\n"));
    expect(ipc.left).toBe(0);
    expect(ipc.cut.length).toBeGreaterThan(0);
  });

  it("never opens a block's own editor on a document written on as its page", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} wholeable />);
    const page = await loaded(await shown(view.container));
    await waitFor(() =>
      expect(page.querySelector(".page")?.getAttribute("contenteditable")).toBe("true"),
    );
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(ipc.opened).toHaveLength(0);
  });

  it("keeps the last document's page in view until the next one is drawn in its place", async () => {
    const view = render(<Sheet file="a3f1-0001" value={BODY} label="Compras" onWrite={() => {}} />);
    const frame = await shown(view.container);
    const page = await loaded(frame);
    const room = view.container.querySelector<HTMLElement>(".scroller") as HTMLElement;
    room.scrollTop = 500;
    ipc.drawn = '<p data-at="0-6">Otra</p>';
    view.rerender(<Sheet file="a3f1-0002" value="Otra.\n" label="Otra" onWrite={() => {}} />);
    expect(page.querySelector(".page")?.textContent).toContain("Compras");
    await waitFor(() => expect(page.querySelector(".page")?.textContent).toContain("Otra"));
    expect(Array.from(view.container.querySelectorAll("iframe"))).toEqual([frame]);
    expect(frame.classList.contains("invisible")).toBe(false);
    expect(room.scrollTop).toBe(0);
  });

  it("lays files dropped on the page after the block they land on, or at the end", async () => {
    const writes: string[] = [];
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    await loaded(await shown(view.container));
    const room = view.container.querySelector("[data-catches-files]") as Element;
    expect(handTo(room, "![foto](foto.png)")).toBe(true);
    expect(writes[writes.length - 1]).toBe("# Compras\n\nleche\n\npan\n\n![foto](foto.png)\n");
  });

  it("takes no files while the document is only read", async () => {
    const view = render(<Sheet value={BODY} label="Compras" reading onWrite={() => {}} />);
    await loaded(await shown(view.container));
    expect(view.container.querySelector("[data-catches-files]")).toBeNull();
    const room = view.container.querySelector(".gutter") as Element;
    expect(handTo(room, "![foto](foto.png)")).toBe(false);
  });

  it("ticks a stage from its box on the page without opening the block", async () => {
    const writes: string[] = [];
    const body = "# Plan\n\n- [ ] uno\n- [x] dos\n";
    ipc.drawn =
      '<h1 data-at="0-6">Plan</h1><ul data-at="8-28"><li class="task" data-at="8-18"><span class="check" data-done="false"></span>uno</li><li class="task" data-at="18-28"><span class="check" data-done="true"></span>dos</li></ul>';
    const view = render(<Sheet value={body} label="Plan" onWrite={(text) => writes.push(text)} />);
    const page = await loaded(await shown(view.container));
    page.querySelector(".check")?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(writes).toEqual(["# Plan\n\n- [x] uno\n- [x] dos\n"]);
    expect(ipc.opened).toHaveLength(0);
  });

  it("tells a box ticked on the page as a change of its own", async () => {
    const settled: [string, string][] = [];
    const body = "# Plan\n\n- [ ] uno\n";
    ipc.drawn =
      '<h1 data-at="0-6">Plan</h1><ul data-at="8-18"><li class="task" data-at="8-18"><span class="check" data-done="false"></span>uno</li></ul>';
    const view = render(
      <Sheet
        value={body}
        label="Plan"
        onWrite={() => {}}
        onSettle={(before, now) => settled.push([before, now])}
      />,
    );
    const page = await loaded(await shown(view.container));
    page.querySelector(".check")?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(settled).toEqual([[body, "# Plan\n\n- [x] uno\n"]]);
  });

  it("ticks a stage inside a quote as well", async () => {
    const writes: string[] = [];
    const body = "# Plan\n\n> - [ ] uno\n";
    ipc.drawn =
      '<h1 data-at="0-6">Plan</h1><blockquote data-at="8-20"><ul><li class="task" data-at="8-20"><span class="check" data-done="false"></span>uno</li></ul></blockquote>';
    const view = render(<Sheet value={body} label="Plan" onWrite={(text) => writes.push(text)} />);
    const page = await loaded(await shown(view.container));
    page.querySelector(".check")?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(writes).toEqual(["# Plan\n\n> - [x] uno\n"]);
  });

  it("marks its room once the page runs under the bar at the top, and unmarks it back at the top", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} />);
    await loaded(await shown(view.container));
    const room = view.container.querySelector<HTMLElement>(".scroller") as HTMLElement;
    room.scrollTop = 120;
    room.dispatchEvent(new Event("scroll"));
    await waitFor(() => expect(room.classList.contains("stuck")).toBe(true));
    room.scrollTop = 0;
    room.dispatchEvent(new Event("scroll"));
    await waitFor(() => expect(room.classList.contains("stuck")).toBe(false));
  });

  it("follows new words from outside while nothing is being written", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} />);
    await loaded(await shown(view.container));
    ipc.drawn = '<p data-at="0-6">Otra</p>';
    view.rerender(<Sheet value="Otra.\n" label="Compras" onWrite={() => {}} />);
    await shown(view.container, "Otra");
  });
});

describe("a block written empty", () => {
  it("goes with the blank line that kept it apart, wherever it was", () => {
    const text = "# Plan\n\nuno\n\ndos\n";
    expect(closed(text, 8, 12)).toBe("# Plan\n\ndos\n");
    expect(closed(text, 13, 17)).toBe("# Plan\n\nuno\n");
    expect(closed("uno\n\ndos", 0, 4)).toBe("dos");
    expect(closed("a\r\n\r\nb\r\n\r\nc", 5, 8)).toBe("a\r\n\r\nc");
  });

  it("asks a new block for only the newlines its neighbour lacks, in the file's own line ending", () => {
    expect(parting("uno", false)).toBe("\n\n");
    expect(parting("uno\n", false)).toBe("\n");
    expect(parting("uno\n\n", false)).toBe("");
    expect(parting("uno\r\n\r\n", false, "\r\n")).toBe("");
    expect(parting("\r\ndos", true, "\r\n")).toBe("\r\n");
    expect(parting("  ", true)).toBe("");
  });
});

describe("a block laid at a point of the text", () => {
  it("stands apart from the words around it, and the file still ends in a newline", () => {
    expect(laid("uno\n\ndos\n", 4, "![a](a.png)")).toBe("uno\n\n![a](a.png)\n\ndos\n");
    expect(laid("uno\n", 4, "![a](a.png)")).toBe("uno\n\n![a](a.png)\n");
    expect(laid("", 0, "![a](a.png)")).toBe("![a](a.png)\n");
    expect(laid("a\r\n\r\nb", 3, "x")).toBe("a\r\n\r\nx\r\n\r\nb");
  });
});

describe("the whole-document sheet", () => {
  it("folds the pages under it into one line that opens on a press, whatever stands above", async () => {
    const view = render(
      <Sheet
        value={BODY}
        label="Compras"
        onWrite={() => {}}
        wholeable
        above={<p>the languages</p>}
        belowFolds
        below={<p>under the sheet</p>}
      />,
    );
    const page = await loaded(await shown(view.container));
    await waitFor(() => expect(page.documentElement.classList.contains("lively")).toBe(true));
    expect(view.queryByText("under the sheet")).toBeNull();
    view.getByRole("button", { name: "Pages" }).click();
    await waitFor(() => expect(view.getByText("under the sheet")).toBeTruthy());
  });

  it("shows the link to the next page as it is, folding nothing on a page", async () => {
    const view = render(
      <Sheet
        value={BODY}
        label="Compras"
        onWrite={() => {}}
        wholeable
        above={<p>the book</p>}
        below={<p>next page</p>}
      />,
    );
    const page = await loaded(await shown(view.container));
    await waitFor(() => expect(page.documentElement.classList.contains("lively")).toBe(true));
    expect(view.getByText("next page")).toBeTruthy();
    expect(view.queryByRole("button", { name: "Pages" })).toBeNull();
  });
});

const PLACED =
  '<h1 data-at="0-10"><span data-src="2-9">Compras</span></h1>' +
  '<p data-at="11-17"><span data-src="11-16">leche</span></p>' +
  '<p data-at="18-22"><span data-src="18-21">pan</span></p>';

describe("the menus of a document written on as its page", () => {
  const opened = async () => {
    ipc.drawn = PLACED;
    const writes: string[] = [];
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={(text) => writes.push(text)} wholeable />,
    );
    const page = await loaded(await shown(view.container));
    await waitFor(() =>
      expect(page.querySelector(".page")?.getAttribute("contenteditable")).toBe("true"),
    );
    const measured = page.defaultView?.Range.prototype as unknown as {
      getBoundingClientRect?: () => unknown;
    };
    measured.getBoundingClientRect ??= () => ({ top: 0, left: 0, bottom: 0, width: 0, height: 0 });
    const words = page.querySelector('[data-src="11-16"]') as HTMLElement;
    const range = page.createRange();
    range.setStart(words.firstChild as Node, 0);
    range.setEnd(words.firstChild as Node, 3);
    page.getSelection()?.removeAllRanges();
    page.getSelection()?.addRange(range);
    return { page, words, writes };
  };

  it("cuts and copies the Markdown of the words chosen, and formats them", async () => {
    const { words, writes } = await opened();
    words.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true }));
    await waitFor(() => expect(offered()).toContain("Copy"));
    expect(offered()).toEqual(expect.arrayContaining(["Cut", "Copy", "Bold", "Italic"]));
    chosen("Copy");
    await waitFor(() => expect(ipc.copied).toContain("lec"));
    words.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true }));
    await waitFor(() => expect(offered()).toContain("Bold"));
    chosen("Bold");
    await waitFor(() => expect(writes[writes.length - 1]).toBe("# Compras\n\n**lec**he\n\npan\n"));
  });

  it("offers the blocks with Ctrl and a slash, searched by name", async () => {
    const { page, writes } = await opened();
    page
      .querySelector(".page")
      ?.dispatchEvent(new KeyboardEvent("keydown", { key: "/", ctrlKey: true, bubbles: true }));
    await waitFor(() => expect(offered()).toContain("Table"));
    expect(document.querySelector('input[placeholder="Find a block"]')).not.toBeNull();
    expect(offered()).not.toContain("Bold");
    expect(writes).toHaveLength(0);
  });
});

describe("the Markdown view of a document", () => {
  const opened = async (props: Partial<Parameters<typeof Sheet>[0]> = {}) => {
    const writes: string[] = [];
    const sheet = (more: Partial<Parameters<typeof Sheet>[0]>) => (
      <Sheet
        value={BODY}
        label="Compras"
        onWrite={(text) => writes.push(text)}
        wholeable
        wholeAsked={1}
        {...props}
        {...more}
      />
    );
    const view = render(sheet({}));
    await loaded(await shown(view.container));
    await waitFor(() => expect(ipc.opened.map((one) => one.said)).toEqual([BODY]));
    return { view, sheet, writes };
  };

  it("writes back what is typed, the file's own last line end kept", async () => {
    const { writes } = await opened();
    ipc.opened[0]?.change("# Compras\n\nleche\n\npan y vino");
    expect(writes).toEqual(["# Compras\n\nleche\n\npan y vino\n"]);
  });

  it("tells the view whether its words are kept yet", async () => {
    const { view, sheet } = await opened({ kept: false });
    expect(ipc.kept).toContain(false);
    view.rerender(sheet({ kept: true }));
    expect(ipc.kept[ipc.kept.length - 1]).toBe(true);
  });

  it("leaves the view when another document comes in", async () => {
    const { view, sheet } = await opened({ file: "a.md" });
    view.rerender(sheet({ file: "b.md", value: "# Otra\n" }));
    expect(ipc.left).toBe(1);
  });
});

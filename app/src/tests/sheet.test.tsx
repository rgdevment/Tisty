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
  wrapped: [] as [string, string][],
  flipped: 0,
  copied: [] as string[],
  picked: { from: 0, to: 0 },
  cut: [] as [number, number][],
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

vi.mock("../ui/formatting", async (original) => ({
  ...(await original<typeof import("../ui/formatting")>()),
  wrapped: (_: unknown, open: string, close: string) => {
    ipc.wrapped.push([open, close]);
    return true;
  },
}));

vi.mock("../ui/barring", async (original) => ({
  ...(await original<typeof import("../ui/barring")>()),
  flipped: () => {
    ipc.flipped += 1;
    return true;
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "doc_drawn") return Promise.resolve(new TextEncoder().encode(ipc.drawn).buffer);
    if (cmd === "mends_in") return Promise.resolve(ipc.mends);
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
}));

vi.mock("../ui/blocking", () => ({
  caretAt: (source: string, read: string) => (read ? source.indexOf(read) + read.length : 0),
  edited: (
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
        },
        dispatch: (spec: {
          selection?: { anchor: number };
          changes?: { from: number; to: number };
        }) => {
          if (spec.selection) opened.caret = spec.selection.anchor;
          if (spec.changes) ipc.cut.push([spec.changes.from, spec.changes.to]);
        },
        focus: () => {},
      },
      leave: () => {
        ipc.left += 1;
        leave(said);
      },
      kept: (yes: boolean) => ipc.kept.push(yes),
    };
  },
}));

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
      (one) => !said || one.srcdoc.includes(said),
    );
    expect(found).toBeTruthy();
    return found as HTMLIFrameElement;
  });

beforeEach(() => {
  ipc.drawn = DRAWN;
  ipc.opened = [];
  ipc.left = 0;
  ipc.kept = [];
  ipc.wrapped = [];
  ipc.flipped = 0;
  ipc.copied = [];
  ipc.picked = { from: 0, to: 0 };
  ipc.cut = [];
  ipc.mends = [];
});

const pick = (page: Document, from: number, to: number) => {
  // The page's frame has a Range of its own, which the test window's polyfill never reached.
  const measured = page.defaultView?.Range.prototype as unknown as {
    getBoundingClientRect?: () => unknown;
  };
  measured.getBoundingClientRect ??= () => ({ top: 0, left: 0, width: 0, height: 0 });
  const words = page.querySelector('p[data-at="11-17"]')?.firstChild as Node;
  const range = page.createRange();
  range.setStart(words, from);
  range.setEnd(words, to);
  page.getSelection()?.removeAllRanges();
  page.getSelection()?.addRange(range);
  return words;
};

const offered = () =>
  Array.from(document.querySelectorAll('[role="menuitem"]')).map((one) => one.textContent?.trim());

const chosen = (label: string) =>
  Array.from(document.querySelectorAll<HTMLElement>('[role="menuitem"]'))
    .find((one) => one.textContent?.trim() === label)
    ?.click();

describe("words picked on the sheet", () => {
  it("writes a quick format around them and goes back to reading, but a link stays open for its address", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} />);
    const page = await loaded(await shown(view.container));
    pick(page, 0, 5);
    page.dispatchEvent(new MouseEvent("mouseup"));
    await waitFor(() => expect(page.querySelector<HTMLElement>(".quick-bar")?.hidden).toBe(false));
    page.querySelector<HTMLButtonElement>(".quick-bar button")?.click();
    expect(ipc.opened.map((one) => one.said)).toEqual(["leche\n"]);
    expect(ipc.wrapped).toEqual([["**", "**"]]);
    expect(ipc.left).toBe(1);
  });

  it("keeps a link's block open so its address can be written", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} />);
    const page = await loaded(await shown(view.container));
    pick(page, 0, 5);
    page.dispatchEvent(new MouseEvent("mouseup"));
    await waitFor(() => expect(page.querySelector<HTMLElement>(".quick-bar")?.hidden).toBe(false));
    page.querySelectorAll<HTMLButtonElement>(".quick-bar button")[5]?.click();
    expect(ipc.wrapped).toEqual([["[", "]()"]]);
    expect(ipc.left).toBe(0);
  });

  it("offers copy, the formats and the block on a right click, and only copy where it is read", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} />);
    const page = await loaded(await shown(view.container));
    const words = pick(page, 0, 5);
    words.parentElement?.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true }));
    await waitFor(() => expect(offered()).toContain("Edit this block"));
    expect(offered()).toEqual(expect.arrayContaining(["Copy", "Bold"]));
    chosen("Copy");
    await waitFor(() => expect(ipc.copied).toEqual(["leche"]));
    words.parentElement?.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true }));
    await waitFor(() => expect(offered()).toContain("Edit this block"));
    chosen("Edit this block");
    expect(ipc.opened.map((one) => one.said)).toEqual(["leche\n"]);
    view.unmount();

    const read = render(<Sheet value={BODY} label="Compras" reading onWrite={() => {}} />);
    const quiet = await loaded(await shown(read.container));
    pick(quiet, 0, 5).parentElement?.dispatchEvent(
      new MouseEvent("contextmenu", { bubbles: true }),
    );
    await waitFor(() => expect(offered()).toEqual(["Copy"]));
  });

  it("offers cut, copy, the formats and the marks inside a block being written", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} />);
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    ipc.picked = { from: 0, to: 5 };
    const writing = () =>
      page
        .querySelector(".editing")
        ?.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true }));
    writing();
    await waitFor(() => expect(offered()).toContain("Show every mark"));
    expect(offered()).toEqual(expect.arrayContaining(["Cut", "Copy", "Bold"]));
    chosen("Bold");
    expect(ipc.wrapped).toEqual([["**", "**"]]);
    writing();
    await waitFor(() => expect(offered()).toContain("Show every mark"));
    chosen("Show every mark");
    expect(ipc.flipped).toBe(1);
    writing();
    await waitFor(() => expect(offered()).toContain("Cut"));
    chosen("Cut");
    await waitFor(() => expect(ipc.cut).toEqual([[0, 5]]));
    expect(ipc.copied).toEqual(["leche"]);
    expect(ipc.left).toBe(0);
  });
});

describe("the sheet in the main window", () => {
  it("writes a block where it was clicked, and draws the page again once it is left", async () => {
    const writes: string[] = [];
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(ipc.opened.map((one) => one.said)).toEqual(["leche\n"]);
    expect(page.querySelector(".editing")).not.toBeNull();

    ipc.opened[0].change("leche y fruta");
    expect(writes).toEqual(["# Compras\n\nleche y fruta\n\npan\n"]);

    ipc.drawn = '<h1 data-at="0-10">Compras</h1><p data-at="11-25">leche y fruta</p>';
    ipc.opened[0].leave("leche y fruta");
    await shown(view.container, "leche y fruta");
  });

  it("tells nothing for a block left as it was", async () => {
    const settled: [string, string][] = [];
    const view = render(
      <Sheet
        value={BODY}
        label="Compras"
        onWrite={() => {}}
        onSettle={(before, now) => settled.push([before, now])}
      />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    ipc.opened[0].leave("leche");
    expect(settled).toEqual([]);
  });

  it("tells the text before and after a block it left", async () => {
    const settled: [string, string][] = [];
    const view = render(
      <Sheet
        value={BODY}
        label="Compras"
        onWrite={() => {}}
        onSettle={(before, now) => settled.push([before, now])}
      />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    ipc.opened[0].change("leche y fruta");
    ipc.opened[0].leave("leche y fruta");
    expect(settled).toEqual([[BODY, "# Compras\n\nleche y fruta\n\npan\n"]]);
  });

  it("leaves the block open when another is clicked, instead of opening a second one", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} />);
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    page
      .querySelector('p[data-at="18-22"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(ipc.opened).toHaveLength(1);
    expect(ipc.left).toBe(1);
  });

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

  it("splices each change into the latest text, however many come", async () => {
    const writes: string[] = [];
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    ipc.opened[0].change("leche y");
    ipc.opened[0].change("leche y fruta");
    ipc.opened[0].change("l");
    expect(writes).toEqual([
      "# Compras\n\nleche y\n\npan\n",
      "# Compras\n\nleche y fruta\n\npan\n",
      "# Compras\n\nl\n\npan\n",
    ]);
  });

  it("keeps the block open when its own words come back late, as they do under fast typing", async () => {
    const writes: string[] = [];
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    ipc.opened[0].change("leche y");
    ipc.opened[0].change("leche y pan");
    ipc.opened[0].change("leche y pan y fruta");
    view.rerender(
      <Sheet value={writes[0]} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    view.rerender(
      <Sheet value={writes[1]} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    expect(ipc.left).toBe(0);
    expect(page.querySelector(".editing")).not.toBeNull();
    view.rerender(
      <Sheet value={writes[2]} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    ipc.opened[0].change("leche y pan y fruta fresca");
    expect(writes[writes.length - 1]).toBe("# Compras\n\nleche y pan y fruta fresca\n\npan\n");
  });

  it("ends the block when words from outside match an older text of its own once it went past it", async () => {
    const writes: string[] = [];
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    ipc.opened[0].change("leche y");
    ipc.opened[0].change("leche y pan");
    ipc.opened[0].change("leche y pan y fruta");
    view.rerender(<Sheet value={writes[1]} label="Compras" onWrite={() => {}} />);
    expect(ipc.left).toBe(0);
    view.rerender(<Sheet value={writes[0]} label="Compras" onWrite={() => {}} />);
    expect(ipc.left).toBe(1);
  });

  it("opens the whole document in one editor when asked, and gives the blocks back as it is left", async () => {
    const sheet = (n: number) => (
      <Sheet value={BODY} label="Compras" onWrite={() => {}} wholeable wholeAsked={n} />
    );
    const view = render(sheet(0));
    const page = await loaded(await shown(view.container));
    view.rerender(sheet(1));
    expect(ipc.opened.map((one) => one.said)).toEqual([BODY]);
    expect(page.querySelector(".page")?.classList.contains("whole")).toBe(true);
    view.rerender(sheet(2));
    expect(ipc.left).toBe(1);
    expect(page.querySelector(".page")?.classList.contains("whole")).toBe(false);
    expect(page.querySelectorAll(".page > div:not([data-at])")).toHaveLength(0);
  });

  it("never opens a task's page whole, from a key or from the menu", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} wholeAsked={0} />);
    const page = await loaded(await shown(view.container));
    view.rerender(<Sheet value={BODY} label="Compras" onWrite={() => {}} wholeAsked={1} />);
    expect(ipc.opened).toHaveLength(0);
    expect(page.querySelector(".page")?.classList.contains("whole")).toBe(false);
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true }));
    await waitFor(() => expect(offered()).toContain("Edit this block"));
    expect(offered()).not.toContain("Edit the whole document");
  });

  it("offers a document's whole in the menu beside its block", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} wholeable />);
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true }));
    await waitFor(() => expect(offered()).toContain("Edit the whole document"));
    chosen("Edit the whole document");
    expect(ipc.opened.map((one) => one.said)).toEqual([BODY]);
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

  it("takes words written from outside into the whole document without leaving it", async () => {
    const sheet = (value: string, n: number) => (
      <Sheet value={value} label="Compras" onWrite={() => {}} wholeable wholeAsked={n} />
    );
    const view = render(sheet(BODY, 0));
    await loaded(await shown(view.container));
    view.rerender(sheet(BODY, 1));
    expect(ipc.opened.map((one) => one.said)).toEqual([BODY]);
    view.rerender(sheet(`${BODY}\nfruta\n`, 1));
    expect(ipc.left).toBe(0);
    expect(ipc.cut).toEqual([[BODY.length, BODY.length]]);
  });

  it("leaves the whole document when words from outside come with Windows line ends", async () => {
    const sheet = (value: string, n: number) => (
      <Sheet value={value} label="Compras" onWrite={() => {}} wholeable wholeAsked={n} />
    );
    const view = render(sheet(BODY, 0));
    await loaded(await shown(view.container));
    view.rerender(sheet(BODY, 1));
    view.rerender(sheet("# Compras\r\n\r\nleche\r\n", 1));
    expect(ipc.left).toBe(1);
  });

  it("keeps the block being written open when words from outside land elsewhere", async () => {
    const writes: string[] = [];
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    view.rerender(
      <Sheet
        value={"# Compras de hoy\n\nleche\n\npan\n\nfruta\n"}
        label="Compras"
        onWrite={(text) => writes.push(text)}
      />,
    );
    expect(ipc.left).toBe(0);
    ipc.opened[0].change("leche fresca");
    expect(writes[writes.length - 1]).toBe("# Compras de hoy\n\nleche fresca\n\npan\n\nfruta\n");
  });

  it("ends the block being written when its words change from outside", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} />);
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    ipc.drawn = '<p data-at="0-6">Otra</p>';
    view.rerender(<Sheet value="Otra.\n" label="Compras" onWrite={() => {}} />);
    expect(ipc.left).toBe(1);
    await shown(view.container, "Otra");
  });

  it("ends the block being written before a page's card goes in, and keeps what was typed", async () => {
    const writes: string[] = [];
    const got: { put?: ((file: string, title: string) => boolean) | null } = {};
    const view = render(
      <Sheet
        value={BODY}
        label="Compras"
        onWrite={(text) => writes.push(text)}
        onInsert={(one) => {
          got.put = one;
        }}
      />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    ipc.opened[0].change("leche y fruta");
    expect(got.put?.("a3f1-0002", "Fruta")).toBe(true);
    expect(ipc.left).toBe(1);
    expect(writes[writes.length - 1]).toBe(
      "# Compras\n\nleche y fruta\n\npan\n\n![Fruta](tisty:doc/a3f1-0002)\n",
    );
  });

  it("keeps the last document's page in view until the next one is drawn", async () => {
    const view = render(<Sheet file="a3f1-0001" value={BODY} label="Compras" onWrite={() => {}} />);
    const first = await shown(view.container);
    await loaded(first);
    const room = view.container.querySelector<HTMLElement>(".scroller") as HTMLElement;
    room.scrollTop = 500;
    ipc.drawn = '<p data-at="0-6">Otra</p>';
    view.rerender(<Sheet file="a3f1-0002" value="Otra.\n" label="Otra" onWrite={() => {}} />);
    const next = await shown(view.container, "Otra");
    expect(next.classList.contains("invisible")).toBe(true);
    expect(first.classList.contains("invisible")).toBe(false);
    expect(view.container.contains(first)).toBe(true);
    await loaded(next);
    await waitFor(() => expect(view.container.querySelectorAll("iframe")).toHaveLength(1));
    expect(next.classList.contains("invisible")).toBe(false);
    expect(room.scrollTop).toBe(0);
  });

  it("ends the block being written when another document comes in", async () => {
    const view = render(<Sheet file="a3f1-0001" value={BODY} label="Compras" onWrite={() => {}} />);
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    view.rerender(<Sheet file="a3f1-0002" value="Otra.\n" label="Otra" onWrite={() => {}} />);
    expect(ipc.left).toBe(1);
  });

  it("opens a block exactly as the file holds it and writes back exactly what was typed", async () => {
    const writes: string[] = [];
    const body = "# Compras\n\n++leche++ y ==pan=={green}\n";
    ipc.drawn = '<h1 data-at="0-10">Compras</h1><p data-at="11-38">leche y pan</p>';
    const view = render(
      <Sheet value={body} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-38"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(ipc.opened[0].said).toBe("++leche++ y ==pan=={green}\n");
    ipc.opened[0].change("++leche++ y ==pan=={blue}");
    expect(writes).toEqual(["# Compras\n\n++leche++ y ==pan=={blue}\n"]);
  });

  it("takes the blank line of a block left empty along with it", async () => {
    const writes: string[] = [];
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    ipc.opened[0].change("");
    ipc.opened[0].leave("");
    expect(writes[writes.length - 1]).toBe("# Compras\n\npan\n");
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

  it("only ends the block being written when a box is pressed meanwhile, ticking nothing", async () => {
    const writes: string[] = [];
    const body = "# Plan\n\nnota\n\n- [ ] uno\n";
    ipc.drawn =
      '<h1 data-at="0-6">Plan</h1><p data-at="8-13">nota</p><ul data-at="14-24"><li class="task" data-at="14-24"><span class="check" data-done="false"></span>uno</li></ul>';
    const view = render(<Sheet value={body} label="Plan" onWrite={(text) => writes.push(text)} />);
    const page = await loaded(await shown(view.container));
    page.querySelector("p")?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    ipc.opened[0].change("nota larga\n");
    const before = writes.length;
    page.querySelector(".check")?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(ipc.left).toBe(1);
    expect(writes.length).toBe(before);
  });

  it("writes nothing more when words from outside end a block left empty", async () => {
    const writes: string[] = [];
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    ipc.opened[0].change("");
    const before = writes.length;
    view.rerender(
      <Sheet
        value={"# Compras\n\n  \n\npan\n"}
        label="Compras"
        onWrite={(text) => writes.push(text)}
      />,
    );
    expect(ipc.left).toBe(1);
    expect(writes.length).toBe(before);
    view.unmount();
    expect(writes.length).toBe(before);
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

  it("writes a title from its empty slot, and nothing while only its mark is there", async () => {
    const writes: string[] = [];
    ipc.drawn = "";
    const view = render(<Sheet value="" label="" onWrite={(text) => writes.push(text)} />);
    const page = await loaded(await shown(view.container));
    page
      .querySelector(".page > h1[data-title]")
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(ipc.opened.map((one) => [one.said, one.caret])).toEqual([["# ", 2]]);
    ipc.opened[0].change("# ");
    ipc.opened[0].change("# Plan");
    expect(writes).toEqual(["", "# Plan"]);
  });

  it("begins a block past the last one, apart from it by a blank line", async () => {
    const writes: string[] = [];
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector(".page")
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true, clientY: 9999 }));
    expect(ipc.opened.map((one) => one.said)).toEqual([""]);
    ipc.opened[0].change("fruta");
    expect(writes).toEqual(["# Compras\n\nleche\n\npan\n\nfruta"]);
  });

  it("ends the block on a press elsewhere in the window, but not on a picker it opened", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} />);
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    const picker = document.createElement("div");
    picker.dataset.keepsWriting = "";
    document.body.append(picker);
    picker.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    expect(ipc.left).toBe(0);
    document.body.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    expect(ipc.left).toBe(1);
    picker.remove();
  });

  it("tells the window a card would come first only when nothing is written before the block", async () => {
    const firsts: boolean[] = [];
    const asking = {
      kinds: ["doc"] as const,
      ask: (_: unknown, __: unknown, ___: unknown, first: boolean) => firsts.push(first),
    };
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={() => {}} asking={asking as never} />,
    );
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    const handed = ipc.opened[0].asking?.() as typeof asking;
    handed.ask("doc", { x: 0, y: 0 }, () => true, true);
    expect(firsts).toEqual([false]);
  });

  it("tells the block being written whether its words are kept yet", async () => {
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} kept={false} />);
    const page = await loaded(await shown(view.container));
    page
      .querySelector('p[data-at="11-17"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    view.rerender(<Sheet value={BODY} label="Compras" onWrite={() => {}} kept />);
    expect(ipc.kept).toEqual([false, true]);
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

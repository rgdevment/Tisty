import { render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Sheet, { closed } from "../ui/Sheet";

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
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) =>
    cmd === "doc_drawn"
      ? Promise.resolve(new TextEncoder().encode(ipc.drawn).buffer)
      : Promise.resolve(null),
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
        state: { sliceDoc: () => said },
        dispatch: (spec: { selection: { anchor: number } }) => {
          opened.caret = spec.selection.anchor;
        },
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
});

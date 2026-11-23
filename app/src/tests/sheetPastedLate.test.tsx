import { render, waitFor } from "@testing-library/react";
import type { Asking } from "tisty-editor/edit";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { caughtBy } from "../dropped";
import { t } from "../locales";
import type { Seat } from "../ui/pageWriter";
import Sheet from "../ui/Sheet";

const ipc = vi.hoisted(() => ({
  placed: [] as string[],
  translating: [] as ((said: string) => void)[],
  opened: 0,
}));

const BODY = "# Compras\n\nleche\n";
const DRAWN =
  '<h1 data-at="0-10"><span data-src="2-9">Compras</span></h1>' +
  '<p data-at="11-17"><span data-src="11-16">leche</span></p>';
const OTHER = "# Otra\n\npan\n";
const OTHER_DRAWN =
  '<h1 data-at="0-7"><span data-src="2-6">Otra</span></h1>' +
  '<p data-at="8-12"><span data-src="8-11">pan</span></p>';

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: { body?: string }) => {
    if (cmd === "doc_drawn" || cmd === "doc_placed") {
      const body = args?.body ?? "";
      ipc.placed.push(body);
      const drawn = body.startsWith("# Otra") ? OTHER_DRAWN : DRAWN;
      return Promise.resolve(new TextEncoder().encode(drawn).buffer);
    }
    if (cmd === "doc_from_html")
      return new Promise((done) => {
        ipc.translating.push(done);
      });
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
}));

vi.mock("../ui/hosting", async (original) => ({
  ...(await original<typeof import("../ui/hosting")>()),
  edited: (block: HTMLElement, said: string) => {
    ipc.opened += 1;
    const host = block.ownerDocument.createElement("div");
    block.replaceWith(host);
    return {
      view: {
        state: {
          sliceDoc: () => said,
          selection: { main: { from: 0, to: 0, head: 0 } },
          doc: { length: said.length, lineAt: () => ({ number: 1 }) },
          lineBreak: "\n",
        },
        dispatch: () => {},
        focus: () => {},
      },
      leave: () => {},
      kept: () => {},
    };
  },
}));

const loaded = async (frame: HTMLIFrameElement) => {
  await waitFor(() => expect(frame.srcdoc).toContain("<article"));
  const page = frame.contentDocument as Document;
  page.open();
  page.write(frame.srcdoc);
  page.close();
  frame.dispatchEvent(new Event("load"));
  return page;
};

const framed = (container: HTMLElement) =>
  waitFor(() => {
    const found = container.querySelector("iframe");
    expect(found).toBeTruthy();
    return found as HTMLIFrameElement;
  });

const writable = async (container: HTMLElement) => {
  const page = await loaded(await framed(container));
  await waitFor(() =>
    expect(page.querySelector(".page")?.getAttribute("contenteditable")).toBe("true"),
  );
  return page;
};

const atLeche = (page: Document) => {
  const words = page.querySelector('.page [data-src="11-16"]')?.firstChild;
  if (words) page.getSelection()?.collapse(words, 5);
};

const filePasted = (page: Document) => {
  atLeche(page);
  const paste = new Event("paste", { bubbles: true, cancelable: true });
  Object.defineProperty(paste, "clipboardData", {
    value: { files: [new File(["x"], "x.png")], getData: () => "" },
  });
  page.querySelector(".page")?.dispatchEvent(paste);
};

const wordsPasted = (page: Document) => {
  atLeche(page);
  const paste = new InputEvent("beforeinput", {
    inputType: "insertFromPaste",
    bubbles: true,
    cancelable: true,
  });
  Object.defineProperty(paste, "getTargetRanges", { value: () => [] });
  Object.defineProperty(paste, "dataTransfer", {
    value: { getData: (kind: string) => (kind === "text/html" ? "<b>y</b>" : "y") },
  });
  page.querySelector(".page")?.dispatchEvent(paste);
};

const telling = (told: unknown[]): Asking => ({
  kinds: [],
  ask: () => {},
  lost: (notice) => told.push(notice ?? t("attachmentLost")),
});

const attaching = (told: unknown[] = []) => {
  const held: { answer: (said: string) => void } = { answer: () => {} };
  const asking: Asking = {
    ...telling(told),
    pasted: () =>
      new Promise((done) => {
        held.answer = done;
      }),
  };
  return { asking, held };
};

const settled = () => new Promise((done) => setTimeout(done, 50));

beforeEach(() => {
  ipc.placed = [];
  ipc.translating = [];
  ipc.opened = 0;
});

describe("a paste answered after the sheet moved to another document", () => {
  it("writes no file into the next document, and says it came for a page now gone", async () => {
    const told: unknown[] = [];
    const { asking, held } = attaching(told);
    const intoFirst: string[] = [];
    const intoNext: string[] = [];
    const sheet = (file: string, value: string, into: string[]) => (
      <Sheet
        file={file}
        value={value}
        label={file}
        asking={asking}
        onWrite={(text) => into.push(text)}
      />
    );
    const view = render(sheet("compras.md", BODY, intoFirst));
    filePasted(await writable(view.container));

    view.rerender(sheet("otra.md", OTHER, intoNext));
    held.answer("![x](x.png)");
    await settled();

    expect(intoNext).toEqual([]);
    expect(intoFirst).toEqual([]);
    expect(told).toEqual([t("attachmentLost")]);
  });

  it("writes no words into the next document, and says what was pasted came too late", async () => {
    const intoFirst: string[] = [];
    const intoNext: string[] = [];
    const told: unknown[] = [];
    const sheet = (file: string, value: string, into: string[]) => (
      <Sheet
        file={file}
        value={value}
        label={file}
        asking={telling(told)}
        onWrite={(text) => into.push(text)}
      />
    );
    const view = render(sheet("compras.md", BODY, intoFirst));
    wordsPasted(await writable(view.container));
    await waitFor(() => expect(ipc.translating).toHaveLength(1));

    view.rerender(sheet("otra.md", OTHER, intoNext));
    ipc.translating[0]("**y**");
    await settled();

    expect(intoNext).toEqual([]);
    expect(intoFirst).toEqual([]);
    expect(told).toEqual([t("pastedLost")]);
  });

  it("says so through the sheet's own failures when its host gives no way to say it", async () => {
    const intoNext: string[] = [];
    const failed: unknown[] = [];
    const sheet = (file: string, value: string, into: string[]) => (
      <Sheet
        file={file}
        value={value}
        label={file}
        onWrite={(text) => into.push(text)}
        onFail={(said) => failed.push(said)}
      />
    );
    const view = render(sheet("compras.md", BODY, []));
    wordsPasted(await writable(view.container));
    await waitFor(() => expect(ipc.translating).toHaveLength(1));

    view.rerender(sheet("otra.md", OTHER, intoNext));
    ipc.translating[0]("**y**");
    await settled();

    expect(intoNext).toEqual([]);
    expect(failed).toEqual([t("pastedLost")]);
  });

  it("lands no dropped file in the next document", async () => {
    const intoNext: string[] = [];
    const sheet = (file: string, value: string, into: string[]) => (
      <Sheet file={file} value={value} label={file} onWrite={(text) => into.push(text)} />
    );
    const view = render(sheet("compras.md", BODY, []));
    await writable(view.container);
    const room = view.container.querySelector("[data-catches-files]") as Element;
    const landing = caughtBy(room);

    view.rerender(sheet("otra.md", OTHER, intoNext));

    expect(landing?.("![x](x.png)")).toBe(false);
    expect(intoNext).toEqual([]);
  });
});

describe("a paste answered while its page takes its file", () => {
  it("lands on the same page and is drawn there when its holder keeps the page", async () => {
    const told: unknown[] = [];
    const { asking, held } = attaching(told);
    const seat: Seat = { current: null };
    const writes: string[] = [];
    const sheet = (file: string) => (
      <Sheet
        file={file}
        fileNow={() => "t_one-0001"}
        seat={seat}
        value={BODY}
        label="Compras"
        inline
        asking={asking}
        onWrite={(text) => writes.push(text)}
      />
    );
    const view = render(sheet("unwritten:one"));
    filePasted(await writable(view.container));

    view.rerender(sheet("t_one-0001"));
    held.answer("![x](x.png)");
    await settled();

    const pasted = "# Compras\n\nleche\n\n![x](x.png)\n";
    expect(writes).toEqual([pasted]);
    await waitFor(() => expect(ipc.placed).toContain(pasted));
    expect(told).toEqual([]);
  });

  it("lands on the page shown again when the page is laid on a new sheet", async () => {
    const { asking, held } = attaching();
    const seat: Seat = { current: null };
    const intoFirst: string[] = [];
    const intoSecond: string[] = [];
    const sheet = (key: string, file: string, into: string[]) => (
      <Sheet
        key={key}
        file={file}
        seat={seat}
        value={BODY}
        label="Compras"
        inline
        asking={asking}
        onWrite={(text) => into.push(text)}
      />
    );
    const view = render(sheet("first", "unwritten:one", intoFirst));
    filePasted(await writable(view.container));

    view.rerender(sheet("second", "t_one-0001", intoSecond));
    held.answer("![x](x.png)");
    await settled();

    expect(intoFirst).toEqual([]);
    expect(intoSecond).toEqual(["# Compras\n\nleche\n\n![x](x.png)\n"]);
  });

  it("is not laid on the new sheet when the words it was pasted among are gone, and says so", async () => {
    const told: unknown[] = [];
    const { asking, held } = attaching(told);
    const seat: Seat = { current: null };
    const intoSecond: string[] = [];
    const sheet = (key: string, value: string, into: string[]) => (
      <Sheet
        key={key}
        file="unwritten:one"
        seat={seat}
        value={value}
        label="Compras"
        inline
        asking={asking}
        onWrite={(text) => into.push(text)}
      />
    );
    const view = render(sheet("first", BODY, []));
    filePasted(await writable(view.container));

    view.rerender(sheet("second", "# Compras\n\nlechuga y más", intoSecond));
    held.answer("![x](x.png)");
    await settled();

    expect(intoSecond).toEqual([]);
    expect(told).toEqual([t("attachmentLost")]);
  });

  it("lands no dropped file on a sheet gone, but on the one that shows the page now", async () => {
    const seat: Seat = { current: null };
    const intoSecond: string[] = [];
    const sheet = (key: string, into: string[]) => (
      <Sheet
        key={key}
        file="unwritten:one"
        seat={seat}
        value={BODY}
        label="Compras"
        inline
        onWrite={(text) => into.push(text)}
      />
    );
    const view = render(sheet("first", []));
    await writable(view.container);
    const landing = caughtBy(view.container.querySelector("[data-catches-files]") as Element);

    view.rerender(sheet("second", intoSecond));

    expect(landing?.("![x](x.png)")).toBe(true);
    expect(intoSecond).toEqual(["# Compras\n\nleche\n\n![x](x.png)\n"]);
  });
});

describe("a file dropped on a page that turns read-only while it is read", () => {
  it("is not written, so the window says it was lost", async () => {
    const writes: string[] = [];
    const sheet = (reading: boolean) => (
      <Sheet
        file="compras.md"
        value={BODY}
        label="Compras"
        reading={reading}
        onWrite={(text) => writes.push(text)}
      />
    );
    const view = render(sheet(false));
    await writable(view.container);
    const landing = caughtBy(view.container.querySelector("[data-catches-files]") as Element);

    view.rerender(sheet(true));

    expect(landing?.("![x](x.png)")).toBe(false);
    expect(writes).toEqual([]);
  });
});

describe("a paste answered after the page turned to Markdown", () => {
  it("is not written behind the Markdown being edited, and says so", async () => {
    const told: unknown[] = [];
    const { asking, held } = attaching(told);
    const writes: string[] = [];
    const sheet = (markdown: boolean) => (
      <Sheet
        file="compras.md"
        value={BODY}
        label="Compras"
        asking={asking}
        view={{ markdown, onTurn: () => {} }}
        onWrite={(text) => writes.push(text)}
      />
    );
    const view = render(sheet(false));
    filePasted(await writable(view.container));

    view.rerender(sheet(true));
    await waitFor(() => expect(ipc.opened).toBe(1));
    held.answer("![x](x.png)");
    await settled();

    expect(writes).toEqual([]);
    expect(told).toEqual([t("attachmentLost")]);
  });
});

import { act, render, screen, waitFor } from "@testing-library/react";
import type { Asking } from "tisty-editor/edit";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import type { Seat } from "../ui/pageWriter";
import Sheet from "../ui/Sheet";

const ipc = vi.hoisted(() => ({
  edits: [] as ((said: string) => void)[],
  written: [] as Promise<unknown>[],
}));

const TABLED = "# Compras\n\n| Hecho | Cosa |\n| --- | --- |\n| [ ] | leche |\n";
const TABLED_DRAWN =
  '<h1 data-at="0-10"><span data-src="2-9">Compras</span></h1>' +
  '<div class="table" data-at="11-58"><table><thead><tr><th>Hecho</th><th>Cosa</th></tr></thead>' +
  '<tbody><tr><td><button type="button" data-asked="table" data-act="toggle" data-row="0" data-column="0" data-to="1"></button></td>' +
  "<td>leche</td></tr></tbody></table></div>";
const STEPPED = "# Compras\n\n- [ ] Comprar pan\n";
const STEPPED_DRAWN =
  '<h1 data-at="0-10"><span data-src="2-9">Compras</span></h1>' +
  '<ul data-at="11-28"><li class="task" data-at="11-28"><button type="button" data-asked="part"></button>' +
  '<span data-src="17-28">Comprar pan</span></li></ul>';
const WORDS = "# Compras\n\nleche\n";
const WORDS_DRAWN =
  '<h1 data-at="0-10"><span data-src="2-9">Compras</span></h1>' +
  '<p data-at="11-17"><span data-src="11-16">leche</span></p>';
const OTHER_DRAWN = '<h1 data-at="0-7"><span data-src="2-6">Otra</span></h1>';

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: { body?: string }) => {
    if (cmd === "doc_drawn" || cmd === "doc_placed") {
      const body = args?.body ?? "";
      const drawn = body.includes("| Hecho")
        ? TABLED_DRAWN
        : body.includes("- [ ]")
          ? STEPPED_DRAWN
          : body.includes("leche")
            ? WORDS_DRAWN
            : OTHER_DRAWN;
      return Promise.resolve(new TextEncoder().encode(drawn).buffer);
    }
    if (cmd === "table_edit")
      return new Promise((done) => {
        ipc.edits.push(done);
      });
    if (cmd === "table_written")
      return (
        ipc.written.shift() ??
        Promise.resolve({ heads: ["Hecho", "Cosa"], rows: [["[ ]", "leche"]] })
      );
    if (cmd === "table_canon") return Promise.resolve((args as { typed?: string }).typed);
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
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

const writable = async (container: HTMLElement) => {
  const frame = await waitFor(() => {
    const found = container.querySelector("iframe");
    expect(found).toBeTruthy();
    return found as HTMLIFrameElement;
  });
  const page = await loaded(frame);
  await waitFor(() =>
    expect(page.querySelector(".page")?.getAttribute("contenteditable")).toBe("true"),
  );
  return page;
};

const pressed = (page: Document, selector: string) =>
  page.querySelector(selector)?.dispatchEvent(new MouseEvent("click", { bubbles: true }));

const telling = (told: unknown[]): Asking => ({
  kinds: [],
  ask: () => {},
  lost: (notice) => told.push(notice ?? t("attachmentLost")),
});

const settled = () => act(() => new Promise((done) => setTimeout(done, 50)));

const TICKED = TABLED.replace("[ ]", "[x]");

beforeEach(() => {
  ipc.edits = [];
  ipc.written = [];
});

describe("a table's change answered after the sheet moved to another document", () => {
  it("writes nothing of the first document into the next, and says the change came too late", async () => {
    const told: unknown[] = [];
    const intoFirst: string[] = [];
    const intoNext: string[] = [];
    const sheet = (file: string, value: string, into: string[]) => (
      <Sheet
        file={file}
        value={value}
        label={file}
        asking={telling(told)}
        onWrite={(text) => into.push(text)}
      />
    );
    const view = render(sheet("compras.md", TABLED, intoFirst));
    pressed(await writable(view.container), "button[data-act='toggle']");
    await waitFor(() => expect(ipc.edits).toHaveLength(1));

    view.rerender(sheet("otra.md", "# Otra\n", intoNext));
    ipc.edits[0](TICKED);
    await settled();

    expect(intoNext).toEqual([]);
    expect(intoFirst).toEqual([]);
    expect(told).toEqual([t("tableLost")]);
  });

  it("keeps the change aside for the first document where its window keeps words aside", async () => {
    const told: unknown[] = [];
    const intoNext: string[] = [];
    const aside: [string, string][] = [];
    const sheet = (file: string, value: string, into: string[]) => (
      <Sheet
        file={file}
        value={value}
        label={file}
        asking={telling(told)}
        onAside={(named, words) => aside.push([named, words])}
        onWrite={(text) => into.push(text)}
      />
    );
    const view = render(sheet("compras.md", TABLED, []));
    pressed(await writable(view.container), "button[data-act='toggle']");
    await waitFor(() => expect(ipc.edits).toHaveLength(1));

    view.rerender(sheet("otra.md", "# Otra\n", intoNext));
    ipc.edits[0](TICKED);
    await settled();

    expect(intoNext).toEqual([]);
    expect(aside).toEqual([["compras.md", TICKED]]);
    expect(told).toEqual([t("tableKept")]);
  });

  it("lands on the sheet that shows the same page now, when its holder keeps the page", async () => {
    const seat: Seat = { current: null };
    const intoFirst: string[] = [];
    const intoSecond: string[] = [];
    const sheet = (key: string, into: string[]) => (
      <Sheet
        key={key}
        file="unwritten:one"
        seat={seat}
        value={TABLED}
        label="Compras"
        onWrite={(text) => into.push(text)}
      />
    );
    const view = render(sheet("first", intoFirst));
    pressed(await writable(view.container), "button[data-act='toggle']");
    await waitFor(() => expect(ipc.edits).toHaveLength(1));

    view.rerender(sheet("second", intoSecond));
    ipc.edits[0](TICKED);
    await settled();

    expect(intoFirst).toEqual([]);
    expect(intoSecond).toEqual([TICKED]);
  });

  it("still lands on the page it was asked on while the sheet shows it", async () => {
    const writes: string[] = [];
    const view = render(
      <Sheet
        file="compras.md"
        value={TABLED}
        label="Compras"
        onWrite={(text) => writes.push(text)}
      />,
    );
    pressed(await writable(view.container), "button[data-act='toggle']");
    await waitFor(() => expect(ipc.edits).toHaveLength(1));

    ipc.edits[0](TICKED);
    await settled();

    expect(writes).toEqual([TICKED]);
  });
});

describe("a step's part made after the sheet moved to another document", () => {
  it("lays no card in the next document, even where it holds the same step, and says so", async () => {
    const told: unknown[] = [];
    const intoNext: string[] = [];
    const made: ((id: string) => void)[] = [];
    const onPart = () =>
      new Promise<string>((done) => {
        made.push(done);
      });
    const sheet = (file: string, value: string, into: string[]) => (
      <Sheet
        file={file}
        value={value}
        label={file}
        asking={telling(told)}
        onPart={onPart}
        onWrite={(text) => into.push(text)}
      />
    );
    const view = render(sheet("compras.md", STEPPED, []));
    pressed(await writable(view.container), "button[data-asked='part']");
    await waitFor(() => expect(made).toHaveLength(1));

    view.rerender(sheet("otra.md", `${STEPPED}\nOtra cosa\n`, intoNext));
    made[0]("t_part-0001");
    await settled();

    expect(intoNext).toEqual([]);
    expect(told).toEqual([t("taskMadeCardLost")]);
  });

  it("lays its card where the step still stands on the same page", async () => {
    const writes: string[] = [];
    const view = render(
      <Sheet
        file="compras.md"
        value={STEPPED}
        label="Compras"
        onPart={() => Promise.resolve("t_part-0001")}
        onWrite={(text) => writes.push(text)}
      />,
    );
    pressed(await writable(view.container), "button[data-asked='part']");
    await settled();

    expect(writes).toEqual(["# Compras\n\n![Comprar pan](tisty:task/t_part-0001)\n"]);
  });
});

describe("a block chosen from the menu after the sheet moved to another document", () => {
  it("writes no block into the next document, and says the choice came too late", async () => {
    const told: unknown[] = [];
    const intoNext: string[] = [];
    const sheet = (file: string, value: string, into: string[]) => (
      <Sheet
        file={file}
        value={value}
        label={file}
        asking={telling(told)}
        onWrite={(text) => into.push(text)}
      />
    );
    const view = render(sheet("compras.md", WORDS, []));
    const page = await writable(view.container);
    const measured = page.defaultView?.Range.prototype as unknown as {
      getBoundingClientRect?: () => unknown;
    };
    measured.getBoundingClientRect ??= () => new DOMRect();
    const words = page.querySelector('.page [data-src="11-16"]')?.firstChild;
    if (words) page.getSelection()?.collapse(words, 5);
    pressed(page, ".page-plus");
    const heading = await screen.findByRole("menuitem", { name: new RegExp(t("bigTitle")) });

    view.rerender(sheet("otra.md", "# Otra\n\npan y queso\n", intoNext));
    act(() => heading.click());
    await settled();

    expect(intoNext).toEqual([]);
    expect(told).toEqual([t("choiceLost")]);
  });
});

describe("a block chosen from the menu while its page takes its file", () => {
  it("is written on that same page, with nothing said", async () => {
    const told: unknown[] = [];
    const writes: string[] = [];
    const seat: Seat = { current: null };
    const sheet = (file: string) => (
      <Sheet
        file={file}
        fileNow={() => "t_one-0001"}
        seat={seat}
        value={WORDS}
        label="Compras"
        asking={telling(told)}
        onWrite={(text) => writes.push(text)}
      />
    );
    const view = render(sheet("unwritten:one"));
    const page = await writable(view.container);
    const measured = page.defaultView?.Range.prototype as unknown as {
      getBoundingClientRect?: () => unknown;
    };
    measured.getBoundingClientRect ??= () => new DOMRect();
    const words = page.querySelector('.page [data-src="11-16"]')?.firstChild;
    if (words) page.getSelection()?.collapse(words, 5);
    pressed(page, ".page-plus");
    const heading = await screen.findByRole("menuitem", { name: new RegExp(t("bigTitle")) });

    view.rerender(sheet("t_one-0001"));
    act(() => heading.click());
    await settled();

    expect(told).toEqual([]);
    expect(writes).toEqual(["# Compras\n\n# leche\n"]);
  });
});

describe("a table's change answered after words came to the same page meanwhile", () => {
  it("is laid over those words instead of writing the text it was asked on", async () => {
    const writes: string[] = [];
    const sheet = (value: string) => (
      <Sheet
        file="compras.md"
        value={value}
        label="Compras"
        onWrite={(text) => writes.push(text)}
      />
    );
    const view = render(sheet(TABLED));
    pressed(await writable(view.container), "button[data-act='toggle']");
    await waitFor(() => expect(ipc.edits).toHaveLength(1));

    view.rerender(sheet(`${TABLED}\nNotas de hoy\n`));
    await settled();
    ipc.edits[0](TICKED);
    await settled();

    expect(writes[writes.length - 1]).toBe(`${TICKED}\nNotas de hoy\n`);
  });
});

describe("keys held for the next cell when that cell cannot open", () => {
  it("are written on a line begun under the table, and it says so", async () => {
    const told: unknown[] = [];
    const writes: string[] = [];
    const view = render(
      <Sheet
        file="compras.md"
        value={TABLED}
        label="Compras"
        asking={telling(told)}
        onWrite={(text) => writes.push(text)}
      />,
    );
    const page = await writable(view.container);
    pressed(page, "tbody td:nth-child(2)");
    const input = await waitFor(() => {
      const found = page.querySelector<HTMLInputElement>(".table-input");
      expect(found).toBeTruthy();
      return found as HTMLInputElement;
    });
    const refusal: { say: (why: string) => void } = { say: () => {} };
    ipc.written.push(
      new Promise((_, refused) => {
        refusal.say = refused;
      }),
    );
    input.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Tab", shiftKey: true, bubbles: true, cancelable: true }),
    );
    await settled();
    for (const key of "pan")
      page.body.dispatchEvent(
        new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }),
      );

    refusal.say("moved");
    await settled();

    expect(writes).toEqual([`${TABLED}\npan`]);
    expect(told).toEqual([t("tableKeysUnder")]);
  });
});

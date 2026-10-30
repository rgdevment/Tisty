import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useEffect } from "react";
import { wordsIn as worded } from "tisty-editor/read";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { DOC } from "../markdown";
import Docs from "../ui/Docs";
import { counted } from "../ui/paperFacts";

const store = vi.hoisted(() => ({
  ran: [] as string[],
  went: [] as string[],
  author: null as string | null,
  editor: null as string | null,
  born: null as string | null,
  sent: [] as string[],
  stamped: [] as boolean[],
}));

vi.mock("@tauri-apps/api/core", async () => ({
  invoke: (await import("./opened")).opening((cmd: string) => {
    store.sent.push(cmd);
    switch (cmd) {
      case "doc_read":
        return Promise.resolve("# Compras\n\nleche");
      case "doc_facts":
        return Promise.resolve({
          made: 1772668800,
          wrote: 1772755200,
          bytes: 8400,
          author: store.author,
          editor: store.editor,
          born: store.born,
        });
      default:
        return Promise.resolve(null);
    }
  }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: () => Promise.resolve("D:/salida/Compras.pdf"),
  open: () => Promise.resolve(null),
  ask: () => Promise.resolve(true),
}));

vi.mock("../ui/printing", () => ({
  pdfOf: () => Promise.resolve(new Blob(["%PDF"], { type: "application/pdf" })),
  STAMPING: { covered: false, credited: false },
  stampOf: (_: string, __: string, chosen: { covered: boolean; credited: boolean }) => {
    store.stamped.push(chosen.credited);
    return Promise.resolve({ credited: chosen.credited });
  },
  factsCard: () => Promise.resolve(undefined),
  seenIn: () => ({ turns: [], opened: [] }),
}));

vi.mock("../ui/Sheet", () => ({
  default: ({
    value,
    onWrite,
    onBlocks,
    onOutline,
    onReady,
  }: {
    value: string;
    onWrite: (text: string) => void;
    onBlocks?: (blocks: unknown[]) => void;
    onOutline?: (heads: unknown[]) => void;
    onReady?: (read: () => unknown) => void;
  }) => {
    useEffect(() => {
      onBlocks?.([
        { key: "h1", label: "Title", hint: "#", icon: "A", run: () => store.ran.push("h1") },
        {
          key: "code",
          label: "Code block",
          hint: "```",
          icon: "C",
          run: () => store.ran.push("code"),
        },
        {
          key: "link",
          label: "Link",
          hint: "[ ]( )",
          icon: "L",
          run: () => store.ran.push("link"),
        },
      ]);
      onOutline?.([
        { key: "1", level: 1, text: "Compras", go: () => store.went.push("Compras") },
        { key: "2", level: 2, text: "Fruta", go: () => store.went.push("Fruta") },
      ]);
      onReady?.(() => ({ type: "doc", content: [] }));
    }, [onBlocks, onOutline, onReady]);
    return <textarea aria-label="editor" value={value} onChange={(e) => onWrite(e.target.value)} />;
  },
}));

const known: Filed[] = [
  { id: "01F", file: "a3f1-0001", title: "Compras", folder: null, archived: false, away: false },
];

const widen = (px: number) => {
  Object.defineProperty(window, "innerWidth", { value: px, configurable: true, writable: true });
};

const resize = (px: number) => {
  act(() => {
    widen(px);
    window.dispatchEvent(new Event("resize"));
  });
};

const show = (open = "a3f1-0001") =>
  render(<Docs open={open} known={known} onKept={vi.fn()} onError={vi.fn()} />);

const showBare = () => render(<Docs known={known} onKept={vi.fn()} onError={vi.fn()} />);

const column = () => screen.queryByRole("complementary", { name: "About this document" });

describe("the column beside a document", () => {
  beforeEach(() => {
    store.ran = [];
    store.went = [];
    store.author = null;
    store.editor = null;
    store.born = null;
    store.sent = [];
    widen(1500);
  });

  it("says nothing about who wrote it while nobody has signed this store", async () => {
    show();

    const aside = await screen.findByRole("complementary", { name: "About this document" });
    await within(aside).findByText("8.4 kB");
    expect(within(aside).queryByText("Author")).toBeNull();
    expect(within(aside).queryByText("Edited by")).toBeNull();
  });

  it("names the author, and names the editor only when somebody else wrote it", async () => {
    store.author = "fulanito";
    store.editor = "rgdevment";
    show();

    const aside = await screen.findByRole("complementary", { name: "About this document" });
    await within(aside).findByText("Author");
    expect(within(aside).getByText("fulanito")).toBeTruthy();
    expect(within(aside).getByText("Edited by")).toBeTruthy();
    expect(within(aside).getByText("rgdevment")).toBeTruthy();
  });

  it("shows itself the first time the window is wide enough", async () => {
    show();

    expect(await screen.findByRole("complementary", { name: "About this document" })).toBeTruthy();
  });

  it("stays away on a narrow window while nobody has asked for it", async () => {
    widen(1100);
    show();

    await screen.findByLabelText("editor");
    expect(column()).toBeNull();
  });

  it("shows itself by default right at the 1440 threshold", async () => {
    widen(1440);
    show();

    expect(await screen.findByRole("complementary", { name: "About this document" })).toBeTruthy();
  });

  it("stays away by default a pixel under the 1440 threshold", async () => {
    widen(1439);
    show();

    await screen.findByLabelText("editor");
    expect(column()).toBeNull();
  });

  it("stays closed while the window stays the size it was", async () => {
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Close this column" }));

    resize(1520);
    expect(column()).toBeNull();
  });

  it("retires when the window is no longer wide enough", async () => {
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    resize(1100);
    expect(column()).toBeNull();
  });

  it("comes back every time the window grows again, closed or not", async () => {
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Close this column" }));
    expect(column()).toBeNull();

    resize(1100);
    resize(1500);

    expect(await screen.findByRole("complementary", { name: "About this document" })).toBeTruthy();
  });

  it("holds a column asked for on a narrow window", async () => {
    widen(1100);
    show();
    await userEvent.click(await screen.findByRole("button", { name: /About the document/ }));

    expect(await screen.findByRole("complementary", { name: "About this document" })).toBeTruthy();
  });

  it("keeps the handle out of sight while the column is up", async () => {
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    expect(screen.queryByRole("button", { name: /About the document/ })).toBeNull();
  });

  it("offers neither column nor handle when no document is open", async () => {
    showBare();

    expect(await screen.findByText("Pick a document, or make one")).toBeTruthy();
    expect(column()).toBeNull();
    expect(screen.queryByRole("button", { name: /About the document/ })).toBeNull();
  });
});

describe("what the column carries", () => {
  beforeEach(() => {
    store.ran = [];
    store.went = [];
    widen(1500);
  });

  it("counts the words of the document, not its markup", async () => {
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    expect(await screen.findByText("2 words")).toBeTruthy();
  });

  it("says what the file takes up on disk", async () => {
    show();

    expect(await screen.findByText("8.4 kB")).toBeTruthy();
  });

  it("jumps to a heading from the outline", async () => {
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Fruta" }));

    expect(store.went).toEqual(["Fruta"]);
  });
});

describe("where the document rests", () => {
  const sheetOf = () => screen.getByLabelText("editor").closest("div");
  const roomOf = () => sheetOf()?.parentElement;

  it("stays centred, always", async () => {
    widen(1500);
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    expect(sheetOf()?.className).toContain("mx-auto");
  });

  it("gives the column its room when what is left still fits the page", async () => {
    widen(1600);
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    expect(roomOf()?.style.paddingRight).toBe("344px");
  });

  it("counts the sidebar at the width it really has, and lets the column float when it does not fit", async () => {
    widen(1500);
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    expect(roomOf()?.style.paddingRight).toBe("0px");
  });

  it("lets the column lie over the text rather than pushing it", async () => {
    widen(1100);
    show();
    await userEvent.click(await screen.findByRole("button", { name: "About the document" }));
    await screen.findByRole("complementary", { name: "About this document" });

    expect(roomOf()?.style.paddingRight).toBe("0px");
  });

  it("reserves nothing while the column is away", async () => {
    widen(1500);
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Close this column" }));

    expect(roomOf()?.style.paddingRight).toBe("0px");
  });

  it("is never narrowed, whatever the window", async () => {
    widen(1100);
    show();
    await userEvent.click(await screen.findByRole("button", { name: "About the document" }));

    expect(sheetOf()?.style.maxWidth).toBe("842px");
  });

  it("reserves room right at 1522, the exact width the column needs to fit beside the page", async () => {
    widen(1522);
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    expect(roomOf()?.style.paddingRight).toBe("344px");
  });

  it("reserves nothing a single pixel short of 1522", async () => {
    widen(1521);
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    expect(roomOf()?.style.paddingRight).toBe("0px");
  });
});

describe("measuring a document", () => {
  it("does not count the markup as words", () => {
    expect(worded("# Compras\n\n- leche\n- pan")).toBe(3);
  });

  it("leaves fenced code out of the count", () => {
    expect(worded("hola\n\n```\nlet x = 1\n```\n\nadios")).toBe(2);
  });

  it("counts the documents the text points at", () => {
    expect(counted(`[a](${DOC}01) y [b](${DOC}02)`, DOC)).toBe(2);
  });
});

describe("what the column offers for the document", () => {
  beforeEach(() => {
    URL.createObjectURL = () => "blob:probe";
    URL.revokeObjectURL = () => {};
  });

  const ways = [
    "Preview the PDF",
    "Export to PDF…",
    "Export as HTML…",
    "Export as Word…",
    "Copy as Markdown",
    "Save a copy as Markdown…",
  ];

  const pick = async (way: string) => {
    await userEvent.click(await screen.findByRole("button", { name: /^export$/i }));
    await userEvent.click(await screen.findByRole("menuitem", { name: new RegExp(way) }));
  };

  it("gathers every way out under one button, in the order they are reached for", async () => {
    widen(1500);
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    await userEvent.click(screen.getByRole("button", { name: /^export$/i }));

    const said = screen.getAllByRole("menuitem").map((one) => one.textContent ?? "");
    expect(ways.map((way) => said.findIndex((one) => one.includes(way)))).toEqual([
      0, 1, 2, 3, 4, 5,
    ]);
  });

  it("asks before a PDF leaves whether it carries the credit, unticked unless the person ticks it", async () => {
    store.author = "rgdevment";
    widen(1500);
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    await pick("Export to PDF");

    const box = await screen.findByText("A quiet “Made with Tisty” at the end");
    const tick = within(box.closest("label") as HTMLElement).getByRole("checkbox");
    expect((tick as HTMLInputElement).checked).toBe(false);
    await userEvent.click(tick);
    expect((tick as HTMLInputElement).checked).toBe(true);
    await userEvent.click(screen.getByRole("button", { name: /^export pdf$/i }));

    await waitFor(() => expect(store.sent).toContain("keep_file"));
    expect(store.stamped[store.stamped.length - 1]).toBe(true);
  });

  it("names each way by what it makes, so no verb has to mean two things", async () => {
    widen(1500);
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    await userEvent.click(screen.getByRole("button", { name: /^export$/i }));

    expect(screen.getByRole("menuitem", { name: /Export to PDF/ })).toBeTruthy();
    expect(screen.getByRole("menuitem", { name: /Save a copy as Markdown/ })).toBeTruthy();
  });

  it("says it goes back to the document, never a Close beside the window's own", async () => {
    widen(1500);
    show();
    await pick("Preview the PDF");

    expect(await screen.findByRole("button", { name: "← Back to the document" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Close" })).toBeNull();
  });

  it("previews with the same choices the export offers, and redraws as they change", async () => {
    store.stamped = [];
    widen(1500);
    show();
    await pick("Preview the PDF");
    const credit = await screen.findByRole("checkbox", {
      name: "A quiet “Made with Tisty” at the end",
    });
    await waitFor(() => expect(store.stamped).toEqual([false]));

    await userEvent.click(credit);

    await waitFor(() => expect(store.stamped[store.stamped.length - 1]).toBe(true));
    expect((credit as HTMLInputElement).checked).toBe(true);
  });

  it("lets the preview go when the section is left, instead of leaking it", async () => {
    const freed: string[] = [];
    URL.revokeObjectURL = (one: string) => {
      freed.push(one);
    };
    widen(1500);
    const view = show();
    await pick("Preview the PDF");
    expect(await screen.findByRole("button", { name: "← Back to the document" })).toBeTruthy();

    view.unmount();

    expect(freed).toContain("blob:probe");
  });

  it("shuts the preview when told to", async () => {
    widen(1500);
    show();
    await pick("Preview the PDF");
    await userEvent.click(await screen.findByRole("button", { name: "← Back to the document" }));

    expect(screen.queryByRole("button", { name: "← Back to the document" })).toBeNull();
  });

  it("says how often the writing leans on each tag it carries", async () => {
    render(
      <Docs
        open="a3f1-0001"
        known={[{ ...known[0], tags: ["legal"] }]}
        onKept={vi.fn()}
        onError={vi.fn()}
      />,
    );
    const editor = await screen.findByLabelText("editor");

    fireEvent.change(editor, { target: { value: "#legal arriba y otra vez #legal" } });

    const chip = await screen.findByTitle("2 times in this document");
    expect(chip.textContent).toBe("#legal2");
  });
});

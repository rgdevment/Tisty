import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useEffect } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { DOC } from "../markdown";
import { counted, worded } from "../ui/Beside";
import Docs from "../ui/Docs";

const store = vi.hoisted(() => ({
  ran: [] as string[],
  went: [] as string[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    switch (cmd) {
      case "doc_read":
        return Promise.resolve("# Compras\n\nleche");
      case "doc_facts":
        return Promise.resolve({ made: 1772668800, wrote: 1772755200, bytes: 8400 });
      default:
        return Promise.resolve(null);
    }
  },
}));

vi.mock("@react-pdf/renderer", () => ({
  pdf: () => ({ toBlob: async () => new Blob(["%PDF"], { type: "application/pdf" }) }),
  Document: () => null,
  Page: () => null,
  Text: () => null,
  View: () => null,
  Image: () => null,
  Link: () => null,
  Font: { registerHyphenationCallback: () => {} },
  StyleSheet: { create: (one: unknown) => one },
}));

vi.mock("../ui/Editor", () => ({
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
  { id: "01F", file: "a3f1-0001", title: "Compras", folder: null, archived: false },
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
    widen(1500);
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

  it("keeps what shapes a paragraph apart from what inserts something new", async () => {
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    expect(await screen.findByText("Format")).toBeTruthy();
    expect(await screen.findByText("Insert")).toBeTruthy();
  });

  it("runs the block the editor gave it", async () => {
    show();
    await userEvent.click(await screen.findByRole("button", { name: /Code block/ }));

    expect(store.ran).toEqual(["code"]);
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
    widen(1500);
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    expect(roomOf()?.style.paddingRight).toBe("344px");
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

    expect(sheetOf()?.style.maxWidth).toBe("820px");
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

  const named = () =>
    screen
      .getAllByRole("button")
      .map((one) => one.textContent?.trim())
      .filter((one) => ["Preview", "Export", "Copy", "Save a copy"].includes(one ?? ""));

  it("puts them in the order they are reached for", async () => {
    widen(1500);
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    expect(named()).toEqual(["Preview", "Export", "Copy", "Save a copy"]);
  });

  it("keeps the two trades apart, so no verb has to mean two things", async () => {
    widen(1500);
    show();
    await screen.findByRole("complementary", { name: "About this document" });

    expect(screen.getByText("PDF")).toBeTruthy();
    expect(screen.getByText("Markdown")).toBeTruthy();
  });

  it("says Close, not the name of another panel", async () => {
    widen(1500);
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Preview" }));

    expect(await screen.findByRole("button", { name: "Close" })).toBeTruthy();
  });

  it("lets the preview go when the section is left, instead of leaking it", async () => {
    const freed: string[] = [];
    URL.revokeObjectURL = (one: string) => {
      freed.push(one);
    };
    widen(1500);
    const view = show();
    await userEvent.click(await screen.findByRole("button", { name: "Preview" }));
    expect(screen.queryByRole("button", { name: "Close" })).toBeTruthy();

    view.unmount();

    expect(freed).toContain("blob:probe");
  });

  it("shuts the preview when told to", async () => {
    widen(1500);
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Preview" }));
    await userEvent.click(await screen.findByRole("button", { name: "Close" }));

    expect(screen.queryByRole("button", { name: "Close" })).toBeNull();
  });
});

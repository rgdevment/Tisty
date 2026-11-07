import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Showing } from "../core";
import { t } from "../locales";
import { natural } from "../ui/viewer/Stage";
import Viewing, { SIDE_FROM } from "../Viewing";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  shown: null as unknown,
  facts: null as unknown,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "view_shown") return Promise.resolve(ipc.shown);
    if (cmd === "view_facts") return Promise.resolve(ipc.facts);
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string, scheme: string) => `http://${scheme}.localhost/${path}`,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
  emit: () => Promise.resolve(),
  emitTo: () => Promise.resolve(),
}));

vi.mock("../ui/printing", () => ({
  pdfOf: (
    _: string[],
    sheet: { stamp?: { credited?: boolean }; turns?: string[]; landscape?: boolean },
  ) => {
    ipc.calls.push({
      cmd: "pdf",
      args: {
        credited: sheet.stamp?.credited ?? null,
        turns: sheet.turns ?? null,
        landscape: sheet.landscape ?? null,
      },
    });
    return Promise.resolve(new Blob(["%PDF"]));
  },
  STAMPING: { covered: false, credited: false },
  stampOf: (_: string, __: string, chosen: { credited: boolean }) =>
    Promise.resolve({ credited: chosen.credited }),
  factsCard: () => Promise.resolve(undefined),
  seenIn: () => ({ turns: [], opened: [] }),
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText: () => Promise.resolve() }));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: () => Promise.resolve("C:\notas.pdf"),
  ask: () => Promise.resolve(true),
  message: () => Promise.resolve(),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    label: "view-0",
    show: () => Promise.resolve(),
    minimize: () => Promise.resolve(),
    toggleMaximize: () => Promise.resolve(),
    close: () => Promise.resolve(),
  }),
}));

URL.createObjectURL = () => "blob:pdf";
URL.revokeObjectURL = () => {};

const pressed = (key: string, more: Partial<KeyboardEventInit> = {}) =>
  fireEvent.keyDown(window, { key, ctrlKey: true, ...more });

const picked = async (label: string) => {
  await userEvent.click(screen.getByRole("button", { name: t("viewMore") }));
  await userEvent.click(screen.getByRole("menuitem", { name: new RegExp(label) }));
};

const SEEN = `Proyectos · ${t("viewOnlyReading")}`;

const tisty: Showing = {
  kind: "doc",
  paper: { id: "D1", file: "f1", title: "Notas", place: "Proyectos" },
  body: "# Notas\n\nTexto.\n\n## Más",
};

beforeEach(() => {
  ipc.calls = [];
  ipc.shown = tisty;
  ipc.facts = null;
  window.innerWidth = SIDE_FROM - 1;
  localStorage.clear();
});

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

describe("the viewer", () => {
  it("shows a document, read only, with its contents and page a press away", async () => {
    render(<Viewing />);

    expect(await screen.findByText(SEEN)).toBeTruthy();
    await userEvent.click(await screen.findByRole("button", { name: t("viewIndex") }));
    expect(await screen.findByText(t("outline"))).toBeTruthy();
    expect(await screen.findByRole("group", { name: t("leafIs") })).toBeTruthy();
    expect(screen.getByRole("button", { name: t("viewEditIn") })).toBeTruthy();
  });

  it("shows the document as Markdown with its lines numbered", async () => {
    render(<Viewing />);
    await screen.findByText(SEEN);
    await userEvent.click(screen.getByRole("button", { name: t("viewIndex") }));

    await userEvent.click(await screen.findByRole("button", { name: t("viewSource") }));

    expect(screen.getByText("# Notas")).toBeTruthy();
    expect(screen.getByRole("button", { name: t("viewCopy") })).toBeTruthy();
  });

  it("draws a widget on a stage with its widths to try", async () => {
    ipc.shown = { kind: "widget", source: "<p>hola</p>", from: null } satisfies Showing;
    render(<Viewing />);

    expect(await screen.findByRole("button", { name: t("viewPhone") })).toBeTruthy();
    expect(screen.getByText(t("viewSealed"))).toBeTruthy();
    await waitFor(() => expect(document.querySelector("iframe.lit-widget")).not.toBeNull());
  });
});

describe("a widget's own width", () => {
  it("is the reach of its content when it stops short of the frame", () => {
    expect(natural({ left: 48, right: 530, view: 1200 })).toBe(578);
  });

  it("is nothing of its own when the content fills the frame", () => {
    expect(natural({ left: 24, right: 1176, view: 1180 })).toBeNull();
  });
});

describe("reading in the viewer", () => {
  it("previews a Tisty document as a PDF with no credit by default, adds it, saves it and closes on Esc", async () => {
    ipc.shown = tisty;
    render(<Viewing />);
    await screen.findByText(SEEN);

    pressed("p");
    expect(await screen.findByTitle(t("seePdf"))).toBeTruthy();
    expect(screen.getByText(t("seePdf")).parentElement?.className).toContain("pr-[7.5rem]");
    await userEvent.click(screen.getByRole("checkbox", { name: t("pdfCreditTick") }));
    await waitFor(() =>
      expect(asked("pdf").map((one) => one.args.credited)).toEqual([false, true]),
    );
    expect(asked("pdf")[0].args.turns).toEqual([]);
    await userEvent.click(screen.getByRole("button", { name: t("toPdf") }));
    await waitFor(() => expect(asked("keep_file")).toHaveLength(1));
    fireEvent.keyDown(window, { key: "Escape" });

    expect(screen.queryByTitle(t("seePdf"))).toBeNull();
  });

  it("opens with the contents folded in a narrow window, shows and hides them, sizes the page, and reads the source by keys", async () => {
    render(<Viewing />);
    await screen.findByText(SEEN);

    expect(screen.queryByText(t("outline"))).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: t("viewIndex") }));
    expect(await screen.findByText(t("outline"))).toBeTruthy();
    pressed("\\");
    expect(screen.queryByText(t("outline"))).toBeNull();
    pressed("\\");
    const letter = await screen.findByRole<HTMLButtonElement>("button", { name: t("leafLetter") });
    expect(letter.disabled).toBe(true);
    await userEvent.click(screen.getByRole("button", { name: t("sheetsShown") }));
    await userEvent.click(screen.getByRole("button", { name: t("leafLetter") }));
    expect(JSON.parse(localStorage.getItem("tisty.viewer") ?? "{}")).toMatchObject({
      leaf: "letter",
    });
    await userEvent.click(screen.getByRole("button", { name: t("sheetsShown") }));
    pressed("M", { shiftKey: true });
    expect(await screen.findByRole("button", { name: t("viewCopy") })).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("viewCopy") }));
    expect(await screen.findByRole("button", { name: t("viewCopied") })).toBeTruthy();
  });

  it("shows sheets turned across only on screen, and the PDF follows them", async () => {
    window.innerWidth = SIDE_FROM;
    render(<Viewing />);
    await screen.findByText(SEEN);

    const sheets = () => screen.getByRole("button", { name: t("sheetsShown") });
    // The column draws its rows once the document's facts arrive, which a slow run reaches after the title.
    expect(
      (await screen.findByRole("button", { name: t("sheetsShown") })).getAttribute("aria-pressed"),
    ).toBe("false");
    await userEvent.click(sheets());
    await userEvent.click(await screen.findByRole("button", { name: t("sheetsAcross") }));
    await waitFor(() => expect(sheets().getAttribute("aria-pressed")).toBe("true"));
    pressed("p");
    await waitFor(() => expect(asked("pdf").map((one) => one.args.landscape)).toContain(true));
  });

  it("opens with the contents beside the page only in a window wide enough for both", async () => {
    const was = window.innerWidth;
    window.innerWidth = SIDE_FROM;
    render(<Viewing />);
    await screen.findByText(SEEN);
    window.innerWidth = was;

    expect(await screen.findByText(t("outline"))).toBeTruthy();
  });

  it("opens folded one pixel short of that width", async () => {
    const was = window.innerWidth;
    window.innerWidth = SIDE_FROM - 1;
    render(<Viewing />);
    await screen.findByText(SEEN);
    window.innerWidth = was;

    expect(screen.queryByText(t("outline"))).toBeNull();
  });

  it("tells who wrote a Tisty document and how big it is beside its contents", async () => {
    ipc.shown = tisty;
    ipc.facts = {
      made: 1_790_000_000,
      wrote: 1_790_000_600,
      bytes: 12_700,
      pages: 2,
      author: "rgdevment",
      editor: null,
      born: null,
    };
    render(<Viewing />);
    await screen.findByText(SEEN);
    await userEvent.click(screen.getByRole("button", { name: t("viewIndex") }));

    expect(await screen.findByText("rgdevment")).toBeTruthy();
    expect(screen.getByText(t("besideDetails"))).toBeTruthy();
    expect(screen.getByText(t("paperWeighs"))).toBeTruthy();
    expect(ipc.calls.filter((one) => one.cmd === "view_facts")).toHaveLength(1);
  });

  it("finds in the text from the menu, and lets go of the search on Esc", async () => {
    render(<Viewing />);
    await screen.findByText(SEEN);

    await picked(t("viewFind"));
    const field = screen.getByPlaceholderText(t("viewFind"));
    await userEvent.type(field, "Notas{Enter}");
    fireEvent.keyDown(field, { key: "Escape" });

    expect(screen.queryByPlaceholderText(t("viewFind"))).toBeNull();
  });

  it("goes back to the main window to edit a Tisty document", async () => {
    ipc.shown = tisty;
    render(<Viewing />);
    await screen.findByText(SEEN);
    await userEvent.click(screen.getByRole("button", { name: t("viewIndex") }));

    await userEvent.click(await screen.findByRole("button", { name: t("viewEditIn") }));

    expect(asked("view_go")[0].args).toEqual({ file: "f1" });
  });

  it("shows a widget's code over the stage and closes it on Esc", async () => {
    ipc.shown = { kind: "widget", source: "<p>hola</p>", from: null } satisfies Showing;
    render(<Viewing />);
    await screen.findByRole("button", { name: t("viewPhone") });

    await userEvent.click(screen.getByRole("button", { name: t("viewTablet") }));
    const drawnFirst = document.querySelector("iframe.lit-widget");
    await userEvent.click(screen.getByRole("button", { name: t("viewReload") }));
    await waitFor(() => expect(document.querySelector("iframe.lit-widget")).not.toBe(drawnFirst));
    await picked(t("viewDark"));
    await picked(t("viewCode"));
    expect(screen.getByText("<p>hola</p>")).toBeTruthy();
    fireEvent.keyDown(window, { key: "Escape" });

    expect(screen.queryByText("<p>hola</p>")).toBeNull();
    expect(screen.getByText(new RegExp(`${t("viewTablet")} · 820 px`))).toBeTruthy();
  });
});

describe("the reader comes only to a window with words to read", () => {
  const READER = "../ui/viewer/Reader";

  const counted = () => {
    const loaded: string[] = [];
    vi.doMock(READER, () => {
      loaded.push("Reader");
      return { default: () => "the reader" };
    });
    return loaded;
  };

  const viewed = async (shown: Showing) => {
    ipc.shown = shown;
    const { default: Standing } = await import("../Standing");
    const { default: Fresh } = await import("../Viewing");
    render(
      <Standing>
        <Fresh />
      </Standing>,
    );
  };

  beforeEach(() => {
    vi.resetModules();
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.doUnmock(READER);
  });

  it("is loaded for a document with words", async () => {
    const loaded = counted();

    await viewed(tisty);

    expect(await screen.findByText("the reader")).toBeTruthy();
    expect(loaded).toEqual(["Reader"]);
  });

  it("is not loaded for a widget", async () => {
    const loaded = counted();

    await viewed({ kind: "widget", source: "<p>hola</p>", from: null });

    expect(await screen.findByRole("button", { name: t("viewPhone") })).toBeTruthy();
    expect(loaded).toEqual([]);
  });

  it("keeps the window saying it is opening until it arrives", async () => {
    const arriving: ((reader: { default: () => string }) => void)[] = [];
    vi.doMock(
      READER,
      () =>
        new Promise<{ default: () => string }>((resolve) => {
          arriving.push(resolve);
        }),
    );

    await viewed(tisty);

    expect(await screen.findByText(SEEN)).toBeTruthy();
    expect(screen.getByText(t("viewOpening"))).toBeTruthy();

    await waitFor(() => expect(arriving).toHaveLength(1));
    arriving[0]({ default: () => "the reader" });

    expect(await screen.findByText("the reader")).toBeTruthy();
    expect(screen.queryByText(t("viewOpening"))).toBeNull();
  });

  it("lets the window look again while it is still on its way", async () => {
    vi.doMock(READER, () => new Promise(() => {}));
    const focused = vi.spyOn(document, "hasFocus");

    await viewed(tisty);
    await waitFor(() => expect(asked("view_shown")).toHaveLength(1));
    expect(await screen.findByText(t("viewOpening"))).toBeTruthy();

    focused.mockReturnValue(false);
    fireEvent.blur(window);
    await new Promise((done) => setTimeout(done, 0));
    focused.mockReturnValue(true);
    fireEvent.focus(window);

    await waitFor(() => expect(asked("view_shown")).toHaveLength(2));
    focused.mockRestore();
  });

  it("says so and offers to draw the window again when it cannot be loaded", async () => {
    vi.doMock(READER, () => {
      throw new Error("the reader cannot be loaded");
    });

    await viewed(tisty);

    expect(await screen.findByText(t("windowFell"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("windowFellAgain") })).toBeTruthy();
  });
});

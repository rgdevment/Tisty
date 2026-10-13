import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Showing } from "../core";
import { t } from "../locales";
import { natural } from "../ui/viewer/Stage";
import Viewing, { SIDE_FROM } from "../Viewing";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  shown: null as unknown,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "view_shown") return Promise.resolve(ipc.shown);
    if (cmd === "view_import")
      return Promise.resolve({ id: "D1", file: "f1", title: "Notas", place: "" });
    if (cmd === "widget_lend") return Promise.resolve("w1");
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
  pdfOfText: (_: string, __: string, signed?: string) => {
    ipc.calls.push({ cmd: "pdf", args: { signed: signed ?? null } });
    return Promise.resolve(new Blob(["%PDF"]));
  },
  signature: () => Promise.resolve("Firmado"),
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText: () => Promise.resolve() }));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: () => Promise.resolve("C:\notas.pdf"),
  ask: () => Promise.resolve(true),
  message: () => Promise.resolve(),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ revealItemInDir: () => Promise.resolve() }));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    label: "view-0",
    minimize: () => Promise.resolve(),
    toggleMaximize: () => Promise.resolve(),
    close: () => Promise.resolve(),
  }),
}));

const file: Showing = {
  kind: "file",
  name: "notas.md",
  place: "Descargas",
  path: "C:\\Descargas\\notas.md",
  body: "# Notas\n\nUn texto.\n\n## Más\n",
};

URL.createObjectURL = () => "blob:pdf";
URL.revokeObjectURL = () => {};

const pressed = (key: string, more: Partial<KeyboardEventInit> = {}) =>
  fireEvent.keyDown(window, { key, ctrlKey: true, ...more });

const picked = async (label: string) => {
  await userEvent.click(screen.getByRole("button", { name: t("viewMore") }));
  await userEvent.click(screen.getByRole("menuitem", { name: new RegExp(label) }));
};

const tisty: Showing = {
  kind: "doc",
  paper: { id: "D1", file: "f1", title: "Notas", place: "Proyectos" },
  body: "# Notas\n\nTexto.\n\n## Más",
};

beforeEach(() => {
  ipc.calls = [];
  ipc.shown = file;
  localStorage.clear();
});

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

describe("the viewer", () => {
  it("shows a file from outside, read only, with its contents and page open", async () => {
    render(<Viewing />);

    expect(await screen.findByText("notas.md")).toBeTruthy();
    expect(screen.getByText(`Descargas · ${t("viewOnlyReading")}`)).toBeTruthy();
    expect(screen.getByText(t("viewInFile"))).toBeTruthy();
    expect(screen.getByText(t("viewPage"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("viewEditIn") })).toBeTruthy();
  });

  it("shows the file as Markdown with its lines numbered", async () => {
    render(<Viewing />);
    await screen.findByText("notas.md");

    await userEvent.click(screen.getByRole("button", { name: t("viewSource") }));

    expect(screen.getByText("# Notas")).toBeTruthy();
    expect(screen.getByRole("button", { name: t("viewCopy") })).toBeTruthy();
  });

  it("brings a file into Tisty to edit it, and becomes that document", async () => {
    render(<Viewing />);
    await screen.findByText("notas.md");
    ipc.shown = {
      kind: "doc",
      paper: { id: "D1", file: "f1", title: "Notas", place: "" },
      body: "# Notas",
    } satisfies Showing;

    await userEvent.click(screen.getByRole("button", { name: t("viewEditIn") }));

    expect(asked("view_import")[0].args).toEqual({ open: true });
    await waitFor(() => expect(screen.getByText(new RegExp(t("viewNowIn")))).toBeTruthy());
    expect(screen.getByText(t("viewInDoc"))).toBeTruthy();
  });

  it("says plainly why a file could not be opened", async () => {
    ipc.shown = {
      kind: "missed",
      name: "grande.md",
      place: "Descargas",
      path: "C:\\Descargas\\grande.md",
      refusal: { code: "documentTooBig", name: "500 KB" },
    } satisfies Showing;
    render(<Viewing />);

    expect(await screen.findByText(t("viewBigTitle").replace("{name}", "grande.md"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("viewRetry") })).toBeTruthy();
  });

  it("draws a widget on a stage with its widths to try", async () => {
    ipc.shown = { kind: "widget", lent: { source: "<p>hola</p>" }, from: null } satisfies Showing;
    render(<Viewing />);

    expect(await screen.findByRole("button", { name: t("viewPhone") })).toBeTruthy();
    expect(screen.getByText(t("viewSealed"))).toBeTruthy();
    await waitFor(() => expect(asked("widget_lend")).toHaveLength(1));
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
  it("previews a Tisty document as a PDF, signs it, saves it and closes on Esc", async () => {
    ipc.shown = tisty;
    render(<Viewing />);
    await screen.findByText(`Proyectos · ${t("viewOnlyReading")}`);

    pressed("p");
    expect(await screen.findByTitle(t("seePdf"))).toBeTruthy();
    expect(screen.getByText(t("seePdf")).parentElement?.className).toContain("pr-[7.5rem]");
    await userEvent.click(screen.getByRole("checkbox", { name: t("viewWithSign") }));
    await waitFor(() =>
      expect(asked("pdf").map((one) => one.args.signed)).toEqual([null, "Firmado"]),
    );
    await userEvent.click(screen.getByRole("button", { name: t("toPdf") }));
    await waitFor(() => expect(asked("keep_pdf")).toHaveLength(1));
    fireEvent.keyDown(window, { key: "Escape" });

    expect(screen.queryByTitle(t("seePdf"))).toBeNull();
  });

  it("opens with the contents folded, shows and hides them, sizes the page, and reads the source by keys", async () => {
    render(<Viewing />);
    await screen.findByText("notas.md");

    expect(screen.queryByText(t("viewPage"))).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: t("viewIndex") }));
    expect(await screen.findByText(t("viewPage"))).toBeTruthy();
    pressed("\\");
    expect(screen.queryByText(t("viewPage"))).toBeNull();
    pressed("\\");
    await userEvent.click(await screen.findByRole("button", { name: t("leafLetter") }));
    expect(JSON.parse(localStorage.getItem("tisty.viewer") ?? "{}")).toEqual({ leaf: "letter" });
    pressed("M", { shiftKey: true });
    expect(await screen.findByRole("button", { name: t("viewCopy") })).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("viewCopy") }));
    expect(await screen.findByRole("button", { name: t("viewCopied") })).toBeTruthy();
  });

  it("opens with the contents beside the page only in a window wide enough for both", async () => {
    const was = window.innerWidth;
    window.innerWidth = SIDE_FROM;
    render(<Viewing />);
    await screen.findByText("notas.md");
    window.innerWidth = was;

    expect(screen.getByText(t("viewPage"))).toBeTruthy();
  });

  it("opens folded one pixel short of that width", async () => {
    const was = window.innerWidth;
    window.innerWidth = SIDE_FROM - 1;
    render(<Viewing />);
    await screen.findByText("notas.md");
    window.innerWidth = was;

    expect(screen.queryByText(t("viewPage"))).toBeNull();
  });

  it("finds in the text from the menu, and lets go of the search on Esc", async () => {
    render(<Viewing />);
    await screen.findByText("notas.md");

    await picked(t("viewFind"));
    const field = screen.getByPlaceholderText(t("viewFind"));
    await userEvent.type(field, "Notas{Enter}");
    fireEvent.keyDown(field, { key: "Escape" });

    expect(screen.queryByPlaceholderText(t("viewFind"))).toBeNull();
  });

  it("brings a file in without opening it, and says where the original stays", async () => {
    render(<Viewing />);
    await screen.findByText("notas.md");

    await picked(t("viewImportQuiet"));

    expect(asked("view_import")[0].args).toEqual({ open: false });
    expect(await screen.findByText(new RegExp(t("viewSavedIn")))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("viewGotIt") }));
    expect(screen.queryByText(new RegExp(t("viewSavedIn")))).toBeNull();
  });

  it("offers to write an empty file in Tisty", async () => {
    ipc.shown = { ...file, body: "  " };
    render(<Viewing />);

    expect(await screen.findByText(t("viewEmptyTitle"))).toBeTruthy();
    const offered = screen.getAllByRole("button", { name: t("viewEditIn") });
    await userEvent.click(offered[offered.length - 1]);
    await waitFor(() => expect(asked("view_import").length).toBeGreaterThan(0));
  });

  it("goes back to the main window to edit a Tisty document", async () => {
    ipc.shown = tisty;
    render(<Viewing />);
    await screen.findByText(`Proyectos · ${t("viewOnlyReading")}`);

    await userEvent.click(screen.getByRole("button", { name: t("viewEditIn") }));

    expect(asked("view_go")[0].args).toEqual({ file: "f1" });
    expect(asked("view_import")).toHaveLength(0);
  });

  it("shows a widget's code over the stage and closes it on Esc", async () => {
    ipc.shown = { kind: "widget", lent: { source: "<p>hola</p>" }, from: null } satisfies Showing;
    render(<Viewing />);
    await screen.findByRole("button", { name: t("viewPhone") });

    await userEvent.click(screen.getByRole("button", { name: t("viewTablet") }));
    await userEvent.click(screen.getByRole("button", { name: t("viewReload") }));
    await waitFor(() => expect(asked("widget_lend").length).toBeGreaterThan(1));
    await picked(t("viewDark"));
    await picked(t("viewCode"));
    expect(screen.getByText("<p>hola</p>")).toBeTruthy();
    fireEvent.keyDown(window, { key: "Escape" });

    expect(screen.queryByText("<p>hola</p>")).toBeNull();
    expect(screen.getByText(new RegExp(`${t("viewTablet")} · 820 px`))).toBeTruthy();
  });
});

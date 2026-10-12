import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Showing } from "../core";
import { t } from "../locales";
import { natural } from "../ui/viewer/Stage";
import Viewing from "../Viewing";

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

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: () => Promise.resolve(null),
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

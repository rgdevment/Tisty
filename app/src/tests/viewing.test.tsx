import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Showing } from "../core";
import { fill, t } from "../locales";
import type { Held } from "../pieceWindow";
import { natural } from "../ui/viewer/Stage";
import Viewing from "../Viewing";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  shown: null as unknown,
  refused: null as unknown,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "view_shown") return Promise.resolve(ipc.shown);
    if (cmd === "piece_save" && ipc.refused) return Promise.reject(ipc.refused);
    if (cmd === "doc_drawn")
      return Promise.resolve(new TextEncoder().encode("<p>drawn</p>").buffer);
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string, scheme: string) => `http://${scheme}.localhost/${path}`,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
  emit: () => Promise.resolve(),
  emitTo: () => Promise.resolve(),
}));

const closing: { heard: (() => Promise<void>) | null } = { heard: null };

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    label: "view-0",
    show: () => Promise.resolve(),
    minimize: () => Promise.resolve(),
    toggleMaximize: () => Promise.resolve(),
    close: () => Promise.resolve(),
    onCloseRequested: (heard: () => Promise<void>) => {
      closing.heard = heard;
      return Promise.resolve(() => {});
    },
  }),
}));

const picked = async (label: string) => {
  await userEvent.click(screen.getByRole("button", { name: t("viewMore") }));
  await userEvent.click(screen.getByRole("menuitem", { name: new RegExp(label) }));
};

const widget: Showing = {
  kind: "piece",
  held: "widget",
  source: "<p>hola</p>",
  from: null,
  code: false,
  writable: false,
};

beforeEach(() => {
  ipc.calls = [];
  ipc.shown = widget;
  ipc.refused = null;
});

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

describe("the viewer", () => {
  it("draws a widget on a stage with its widths to try", async () => {
    render(<Viewing />);

    expect(await screen.findByRole("button", { name: t("viewPhone") })).toBeTruthy();
    expect(screen.getByText(t("viewSealed"))).toBeTruthy();
    await waitFor(() => expect(document.querySelector("iframe.lit-widget")).not.toBeNull());
  });

  it("shows a widget's code over the stage and closes it on Esc", async () => {
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

describe("a widget's own width", () => {
  it("is the reach of its content when it stops short of the frame", () => {
    expect(natural({ left: 48, right: 530, view: 1200 })).toBe(578);
  });

  it("is nothing of its own when the content fills the frame", () => {
    expect(natural({ left: 24, right: 1176, view: 1180 })).toBeNull();
  });
});

describe("a widget's code in its own window", () => {
  const opened = (writable: boolean) => {
    ipc.shown = {
      kind: "piece",
      held: "widget",
      source: "<b>1</b>\n",
      from: { id: "", file: "f1", title: "Notas", place: "" },
      code: true,
      writable,
    } satisfies Showing;
    render(<Viewing />);
  };

  it("opens on the code beside the drawing and saves what is written after a pause", async () => {
    opened(true);
    const field = (await screen.findByRole("textbox", {
      name: t("viewCode"),
    })) as HTMLTextAreaElement;
    expect(field.value).toBe("<b>1</b>\n");
    expect(field.readOnly).toBe(false);
    fireEvent.change(field, { target: { value: "<b>2</b>\n" } });
    expect(asked("piece_save")).toHaveLength(0);
    await act(() => vi.advanceTimersByTimeAsync(800));
    expect(asked("piece_save").map((one) => one.args)).toEqual([{ now: "<b>2</b>\n" }]);
  });

  it("saves nothing while the code is left empty, as an empty widget is no widget", async () => {
    opened(true);
    const field = await screen.findByRole("textbox", { name: t("viewCode") });
    fireEvent.change(field, { target: { value: "  " } });
    await act(() => vi.advanceTimersByTimeAsync(800));
    expect(asked("piece_save")).toHaveLength(0);
  });

  it("only shows the code where the document cannot be written", async () => {
    opened(false);
    const field = (await screen.findByRole("textbox", {
      name: t("viewCode"),
    })) as HTMLTextAreaElement;
    expect(field.readOnly).toBe(true);
  });

  it("says so when the widget changed in the document since it was opened", async () => {
    ipc.refused = { code: "pieceMoved", name: "Notas" };
    opened(true);
    const field = await screen.findByRole("textbox", { name: t("viewCode") });
    fireEvent.change(field, { target: { value: "<b>3</b>" } });
    await act(() => vi.advanceTimersByTimeAsync(800));
    expect(await screen.findByRole("alert")).toBeTruthy();
  });

  it("brings back what was written when the code is closed and opened again", async () => {
    opened(true);
    const field = await screen.findByRole("textbox", { name: t("viewCode") });
    fireEvent.change(field, { target: { value: "<b>4</b>\n" } });
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("textbox", { name: t("viewCode") })).toBeNull();
    await picked(t("viewCode"));
    const again = (await screen.findByRole("textbox", {
      name: t("viewCode"),
    })) as HTMLTextAreaElement;
    expect(again.value).toBe("<b>4</b>\n");
    await act(() => vi.advanceTimersByTimeAsync(800));
    expect(asked("piece_save").map((one) => one.args)).toEqual([{ now: "<b>4</b>\n" }]);
  });

  it("saves what is still waiting when the window is closed", async () => {
    opened(true);
    const field = await screen.findByRole("textbox", { name: t("viewCode") });
    fireEvent.change(field, { target: { value: "<b>5</b>\n" } });
    await act(() => closing.heard?.() ?? Promise.resolve());
    expect(asked("piece_save").map((one) => one.args)).toEqual([{ now: "<b>5</b>\n" }]);
    await act(() => vi.advanceTimersByTimeAsync(800));
    expect(asked("piece_save")).toHaveLength(1);
  });

  it("indents with Tab instead of leaving the code", async () => {
    opened(true);
    const field = (await screen.findByRole("textbox", {
      name: t("viewCode"),
    })) as HTMLTextAreaElement;
    field.setSelectionRange(0, 0);
    fireEvent.keyDown(field, { key: "Tab" });
    expect(field.value).toBe("  <b>1</b>\n");
    await act(() => vi.advanceTimersByTimeAsync(800));
    expect(asked("piece_save").map((one) => one.args)).toEqual([{ now: "  <b>1</b>\n" }]);
  });
});

describe("a formula, a table or the properties in their own window", () => {
  const opened = (held: Held, source: string) => {
    ipc.shown = {
      kind: "piece",
      held,
      source,
      from: { id: "", file: "f1", title: "Notas", place: "" },
      code: true,
      writable: true,
    } satisfies Showing;
    render(<Viewing />);
  };
  const drawn = () => asked("doc_drawn").map((one) => one.args.body);

  it("names a formula, draws it with the core beside its code and offers no widths", async () => {
    opened("formula", "a^2\n");
    const field = (await screen.findByRole("textbox", {
      name: t("viewCode"),
    })) as HTMLTextAreaElement;
    expect(field.value).toBe("a^2\n");
    expect(screen.getByText(fill("viewFormulaOf", "Notas"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("viewPhone") })).toBeNull();
    await waitFor(() => expect(drawn()).toContain("```math\na^2\n```\n"));
    fireEvent.change(field, { target: { value: "b^2" } });
    await act(() => vi.advanceTimersByTimeAsync(800));
    expect(asked("piece_save").map((one) => one.args)).toEqual([{ now: "b^2" }]);
    await waitFor(() => expect(drawn()).toContain("```math\nb^2\n```\n"));
  });

  it.each([
    ["inlineFormula", "x^2", "$$x^2$$\n", "viewFormulaOf"],
    ["table", "| a |\n|---|\n", "| a |\n|---|\n", "viewTableOf"],
    ["front", "title: Plan\n", "---\ntitle: Plan\n---\n", "viewFrontOf"],
  ] as const)("draws %s as the document would", async (held, source, body, named) => {
    opened(held, source);
    expect(await screen.findByText(fill(named, "Notas"))).toBeTruthy();
    await waitFor(() => expect(drawn()).toContain(body));
  });
});

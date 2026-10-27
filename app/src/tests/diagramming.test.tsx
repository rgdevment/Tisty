import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Showing } from "../core";
import { fill, t } from "../locales";
import Viewing from "../Viewing";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  shown: null as unknown,
  refusal: null as unknown,
  asked: [] as string[],
  closing: null as null | ((e: { preventDefault: () => void }) => Promise<void>),
  leave: true as boolean | "fails",
  quitting: null as null | (() => Promise<void>),
  now: "mindmap\n  root((Otra))\n",
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "view_shown") return Promise.resolve(ipc.shown);
    if (cmd === "diagram_save" && ipc.refusal) return Promise.reject(ipc.refusal);
    if (cmd === "diagram_again") return Promise.resolve(ipc.now);
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
  emit: () => Promise.resolve(),
  emitTo: () => Promise.resolve(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: (said: string) => {
    ipc.asked.push(said);
    return ipc.leave === "fails"
      ? Promise.reject(new Error("no dialog"))
      : Promise.resolve(ipc.leave);
  },
  save: () => Promise.resolve(null),
  message: () => Promise.resolve(),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ revealItemInDir: () => Promise.resolve() }));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    label: "view-0",
    minimize: () => Promise.resolve(),
    toggleMaximize: () => Promise.resolve(),
    close: () => Promise.resolve(),
    onCloseRequested: (heard: typeof ipc.closing) => {
      ipc.closing = heard;
      return Promise.resolve(() => {});
    },
    listen: (event: string, heard: () => Promise<void>) => {
      if (event === "diagramLeaving") ipc.quitting = heard;
      return Promise.resolve(() => {});
    },
  }),
}));

vi.mock("../ui/diagrams", () => ({
  drawing: (source: string) =>
    Promise.resolve(
      source.includes("broken")
        ? null
        : `<svg viewBox="0 0 200 100"><text>${source.split("\n")[0]}</text></svg>`,
    ),
}));

const shown: Showing = {
  kind: "diagram",
  paper: { id: "D1", file: "f1", title: "Tisty Share", place: "Proyectos" },
  source: "mindmap\n  root((Share))\n",
};

// A kind the window draws with Mermaid, not with its own canvas.
const flow: Showing = { ...shown, source: "graph TD\n  A-->B\n" };

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);
const lastOf = (cmd: string) => asked(cmd)[asked(cmd).length - 1];
const written = async () => (await screen.findByLabelText(t("diagramText"))) as HTMLTextAreaElement;
const saving = () => screen.getByRole("button", { name: fill("diagramSave", "Tisty Share") });

beforeEach(() => {
  vi.restoreAllMocks();
  ipc.calls = [];
  ipc.shown = shown;
  ipc.refusal = null;
  ipc.asked = [];
  ipc.closing = null;
  ipc.quitting = null;
  ipc.leave = true;
});

describe("the diagram window", () => {
  it("opens on the diagram's text, names its kind and the document it goes back to", async () => {
    render(<Viewing />);

    expect((await written()).value).toBe(shown.source);
    expect(screen.getByText(fill("diagramOf", "Tisty Share"))).toBeTruthy();
    expect(screen.getByText(`${t("diagramMindmap")} · Proyectos`)).toBeTruthy();
    expect(screen.getByText(t("diagramSaved"))).toBeTruthy();
    expect((saving() as HTMLButtonElement).disabled).toBe(true);
  });

  it("opens a diagram whose last line has no break as saved, not as changed", async () => {
    ipc.shown = { ...shown, source: "mindmap\n  root((Share))" };
    render(<Viewing />);

    expect((await written()).value).toBe("mindmap\n  root((Share))");
    expect(screen.getByText(t("diagramSaved"))).toBeTruthy();
    expect((saving() as HTMLButtonElement).disabled).toBe(true);
  });

  it("draws what the text says, live, and keeps the last drawing while it cannot", async () => {
    ipc.shown = flow;
    const { container } = render(<Viewing />);
    const text = await written();

    await waitFor(() => expect(container.querySelector(".diagram-drawn svg")).toBeTruthy());
    fireEvent.change(text, { target: { value: "graph broken" } });

    expect(await screen.findByText(t("diagramUnreadable"))).toBeTruthy();
    expect(container.querySelector(".diagram-drawn svg")?.textContent).toBe("graph TD");
  });

  it("saves the text it holds into the document, by the button or by Ctrl+S", async () => {
    render(<Viewing />);
    const text = await written();

    fireEvent.change(text, { target: { value: "mindmap\n  root((Share))\n    Cifrado" } });
    expect(screen.getByText(t("diagramUnsaved"))).toBeTruthy();
    await userEvent.click(saving());

    expect(asked("diagram_save")[0]?.args).toEqual({
      now: "mindmap\n  root((Share))\n    Cifrado",
    });
    expect(await screen.findByText(t("diagramSaved"))).toBeTruthy();

    fireEvent.change(text, { target: { value: "erDiagram" } });
    fireEvent.keyDown(window, { key: "s", ctrlKey: true });
    await waitFor(() => expect(asked("diagram_save")).toHaveLength(2));
    expect(screen.getByText(`${t("diagramSchema")} · Proyectos`)).toBeTruthy();
  });

  it("says plainly when the diagram changed in the document meanwhile, and keeps what was typed", async () => {
    ipc.refusal = { code: "diagramMoved", name: "Tisty Share" };
    render(<Viewing />);
    const text = await written();

    fireEvent.change(text, { target: { value: "mindmap\n  root((Otra))" } });
    await userEvent.click(saving());

    expect((await screen.findByRole("alert")).textContent).toBe(
      fill("diagramMoved", "Tisty Share"),
    );
    expect(text.value).toBe("mindmap\n  root((Otra))");
    expect(screen.getByText(t("diagramUnsaved"))).toBeTruthy();
  });

  it("asks before closing over unsaved work, and lets a saved window go", async () => {
    render(<Viewing />);
    const text = await written();
    const stopped = vi.fn();

    await ipc.closing?.({ preventDefault: stopped });
    expect(ipc.asked).toEqual([]);

    fireEvent.change(text, { target: { value: "mindmap\n  root((Otra))" } });
    ipc.leave = false;
    await ipc.closing?.({ preventDefault: stopped });

    expect(ipc.asked).toEqual([t("diagramLeave")]);
    expect(stopped).toHaveBeenCalledOnce();
  });

  it("zooms with its buttons and fits the drawing back", async () => {
    ipc.shown = flow;
    vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(800);
    vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(600);
    const { container } = render(<Viewing />);
    await waitFor(() => expect(container.querySelector(".diagram-drawn svg")).toBeTruthy());
    const shown = screen.getByText("100%");

    await userEvent.click(screen.getByRole("button", { name: t("diagramZoomIn") }));
    expect(shown.textContent).toBe("120%");
    await userEvent.click(screen.getByRole("button", { name: t("diagramZoomOut") }));
    await userEvent.click(screen.getByRole("button", { name: t("diagramZoomOut") }));
    expect(shown.textContent).toBe("83%");
    await userEvent.click(screen.getByRole("button", { name: t("diagramFit") }));
    expect(shown.textContent).toBe("100%");
  });

  it("moves the drawing by dragging or scrolling, and zooms under the pointer with Ctrl", async () => {
    ipc.shown = flow;
    vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(800);
    vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(600);
    const { container } = render(<Viewing />);
    await waitFor(() => expect(container.querySelector(".diagram-drawn svg")).toBeTruthy());
    const canvas = screen.getByRole("region", { name: t("diagramCanvas") });
    const drawn = () => (container.querySelector(".diagram-drawn") as HTMLElement).style.transform;

    expect(drawn()).toBe("translate(300px, 250px) scale(1)");
    fireEvent.pointerDown(canvas, { clientX: 10, clientY: 10, pointerId: 1 });
    fireEvent.pointerMove(canvas, { clientX: 40, clientY: 30, pointerId: 1, buttons: 1 });
    fireEvent.pointerUp(canvas, { pointerId: 1 });
    fireEvent.pointerMove(canvas, { clientX: 90, clientY: 90, pointerId: 1, buttons: 1 });
    expect(drawn()).toBe("translate(330px, 270px) scale(1)");

    // A drag the system cancels, or one released where the canvas never heard it, ends there.
    for (const ending of ["cancel", "release unheard"]) {
      fireEvent.pointerDown(canvas, { clientX: 0, clientY: 0, pointerId: 2 });
      if (ending === "cancel") fireEvent.pointerCancel(canvas, { pointerId: 2 });
      else fireEvent.pointerMove(canvas, { clientX: 5, clientY: 5, pointerId: 2, buttons: 0 });
      fireEvent.pointerMove(canvas, { clientX: 50, clientY: 50, pointerId: 2, buttons: 1 });
      expect(drawn(), ending).toBe("translate(330px, 270px) scale(1)");
    }

    fireEvent.wheel(canvas, { deltaX: 30, deltaY: 70 });
    expect(drawn()).toBe("translate(300px, 200px) scale(1)");
    fireEvent.wheel(canvas, { deltaY: -1, ctrlKey: true, clientX: 0, clientY: 0 });
    expect(drawn()).toBe("translate(360px, 240px) scale(1.2)");
  });

  it("after a refusal, takes the document's diagram or saves its own over it", async () => {
    ipc.refusal = { code: "diagramMoved", name: "Tisty Share" };
    render(<Viewing />);
    const text = await written();

    fireEvent.change(text, { target: { value: "mindmap\n  root((Mía))" } });
    await userEvent.click(saving());
    await userEvent.click(await screen.findByRole("button", { name: t("diagramTakeTheirs") }));

    await waitFor(() => expect(text.value).toBe(ipc.now));
    expect(screen.getByText(t("diagramSaved"))).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();

    fireEvent.change(text, { target: { value: "mindmap\n  root((Mía))" } });
    await userEvent.click(saving());
    ipc.refusal = null;
    await userEvent.click(await screen.findByRole("button", { name: t("diagramKeepMine") }));

    await waitFor(() => expect(screen.getByText(t("diagramSaved"))).toBeTruthy());
    expect(lastOf("diagram_save")?.args).toEqual({ now: "mindmap\n  root((Mía))" });
    expect(asked("diagram_again")).toHaveLength(2);
  });

  it("tells the app what is unsaved, and asks before the app quits over it", async () => {
    render(<Viewing />);
    const text = await written();

    fireEvent.change(text, { target: { value: "mindmap\n  root((Otra))" } });
    await waitFor(() => expect(lastOf("diagram_unsaved")?.args).toEqual({ unsaved: true }));

    ipc.leave = false;
    await ipc.quitting?.();
    expect(asked("diagram_leave")).toHaveLength(0);
    ipc.leave = true;
    await ipc.quitting?.();
    expect(ipc.asked).toEqual([t("diagramLeaveApp"), t("diagramLeaveApp")]);
    expect(asked("diagram_leave")).toHaveLength(1);
  });

  it("never traps the window when the dialog cannot open: it says so once, then lets go", async () => {
    render(<Viewing />);
    const text = await written();
    const stopped = vi.fn();

    fireEvent.change(text, { target: { value: "mindmap\n  root((Otra))" } });
    ipc.leave = "fails";
    await ipc.closing?.({ preventDefault: stopped });
    expect(stopped).toHaveBeenCalledOnce();
    expect(await screen.findByText(t("diagramUnasked"))).toBeTruthy();

    await ipc.closing?.({ preventDefault: stopped });
    expect(stopped).toHaveBeenCalledOnce();
  });
});

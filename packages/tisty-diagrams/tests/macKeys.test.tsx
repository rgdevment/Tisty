import { cleanup, fireEvent, render, renderHook, screen } from "@testing-library/react";
import type { KeyboardEvent } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Mind } from "../src/mindmap";

const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)";
const WINDOWS = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";
const MAP = "mindmap\n  root((Share))\n    Cifrado\n";

type Held = { readonly key: string; readonly ctrlKey?: boolean; readonly metaKey?: boolean };

const on = (agent: string) => {
  vi.resetModules();
  vi.spyOn(navigator, "userAgent", "get").mockReturnValue(agent);
};

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

const canvasOn = async (agent: string) => {
  on(agent);
  const { ReactFlowProvider } = await import("@xyflow/react");
  const { useCanvasHands } = await import("../src/useCanvasHands");
  const acts = { remove: null, undo: vi.fn(), redo: vi.fn(), duplicate: vi.fn() };
  const { result } = renderHook(() => useCanvasHands(acts), { wrapper: ReactFlowProvider });
  const press = (held: Held) =>
    result.current.keyed({
      ctrlKey: false,
      metaKey: false,
      shiftKey: false,
      ...held,
      target: document.body,
      preventDefault: () => {},
    } as unknown as KeyboardEvent);
  return { acts, press };
};

const mapOn = async (agent: string) => {
  on(agent);
  const { default: IdeaMap } = await import("../src/IdeaMap");
  const { readMind } = await import("../src/mindmap");
  const undo = vi.fn();
  const redo = vi.fn();
  render(
    <IdeaMap
      mind={readMind(MAP) as Mind}
      chosen={[]}
      fresh="Idea"
      undo={undo}
      redo={redo}
      onChoose={() => {}}
      onChange={() => {}}
    />,
  );
  const press = (held: Held) =>
    fireEvent.keyDown(screen.getByRole("button", { name: "Share" }), held);
  return { undo, redo, press };
};

describe("the canvas keys for what is chosen", () => {
  it("on a Mac answer Cmd and leave Control alone", async () => {
    const { acts, press } = await canvasOn(MAC);

    press({ key: "z", ctrlKey: true });
    press({ key: "y", ctrlKey: true });
    press({ key: "d", ctrlKey: true });
    expect(acts.undo).not.toHaveBeenCalled();
    expect(acts.redo).not.toHaveBeenCalled();
    expect(acts.duplicate).not.toHaveBeenCalled();

    press({ key: "z", metaKey: true });
    press({ key: "y", metaKey: true });
    press({ key: "d", metaKey: true });
    expect(acts.undo).toHaveBeenCalledOnce();
    expect(acts.redo).toHaveBeenCalledOnce();
    expect(acts.duplicate).toHaveBeenCalledOnce();
  });

  it("on Windows answer Control", async () => {
    const { acts, press } = await canvasOn(WINDOWS);

    press({ key: "z", ctrlKey: true });
    press({ key: "y", ctrlKey: true });
    press({ key: "d", ctrlKey: true });
    expect(acts.undo).toHaveBeenCalledOnce();
    expect(acts.redo).toHaveBeenCalledOnce();
    expect(acts.duplicate).toHaveBeenCalledOnce();
  });
});

describe("the Ideas map's undo and redo", () => {
  it("on a Mac answer Cmd and leave Control alone", async () => {
    const { undo, redo, press } = await mapOn(MAC);

    press({ key: "z", ctrlKey: true });
    press({ key: "y", ctrlKey: true });
    expect(undo).not.toHaveBeenCalled();
    expect(redo).not.toHaveBeenCalled();

    press({ key: "z", metaKey: true });
    press({ key: "y", metaKey: true });
    expect(undo).toHaveBeenCalledOnce();
    expect(redo).toHaveBeenCalledOnce();
  });

  it("on Windows answer Control", async () => {
    const { undo, redo, press } = await mapOn(WINDOWS);

    press({ key: "z", ctrlKey: true });
    press({ key: "y", ctrlKey: true });
    expect(undo).toHaveBeenCalledOnce();
    expect(redo).toHaveBeenCalledOnce();
  });
});

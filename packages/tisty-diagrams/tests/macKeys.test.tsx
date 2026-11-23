import { cleanup, fireEvent, render, renderHook, screen } from "@testing-library/react";
import type { KeyboardEvent } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Mind } from "../src/mindmap";

const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)";
const WINDOWS = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";
const MAP = "mindmap\n  root((Share))\n    Cifrado\n";

type Held = {
  readonly key: string;
  readonly code?: string;
  readonly ctrlKey?: boolean;
  readonly metaKey?: boolean;
  readonly shiftKey?: boolean;
  readonly altKey?: boolean;
};

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
      code: "",
      altKey: false,
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

describe("the canvas keys on a layout that writes no Latin letters", () => {
  it("follow the key pressed with Cmd on a Russian Mac", async () => {
    const { acts, press } = await canvasOn(MAC);

    press({ key: "я", code: "KeyZ", metaKey: true });
    press({ key: "Я", code: "KeyZ", metaKey: true, shiftKey: true });
    press({ key: "в", code: "KeyD", metaKey: true });
    expect(acts.undo).toHaveBeenCalledOnce();
    expect(acts.redo).toHaveBeenCalledOnce();
    expect(acts.duplicate).toHaveBeenCalledOnce();
  });

  it("follow the key pressed with Control on Thai Windows, where Y writes a vowel sign", async () => {
    const { acts, press } = await canvasOn(WINDOWS);

    press({ key: "ผ", code: "KeyZ", ctrlKey: true });
    press({ key: "ั", code: "KeyY", ctrlKey: true });
    press({ key: "ก", code: "KeyD", ctrlKey: true });
    expect(acts.undo).toHaveBeenCalledOnce();
    expect(acts.redo).toHaveBeenCalledOnce();
    expect(acts.duplicate).toHaveBeenCalledOnce();
  });
});

describe("the keys with Option on a Mac", () => {
  it("leave the canvas alone, though Option writes a Greek letter on a Latin layout", async () => {
    const { acts, press } = await canvasOn(MAC);

    press({ key: "Ω", code: "KeyZ", metaKey: true, altKey: true });
    press({ key: "∂", code: "KeyD", metaKey: true, altKey: true });
    expect(acts.undo).not.toHaveBeenCalled();
    expect(acts.duplicate).not.toHaveBeenCalled();
  });

  it("leave the Ideas map alone", async () => {
    const { undo, press } = await mapOn(MAC);

    press({ key: "Ω", code: "KeyZ", metaKey: true, altKey: true });
    expect(undo).not.toHaveBeenCalled();
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

describe("the Ideas map's undo and redo on a layout that writes no Latin letters", () => {
  it("follow the key pressed with Cmd on a Russian Mac", async () => {
    const { undo, redo, press } = await mapOn(MAC);

    press({ key: "я", code: "KeyZ", metaKey: true });
    press({ key: "н", code: "KeyY", metaKey: true });
    expect(undo).toHaveBeenCalledOnce();
    expect(redo).toHaveBeenCalledOnce();
  });

  it("follow the key pressed with Control on Thai Windows", async () => {
    const { undo, redo, press } = await mapOn(WINDOWS);

    press({ key: "ผ", code: "KeyZ", ctrlKey: true });
    press({ key: "ั", code: "KeyY", ctrlKey: true });
    expect(undo).toHaveBeenCalledOnce();
    expect(redo).toHaveBeenCalledOnce();
  });
});

const typed = (key: string, code: string, altKey = false) => ({ key, code, altKey });

describe("the letter a shortcut means", () => {
  it("is the key pressed when the layout writes no Latin letter, and none for a key that holds none", async () => {
    const { letterOf } = await import("../src/macKeys");
    expect(letterOf(typed("ภ", "Digit4"))).toBe("");
    expect(letterOf(typed("़", "BracketRight"))).toBe("");
    expect(letterOf(typed("ว", "Semicolon"))).toBe("");
    expect(letterOf(typed("ф", "KeyA"))).toBe("a");
    expect(letterOf(typed("ิ", "KeyB"))).toBe("b");
  });

  it("is the letter written on a Latin layout, wherever its key sits", async () => {
    const { letterOf } = await import("../src/macKeys");
    expect(letterOf(typed("a", "KeyQ"))).toBe("a");
    expect(letterOf(typed("Z", "KeyW"))).toBe("z");
    expect(letterOf(typed("ğ", "KeyE"))).toBe("");
    expect(letterOf(typed("Delete", "Delete"))).toBe("");
  });

  it("is only the letter written with Option or AltGr, which write other letters on a Latin layout", async () => {
    const { letterOf } = await import("../src/macKeys");
    expect(letterOf(typed("Ω", "KeyZ", true))).toBe("");
    expect(letterOf(typed("π", "KeyP", true))).toBe("");
    expect(letterOf(typed("z", "KeyZ", true))).toBe("z");
  });
});

describe("the key a shortcut means", () => {
  it("is the letter by the key pressed, and any other key as written", async () => {
    const { keyOf } = await import("../src/macKeys");
    expect(keyOf(typed("ц", "KeyW"))).toBe("w");
    expect(keyOf(typed("F", "KeyF"))).toBe("f");
    expect(keyOf(typed("0", "Digit0"))).toBe("0");
    expect(keyOf(typed("\\", "Backslash"))).toBe("\\");
    expect(keyOf(typed("Escape", "Escape"))).toBe("escape");
  });

  it("is the digit of a key that writes a letter there, as AZERTY and Thai do", async () => {
    const { keyOf } = await import("../src/macKeys");
    expect(keyOf(typed("à", "Digit0"))).toBe("0");
    expect(keyOf(typed("é", "Digit2"))).toBe("2");
    expect(keyOf(typed("จ", "Digit0"))).toBe("0");
    expect(keyOf(typed("ๅ", "Digit1"))).toBe("1");
    expect(keyOf(typed("-", "Digit6"))).toBe("-");
    expect(keyOf(typed("º", "Digit0", true))).toBe("º");
  });
});

describe("the name of a shortcut", () => {
  it("is Cmd on a Mac and Ctrl on Windows", async () => {
    on(MAC);
    expect((await import("../src/macKeys")).shortcut("R")).toBe("⌘R");
    on(WINDOWS);
    expect((await import("../src/macKeys")).shortcut("R")).toBe("Ctrl+R");
  });
});

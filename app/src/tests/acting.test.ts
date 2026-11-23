import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Showing } from "../core";
import { t } from "../locales";
import { darkNow } from "../ui/theming";
import { type Acts, choicesFor, keysFor } from "../ui/viewer/acting";

const ipc = vi.hoisted(() => ({ calls: [] as { cmd: string; args?: Record<string, unknown> }[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args });
    return Promise.resolve(null);
  },
}));

const widget: Showing = {
  kind: "piece",
  held: "widget",
  source: "<p>x</p>",
  from: { id: "", file: "f1", title: "Notas", place: "" },
  code: false,
  writable: false,
};

const acts = (on: Showing | null): Acts & Record<string, ReturnType<typeof vi.fn>> => {
  const spy = () => vi.fn();
  return {
    shown: on,
    dark: null,
    actual: false,
    fail: spy(),
    again: spy(),
    code: spy(),
    shade: spy(),
    fit: spy(),
    preset: spy(),
    close: spy(),
  } as unknown as Acts & Record<string, ReturnType<typeof vi.fn>>;
};

const settled = () => new Promise((done) => setTimeout(done, 0));

beforeEach(() => {
  ipc.calls = [];
  document.documentElement.removeAttribute("data-theme");
});

const picked = (all: ReturnType<typeof choicesFor>, label: string) => {
  const one = all.find((choice) => choice.label.endsWith(label));
  one?.onPick?.();
  return one;
};

describe("the viewer's menu", () => {
  it("offers a widget its code, its document and how it is drawn", async () => {
    const a = acts(widget);
    const all = choicesFor(a);

    picked(all, t("viewReload"));
    picked(all, t("viewCode"));
    picked(all, t("viewInDocGo"));
    picked(all, t("viewDark"));
    picked(all, t("viewActual"));
    await settled();

    expect(a.again).toHaveBeenCalled();
    expect(a.code).toHaveBeenCalled();
    expect(a.shade).toHaveBeenCalled();
    expect(a.fit).toHaveBeenCalled();
    expect(ipc.calls).toContainEqual({ cmd: "view_go", args: { file: "f1" } });
  });

  it("ticks the dark widget by the theme it is really drawn in", () => {
    document.documentElement.setAttribute("data-theme", "dark");

    const ticked = choicesFor(acts(widget)).find((one) => one.key === "dark");

    expect(darkNow()).toBe(true);
    expect(ticked?.label.startsWith("✓")).toBe(true);
  });
});

const key = (key: string, shiftKey = false) => new KeyboardEvent("keydown", { key, shiftKey });

describe("the viewer's keys", () => {
  it("picks a widget's width by number and its true size by zero", () => {
    const a = acts(widget);
    keysFor(a, key("2"))?.();
    keysFor(a, key("0"))?.();

    expect(a.preset).toHaveBeenCalledWith("390");
    expect(a.fit).toHaveBeenCalled();
    expect(keysFor(a, key("f"))).toBeUndefined();
  });

  it("closes and reads again by the key pressed when the layout writes no Latin letters", () => {
    const a = acts(widget);
    keysFor(a, new KeyboardEvent("keydown", { key: "ц", code: "KeyW" }))?.();
    keysFor(a, new KeyboardEvent("keydown", { key: "к", code: "KeyR" }))?.();

    expect(a.close).toHaveBeenCalled();
    expect(a.again).toHaveBeenCalled();
  });

  it("picks a width and the true size by the digit keys where they write letters, as AZERTY and Thai do", () => {
    const a = acts(widget);
    keysFor(a, new KeyboardEvent("keydown", { key: "à", code: "Digit0" }))?.();
    keysFor(a, new KeyboardEvent("keydown", { key: "ๅ", code: "Digit1" }))?.();

    expect(a.fit).toHaveBeenCalled();
    expect(a.preset).toHaveBeenCalledWith("fit");
  });

  it("offers a formula, a table or the properties only their code and their document", () => {
    const formula: Showing = { ...widget, held: "formula" };
    const a = acts(formula);
    const all = choicesFor(a);

    expect(all.map((one) => one.key)).toEqual(["code", "doc"]);
    keysFor(a, key("2"))?.();
    expect(a.preset).not.toHaveBeenCalled();
  });
});

const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)";
const WINDOWS = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";

const hintsOn = async (agent: string) => {
  vi.resetModules();
  vi.spyOn(navigator, "userAgent", "get").mockReturnValue(agent);
  const fresh = await import("../ui/viewer/acting");
  return fresh.choicesFor(acts(widget)).flatMap((one) => (one.hint ? [[one.key, one.hint]] : []));
};

describe("the viewer's menu hints", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("name Cmd on a Mac, the key the viewer answers there", async () => {
    expect(await hintsOn(MAC)).toEqual([
      ["again", "⌘R"],
      ["actual", "⌘0"],
    ]);
  });

  it("name Ctrl on Windows", async () => {
    expect(await hintsOn(WINDOWS)).toEqual([
      ["again", "Ctrl+R"],
      ["actual", "Ctrl+0"],
    ]);
  });
});

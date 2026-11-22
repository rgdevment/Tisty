import { afterEach, describe, expect, it, vi } from "vitest";

const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)";
const WINDOWS = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";

const keysOn = (agent: string) => {
  vi.resetModules();
  vi.spyOn(navigator, "userAgent", "get").mockReturnValue(agent);
  return import("../src/keys");
};

const keyed = () => ({
  find: vi.fn(),
  side: vi.fn(),
  theme: vi.fn(),
  zoom: vi.fn(),
  head: vi.fn(),
  back: vi.fn(),
  source: vi.fn(),
  print: vi.fn(),
  close: vi.fn(),
  escape: vi.fn(),
});

const pressing = async (agent: string) => {
  const { keyHeard, shortcut } = await keysOn(agent);
  const press = (init: KeyboardEventInit) => {
    const acts = keyed();
    const event = new KeyboardEvent("keydown", { cancelable: true, ...init });
    keyHeard(acts)(event);
    return { acts, event };
  };
  return { press, shortcut };
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe("the command key", () => {
  it("is Cmd on a Mac, and Control there is left alone", async () => {
    const { press } = await pressing(MAC);

    const control = press({ key: "f", ctrlKey: true });
    expect(control.acts.find).not.toHaveBeenCalled();
    expect(control.event.defaultPrevented).toBe(false);
    expect(press({ key: "w", ctrlKey: true }).acts.close).not.toHaveBeenCalled();

    const command = press({ key: "f", metaKey: true });
    expect(command.acts.find).toHaveBeenCalled();
    expect(command.event.defaultPrevented).toBe(true);
    expect(press({ key: "M", metaKey: true, shiftKey: true }).acts.source).toHaveBeenCalled();
  });

  it("is Control on Windows", async () => {
    const { press } = await pressing(WINDOWS);

    expect(press({ key: "f", ctrlKey: true }).acts.find).toHaveBeenCalled();
    expect(press({ key: "w", ctrlKey: true }).acts.close).toHaveBeenCalled();
  });

  it("is the one the menu names", async () => {
    expect((await pressing(MAC)).shortcut("F")).toBe("⌘F");
    expect((await pressing(WINDOWS)).shortcut("F")).toBe("Ctrl+F");
  });
});

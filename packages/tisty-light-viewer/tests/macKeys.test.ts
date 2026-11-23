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

  it("follows the key pressed on a layout that writes no Latin letters", async () => {
    const mac = await pressing(MAC);
    expect(mac.press({ key: "а", code: "KeyF", metaKey: true }).acts.find).toHaveBeenCalled();
    expect(
      mac.press({ key: "Ь", code: "KeyM", metaKey: true, shiftKey: true }).acts.source,
    ).toHaveBeenCalled();
    const plain = mac.press({ key: "0", code: "Digit0", metaKey: true });
    expect(plain.acts.zoom).toHaveBeenCalledWith(0);

    const windows = await pressing(WINDOWS);
    expect(windows.press({ key: "ไ", code: "KeyW", ctrlKey: true }).acts.close).toHaveBeenCalled();
    expect(windows.press({ key: "з", code: "KeyP", ctrlKey: true }).acts.print).toHaveBeenCalled();
  });

  it("is no shortcut with Option, which writes a Greek letter on a Latin Mac", async () => {
    const { press } = await pressing(MAC);

    const option = press({ key: "π", code: "KeyP", metaKey: true, altKey: true });
    expect(option.acts.print).not.toHaveBeenCalled();
    expect(option.event.defaultPrevented).toBe(false);
  });

  it("takes a sign written with AltGr, as Ctrl+\\ is on a German Windows", async () => {
    const { press } = await pressing(WINDOWS);

    const side = press({ key: "\\", code: "Minus", ctrlKey: true, altKey: true });
    expect(side.acts.side).toHaveBeenCalled();
  });

  it("brings the true size back by the zero key where it writes a letter, as AZERTY and Thai do", async () => {
    const mac = await pressing(MAC);
    const azerty = mac.press({ key: "à", code: "Digit0", metaKey: true });
    expect(azerty.acts.zoom).toHaveBeenCalledWith(0);

    const windows = await pressing(WINDOWS);
    const thai = windows.press({ key: "จ", code: "Digit0", ctrlKey: true });
    expect(thai.acts.zoom).toHaveBeenCalledWith(0);
  });

  it("is the one the menu names", async () => {
    expect((await pressing(MAC)).shortcut("F")).toBe("⌘F");
    expect((await pressing(WINDOWS)).shortcut("F")).toBe("Ctrl+F");
  });
});

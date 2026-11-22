import { afterEach, describe, expect, it, vi } from "vitest";

const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)";
const WINDOWS = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";

const pressedOn = async (agent: string) => {
  vi.resetModules();
  vi.spyOn(navigator, "userAgent", "get").mockReturnValue(agent);
  return (await import("../../src/edit/slashing")).pressed;
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe("the shortcuts the menu and the bar name", () => {
  it("say Cmd on a Mac", async () => {
    expect((await pressedOn(MAC))("E")).toBe("⌘ E");
  });

  it("say Control on Windows", async () => {
    expect((await pressedOn(WINDOWS))("E")).toBe("Ctrl E");
  });
});

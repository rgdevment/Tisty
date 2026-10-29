import { describe, expect, it, vi } from "vitest";
import { darkNow, followTheme } from "../ui/theming";

describe("the host's theme", () => {
  it("is dark only while the host says so", () => {
    document.documentElement.dataset.theme = "dark";
    expect(darkNow()).toBe(true);
    document.documentElement.dataset.theme = "light";
    expect(darkNow()).toBe(false);
    document.documentElement.removeAttribute("data-theme");
    expect(darkNow()).toBe(false);
  });

  it("is followed each time it turns, and no more once stopped", async () => {
    const retheme = vi.fn();
    const stop = followTheme(retheme);
    document.documentElement.dataset.theme = "dark";
    await Promise.resolve();
    expect(retheme).toHaveBeenCalledOnce();
    document.documentElement.lang = "es";
    await Promise.resolve();
    expect(retheme).toHaveBeenCalledOnce();
    stop();
    document.documentElement.dataset.theme = "light";
    await Promise.resolve();
    expect(retheme).toHaveBeenCalledOnce();
  });
});

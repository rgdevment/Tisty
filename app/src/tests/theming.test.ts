import { beforeEach, describe, expect, it, vi } from "vitest";

const win = vi.hoisted(() => ({ label: "main" }));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ label: win.label, onFocusChanged: () => Promise.resolve(() => {}) }),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => new Promise(() => {}) }));

describe("every window paints itself in the system theme", () => {
  const dark = (yes: boolean) => {
    window.matchMedia = ((query: string) => ({
      matches: yes,
      media: query,
      onchange: null,
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      dispatchEvent: () => false,
    })) as unknown as typeof window.matchMedia;
  };

  const load = async (label: string, wantsDark: boolean) => {
    win.label = label;
    dark(wantsDark);
    document.documentElement.removeAttribute("data-theme");
    document.body.innerHTML = '<div id="root"></div>';
    vi.resetModules();
    await import("../main");
  };

  beforeEach(() => {
    vi.resetModules();
  });

  it("paints the main window dark where the desktop is dark", async () => {
    await load("main", true);

    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });

  it("paints quick capture dark too, which is what it never did", async () => {
    await load("quick", true);

    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });

  it("paints quick capture light where the desktop is light", async () => {
    await load("quick", false);

    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });
});

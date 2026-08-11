import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";
import { adopt } from "../locales";

// There is no Tauri host under jsdom, so the window's event bridge has to be
// stood in for: without it every suite that mounts App leaks a rejection.
vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
  emit: () => Promise.resolve(),
}));

adopt("en");

// jsdom ships no media queries, and the theme reads one on mount.
window.matchMedia ??= ((query: string) => ({
  matches: false,
  media: query,
  onchange: null,
  addEventListener: () => {},
  removeEventListener: () => {},
  addListener: () => {},
  removeListener: () => {},
  dispatchEvent: () => false,
})) as unknown as typeof window.matchMedia;

// jsdom has no layout, so nothing ever resizes; the app measures with this.
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver;

afterEach(cleanup);

import { cleanup } from "@testing-library/react";
import { afterEach, beforeAll, vi } from "vitest";
import { adopt } from "../locales";

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
  emit: () => Promise.resolve(),
}));

beforeAll(() => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  vi.setSystemTime(new Date(2026, 5, 10, 9, 0, 0));
});

adopt("en");

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

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver;

const flat = () => {
  const rect = { x: 0, y: 0, top: 0, left: 0, right: 0, bottom: 0, width: 0, height: 0 };
  return { length: 0, item: () => null, [Symbol.iterator]: function* () {}, ...rect };
};

const laid = Text.prototype as unknown as { getClientRects?: () => unknown };
laid.getClientRects ??= flat;

const spanned = Range.prototype as unknown as {
  getClientRects?: () => unknown;
  getBoundingClientRect?: () => unknown;
};
spanned.getClientRects ??= flat;
spanned.getBoundingClientRect ??= flat;

// An unmounted editor leaves timers behind, and on a fake clock they fire once the environment is
// already gone, which ends the run on `window is not defined` with every test passing.
afterEach(() => {
  cleanup();
  if (vi.isFakeTimers()) {
    vi.advanceTimersByTime(50);
  }
});

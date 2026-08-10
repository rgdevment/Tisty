import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";
import { adopt } from "../locales";

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

afterEach(cleanup);

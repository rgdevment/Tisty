import { beforeEach, describe, expect, it, vi } from "vitest";

const noted = vi.hoisted(() => ({ breaks: [] as [string, string | null, string][] }));

vi.mock("../core", () => ({
  noteBreak: (kind: string, said: string | null, frames: string) => {
    noted.breaks.push([kind, said, frames]);
    return Promise.resolve();
  },
}));

import { thrown } from "../broke";

let saidSoFar = 0;
const fresh = () => `said ${++saidSoFar}`;

describe("a window error", () => {
  beforeEach(() => {
    noted.breaks.length = 0;
  });

  it("is not noted when Chromium only settles a resize on the next frame", () => {
    thrown(
      new ErrorEvent("error", {
        message: "ResizeObserver loop completed with undelivered notifications.",
      }),
    );
    expect(noted.breaks).toEqual([]);
  });

  it("keeps what the page said and where when nothing was thrown with it", () => {
    const said = fresh();
    thrown(
      new ErrorEvent("error", {
        message: said,
        filename: "http://tauri.localhost/assets/index.js",
        lineno: 12,
        colno: 7,
      }),
    );
    expect(noted.breaks).toEqual([
      ["Error", said, "at http://tauri.localhost/assets/index.js:12:7"],
    ]);
  });

  it("keeps what the page said when a plain string was thrown", () => {
    const said = fresh();
    thrown(new ErrorEvent("error", { message: `Uncaught ${said}`, error: said }));
    expect(noted.breaks).toEqual([["Error", `Uncaught ${said}`, ""]]);
  });

  it("reads a thrown error before the page's words", () => {
    const fault = new TypeError(fresh());
    fault.stack = `TypeError: ${fault.message}\n    at drawn (http://tauri.localhost/a.js:1:2)`;
    thrown(new ErrorEvent("error", { message: "Uncaught", error: fault }));
    expect(noted.breaks).toEqual([
      ["TypeError", fault.message, "at drawn (http://tauri.localhost/a.js:1:2)"],
    ]);
  });
});

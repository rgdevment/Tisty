import { beforeEach, describe, expect, it, vi } from "vitest";

type Noted = [string, string | null, string, boolean];

const noted = vi.hoisted(() => ({ breaks: [] as Noted[] }));

vi.mock("../core", () => ({
  noteBreak: (kind: string, said: string | null, frames: string, stopped: boolean) => {
    noted.breaks.push([kind, said, frames, stopped]);
    return Promise.resolve();
  },
}));

import { broke, rejected, thrown } from "../broke";

let saidSoFar = 0;
const fresh = () => `said ${++saidSoFar}`;

const rejection = (reason: unknown) => ({ reason }) as unknown as PromiseRejectionEvent;

beforeEach(() => {
  noted.breaks.length = 0;
});

describe("a window that falls", () => {
  it("is noted as having stopped drawing, once for the same break", () => {
    const said = fresh();
    broke("TypeError", said, "TypeError\n    at drawn (http://tauri.localhost/a.js:1:2)");
    broke("TypeError", said, "TypeError\n    at drawn (http://tauri.localhost/a.js:1:2)");
    expect(noted.breaks).toEqual([
      ["TypeError", said, "at drawn (http://tauri.localhost/a.js:1:2)", true],
    ]);
  });
});

describe("a window error", () => {
  it("is not noted when Chromium only settles a resize on the next frame", () => {
    thrown(
      new ErrorEvent("error", {
        message: "ResizeObserver loop completed with undelivered notifications.",
      }),
    );
    expect(noted.breaks).toEqual([]);
  });

  it("is noted when it only mentions the resize notice", () => {
    const said = `${fresh()} after a ResizeObserver loop`;
    thrown(new ErrorEvent("error", { message: said }));
    expect(noted.breaks).toEqual([["Error", said, "", false]]);
  });

  it("keeps what the browser said and where when nothing was thrown with it", () => {
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
      ["Error", said, "at http://tauri.localhost/assets/index.js:12:7", false],
    ]);
  });

  it("keeps only the shape of a thrown string, never its words", () => {
    const words = fresh();
    thrown(
      new ErrorEvent("error", {
        message: `Uncaught ${words}`,
        error: words,
        filename: "http://tauri.localhost/b.js",
        lineno: 3,
        colno: 4,
      }),
    );
    expect(noted.breaks).toEqual([
      ["Error", "a string without a message", "at http://tauri.localhost/b.js:3:4", false],
    ]);
  });

  it("reads a thrown error before the browser's words", () => {
    const fault = new TypeError(fresh());
    fault.stack = `TypeError: ${fault.message}\n    at drawn (http://tauri.localhost/a.js:1:2)`;
    thrown(new ErrorEvent("error", { message: "Uncaught", error: fault }));
    expect(noted.breaks).toEqual([
      ["TypeError", fault.message, "at drawn (http://tauri.localhost/a.js:1:2)", false],
    ]);
  });

  it("falls back to where it happened when the error's stack names no frame", () => {
    const fault = new RangeError(fresh());
    fault.stack = `RangeError: ${fault.message}`;
    thrown(
      new ErrorEvent("error", {
        error: fault,
        filename: "http://tauri.localhost/c.js",
        lineno: 9,
        colno: 1,
      }),
    );
    expect(noted.breaks).toEqual([
      ["RangeError", fault.message, "at http://tauri.localhost/c.js:9:1", false],
    ]);
  });

  it("is noted even when it reaches the window as a plain event", () => {
    expect(() => thrown(new Event("error") as ErrorEvent)).not.toThrow();
    expect(noted.breaks).toEqual([["Error", "", "", false]]);
  });
});

describe("a promise nobody answered", () => {
  it("keeps the error it was refused with", () => {
    const fault = new SyntaxError(fresh());
    fault.stack = `SyntaxError: ${fault.message}\n    at read (http://tauri.localhost/d.js:2:5)`;
    rejected(rejection(fault));
    expect(noted.breaks).toEqual([
      ["SyntaxError", fault.message, "at read (http://tauri.localhost/d.js:2:5)", false],
    ]);
  });

  it("names the refusal code Tisty answered with, never the name beside it", () => {
    const code = fresh();
    rejected(rejection({ code, name: "Recetas de la abuela" }));
    expect(noted.breaks).toEqual([["Refusal", code, "", false]]);
  });

  it("keeps only the shape of a rejected string or object", () => {
    rejected(rejection(fresh()));
    rejected(rejection({ title: fresh() }));
    expect(noted.breaks).toEqual([
      ["Rejection", "a string without a message", "", false],
      ["Rejection", "an object without a message", "", false],
    ]);
  });

  it("is noted with nothing said when it was refused with nothing", () => {
    rejected(rejection(undefined));
    expect(noted.breaks).toEqual([["Rejection", null, "", false]]);
  });
});

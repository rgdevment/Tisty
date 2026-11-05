import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vitest";
import { dressedAsWindow, framed } from "../ui/framed";
import task from "../ui/taskpage.css?raw";

const page = (html: string): Document => new DOMParser().parseFromString(html, "text/html");

afterEach(() => document.documentElement.removeAttribute("data-theme"));

describe("a page in the window", () => {
  it("is built in the window's theme", () => {
    document.documentElement.dataset.theme = "dark";
    expect(page(framed("<p>x</p>")).documentElement.classList.contains("dark")).toBe(true);
    document.documentElement.dataset.theme = "light";
    expect(page(framed("<p>x</p>")).documentElement.classList.contains("dark")).toBe(false);
  });

  it("wears a writer's bar before its own looks, and none when it only reads", () => {
    expect(framed("<p>x</p>")).not.toContain(".brought-bar");
    expect(framed("<p>x</p>", ".brought-bar{}")).toContain(`.brought-bar{}${task}`);
  });

  it("turns as the window turns", () => {
    const drawn = page(framed("<p>x</p>"));
    document.documentElement.dataset.theme = "dark";
    dressedAsWindow(drawn);
    expect(drawn.documentElement.classList.contains("dark")).toBe(true);
    document.documentElement.dataset.theme = "light";
    dressedAsWindow(drawn);
    expect(drawn.documentElement.classList.contains("dark")).toBe(false);
  });
});

describe("what only reads", () => {
  const SRC = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const PACKAGES = resolve(SRC, "..", "..", "packages");
  const IMPORTS =
    /^(?:import\s+(?!type\b)[^;"]*?from\s+|import\s+|export\s+(?:\*|\{)[^;"]*?from\s+)"([^"]+)"/gm;

  const fileOf = (from: string, said: string): string | undefined => {
    const [spec] = said.split("?");
    const door = /^(tisty-editor|tisty-diagrams)\/(.+)$/.exec(spec);
    const at = door ? join(PACKAGES, door[1], "src", door[2]) : resolve(dirname(from), spec);
    return [`${at}.ts`, `${at}.tsx`, at].find(existsSync);
  };

  const reachedFrom = (root: string): string[] => {
    const seen = new Set<string>();
    const named = new Set<string>();
    const waiting = [join(SRC, root)];
    for (const at of waiting) {
      if (seen.has(at)) continue;
      seen.add(at);
      if (!/\.tsx?$/.test(at)) continue;
      for (const [, spec] of readFileSync(at, "utf8").matchAll(IMPORTS)) {
        named.add(spec);
        const found =
          spec.startsWith(".") || spec.startsWith("tisty-") ? fileOf(at, spec) : undefined;
        if (found) waiting.push(found);
      }
    }
    return [...named];
  };

  it.each(["ui/Page.tsx", "ui/viewer/Reader.tsx", "ui/HelpArticle.tsx", "ui/MarkdownArticle.tsx"])(
    "%s reaches nothing of the editor",
    (root) => {
      const reached = reachedFrom(root);
      expect(reached).toContain("tisty-editor/read");
      expect(reached.filter((one) => /^(tisty-editor\/edit|@codemirror|@lezer)/.test(one))).toEqual(
        [],
      );
    },
  );
});

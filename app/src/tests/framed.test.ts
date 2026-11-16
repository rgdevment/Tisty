import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { MARKED_FOR } from "tisty-editor/edit";
import { afterEach, describe, expect, it } from "vitest";
import { MARK_FADES } from "../ui/anchoring";
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

describe("the mark an agent's words leave", () => {
  it("fades when the editor lets it go", () => {
    expect(MARK_FADES).toBe(MARKED_FOR);
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

  it.each([
    "ui/Page.tsx",
    "ui/viewer/Reader.tsx",
    "ui/HelpArticle.tsx",
    "ui/MarkdownArticle.tsx",
    "Viewing.tsx",
    "App.tsx",
  ])("%s reaches nothing of the editor", (root) => {
    const reached = reachedFrom(root);
    expect(reached).toContain("tisty-editor/read");
    expect(reached.filter((one) => /^(tisty-editor\/edit|@codemirror|@lezer)/.test(one))).toEqual(
      [],
    );
  });
});

const rulesOf = (css: string) =>
  [...css.replace(/\/\*[\s\S]*?\*\//g, "").matchAll(/([^{}]+)\{[^{}]*\}/g)].flatMap(([, said]) =>
    said.split(/,(?![^(]*\))/).map((one) => one.trim()),
  );

const compoundsOf = (selector: string) => {
  const found: string[] = [];
  let depth = 0;
  let one = "";
  for (const letter of selector) {
    if (letter === "(") depth += 1;
    if (letter === ")") depth -= 1;
    if (depth === 0 && /[\s>+~]/.test(letter)) {
      if (one) found.push(one);
      one = "";
    } else one += letter;
  }
  return one ? [...found, one] : found;
};

const ABOVE_EVERY_BLOCK = /^(html|:root|body)(?![\w-])|\.page(?![\w-])/;

const here = dirname(fileURLToPath(import.meta.url));
const LOOKS = [
  ...["page.css", "tables.css", "diary.css"].map((one) =>
    join(here, "../../../packages/tisty-sheet/src", one),
  ),
  join(here, "../ui/taskpage.css"),
]
  .map((one) => readFileSync(one, "utf8"))
  .join("\n");

describe("the page's looks, kept cheap to restyle while it is written on", () => {
  it("never asks the root, the body or the sheet what they hold to style something inside them", () => {
    const asking = rulesOf(LOOKS).filter((selector) =>
      compoundsOf(selector)
        .slice(0, -1)
        .some((one) => one.includes(":has(") && ABOVE_EVERY_BLOCK.test(one)),
    );
    expect(asking).toEqual([]);
  });

  it("still sees the guard on a rule written that way", () => {
    const one = "html:not(.tall):has(.diary-month) .page";
    expect(compoundsOf(one)).toEqual(["html:not(.tall):has(.diary-month)", ".page"]);
    expect(ABOVE_EVERY_BLOCK.test(compoundsOf(one)[0] ?? "")).toBe(true);
  });

  it("lays a diary's month on the desk, and no other page", () => {
    const month = rulesOf(LOOKS).filter((one) => one.includes("diary-month)"));
    expect(month).toHaveLength(2);
    const diary = page(framed('<div class="diary-month"></div>'));
    const other = page(framed('<div><div class="diary-month"></div></div>'));
    for (const one of month) {
      expect(diary.querySelector(one), one).not.toBeNull();
      expect(other.querySelector(one), one).toBeNull();
    }
  });
});

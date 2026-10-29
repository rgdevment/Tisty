import { beforeEach, describe, expect, it } from "vitest";
import { dressed, escaped, PAGE_CSS, saidKinds, shell, TOKENS } from "../src/shell";
import { source } from "./source";

const page = (html: string): Document =>
  new DOMParser().parseFromString(shell(html, false), "text/html");

beforeEach(() => {
  document.documentElement.removeAttribute("style");
  document.documentElement.lang = "";
});

describe("the shell of a drawn page", () => {
  it("speaks the host's language, starts untouchable and wears the theme it is given", () => {
    document.documentElement.lang = "es";
    const said = shell("<p>x</p>", true);
    expect(said).toContain('lang="es"');
    expect(said).toContain('class="dark"');
    expect(said).toContain('<article class="page" inert><p>x</p></article>');
    expect(said).toContain(":root{--tisty-sheet:");
    expect(shell("<p>x</p>", false)).toContain('class=""');
  });

  it("wears the page's CSS, then the host's own over it", () => {
    expect(shell("<p>x</p>", false, ".mine{color:red}")).toContain(
      `<style>${PAGE_CSS}.mine{color:red}</style>`,
    );
    expect(shell("<p>x</p>", false)).toContain(`<style>${PAGE_CSS}</style>`);
  });

  it("carries the host's tokens, and none that could close the style", () => {
    document.documentElement.style.setProperty("--tisty-ink", "#111111");
    document.documentElement.style.setProperty("--tisty-soft", "red}</style><script>");
    const said = shell("<p>x</p>", false);
    expect(said).toContain("--tisty-ink:#111111");
    expect(said).not.toContain("--tisty-soft:");
  });

  it("escapes what it is asked to show as plain words", () => {
    expect(escaped("<b>&</b>")).toBe("&lt;b&gt;&amp;&lt;/b&gt;");
  });
});

describe("the page the writing sits on", () => {
  it("is paper by default, whatever the theme", () => {
    expect(source("page.css")).toMatch(
      /html:not\(\.tall\) \.page \{[^}]*background:\s*var\(--tisty-sheet\)/,
    );
  });

  it("names every token it reads, hues and pens included", () => {
    expect(TOKENS).toContain("--tisty-sheet");
    expect(TOKENS).toContain("--tisty-hue-red");
    expect(TOKENS).toContain("--tisty-pen-yellow");
    expect(TOKENS.every((one) => one.startsWith("--tisty-"))).toBe(true);
  });
});

describe("dressing a drawn page", () => {
  it("wears the theme it is given and the host's tokens", () => {
    const drawn = page("<p>x</p>");
    document.documentElement.style.setProperty("--tisty-ink", "#222222");
    dressed(drawn, true);
    expect(drawn.documentElement.classList.contains("dark")).toBe(true);
    expect(drawn.documentElement.style.getPropertyValue("--tisty-ink")).toBe("#222222");
    dressed(drawn, false);
    expect(drawn.documentElement.classList.contains("dark")).toBe(false);
  });

  it("names every callout in the host's words, and leaves a plain quote alone", () => {
    const drawn = page(
      '<blockquote data-callout="tip"><p>x</p></blockquote><blockquote><p>z</p></blockquote>',
    );
    saidKinds(drawn, (kind) => `said ${kind}`);
    const [tip, plain] = Array.from(drawn.querySelectorAll("blockquote"));
    expect(tip.dataset.said).toBe("said tip");
    expect(plain.dataset.said).toBeUndefined();
  });
});

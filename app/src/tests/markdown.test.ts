import { describe, expect, it } from "vitest";
import { composed } from "../markdown";

describe("what gets composed", () => {
  it("keeps the ordinary Markdown a file needs to stay readable without Tisty", () => {
    expect(composed("**firm** and *soft*")).toContain("<strong>firm</strong>");
    expect(composed("### A heading")).toContain("<h3>A heading</h3>");
    expect(composed("- one\n- two")).toContain("<li>one</li>");
    expect(composed("run `PAY_SANDBOX=1`")).toContain("<code>PAY_SANDBOX=1</code>");
  });

  it("paints a reference as one, without turning it into a link that goes nowhere", () => {
    const html = composed("it came from [[OPS-3465]]");
    expect(html).toContain('<span class="ref">OPS-3465</span>');
    expect(html).not.toContain("<a");
  });

  it("leaves an unclosed reference as the text it is", () => {
    expect(composed("[[never closed")).toContain("[[never closed");
  });

  it("marks what lives under the data root so the view can resolve it", () => {
    const inside = composed("![shot](<attachments/ab/cd.png>)");
    expect(inside).toContain('data-inside="attachments/ab/cd.png"');
    expect(inside).toContain('src=""');

    const outside = composed("[OPS-3465](https://jira.example/browse/OPS-3465)");
    expect(outside).not.toContain("data-inside");
    expect(outside).toContain('href="https://jira.example/browse/OPS-3465"');
  });

  it("leaves an absolute path alone, whatever the platform writes", () => {
    for (const at of ["C:/Users/Mario/clip.mkv", "/home/mario/clip.mkv", "//server/share/clip.mkv"]) {
      expect(composed(`[clip](<${at}>)`)).not.toContain("data-inside");
    }
    expect(composed("![shot](<attachments/ab/cd.png>)")).toContain("data-inside");
  });

  it("finds a bare address without being asked", () => {
    expect(composed("see https://x.example/1")).toContain('href="https://x.example/1"');
  });
});

describe("what does not get through", () => {
  it("escapes raw HTML instead of running it", () => {
    const html = composed('<img src=x onerror="alert(1)">');
    expect(html).not.toContain("<img");
    expect(html).toContain("&lt;img");
  });

  it("refuses a script scheme behind a well-behaved label", () => {
    const html = composed("[press me](javascript:alert(1))");
    expect(html).not.toContain("<a");
    expect(html).toContain("[press me]");
  });

  it("escapes a reference that carries markup in its name", () => {
    expect(composed("[[<b>bold</b>]]")).not.toContain("<b>");
  });
});

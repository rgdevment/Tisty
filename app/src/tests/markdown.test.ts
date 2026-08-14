import { describe, expect, it } from "vitest";
import { composed, docLink, docOf } from "../markdown";

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

describe("a link to a document, built with a title nobody chose carefully", () => {
  it("still makes a link out of a title with nothing in it", () => {
    const html = composed(docLink("mac0-0001", ""));
    expect(html).toContain('href="tisty:doc/mac0-0001"');
    expect(html).toContain('class="paper"');
  });

  it("keeps a title made only of spaces instead of collapsing it away", () => {
    const html = composed(docLink("mac0-0001", "   "));
    expect(html).toContain('href="tisty:doc/mac0-0001"');
    expect(html).toContain(">   <");
  });

  it("survives a title with a loose opening bracket too", () => {
    const written = docLink("mac0-0001", "Informe [ borrador");

    expect(composed(written)).toContain(">Informe [ borrador<");
    expect(composed(written)).toContain('href="tisty:doc/mac0-0001"');
  });

  it("does not need to escape the parentheses a title carries", () => {
    const written = docLink("mac0-0001", "Nota (borrador)");

    expect(composed(written)).toContain(">Nota (borrador)<");
  });

  it("keeps one link, not two, when the title has a line break in it", () => {
    const written = docLink("mac0-0001", "linea uno\nlinea dos");
    const html = composed(written);

    expect(html.match(/<a /g)).toHaveLength(1);
    expect(html).toContain('href="tisty:doc/mac0-0001"');
  });

  it("shows a title that already looks like a link as plain text, not a nested one", () => {
    const written = docLink("mac0-0001", "[ya es un link](http://evil.com)");
    const html = composed(written);

    expect(html.match(/<a /g)).toHaveLength(1);
    expect(html).toContain(">[ya es un link](http://evil.com)<");
    expect(html).toContain('href="tisty:doc/mac0-0001"');
    expect(html).not.toContain("evil.com\"");
  });
});

describe("what docOf makes of an address", () => {
  it("says no to anything that is not one of ours", () => {
    expect(docOf("")).toBeNull();
    expect(docOf("tisty:do")).toBeNull();
    expect(docOf("https://tisty:doc/mac0-0001")).toBeNull();
  });

  it("reads an id with nothing after the scheme as an empty one, not a missing one", () => {
    expect(docOf("tisty:doc/")).toBe("");
  });

  it("takes whatever comes after the scheme, slashes included", () => {
    expect(docOf("tisty:doc/a/b")).toBe("a/b");
  });
});

import { checkHtml } from "@tisty/contracts";
import { afterEach, describe, expect, it } from "vitest";
import { finished } from "../src/finishing";
import { flattened } from "../src/flattening";

afterEach(() => {
  document.head.innerHTML = "";
  document.body.innerHTML = "";
});

const NOT_DRAWN = "Not drawn";
const drawnBy =
  (svg: (dark: boolean) => string | null) =>
  async (_: string, dark: boolean): Promise<string | null> =>
    svg(dark);

const DRAWING = (dark: boolean) =>
  `<svg xmlns="http://www.w3.org/2000/svg" id="m" class="flowchart" viewBox="0 0 100 50" style="max-width:100px" role="graphics-document"><style>#m .node{fill:${dark ? "#000" : "#fff"};}</style><defs><marker id="a" viewBox="0 0 1 1" refX="1" refY="1" markerWidth="1" markerHeight="1" orient="auto"><path d="M0 0" class="arrow"></path></marker><marker id="unused"><path d="M0 0"></path></marker><filter id="s"><feDropShadow></feDropShadow></filter></defs><g class="node" id="n1" data-id="x"><rect class="box" x="1" y="2" width="3" height="4" style="stroke:#333"></rect><text x="1" y="1"><tspan x="1">Hola</tspan></text><path d="M0 0" marker-end="url(#a)"></path></g><switch><foreignObject><div>x</div></foreignObject><text>y</text></switch><a xlink:href="https://a.example"><circle cx="1" cy="1" r="1"></circle></a></svg>`;

const mermaid = (source: string) =>
  `<figure class="diagram"><pre><code class="language-mermaid">${source}</code></pre></figure>`;

describe("a document finished to travel", () => {
  it("colours its code the way the contract says", async () => {
    const { html } = await finished(
      '<figure class="code"><pre><code class="language-ts">class A extends B { f() { g(); } }</code></pre></figure>',
    );
    expect(html).toContain("hljs-title");
    expect(html).toContain("class_");
    expect(checkHtml(html)).toEqual([]);
  });

  it("leaves code it does not know as it is, a block past its limit too", async () => {
    const { html } = await finished(
      '<figure class="code"><pre><code class="language-nope">x</code></pre></figure><figure class="code"><pre><code class="language-widget">&lt;div&gt;&lt;/div&gt;</code></pre></figure>',
    );
    expect(html).not.toContain("hljs");
    expect(checkHtml(html)).toEqual([]);
  });

  it("draws an icon so whoever reads needs no set of its own", async () => {
    const { html } = await finished(
      '<p><span class="ico" data-ico="clock" data-hue="red" title="clock"></span><span class="ico" data-ico="nope" data-hue="" title="nope"></span></p>',
    );
    const page = new DOMParser().parseFromString(html, "text/html");
    const [known, unknown] = page.querySelectorAll("span.ico");
    expect(known.querySelector("svg")?.getAttribute("stroke")).toBe("currentColor");
    expect(known.querySelector("circle")).not.toBeNull();
    expect(unknown.querySelector("svg")).toBeNull();
    expect(checkHtml(html)).toEqual([]);
  });

  it("leaves no class where a solid glyph wore one", async () => {
    const { html } = await finished('<span class="ico" data-ico="star"></span>');
    expect(html).not.toContain("solid");
    expect(checkHtml(html)).toEqual([]);
  });

  it("gives each diagram both drawings, and they share no name", async () => {
    const { html, notDrawn } = await finished(
      mermaid("graph") + mermaid("graph"),
      drawnBy(DRAWING),
    );
    const page = new DOMParser().parseFromString(html, "text/html");
    expect(notDrawn).toBe(0);
    const figures = page.querySelectorAll("figure.diagram");
    expect(figures).toHaveLength(2);
    for (const figure of figures) {
      const themes = [...figure.querySelectorAll("svg")].map((one) =>
        one.getAttribute("data-theme"),
      );
      expect(themes).toEqual(["light", "dark"]);
    }
    const ids = [...page.querySelectorAll("[id]")].map((one) => one.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(checkHtml(html)).toEqual([]);
  });

  it("says when a diagram could not be drawn, and it still reads, as code", async () => {
    const { html, notDrawn } = await finished(
      mermaid("graph &lt;"),
      drawnBy(() => null),
    );
    expect(notDrawn).toBe(1);
    expect(html).toBe('<figure class="code"><pre><code>graph &lt;</code></pre></figure>');
    expect(checkHtml(html)).toEqual([]);
  });

  it("does not take half a diagram: both themes or none", async () => {
    const { notDrawn } = await finished(
      mermaid("graph"),
      drawnBy((dark) => (dark ? null : DRAWING(false))),
    );
    expect(notDrawn).toBe(1);
  });

  it("leaves nothing behind in the window", async () => {
    await finished(mermaid("graph"), drawnBy(DRAWING));
    expect(document.body.children).toHaveLength(0);
  });

  it("does not leave a page half done when a drawing throws", async () => {
    await expect(
      finished(mermaid("graph"), () => Promise.reject(new Error(NOT_DRAWN))),
    ).rejects.toThrow(NOT_DRAWN);
    expect(document.body.children).toHaveLength(0);
  });
});

describe("a drawing flattened", () => {
  const placed = (svg: string) => {
    document.body.innerHTML = svg;
    return document.body.querySelector("svg") as SVGSVGElement;
  };

  it("keeps its shapes and its words, and nothing a reader cannot use", () => {
    const flat = flattened(placed(DRAWING(false)), "p") ?? "";
    for (const gone of [
      "<style",
      "class=",
      "style=",
      "data-id",
      "foreignObject",
      "filter",
      "feDropShadow",
      "<a ",
      "xlink",
    ]) {
      expect(flat).not.toContain(gone);
    }
    for (const kept of [
      "<rect",
      "<tspan",
      "Hola",
      "<circle",
      'viewBox="0 0 100 50"',
      "<text>y</text>",
    ]) {
      expect(flat).toContain(kept);
    }
  });

  it("drops the markers nothing uses and renames those it keeps", () => {
    document.head.innerHTML = "<style>path{marker-end:url(#a)}</style>";
    const flat = flattened(placed(DRAWING(false)), "p") ?? "";
    expect(flat).not.toContain("unused");
    expect(flat).toContain('<marker id="p-0"');
    expect(flat).toContain('marker-end="url(#p-0)"');
    expect(flat).not.toContain('id="a"');
  });

  it("gives its size from what it was drawn on, not from the page's style", () => {
    const flat = flattened(
      placed('<svg viewBox="0 0 120.4 60.6" style="max-width:9px"><path d="M0 0"></path></svg>'),
      "p",
    );
    expect(flat).toContain('width="120"');
    expect(flat).toContain('height="61"');
  });
});

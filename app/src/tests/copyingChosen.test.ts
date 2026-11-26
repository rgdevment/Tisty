import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../core", () => ({ copiedHtml: () => Promise.resolve() }));

import { chosenContents, chosenWhole, copiedAs } from "../ui/copying";

const CODE = `<figure class="code"><pre><code class="language-rust">fn a() {\n    let b = 1;\n    let c = 2;\n}\n</code></pre></figure>`;
const TABLE = `<div class="table"><table><thead><tr><th>Nombre</th><th>Precio</th></tr></thead><tbody><tr><td>a</td><td>$100</td></tr><tr><td>b</td><td>$200</td></tr></tbody></table></div>`;

const pageWith = (html: string): HTMLElement => {
  const page = document.createElement("div");
  page.className = "page";
  page.innerHTML = html;
  document.body.append(page);
  return page;
};

const textOf = (page: Element, selector: string): Text => {
  const found = page.querySelector(selector)?.firstChild;
  if (!found || found.nodeType !== Node.TEXT_NODE) throw new Error(`no text in ${selector}`);
  return found as Text;
};

const rangeOf = (start: Node, from: number, end: Node, to: number): Range => {
  const range = document.createRange();
  range.setStart(start, from);
  range.setEnd(end, to);
  return range;
};

const both = (range: Range) => ({
  now: copiedAs("", chosenContents(range)),
  old: copiedAs("", range.cloneContents()),
});

afterEach(() => {
  document.body.innerHTML = "";
});

describe("a partial selection keeps the shape it was chosen in", () => {
  it("keeps some lines of one code block as code", () => {
    const page = pageWith(CODE);
    const text = textOf(page, "code");
    const value = text.data;
    const from = value.indexOf("    let b");
    const to = value.indexOf("2;") + 2;
    const { now, old } = both(rangeOf(text, from, text, to));
    expect(now.plain).toBe("    let b = 1;\n    let c = 2;");
    expect(now.html).toMatch(/^<pre><code>/);
    expect(now.html).toContain("let b = 1;");
    expect(now.html).toContain("let c = 2;");
    expect(old.html).not.toMatch(/^<pre>/);
  });

  it("keeps two cells of one row as a table", () => {
    const page = pageWith(TABLE);
    const cells = page.querySelectorAll("tbody td");
    const { now, old } = both(
      rangeOf(cells[0].firstChild as Text, 0, cells[1].firstChild as Text, 2),
    );
    expect(now.html).toMatch(/^<table>/);
    expect(now.html).toContain("<tr><td>a</td><td>$1");
    expect(now.plain).toBe("a\t$1");
    expect(old.html).not.toMatch(/^<table>/);
  });

  it("keeps cells across rows as a table with both rows", () => {
    const page = pageWith(TABLE);
    const cells = page.querySelectorAll("tbody td");
    const { now, old } = both(
      rangeOf(cells[0].firstChild as Text, 0, cells[3].firstChild as Text, 4),
    );
    expect(now.html).toMatch(/^<table><tbody><tr>.*<\/tr><tr>.*<\/tr><\/tbody><\/table>$/);
    expect(now.plain).toBe("a\t$100\nb\t$200");
    expect(old.html).not.toMatch(/^<table>/);
  });

  it("goes on counting an ordered list with a start", () => {
    const page = pageWith(
      `<ol start="3"><li><span>uno</span></li><li><span>dos</span></li><li><span>tres</span></li></ol>`,
    );
    const dos = textOf(page, "li:nth-child(2) span");
    const tres = textOf(page, "li:nth-child(3) span");
    const { now, old } = both(rangeOf(dos, 1, tres, 4));
    expect(now.html).toMatch(/^<ol start="4">/);
    expect(now.plain).toBe("4. os\n5. tres");
    expect(old.html).not.toMatch(/^<ol/);
  });

  it("counts from 2 an ordered list without a start", () => {
    const page = pageWith(
      `<ol><li><span>uno</span></li><li><span>dos</span></li><li><span>tres</span></li></ol>`,
    );
    const dos = textOf(page, "li:nth-child(2) span");
    const tres = textOf(page, "li:nth-child(3) span");
    const { now, old } = both(rangeOf(dos, 0, tres, 4));
    expect(now.html).toContain('<ol start="2">');
    expect(now.plain).toBe("2. dos\n3. tres");
    expect(old.plain).not.toMatch(/^2\. /);
  });

  it("keeps a partial bulleted list as a list", () => {
    const page = pageWith(`<ul><li><span>uno</span></li><li><span>dos</span></li></ul>`);
    const uno = textOf(page, "li:nth-child(1) span");
    const dos = textOf(page, "li:nth-child(2) span");
    const { now, old } = both(rangeOf(uno, 1, dos, 2));
    expect(now.html).toMatch(/^<ul>/);
    expect(now.plain).toBe("- no\n- do");
    expect(old.html).not.toMatch(/^<ul>/);
  });

  it("keeps the bold and the link of one paragraph without its wrapper", () => {
    const page = pageWith(
      `<p><span>hola </span><strong><span>negrita</span></strong> <a href="https://example.com"><span>web</span></a> fin</p>`,
    );
    const first = textOf(page, "p > span");
    const web = textOf(page, "a span");
    const { now } = both(rangeOf(first, 1, web, 2));
    expect(now.html).not.toContain("<p>");
    expect(now.html).toBe('ola <strong>negrita</strong> <a href="https://example.com">we</a>');
  });

  it("adds no wrapper across two top-level paragraphs", () => {
    const page = pageWith(`<p><span>uno</span></p><p><span>dos</span></p>`);
    const a = textOf(page, "p:nth-child(1) span");
    const b = textOf(page, "p:nth-child(2) span");
    const { now } = both(rangeOf(a, 1, b, 2));
    expect(now.html).toMatch(/^<p>/);
    expect(now.html).toBe("<p>no</p><p>do</p>");
  });

  it("leaves words inside one list item as words", () => {
    const page = pageWith(
      `<ol start="2"><li><span>uno</span></li><li><span>dos tres</span></li></ol>`,
    );
    const text = textOf(page, "li:nth-child(2) span");
    const { now, old } = both(rangeOf(text, 0, text, 3));
    expect(now.html).toBe("dos");
    expect(now.plain).toBe("dos");
    expect(old.html).toBe("dos");
  });

  it("opens a list at the item the range starts in", () => {
    const page = pageWith("<ol><li>a</li><li>b</li><li>c</li></ol><p>tras</p>");
    const c = textOf(page, "li:nth-child(3)");
    const tras = textOf(page, "p");
    const { now, old } = both(rangeOf(c, 0, tras, 4));
    expect(now.plain).toBe("3. c\n\ntras");
    expect(now.html).toBe('<ol start="3"><li>c</li></ol><p>tras</p>');
    expect(old.plain).not.toBe("3. c\n\ntras");
  });

  it("counts from the child a range is anchored on in the list", () => {
    const page = pageWith("<ol><li>a</li><li>b</li><li>c</li></ol>");
    const ol = page.querySelector("ol") as Element;
    const c = textOf(page, "li:nth-child(3)");
    const { now, old } = both(rangeOf(ol, 1, c, 1));
    expect(now.plain).toBe("2. b\n3. c");
    expect(old.plain).not.toBe("2. b\n3. c");
  });

  it("leaves a word inside one cell as the word", () => {
    const page = pageWith(TABLE);
    const cell = page.querySelectorAll("tbody td")[1].firstChild as Text;
    const { now } = both(rangeOf(cell, 1, cell, 4));
    expect(now.plain).toBe("100");
    expect(now.html).toBe("100");
  });

  it("leaves the words of one heading without the heading", () => {
    const page = pageWith("<h1><span>Un título</span></h1>");
    const text = textOf(page, "span");
    const { now } = both(rangeOf(text, 3, text, 9));
    expect(now.html).not.toContain("<h1");
    expect(now.plain).toBe("título");
  });

  it("does not wrap words in the drawing of a formula", () => {
    const page = pageWith(
      '<span class="katex"><span class="katex-html" aria-hidden="true"><span>a</span><span>+b</span></span></span>',
    );
    const a = textOf(page, "span span span");
    const b = textOf(page, "span span span + span");
    const { now } = both(rangeOf(a, 0, b, 2));
    expect(now.plain).toBe("a+b");
    expect(now.html).not.toContain("aria-hidden");
  });
});

describe("a quote that opens with code", () => {
  it("keeps the indent of the first line", () => {
    const shown = document
      .createRange()
      .createContextualFragment(
        `<blockquote><figure class="code"><pre><code>    x = 1\n    y = 2\n</code></pre></figure></blockquote>`,
      );
    expect(copiedAs("", shown).plain).toBe("    x = 1\n    y = 2");
  });
});

describe("text that looks like a marker", () => {
  it("keeps a real U+E000 in prose", () => {
    const shown = document
      .createRange()
      .createContextualFragment("<p><span>prompt  listo</span></p>");
    const { plain } = copiedAs("", shown);
    expect(plain).toBe("prompt  listo");
    expect(plain).not.toContain("\n");
  });

  it("keeps blank lines and indent in code", () => {
    const shown = document
      .createRange()
      .createContextualFragment("<pre><code>a\n\n  b\n</code></pre>");
    expect(copiedAs("", shown).plain).toBe("a\n\n  b");
  });
});

describe("whole blocks chosen keep their list", () => {
  const LIST = "<ol><li>a</li><li>b</li><li>c</li><li>d</li></ol><p>tras</p>";

  const pick = (html: string, selectors: string[]) => {
    const page = pageWith(html);
    const chosen = selectors.map((one) => {
      const found = page.querySelector(one);
      if (!found) throw new Error(`no ${one}`);
      return found;
    });
    const old = document.createDocumentFragment();
    for (const one of chosen) old.append(one.cloneNode(true));
    return { now: copiedAs("", chosenWhole(chosen)), old: copiedAs("", old) };
  };

  it("counts on from the first chosen item", () => {
    const { now, old } = pick(LIST, ["li:nth-child(3)", "li:nth-child(4)"]);
    expect(now.plain).toBe("3. c\n4. d");
    expect(now.html).toBe('<ol start="3"><li>c</li><li>d</li></ol>');
    expect(old.plain).not.toBe("3. c\n4. d");
  });

  it("counts a single chosen item", () => {
    const { now, old } = pick(LIST, ["li:nth-child(3)"]);
    expect(now.plain).toBe("3. c");
    expect(old.plain).not.toBe("3. c");
  });

  it("separates the list from the paragraph after it", () => {
    const { now, old } = pick(LIST, ["li:nth-child(4)", "p"]);
    expect(now.plain).toBe("4. d\n\ntras");
    expect(now.html).toBe('<ol start="4"><li>d</li></ol><p>tras</p>');
    expect(old.plain).not.toBe("4. d\n\ntras");
  });

  it("adds the index to the list's own start", () => {
    const { now, old } = pick('<ol start="5"><li>x</li><li>y</li></ol>', ["li:nth-child(2)"]);
    expect(now.plain).toBe("6. y");
    expect(now.html).toBe('<ol start="6"><li>y</li></ol>');
    expect(old.plain).not.toBe("6. y");
  });

  it("keeps a bullet in its ul", () => {
    const { now, old } = pick("<ul><li>x</li><li>y</li></ul>", ["li:nth-child(2)"]);
    expect(now.plain).toBe("- y");
    expect(now.html).toBe("<ul><li>y</li></ul>");
    expect(old.html).not.toContain("<ul>");
  });

  it("keeps items of two lists in two lists", () => {
    const { now, old } = pick("<ol><li>a</li><li>b</li></ol><ol><li>c</li><li>d</li></ol>", [
      "ol:nth-of-type(1) li:nth-child(2)",
      "ol:nth-of-type(2) li:nth-child(1)",
    ]);
    expect(now.html).toBe('<ol start="2"><li>b</li></ol><ol><li>c</li></ol>');
    expect(old.html).not.toContain("<ol");
  });

  it("clones a block that is not an item as it is", () => {
    const { now, old } = pick("<p>solo</p>", ["p"]);
    expect(now.html).toBe("<p>solo</p>");
    expect(now.plain).toBe(old.plain);
  });
});

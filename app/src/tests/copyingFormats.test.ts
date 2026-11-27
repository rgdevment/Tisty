import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const board = vi.hoisted(() => ({ html: null as string | null, plain: null as string | null }));

vi.mock("../core", () => ({
  copiedHtml: (html: string, plain: string) => {
    board.html = html;
    board.plain = plain;
    return Promise.resolve();
  },
}));

import { TISTY_MARKDOWN } from "../ui/clipboarding";
import { clippedWords, copiedAs, copiedWords } from "../ui/copying";

const WORDS = "==amarillo== y ==azul=={blue}";

const PAGE = `<div aria-hidden="true">gutter</div><button>+</button><svg aria-hidden="true"><path d="M0 0"/></svg><span data-ghost>fantasma</span><div class="page-drop">DROPTXT</div><span class="page-drop-caret">CARETTXT</span><div class="tag-list">TAGLISTTXT</div><div class="diagram-bar">DIAGRAMTXT</div><div class="math-bar">MATHBARTXT</div><span class="picture-away">PICTUREAWAYTXT</span><span class="note-off">NOTEOFFTXT</span><span class="to-part">TOPARTTXT</span><h1 id="título" data-at="0-10"><span data-src="2-9">Título</span></h1><p data-at="11-95"><span data-src="11-21">Texto con </span><strong><span data-src="23-30">negrita</span></strong><span data-src="32-34">, </span><mark><span data-src="36-44">amarillo</span></mark><span data-src="46-49"> y </span><mark data-pen="blue"><span data-src="51-55">azul</span></mark><span data-src="63-69"> más </span><span class="ico" data-ico="star" data-hue="" title="star" data-src="69-77" contenteditable="false"></span><span data-src="77-80"> y </span><span class="ico" data-ico="heart" data-hue="red" title="heart" data-src="80-93" contenteditable="false"></span><span data-src="93-94">.</span></p><p data-at="96-222"><span data-src="96-104">Enlaces </span><a href="tisty:doc/abc-123"><span data-src="105-108">doc</span></a><span data-src="128-130">, </span><a href="https://example.com"><span data-src="131-134">web</span></a><span data-src="156-159"> y </span><a href="tisty:task/01ABC"><span data-src="160-161">t</span></a><span data-src="180-182">, </span><span class="ref" data-ref="Referencia" data-src="182-196" contenteditable="false">Referencia</span><span data-src="196-199"> y </span><span class="tag" data-tag="etiqueta" data-src="199-208" contenteditable="false">#etiqueta</span><span data-src="208-215">. Math </span><span data-src="215-220" contenteditable="false"><math><msup><mi>x</mi><mn>2</mn></msup></math></span><span data-src="220-221">.</span></p><ul data-at="223-272"><li class="task" data-at="223-263"><span class="check" data-done="false"></span><span data-src="229-244">tarea pendiente</span><ul data-at="247-263"><li class="task" data-at="247-263"><span class="check" data-done="true"></span><span data-src="253-262">sub hecha</span></li></ul></li><li data-at="263-272"><span data-src="265-270">punto</span></li></ul><ol data-at="272-287"><li data-at="272-279"><span data-src="275-278">uno</span></li><li data-at="279-287"><span data-src="282-285">dos</span></li></ol><details data-at="287-330"><summary>Ver más</summary><p data-at="308-325"><span data-src="308-315">Oculto </span><strong><span data-src="317-322">aquí</span></strong></p></details><div class="table" data-at="331-401"><table contenteditable="false"><colgroup><col><col></colgroup><thead><tr><th scope="col">Nombre</th><th scope="col" data-type="money" data-currency="CLP" data-total="sum" style="text-align:right">Precio</th></tr></thead><tbody><tr><td>a</td><td data-key="100" class="whole" style="text-align:right">$100</td></tr><tr><td>b</td><td data-key="200" class="whole" style="text-align:right">$200</td></tr></tbody><tfoot><tr><td>Total</td><td class="whole" style="text-align:right">$300</td></tr></tfoot></table></div><blockquote class="callout" data-callout="tip" data-at="402-440"><p data-at="404-440"><span data-src="411-418">Consejo</span><br data-src="418-419" contenteditable="false"><span data-src="421-439">Cuerpo del consejo</span></p></blockquote><figure class="code" data-at="441-463"><pre><code class="language-rust" data-src="449-460">let a = 1;
</code></pre></figure><p><span class="doc-card is-doc" contenteditable="false" tabindex="0" role="link" data-doc="abc-123"><span class="card-tile">CARDTILE</span><span class="card-name">Minuta semanal</span><span class="card-blurb">CARDBLURB</span><span class="card-meta">CARDMETA<button type="button" class="card-more" aria-label="more">⋯</button><span class="card-go">›</span></span></span></p>`;

const FURNITURE_TEXT = [
  "gutter",
  "fantasma",
  "DROPTXT",
  "CARETTXT",
  "TAGLISTTXT",
  "DIAGRAMTXT",
  "MATHBARTXT",
  "PICTUREAWAYTXT",
  "NOTEOFFTXT",
  "TOPARTTXT",
];

const fragment = (html: string, from: Document = document) =>
  from.createRange().createContextualFragment(html);

const shown = () => fragment(PAGE);

class FakeData {
  store = new Map<string, string>();
  setData(type: string, value: string) {
    this.store.set(type, value);
  }
}

const copyEvent = (data: FakeData | null) => {
  const event = new Event("copy", { cancelable: true, bubbles: true }) as ClipboardEvent;
  Object.defineProperty(event, "clipboardData", { value: data });
  return event;
};

const pageWithWindow = () => {
  const page = document.implementation.createHTMLDocument("t");
  Object.defineProperty(page, "defaultView", { value: window, configurable: true });
  return page;
};

beforeEach(() => {
  board.html = null;
  board.plain = null;
});

describe("copying the page as html", () => {
  const { html } = copiedAs(WORDS, shown());

  it("leaves no trace of the editor in the markup", () => {
    for (const trace of [
      "class=",
      "data-",
      "contenteditable",
      "tisty:",
      "<svg",
      "<button",
      "<span",
      "<div",
    ])
      expect(html).not.toContain(trace);
  });

  it("keeps the shapes another app understands", () => {
    expect(html).toContain("<strong>");
    expect(html).toContain("<table>");
    expect(html).toContain("<details>");
    expect(html).toContain("<summary>Ver más</summary>");
    expect(html).toContain('<mark style="background-color:#ccdefd">');
    expect(html).toContain('<mark style="background-color:#fdeeb4">');
    expect(html).toContain('href="https://example.com"');
  });

  it("turns the task boxes into characters", () => {
    expect(html).toContain("☐ ");
    expect(html).toContain("☑ ");
  });

  it("leaves links into Tisty as plain words", () => {
    expect(html).toContain("doc");
    expect(html).not.toMatch(/<a [^>]*href="tisty:/);
    expect(html).not.toContain(">doc</a>");
  });

  it("drops the furniture but keeps the table whole", () => {
    for (const text of FURNITURE_TEXT) expect(html).not.toContain(text);
    expect(html).not.toContain("+<");
    for (const cell of ["Nombre", "Precio", "100", "200", "$300", "Total"])
      expect(html).toContain(cell);
    expect(html).toContain("<thead>");
    expect(html).toContain("<tfoot>");
  });

  it("leaves a document card as its title alone", () => {
    expect(html).toContain("Minuta semanal");
    for (const text of ["CARDBLURB", "CARDMETA", "CARDTILE"]) expect(html).not.toContain(text);
  });

  it("keeps the colspan, rowspan and start that give the shape", () => {
    const out = copiedAs(
      "",
      fragment(
        '<ol start="3"><li>c</li></ol><table><tr><td colspan="2" rowspan="3" class="x">a</td></tr></table>',
      ),
    ).html;

    expect(out).toContain('<ol start="3">');
    expect(out).toContain('colspan="2"');
    expect(out).toContain('rowspan="3"');
    expect(out).not.toContain("class");
  });

  it("keeps mail links and drops script links", () => {
    const out = copiedAs(
      "",
      fragment('<p><a href="mailto:a@b.c">m</a><a href="javascript:alert(1)">j</a></p>'),
    ).html;

    expect(out).toContain('href="mailto:a@b.c"');
    expect(out).not.toContain("javascript");
    expect(out).toContain("j");
  });

  it("paints a pen with an unknown colour as the yellow one", () => {
    const out = copiedAs("", fragment('<mark data-pen="nope">x</mark>')).html;

    expect(out).toContain("#fdeeb4");
  });

  it("gives the words back untouched as Tisty markdown", () => {
    expect(copiedAs(WORDS, shown()).tisty).toBe(WORDS);
  });

  it("skips comments in the page", () => {
    expect(copiedAs("", fragment("<p>a<!-- no --></p>")).html).toBe("<p>a</p>");
  });
});

describe("copying the page as plain text", () => {
  const { plain } = copiedAs(WORDS, shown());
  const lines = plain.split("\n");

  it("marks each item of a list", () => {
    expect(lines).toContain("- punto");
    expect(lines).toContain("1. uno");
    expect(lines).toContain("2. dos");
  });

  it("indents the item inside another", () => {
    expect(lines).toContain("- ☐ tarea pendiente");
    expect(lines).toContain("  - ☑ sub hecha");
  });

  it("separates the cells of a row by a tab", () => {
    expect(lines).toContain("Nombre\tPrecio");
    expect(lines).toContain("a\t$100");
  });

  it("leaves no Tisty syntax behind", () => {
    for (const syntax of ["==", "::", "[[", "{blue}", "tisty:"])
      expect(plain).not.toContain(syntax);
  });

  it("keeps no triple blank lines nor trailing spaces", () => {
    expect(plain).not.toMatch(/\n{3,}/);
    expect(plain).not.toMatch(/[ \t]+\n/);
    expect(plain).toBe(plain.trim());
  });

  it("copies a run of spaces drawn with no-break spaces as plain spaces", () => {
    expect(copiedAs("", fragment("<p>uno   dos</p>")).plain).toBe("uno   dos");
  });

  it("drops the furniture", () => {
    for (const text of FURNITURE_TEXT) expect(plain).not.toContain(text);
  });

  it("leaves a document card as its title alone", () => {
    expect(plain).toContain("Minuta semanal");
    for (const text of ["CARDBLURB", "CARDMETA", "CARDTILE"]) expect(plain).not.toContain(text);
  });

  it("keeps the words of the page", () => {
    for (const word of [
      "Título",
      "negrita",
      "amarillo",
      "Referencia",
      "#etiqueta",
      "Ver más",
      "let a = 1;",
    ])
      expect(plain).toContain(word);
  });

  it("numbers a list from its start without blank lines between items", () => {
    expect(copiedAs("", fragment('<ol start="3"><li>c</li><li>d</li></ol>')).plain).toBe(
      "3. c\n4. d",
    );
  });

  it("breaks the line at a br", () => {
    expect(copiedAs("", fragment("<p>a<br>b</p>")).plain).toBe("a\nb");
  });
});

describe("filling a copy event", () => {
  it("writes the three types and takes the event over", () => {
    const data = new FakeData();
    const event = copyEvent(data);
    const expected = copiedAs(WORDS, shown());

    clippedWords(event, WORDS, shown());

    expect(event.defaultPrevented).toBe(true);
    expect(data.store.get(TISTY_MARKDOWN)).toBe(expected.tisty);
    expect(data.store.get("text/plain")).toBe(expected.plain);
    expect(data.store.get("text/html")).toBe(expected.html);
    expect(data.store.size).toBe(3);
  });

  it("does nothing without clipboard data", () => {
    const event = copyEvent(null);

    clippedWords(event, WORDS, shown());

    expect(event.defaultPrevented).toBe(false);
  });
});

describe("the menu's copy", () => {
  const raising = (data: FakeData | null, seen: string[] = []) => {
    const page = pageWithWindow();
    page.execCommand = () => {
      for (const name of ["beforecopy", "copy"]) {
        const event = new Event(name, { cancelable: true }) as ClipboardEvent;
        Object.defineProperty(event, "clipboardData", { value: data });
        window.dispatchEvent(event);
        seen.push(name + (event.defaultPrevented ? ":prevented" : ""));
      }
      return true;
    };
    return page;
  };

  it("fills every type through a copy event and never touches the fallback", async () => {
    const data = new FakeData();
    const seen: string[] = [];
    const outside = vi.fn();
    document.addEventListener("copy", outside);
    const expected = copiedAs(WORDS, shown());

    await copiedWords(WORDS, shown(), raising(data, seen));
    document.removeEventListener("copy", outside);

    expect(seen).toEqual(["beforecopy:prevented", "copy:prevented"]);
    expect(data.store.get(TISTY_MARKDOWN)).toBe(WORDS);
    expect(data.store.get("text/plain")).toBe(expected.plain);
    expect(data.store.get("text/html")).toBe(expected.html);
    expect(board.html).toBeNull();
    expect(outside).not.toHaveBeenCalled();
  });

  it("falls back to html and words when no event arrives", async () => {
    const page = pageWithWindow();
    page.execCommand = () => false;
    const expected = copiedAs(WORDS, shown());

    await copiedWords(WORDS, shown(), page);

    expect(board.html).toBe(expected.html);
    expect(board.plain).toBe(expected.plain);
  });

  it("falls back when the page has no window", async () => {
    const page = document.implementation.createHTMLDocument("t");

    await copiedWords(WORDS, shown(), page);

    expect(board.html).toBe(copiedAs(WORDS, shown()).html);
  });

  it("falls back when the event carries no clipboard data", async () => {
    await copiedWords(WORDS, shown(), raising(null));

    expect(board.html).not.toBeNull();
  });

  it("lets go of its listeners afterwards", async () => {
    await copiedWords(WORDS, shown(), raising(new FakeData()));
    const data = new FakeData();
    const later = copyEvent(data);

    window.dispatchEvent(later);

    expect(later.defaultPrevented).toBe(false);
    expect(data.store.size).toBe(0);
  });

  it("falls back to html and words, and lets go of its listeners, when the copy command throws", async () => {
    const page = pageWithWindow();
    page.execCommand = () => {
      throw new Error("no");
    };

    await copiedWords(WORDS, shown(), page);

    expect(board.html).not.toBeNull();
    expect(board.plain).not.toBeNull();

    const later = copyEvent(new FakeData());
    window.dispatchEvent(later);
    expect(later.defaultPrevented).toBe(false);
  });
});

describe("a page from another document", () => {
  it("is cleaned with the document it came from", () => {
    const other = document.implementation.createHTMLDocument("frame");
    const part = fragment(PAGE, other);

    const out = copiedAs(WORDS, part);

    expect(part.ownerDocument).toBe(other);
    expect(out.html).toContain("<strong>");
    expect(out.html).not.toContain("<button");
    expect(out.plain).toContain("- punto");
  });
});

describe("math, pictures and rules", () => {
  const squared = "<p>a <math><msup><mi>x</mi><mn>2</mn></msup></math> b</p>";
  const half =
    "<p>c <math><mfrac><mrow><mi>a</mi><mo>+</mo><mi>b</mi></mrow><mn>2</mn></mfrac></math> d</p>";

  it("keeps the math in the html", () => {
    const html = copiedAs("", fragment(squared + half)).html;

    for (const tag of ["<math>", "<msup>", "<mi>", "<mfrac>"]) expect(html).toContain(tag);
  });

  it("says a power with a caret", () => {
    expect(copiedAs("", fragment(squared)).plain).toContain("x^2");
  });

  it("says a fraction with a slash and its top in brackets", () => {
    expect(copiedAs("", fragment(half)).plain).toContain("(a+b)/2");
  });

  it("keeps a picture from the web with its description", () => {
    const { html, plain } = copiedAs(
      "",
      fragment(
        '<p><img src="https://example.com/a.png" alt="Gato" loading="lazy" data-src="1-2"></p>',
      ),
    );

    expect(html).toContain('<img src="https://example.com/a.png" alt="Gato">');
    expect(html).not.toContain("data-");
    expect(plain).toBe("Gato");
  });

  it("leaves a picture of this machine as its description alone", () => {
    const { html, plain } = copiedAs(
      "",
      fragment('<p><img src="asset://localhost/x.png" alt="Local"></p>'),
    );

    expect(html).toBe("<p>Local</p>");
    expect(plain).toBe("Local");
  });

  it("says a rule as a line of dashes", () => {
    expect(copiedAs("", fragment("<p>a</p><hr><p>b</p>")).plain.split("\n")).toContain("---");
  });

  it("brackets a fraction part that is itself a power", () => {
    const html = "<p><math><mfrac><msup><mi>x</mi><mn>2</mn></msup><mn>2</mn></mfrac></math></p>";

    expect(copiedAs("", fragment(html)).plain).toBe("(x^2)/2");
  });

  it("says a long exponent without brackets", () => {
    const html = "<p><math><msup><mi>x</mi><mn>10</mn></msup></math></p>";

    expect(copiedAs("", fragment(html)).plain).toBe("x^10");
  });

  it("says a symbol with a subscript and a power", () => {
    const html = "<p><math><msubsup><mi>x</mi><mi>i</mi><mn>2</mn></msubsup></math></p>";

    expect(copiedAs("", fragment(html)).plain).toBe("x_i^2");
  });

  it("keeps a math block displayed as a block", () => {
    const html =
      '<div class="math" data-at="42-59"><math display="block"><msup><mi>x</mi><mn>2</mn></msup><mo>+</mo><mn>1</mn></math></div>';

    expect(copiedAs("", fragment(html)).html).toContain('<math display="block">');
  });
});

describe("the letter that shows nothing", () => {
  const UNSEEN = String.fromCharCode(0x200b);

  const onlyTheLetter = () => {
    const part = document.createDocumentFragment();
    part.appendChild(document.createTextNode(UNSEEN));
    return part;
  };

  it("leaves a fragment of only the letter with nothing to paste", () => {
    const out = copiedAs(WORDS, onlyTheLetter());

    expect(out.plain).toBe("");
    expect(out.html).toBe("");
    expect(out.tisty).toBe(WORDS);
  });

  it("leaves blocks holding only the letter with nothing to paste", () => {
    const out = copiedAs("", fragment(`<p>${UNSEEN}</p><p>${UNSEEN}</p>`));

    expect(out.plain).toBe("");
    expect(out.html).toBe("");
  });

  it("takes the letter out from beside the pieces", () => {
    const out = copiedAs("", fragment(`<p>a${UNSEEN}<strong>b</strong>${UNSEEN}</p>`));

    expect(out.plain).toBe("ab");
    expect(out.html).toBe("<p>a<strong>b</strong></p>");
    expect(out.html).not.toContain(UNSEEN);
  });

  it("still keeps a picture from the web that has no words", () => {
    const out = copiedAs("", fragment('<img src="https://example.com/a.png" alt="">'));

    expect(out.html).not.toBe("");
    expect(out.html).toContain("<img");
  });

  it("writes empty text and html but the words as Tisty markdown", () => {
    const data = new FakeData();

    clippedWords(copyEvent(data), WORDS, onlyTheLetter());

    expect(data.store.get("text/plain")).toBe("");
    expect(data.store.get("text/html")).toBe("");
    expect(data.store.get(TISTY_MARKDOWN)).toBe(WORDS);
  });
});

describe("what a copy never loses", () => {
  const UNSEEN = String.fromCharCode(0x200b);
  const mounted: HTMLElement[] = [];

  const mount = (html: string) => {
    const host = document.createElement("div");
    host.innerHTML = html;
    document.body.appendChild(host);
    mounted.push(host);
    return host;
  };

  afterEach(() => {
    for (const host of mounted.splice(0)) host.remove();
  });

  const textNodesOf = (host: HTMLElement) => {
    const found: Text[] = [];
    const walker = document.createTreeWalker(host, NodeFilter.SHOW_TEXT);
    for (let at = walker.nextNode(); at; at = walker.nextNode()) found.push(at as Text);
    return found;
  };

  const FURNITURE = [
    "button",
    "svg",
    "input",
    "script",
    "style",
    "[data-ghost]",
    "[aria-hidden='true']",
    ".ico",
    ".page-drop",
    ".page-drop-caret",
    ".tag-list",
    ".diagram-bar",
    ".math-bar",
    ".picture-away",
    ".note-off",
    ".to-part",
    ".check",
  ].join(", ");

  const visibleWords = (clone: DocumentFragment): string[] => {
    const box = document.createElement("div");
    box.appendChild(clone);
    for (const card of Array.from(box.querySelectorAll(".doc-card"))) {
      const title = card.querySelector(".card-name")?.textContent ?? "";
      card.replaceWith(document.createTextNode(title));
    }
    for (const one of Array.from(box.querySelectorAll(FURNITURE))) one.remove();
    return textNodesOf(box).flatMap((one) => one.data.split(UNSEEN).join("").match(/\S+/g) ?? []);
  };

  const bare = (text: string) => text.replace(/[\s^_/()]/g, "");

  describe("the words as written", () => {
    const WRITTEN = [
      "ne**gri",
      "[enlace](https://exa",
      "| a | b |\n|---|---|\n| c",
      "==amarillo== y ==azul=={blue}",
      "",
    ];

    it("comes back byte for byte whatever the page shows", () => {
      for (const words of WRITTEN) expect(copiedAs(words, shown()).tisty).toBe(words);
    });

    it("is what a copy event writes as Tisty markdown", () => {
      for (const words of WRITTEN) {
        const data = new FakeData();

        clippedWords(copyEvent(data), words, shown());

        expect(data.store.get(TISTY_MARKDOWN)).toBe(words);
      }
    });
  });

  describe("the words that show", () => {
    it("all reach the plain text from any part of the page", () => {
      const host = mount(PAGE);
      const nodes = textNodesOf(host);
      const lost: string[] = [];
      let tried = 0;

      for (let from = 0; from < nodes.length; from += 3) {
        const start = nodes[from];
        const startAts = [0, 1, Math.floor(start.length / 2)];
        for (const gap of [0, 1, 4]) {
          const end = nodes[Math.min(from + gap, nodes.length - 1)];
          const endAts = [end.length, Math.floor(end.length / 2)];
          for (const startAt of startAts)
            for (const endAt of endAts) {
              if (end === start && endAt < startAt) continue;
              const range = document.createRange();
              range.setStart(start, Math.min(startAt, start.length));
              range.setEnd(end, endAt);
              const clone = range.cloneContents();
              const clean = bare(copiedAs("", range.cloneContents()).plain);
              tried++;
              for (const word of visibleWords(clone))
                if (bare(word) !== "" && !clean.includes(bare(word)))
                  lost.push(`${from}+${gap} [${startAt},${endAt}] ${word}`);
            }
        }
      }

      expect(tried).toBeGreaterThan(100);
      expect(lost).toEqual([]);
    });
  });

  describe("the edges", () => {
    it("gives nothing for an empty fragment", () => {
      const out = copiedAs("", document.createDocumentFragment());

      expect(out.plain).toBe("");
      expect(out.html).toBe("");
    });

    it("gives a cell chosen from inside as its text", () => {
      const host = mount("<table><tbody><tr><td>uno</td><td>dos</td></tr></tbody></table>");
      const text = host.querySelector("td")?.firstChild as Text;
      const range = document.createRange();
      range.setStart(text, 1);
      range.setEnd(text, 3);

      expect(copiedAs("", range.cloneContents()).plain).toBe("no");
    });

    it("gives a whole cell as its text with no tab after it", () => {
      const host = mount("<table><tbody><tr><td>uno</td><td>dos</td></tr></tbody></table>");
      const cell = host.querySelector("td") as Element;
      const range = document.createRange();
      range.selectNode(cell);

      expect(copiedAs("", range.cloneContents()).plain).toBe("uno");
    });

    it("keeps a block of code exactly", () => {
      const code = "fn a() {\n    b();\n\n}";
      const out = copiedAs(
        "",
        fragment(`<figure class="code"><pre><code>${code}</code></pre></figure>`),
      );

      expect(out.plain).toBe(code);
      expect(out.html).toContain(`<pre><code>${code}</code></pre>`);
    });

    it("gives a formula alone as math and as a power", () => {
      const out = copiedAs("", fragment("<p><math><msup><mi>x</mi><mn>2</mn></msup></math></p>"));

      expect(out.html).toContain("<math>");
      expect(out.plain).toBe("x^2");
    });

    it("gives a picture from the web alone", () => {
      const out = copiedAs("", fragment('<img src="https://example.com/a.png" alt="Gato">'));

      expect(out.html).toBe('<img src="https://example.com/a.png" alt="Gato">');
      expect(out.plain).toBe("Gato");
    });

    it("keeps the title and the first paragraph on lines of their own", () => {
      const host = mount(PAGE);
      const title = host.querySelector("h1 span")?.firstChild as Text;
      const first = host.querySelector("p span")?.firstChild as Text;
      const range = document.createRange();
      range.setStart(title, 0);
      range.setEnd(first, first.length);

      const lines = copiedAs("", range.cloneContents()).plain.split("\n");

      expect(lines).toContain("Título");
      expect(lines).toContain("Texto con");
    });
  });
});

describe("an icon left out", () => {
  const ICON = '<span class="ico" data-ico="star" data-hue="" title="star"></span>';

  it("leaves one space where it sat between two words", () => {
    const out = copiedAs("", fragment(`<p>con ${ICON} y</p>`));

    expect(out.plain).toBe("con y");
    expect(out.html).toBe("<p>con y</p>");
  });

  it("leaves no double space at the start of the line", () => {
    const out = copiedAs("", fragment(`<p>${ICON} Hola</p>`));

    expect(out.plain).toBe("Hola");
    expect(out.html).not.toMatch(/^\s{2}/);
    expect(out.html).not.toContain("  ");
  });

  it("adds no space between two words it joined", () => {
    const out = copiedAs("", fragment(`<p>a${ICON}b</p>`));

    expect(out.plain).toBe("ab");
    expect(out.html).toBe("<p>ab</p>");
  });

  it("keeps the indent of a block of code after a button", () => {
    const code = "fn a() {\n    b();\n        c();\n}";
    const out = copiedAs(
      "",
      fragment(
        `<figure class="code"><button class="code-lang">rust</button><pre><code>${code}</code></pre></figure>`,
      ),
    );

    expect(out.plain).toBe(code);
    expect(out.html).toContain(`<pre><code>${code}</code></pre>`);
  });

  it("leaves one space when two icons sit between the words", () => {
    const out = copiedAs("", fragment(`<p>con ${ICON} ${ICON} y</p>`));

    expect(out.plain).toBe("con y");
    expect(out.html).toBe("<p>con y</p>");
  });
});

describe("the indent a copy opens with", () => {
  const CODE = "    b();\n        c();\n}";

  it("keeps the indent of the first line of a block of code after a button", () => {
    const { plain } = copiedAs(
      "",
      fragment(
        `<figure class="code"><button class="code-lang">rust</button><pre><code>${CODE}</code></pre></figure>`,
      ),
    );

    expect(plain.startsWith("    b();")).toBe(true);
    expect(plain).toContain("\n        c();");
  });

  it("keeps it too when the code has no figure around it", () => {
    const { plain } = copiedAs("", fragment(`<pre><code>${CODE}</code></pre>`));

    expect(plain.startsWith("    b();")).toBe(true);
    expect(plain).toContain("\n        c();");
  });

  it("starts a paragraph at its first word", () => {
    expect(copiedAs("", fragment("<p>   Hola</p>")).plain).toBe("Hola");
  });

  it("starts at the text before a block of code and leaves the code its indent", () => {
    const { plain } = copiedAs("", fragment(`<p>Antes</p><pre><code>${CODE}</code></pre>`));

    expect(plain.startsWith("Antes")).toBe(true);
    expect(plain.split("\n")).toContain("    b();");
    expect(plain.split("\n")).toContain("        c();");
  });

  it("ends with no spaces nor line breaks, even when the code ends in one", () => {
    const code = copiedAs("", fragment("<pre><code>    b();\n</code></pre>")).plain;
    const text = copiedAs("", fragment("<p>Hola   </p>")).plain;

    expect(code).toBe("    b();");
    expect(text).toBe("Hola");
  });
});

describe("an icon as the page draws it, between the spans of the words", () => {
  const DRAWN =
    '<span class="ico" data-ico="star" data-hue="" data-src="5-13" contenteditable="false"><svg viewBox="0 0 1 1"><path d="M0 0"/></svg></span>';
  const words = (text: string, at = "0-1") => `<span data-src="${at}">${text}</span>`;

  it("leaves one space in the plain text and the html between the words", () => {
    const out = copiedAs("", fragment(`<p>${words("medio ")}${DRAWN}${words(" luego")}</p>`));

    expect(out.plain).toBe("medio luego");
    expect(out.html).toBe("<p>medio luego</p>");
  });

  it("adds no blank when the choice starts with the icon", () => {
    const out = copiedAs("", fragment(`<p>${DRAWN}${words(" luego")}</p>`));

    expect(out.plain).toBe("luego");
    expect(out.html).not.toContain("  ");
  });

  it("keeps the indent of a code block that follows a paragraph ending in an icon", () => {
    const code = "<pre><code>    let x = 1;\n</code></pre>";
    const out = copiedAs("", fragment(`<p>${words("texto ")}${DRAWN}</p>${code}`));

    expect(out.html).toContain("<pre><code>    let x = 1;\n</code></pre>");
    expect(out.plain).toContain("    let x = 1;");
  });

  it("keeps the words of a second paragraph that opens with a space after a trailing icon", () => {
    const out = copiedAs("", fragment(`<p>${words("texto ")}${DRAWN}</p><p>${words(" otro")}</p>`));

    expect(out.plain).toContain("texto");
    expect(out.plain).toContain("otro");
    expect(out.html).toContain("otro");
  });

  it("adds no blank when the choice ends with the icon", () => {
    const out = copiedAs("", fragment(`<p>${words("medio")}${DRAWN}</p>`));

    expect(out.plain).toBe("medio");
    expect(out.html).toBe("<p>medio</p>");
  });

  it("leaves one space when two icons sit between the words", () => {
    const out = copiedAs(
      "",
      fragment(`<p>${words("medio ")}${DRAWN}${words(" ")}${DRAWN}${words(" luego")}</p>`),
    );

    expect(out.plain).toBe("medio luego");
    expect(out.html).toBe("<p>medio luego</p>");
  });

  it("leaves one space before words drawn inside a strong", () => {
    const out = copiedAs(
      "",
      fragment(`<p>${words("medio ")}${DRAWN}<strong>${words(" fuerte")}</strong></p>`),
    );

    expect(out.plain).toBe("medio fuerte");
    expect(out.html).toBe("<p>medio <strong>fuerte</strong></p>");
  });

  it("still says the title of a document card after a dropped icon", () => {
    const card =
      '<span class="doc-card is-doc" contenteditable="false"><span class="card-name">Minuta</span></span>';
    const out = copiedAs("", fragment(`<p>${words("medio ")}${DRAWN}${card}</p>`));

    expect(out.plain).toBe("medio Minuta");
  });

  it("still says the box of a task after a dropped icon", () => {
    const check = '<span class="check" data-done="false"></span>';
    const out = copiedAs(
      "",
      fragment(
        `<ul><li class="task">${words("medio ")}${DRAWN}${check}${words("tarea")}</li></ul>`,
      ),
    );

    expect(out.plain).toContain("medio ☐ tarea");
  });
});

describe("a no-break space in what is copied", () => {
  const NB = " ";

  it("goes to the plain text as an ordinary space", () => {
    const out = copiedAs("", fragment(`<p><span data-src="0-8">ls${NB}${NB}-la</span></p>`));

    expect(out.plain).toBe("ls  -la");
    expect(out.plain).not.toContain(NB);
  });

  it("goes as spaces too in the indent of a second line", () => {
    const out = copiedAs(
      "",
      fragment(
        `<p><span data-src="0-3">uno</span><br><span data-src="4-12">${NB.repeat(4)}dos</span></p>`,
      ),
    );

    expect(out.plain).not.toContain(NB);
    expect(out.plain).toBe("uno\n    dos");
  });

  it("goes as a space from the letters of a block of code", () => {
    const out = copiedAs("", fragment(`<pre><code>a${NB}b\n</code></pre>`));

    expect(out.plain).toBe("a b");
    expect(out.plain).not.toContain(NB);
  });

  it("may stay in the html, and the Tisty words keep it as written", () => {
    const words = `ls${NB}${NB}-la`;
    const out = copiedAs(words, fragment(`<p><span data-src="0-8">${words}</span></p>`));

    expect(out.tisty).toBe(words);
    expect(out.html).toContain("ls");
    expect(out.html).toContain("-la");
  });
});

describe("the line breaks of a block of code in the plain text", () => {
  const MARKER = String.fromCharCode(0xe000);
  const MIXED =
    '<p><span data-src="0-5">Antes</span></p><pre><code>a\n\n\n\nb\n</code></pre><p><span data-src="9-14">Luego</span></p>';

  it("keeps every blank line the code has", () => {
    const out = copiedAs("", fragment("<pre><code>a\n\n\n\nb\n</code></pre>"));

    expect(out.plain).toBe("a\n\n\n\nb");
  });

  it("keeps the trailing spaces of a code line", () => {
    const out = copiedAs("", fragment("<pre><code>x  \ny\n</code></pre>"));

    expect(out.plain).toBe("x  \ny");
  });

  it("leaves one blank line between a paragraph, a code block and a paragraph", () => {
    const out = copiedAs("", fragment(MIXED));

    expect(out.plain).toBe("Antes\n\na\n\n\n\nb\n\nLuego");
  });

  it("leaves no triple newline outside the code", () => {
    const out = copiedAs(
      "",
      fragment(
        '<p><span data-src="0-5">Antes</span></p><pre><code>a\nb\n</code></pre><p><span data-src="9-14">Luego</span></p>',
      ),
    );

    expect(out.plain).toBe("Antes\n\na\nb\n\nLuego");
    expect(out.plain).not.toContain("\n\n\n");
  });

  it("never lets the private marker reach the plain text or the html", () => {
    for (const shown of [
      MIXED,
      "<pre><code>x  \ny\n</code></pre>",
      "<pre><code>a\n\nb</code></pre>",
    ]) {
      const out = copiedAs("", fragment(shown));

      expect(out.plain).not.toContain(MARKER);
      expect(out.html).not.toContain(MARKER);
    }
  });
});

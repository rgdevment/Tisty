import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { heads, shell, went } from "../ui/paging";

const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));

const page = (html: string): Document => new DOMParser().parseFromString(shell(html), "text/html");

const clicked = (html: string, selector: string, selecting = false) => {
  const one = page(html).querySelector(selector);
  if (!one) throw new Error(`nothing matches ${selector}`);
  return went(one, selecting);
};

describe("a click on a drawn page", () => {
  it("ticks a stage from its box, and opens the block from its words", () => {
    const list =
      '<ul data-at="0-20"><li class="task" data-at="0-10"><span class="check" data-done="false"></span>uno</li></ul>';
    expect(clicked(list, ".check")).toEqual({ kind: "tick", at: [0, 10] });
    expect(clicked(list, "li")).toEqual({ kind: "block", at: [0, 20] });
    expect(clicked(list, ".check", true)).toBeNull();
  });

  it("opens a card or a tisty link as the document it names", () => {
    expect(clicked('<p data-at="0-9"><a class="card" data-doc="ab12-0001">A</a></p>', "a")).toEqual(
      {
        kind: "doc",
        id: "ab12-0001",
      },
    );
    expect(clicked('<p data-at="0-9"><a href="tisty:doc/ab12-0002">B</a></p>', "a")).toEqual({
      kind: "doc",
      id: "ab12-0002",
    });
  });

  it("sends the web and mail outside, and an attachment to the app", () => {
    expect(clicked('<p data-at="0-9"><a href="https://x.y/z">x</a></p>', "a")).toEqual({
      kind: "web",
      href: "https://x.y/z",
    });
    expect(clicked('<p data-at="0-9"><a href="mailto:a@b.c">x</a></p>', "a")).toEqual({
      kind: "web",
      href: "mailto:a@b.c",
    });
    expect(clicked('<p data-at="0-9"><a href="attachments/a.pdf">x</a></p>', "a")).toEqual({
      kind: "attachment",
      reference: "attachments/a.pdf",
    });
  });

  it("follows nothing it does not know, and nothing a link was cleaned out of", () => {
    expect(clicked('<p data-at="0-9"><a href="#">x</a></p>', "a")).toBeNull();
    expect(clicked('<p data-at="0-9"><a href="ftp://x.y">x</a></p>', "a")).toBeNull();
  });

  it("names the whole block it fell on, by the bytes it was written in", () => {
    expect(clicked('<ul data-at="4-30"><li data-at="4-12">one</li></ul>', "li")).toEqual({
      kind: "block",
      at: [4, 30],
    });
  });

  it("opens a widget's code or the widget apart from its bar, and nothing from the widget itself", () => {
    const widget =
      '<figure class="widget" data-at="4-40"><div class="widget-bar"><span class="widget-name">Rina</span><button data-asked="code">c</button><button data-asked="apart">a</button></div><div><iframe></iframe></div></figure>';
    expect(clicked(widget, "button[data-asked=code]")).toEqual({ kind: "block", at: [4, 40] });
    expect(clicked(widget, "button[data-asked=apart]")).toEqual({ kind: "apart", at: [4, 40] });
    const bare = widget.replace(' data-at="4-40"', "");
    const two = page(`<ul data-at="4-90"><li>${bare}</li><li>${bare}</li></ul>`);
    const second = two.querySelectorAll("button[data-asked=apart]")[1];
    expect(went(second, false)).toEqual({ kind: "apart", at: [4, 90], nth: 1 });
    expect(clicked(widget, ".widget-name")).toBeNull();
    expect(clicked(widget, "iframe")).toBeNull();
    expect(
      clicked('<div class="widget-bar"><button data-asked="code">c</button></div>', "button"),
    ).toBeNull();
  });

  it("opens a drawn card's document, and its block from its edit button", () => {
    const card =
      '<p data-at="4-30"><span class="doc-card" data-doc="ab12-0003"><span class="card-name">C</span><button data-asked="code">e</button></span></p>';
    expect(clicked(card, ".card-name")).toEqual({ kind: "doc", id: "ab12-0003" });
    expect(clicked(card, "button")).toEqual({ kind: "block", at: [4, 30] });
  });

  it("asks nothing while words are being picked", () => {
    expect(clicked('<p data-at="0-9">words</p>', "p", true)).toBeNull();
  });

  it("hands a tag back by its name", () => {
    expect(
      clicked('<p data-at="0-9"><span class="tag" data-tag="work">#work</span></p>', "span"),
    ).toEqual({ kind: "tag", name: "work" });
  });
});

describe("the outline of a drawn page", () => {
  it("reads the headings in order with their level", () => {
    const found = heads(
      page('<h1 id="a" data-at="0-3">A</h1><p>x</p><h3 id="b" data-at="5-9">B</h3>'),
    );
    expect(found.map((one) => [one.level, one.text])).toEqual([
      [1, "A"],
      [3, "B"],
    ]);
  });
});

describe("the window's policy", () => {
  it("lets no document change its base, embed objects or post a form", () => {
    for (const said of [conf.app.security.csp, conf.app.security.devCsp]) {
      expect(said).toContain("base-uri 'none'");
      expect(said).toContain("object-src 'none'");
      expect(said).toContain("form-action 'none'");
    }
  });

  it("runs no inline script in production", () => {
    const scripts = /script-src ([^;]*)/.exec(conf.app.security.csp)?.[1] ?? "";
    expect(scripts).not.toContain("'unsafe-inline'");
    expect(scripts).not.toContain("'unsafe-eval'");
  });
});

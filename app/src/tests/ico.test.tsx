import { Editor } from "@tiptap/core";
import { describe, expect, it } from "vitest";
import { asMarkdown, written } from "../ui/writing";

const made = (content = "") => new Editor({ extensions: written(), content });

const md = (e: Editor) => asMarkdown(e) ?? "";

describe("an icon written into a document", () => {
  it("goes out as markup carrying its name, which another reader can still show", () => {
    const editor = made("<p>hola</p>");
    editor.commands.insertContent({ type: "ico", attrs: { name: "rocket", hue: null } });
    const out = md(editor);
    editor.destroy();

    expect(out).toContain('<span data-ico="rocket">:rocket:</span>');
  });

  it("carries the colour along when it was given one", () => {
    const editor = made("<p>hola</p>");
    editor.commands.insertContent({ type: "ico", attrs: { name: "bread", hue: "teal" } });
    const out = md(editor);
    editor.destroy();

    expect(out).toContain('data-ico="bread"');
    expect(out).toContain('data-hue="teal"');
  });

  it("is read back from a document that already carried one", () => {
    const editor = made('<p>ayer <span data-ico="pill" data-hue="pink">:pill:</span></p>');
    const out = md(editor);
    editor.destroy();

    expect(out).toContain('data-ico="pill"');
    expect(out).toContain('data-hue="pink"');
  });

  it("keeps the name a reader sees even when the file was written with an emoji", () => {
    const editor = made('<p><span data-ico="bread">🍞</span></p>');
    const out = md(editor);
    editor.destroy();

    expect(out).toContain('data-ico="bread"');
    expect(out).toContain(":bread:");
  });

  it("hangs a drawing where the name was, rather than the name itself", () => {
    const editor = made('<p><span data-ico="star"></span></p>');
    const held = editor.view.dom.querySelector(".ico");
    const drawn = held?.querySelector("svg");
    editor.destroy();

    expect(held?.getAttribute("data-ico")).toBe("star");
    expect(drawn?.getAttribute("viewBox")).toBe("0 0 24 24");
    expect(held?.textContent).toBe("");
  });

  it("wears the colour as a class, so the theme can move it", () => {
    const editor = made('<p><span data-ico="pill" data-hue="pink"></span></p>');
    const held = editor.view.dom.querySelector(".ico");
    editor.destroy();

    expect(held?.className).toContain("ico-pink");
  });

  it("says nothing extra for an icon with no colour", () => {
    const editor = made('<p><span data-ico="pill"></span></p>');
    const held = editor.view.dom.querySelector(".ico");
    editor.destroy();

    expect(held?.className).toBe("ico");
    expect(held?.getAttribute("data-hue")).toBeNull();
  });

  it("stands its ground for a name the set does not carry, rather than falling over", () => {
    const editor = made('<p><span data-ico="nosuchthing"></span></p>');
    const held = editor.view.dom.querySelector(".ico");
    const out = asMarkdown(editor) ?? "";
    editor.destroy();

    expect(held).toBeTruthy();
    expect(out).toContain('data-ico="nosuchthing"');
  });
});

describe("an icon a hand edit or a paste put in the file", () => {
  it("goes back as text, never as markup of its own", () => {
    const editor = made(
      '<p>x <span data-ico="&quot; onmouseover=alert(1) a=&quot;">boom</span></p>',
    );
    const out = md(editor);
    editor.destroy();

    expect(out).not.toContain('onmouseover=alert(1) a=""');
    expect(out).toContain("&quot;");
  });

  it("keeps a colour it cannot paint from breaking out of its own attribute", () => {
    const editor = made(
      '<p><span data-ico="pill" data-hue="&quot;><script>alert(1)</script>">z</span></p>',
    );
    const out = md(editor);
    editor.destroy();

    expect(out).not.toContain("<script>");
  });

  it("still carries a name this build cannot draw, so a newer one gets it back", () => {
    const editor = made('<p><span data-ico="not-drawn-here">:not-drawn-here:</span></p>');
    const out = md(editor);
    editor.destroy();

    expect(out).toContain('data-ico="not-drawn-here"');
  });
});

describe("an icon alone in a table cell", () => {
  it("survives the save that used to erase the cell", () => {
    const editor = made(
      "<table><tbody><tr><td><p>a</p></td><td><p>b</p></td></tr>" +
        '<tr><td><p><span data-ico="bread">:bread:</span></p></td><td><p>y</p></td></tr></tbody></table>',
    );
    const out = md(editor);
    editor.destroy();

    expect(out).toContain('data-ico="bread"');
  });
});

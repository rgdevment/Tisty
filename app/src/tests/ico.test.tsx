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
    const editor = made('<p><span data-ico="rocket"></span></p>');
    const held = editor.view.dom.querySelector(".ico");
    const drawn = held?.querySelector("svg");
    editor.destroy();

    expect(held?.getAttribute("data-ico")).toBe("rocket");
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

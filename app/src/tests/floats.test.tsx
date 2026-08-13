import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { Markdown } from "tiptap-markdown";
import Floats from "../ui/Floats";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const made = () =>
  new Editor({
    extensions: [StarterKit, Markdown.configure({ html: true, breaks: true })],
    content: "hola mundo",
  });

const md = (e: Editor) =>
  (e.storage as unknown as { markdown: { getMarkdown: () => string } }).markdown.getMarkdown();

describe("the panel that appears over a selection", () => {
  it("offers only what the document can keep", () => {
    const editor = made();
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    const names = screen.getAllByRole("button").map((one) => one.getAttribute("aria-label"));
    expect(names).toEqual(["Bold", "Italic", "Underline", "Strikethrough", "Code"]);

    editor.destroy();
  });

  it("marks what the text already is, for whoever cannot see it", () => {
    const editor = made();
    editor.chain().selectAll().toggleBold().run();
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    expect(screen.getByRole("button", { name: "Bold" }).getAttribute("aria-pressed")).toBe("true");
    expect(screen.getByRole("button", { name: "Italic" }).getAttribute("aria-pressed")).toBe(
      "false",
    );

    editor.destroy();
  });

  it("writes the format into the document, not just onto the screen", async () => {
    const editor = made();
    editor.commands.selectAll();
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Underline" }));

    expect(md(editor)).toBe("<u>hola mundo</u>");
    editor.destroy();
  });

  it("never steals the selection it is acting on", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Bold" }));

    expect(md(editor)).toBe("**hola** mundo");
    editor.destroy();
  });
});

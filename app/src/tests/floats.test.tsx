import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Editor } from "@tiptap/core";
import { asMarkdown, written } from "../ui/writing";
import Floats from "../ui/Floats";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const made = (content = "hola mundo") => new Editor({ extensions: written(), content });

const md = (e: Editor) => asMarkdown(e) ?? "";

describe("the panel that appears over a selection", () => {
  it("offers only what the document can keep", () => {
    const editor = made();
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    const names = screen.getAllByRole("button").map((one) => one.getAttribute("aria-label"));
    expect(names).toEqual([
      "Bold",
      "Italic",
      "Underline",
      "Strikethrough",
      "Code",
      "Link",
      "Align left",
      "Centre",
      "Align right",
      "Justify",
    ]);

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

  it("writes alignment as the html markdown admits, since it has no syntax for it", async () => {
    const editor = made();
    editor.commands.selectAll();
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Centre" }));

    expect(md(editor)).toBe('<p style="text-align: center">hola mundo</p>');
    editor.destroy();
  });

  it("leaves a paragraph nobody aligned as plain markdown", async () => {
    const editor = made("uno\n\ndos");
    editor.commands.selectAll();
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Align left" }));

    expect(md(editor)).toBe("uno\n\ndos");
    editor.destroy();
  });

  it("takes an address and ties it to the words that were picked", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));
    await userEvent.type(screen.getByLabelText(/Address/), "ejemplo.org{Enter}");

    expect(md(editor)).toBe("[hola](https://ejemplo.org) mundo");
    editor.destroy();
  });

  it("takes an address whole when it already says how to reach it", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));
    await userEvent.type(screen.getByLabelText(/Address/), "mailto:a@b.test{Enter}");

    expect(md(editor)).toContain("mailto:a@b.test");
    editor.destroy();
  });

  it("unties the link when the address is left empty", async () => {
    const editor = made("[hola](https://ejemplo.org) mundo");
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));
    await userEvent.clear(screen.getByLabelText(/Address/));
    await userEvent.keyboard("{Enter}");

    expect(md(editor)).toBe("hola mundo");
    editor.destroy();
  });

  it("shows the words that were picked, ready to be changed", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));

    expect((screen.getByLabelText(/Text to show/) as HTMLInputElement).value).toBe("hola");
    editor.destroy();
  });

  it("writes the words when they were changed, and links those", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));
    await userEvent.clear(screen.getByLabelText(/Text to show/));
    await userEvent.type(screen.getByLabelText(/Text to show/), "el sitio");
    await userEvent.type(screen.getByLabelText(/Address/), "ejemplo.org{Enter}");

    expect(md(editor)).toBe("[el sitio](https://ejemplo.org) mundo");
    editor.destroy();
  });

  it("opens straight into the address when the slash menu asked for it", () => {
    const editor = made();
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} asking />);

    expect(screen.getByLabelText(/Address/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Bold" })).toBeNull();

    editor.destroy();
  });

  it("says it is done so the menu that opened it can let go", async () => {
    const onDone = vi.fn();
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} asking onDone={onDone} />);

    await userEvent.type(screen.getByLabelText(/Address/), "ejemplo.org{Enter}");

    expect(onDone).toHaveBeenCalled();
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

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Editor } from "@tiptap/core";
import { asMarkdown, written } from "../ui/writing";
import Floats from "../ui/Floats";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const made = (content = "hola mundo") => new Editor({ extensions: written(), content });

const md = (e: Editor) => asMarkdown(e) ?? "";

describe("when the panel is allowed on screen at all", () => {
  it("rides over words that were picked", async () => {
    const { perched } = await import("../ui/Editor");

    expect(perched(false, false, false, false)).toBe(true);
  });

  it("stays away from a caret sitting on nothing, and from code", async () => {
    const { perched } = await import("../ui/Editor");

    expect(perched(true, false, false, false)).toBe(false);
    expect(perched(false, true, false, false)).toBe(false);
  });

  it("gives way to the menu the right button opened, selection or not", async () => {
    const { perched } = await import("../ui/Editor");

    expect(perched(false, false, true, false)).toBe(false);
  });

  it("stays away from a whole thing picked, which has no words to dress", async () => {
    const { perched } = await import("../ui/Editor");

    expect(perched(false, false, false, true)).toBe(false);
  });
});

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

  it("writes the address itself when there were no words to link", async () => {
    const editor = made("");
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} asking />);

    await userEvent.type(screen.getByLabelText(/Address/), "ejemplo.org{Enter}");

    expect(md(editor)).toBe("[ejemplo.org](https://ejemplo.org)");
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

  it("answers to a keyboard, which never sends a mouse press", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    screen.getByRole("button", { name: "Bold" }).focus();
    await userEvent.keyboard("{Enter}");

    expect(md(editor)).toBe("**hola** mundo");
    editor.destroy();
  });

  it("is one stop away, and the arrows walk the rest", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.tab();
    expect(document.activeElement?.getAttribute("aria-label")).toBe("Bold");

    await userEvent.keyboard("{ArrowRight}{ArrowRight}");
    expect(document.activeElement?.getAttribute("aria-label")).toBe("Underline");

    await userEvent.keyboard("{ArrowLeft}{ArrowLeft}{ArrowLeft}");
    expect(document.activeElement?.getAttribute("aria-label")).toBe("Link");

    await userEvent.tab();
    expect(document.activeElement?.tagName).toBe("BODY");

    editor.destroy();
  });

  it("follows the selection when it moves without the panel going away", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    const { rerender } = render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    editor.commands.setTextSelection({ from: 6, to: 11 });
    rerender(<Floats editor={editor} at={{ x: 20, y: 40 }} />);
    await userEvent.click(screen.getByRole("button", { name: "Bold" }));

    expect(md(editor)).toBe("hola **mundo**");
    editor.destroy();
  });

  it("ties the link where it was opened, not where the caret drifted to", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));
    editor.commands.setTextSelection({ from: 6, to: 11 });
    await userEvent.type(screen.getByLabelText(/Address/), "ejemplo.org{Enter}");

    expect(md(editor)).toBe("[hola](https://ejemplo.org) mundo");
    editor.destroy();
  });

  it("keeps the format of what it links when a space rides along in the selection", async () => {
    const editor = new Editor({ extensions: written(), content: "hola **mundo** y adios" });
    editor.commands.setTextSelection({ from: 1, to: 12 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));
    await userEvent.type(screen.getByLabelText(/Address/), "ejemplo.org{Enter}");

    expect(md(editor)).toContain("**mundo**");
    expect(md(editor)).toBe("[hola **mundo**](https://ejemplo.org) y adios");

    editor.destroy();
  });

  it("puts the words in as words, never as markup", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));
    await userEvent.clear(screen.getByLabelText(/Text to show/));
    await userEvent.type(screen.getByLabelText(/Text to show/), "<b>ojo</b>");
    await userEvent.type(screen.getByLabelText(/Address/), "ejemplo.org{Enter}");

    expect(editor.getText()).toContain("<b>ojo</b>");
    expect(editor.getHTML()).not.toContain("<b>ojo</b>");
    editor.destroy();
  });

  it("refuses an address with a space instead of writing a broken one", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));
    await userEvent.type(screen.getByLabelText(/Address/), "ejem plo.org{Enter}");

    expect(screen.getByLabelText(/Address/).getAttribute("aria-invalid")).toBe("true");
    expect(md(editor)).toBe("hola mundo");
    editor.destroy();
  });

  it("lets a bare host through, port and all", async () => {
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));
    await userEvent.type(screen.getByLabelText(/Address/), "localhost:3000{Enter}");

    expect(md(editor)).toContain("localhost:3000");
    editor.destroy();
  });

  it("closes and gives the selection back when the press lands elsewhere", async () => {
    const onDone = vi.fn();
    const editor = made();
    editor.commands.setTextSelection({ from: 1, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} onDone={onDone} />);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));
    await userEvent.click(document.body);

    expect(screen.queryByLabelText(/Address/)).toBeNull();
    expect(onDone).toHaveBeenCalled();
    expect(editor.state.selection.from).toBe(1);
    expect(editor.state.selection.to).toBe(5);
    editor.destroy();
  });

  it("does not offer what a code block cannot hold", () => {
    const editor = made();
    editor.chain().selectAll().setCodeBlock().run();
    editor.commands.setTextSelection({ from: 2, to: 5 });
    render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);

    expect((screen.getByRole("button", { name: "Bold" }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole("button", { name: "Link" }) as HTMLButtonElement).disabled).toBe(true);

    editor.destroy();
  });
});

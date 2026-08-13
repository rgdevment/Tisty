import { readFileSync } from "node:fs";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Editor } from "@tiptap/core";
import { written } from "../ui/writing";
import Modal from "../ui/Modal";
import Floats from "../ui/Floats";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const sheet = () => {
  const css = readFileSync("src/index.css", "utf8");
  const from = css.indexOf("@media print");
  expect(from).toBeGreaterThan(-1);
  return css.slice(from);
};

describe("what the printed sheet leaves out", () => {
  it("drops whatever floats over the app, and every live region", () => {
    const printed = sheet();

    expect(printed).toContain(".fixed");
    expect(printed).toContain("[aria-live]");
  });

  it("floats the modal, which is how the sheet knows to drop it", () => {
    render(
      <Modal title="Borrar">
        <p>seguro</p>
      </Modal>,
    );

    expect(screen.getByRole("dialog").classList.contains("fixed")).toBe(true);
  });

  it("floats the format panel and its link form alike", () => {
    const editor = new Editor({ extensions: written(), content: "hola mundo" });
    editor.commands.setTextSelection({ from: 1, to: 5 });

    const { unmount } = render(<Floats editor={editor} at={{ x: 10, y: 40 }} />);
    expect(screen.getByRole("toolbar").classList.contains("fixed")).toBe(true);
    unmount();

    render(<Floats editor={editor} at={{ x: 10, y: 40 }} asking />);
    expect(screen.getByLabelText(/Address/).closest("form")?.classList.contains("fixed")).toBe(
      true,
    );

    editor.destroy();
  });

  it("puts a colour on paper for what the dark theme would leave invisible", () => {
    const printed = sheet();

    expect(printed).toMatch(/\.tisty-doc hr\s*\{[^}]*#/);
    expect(printed).toMatch(/\.tisty-doc li::marker\s*\{[^}]*#/);
    expect(printed).toMatch(/\.tisty-doc code\s*\{[^}]*background:\s*#/);
    expect(printed).toMatch(/\.tisty-doc th\s*\{[^}]*background:\s*#/);
  });
});

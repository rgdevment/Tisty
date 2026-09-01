import { Editor } from "@tiptap/core";
import { describe, expect, it, vi } from "vitest";
import { DOC } from "../markdown";
import { named } from "../paging";
import { asMarkdown, written } from "../ui/writing";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const wrote = (alt: string) => {
  const editor = new Editor({
    extensions: written(),
    content: { type: "doc", content: [{ type: "image", attrs: { src: `${DOC}a3f1-0002`, alt } }] },
  });
  const said = asMarkdown(editor) ?? "";
  editor.destroy();
  return said;
};

describe("the way the editor writes a card", () => {
  it("escapes a bracket in the title, or the line stops naming the page", () => {
    const said = wrote("Capítulo 1 [borrador]");

    expect(said).toContain("![Capítulo 1 \\[borrador\\]](tisty:doc/a3f1-0002)");
    expect([...named(said)]).toEqual(["a3f1-0002"]);
  });

  it("leaves a title without brackets exactly as it was", () => {
    expect(wrote("El túnel")).toContain("![El túnel](tisty:doc/a3f1-0002)");
  });

  it("keeps naming the page after a title that was written escaped is read back", () => {
    const editor = new Editor({ extensions: written(), content: wrote("Uno [dos] tres") });
    const again = asMarkdown(editor) ?? "";
    editor.destroy();

    expect([...named(again)]).toEqual(["a3f1-0002"]);
  });
});

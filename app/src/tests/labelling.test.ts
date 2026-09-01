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

  it("escapes a slash too, or the one before a bracket would free it", () => {
    const said = wrote("Rutas C:\\ y más");

    expect([...named(said)]).toEqual(["a3f1-0002"]);
    expect([...named(wrote("Fin \\[uno]"))]).toEqual(["a3f1-0002"]);
  });

  it("keeps naming the page however often it is read and written again", () => {
    for (const title of ["Uno [dos] tres", "Rutas C: y más", "Fin [uno]"]) {
      let said = wrote(title);
      for (let turn = 0; turn < 3; turn += 1) {
        const editor = new Editor({ extensions: written(), content: said });
        said = asMarkdown(editor) ?? "";
        editor.destroy();
        expect([...named(said)]).toEqual(["a3f1-0002"]);
      }
    }
  });
});

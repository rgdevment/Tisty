import { Editor } from "@tiptap/core";
import { describe, expect, it, vi } from "vitest";
import { shotAt, shotNode } from "../ui/Editor";
import { written } from "../ui/writing";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  convertFileSrc: (at: string) => at,
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn(), openPath: vi.fn() }));

const held = (content: string) => {
  const editor = new Editor({ extensions: written(), content });
  let at: number | null = null;
  editor.state.doc.descendants((node, spot) => {
    if (node.type.name === "image" && at === null) at = spot;
    return true;
  });
  if (at !== null) editor.commands.setNodeSelection(at);
  return editor;
};

describe("picking a photo in a document", () => {
  it("offers what to do with it once it is picked", () => {
    const editor = held("![gato](<attachments/aa/gato.jpg>)");

    expect(shotAt(editor)?.src).toBe("attachments/aa/gato.jpg");

    editor.destroy();
  });

  it("offers nothing for a document card, which has its own handles", () => {
    const editor = held("![Informe](tisty:doc/mac0-0001)");

    expect(shotAt(editor)).toBeNull();

    editor.destroy();
  });

  it("offers nothing for an attachment that is not a picture", () => {
    const editor = held("![Cartola](<attachments/aa/cartola.pdf>)");

    expect(shotAt(editor)).toBeNull();

    editor.destroy();
  });

  it("offers nothing for a picture that lives elsewhere, which is not ours to open", () => {
    const editor = held("![fuera](https://ejemplo.org/foto.jpg)");

    expect(shotAt(editor)).toBeNull();

    editor.destroy();
  });

  it("offers nothing while the writer is typing rather than pointing at a photo", () => {
    const editor = new Editor({ extensions: written(), content: "una línea" });

    expect(shotAt(editor)).toBeNull();

    editor.destroy();
  });
});

describe("the blue wash the browser paints over a picked photo", () => {
  it("is held back while a photo is picked, so it cannot spill past the frame", () => {
    const editor = held("![gato](<attachments/aa/gato.jpg>)");

    expect(shotNode(editor.state)).toBe("attachments/aa/gato.jpg");

    editor.destroy();
  });

  it("is held back for a photo from elsewhere too, which the frame also holds", () => {
    const editor = held("![fuera](https://ejemplo.org/foto.jpg)");

    expect(shotNode(editor.state)).toBe("https://ejemplo.org/foto.jpg");

    editor.destroy();
  });

  it("is left alone while writing, where it is how a writer sees the selection", () => {
    const editor = new Editor({ extensions: written(), content: "una línea" });

    expect(shotNode(editor.state)).toBeNull();

    editor.destroy();
  });

  it("is left alone for a card, which is not a photo in a frame", () => {
    const editor = held("![Informe](tisty:doc/mac0-0001)");

    expect(shotNode(editor.state)).toBeNull();

    editor.destroy();
  });
});

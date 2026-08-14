import { describe, expect, it, vi } from "vitest";
import { Editor } from "@tiptap/core";
import { asMarkdown, written } from "../ui/writing";
import { previewing, type Reach } from "../ui/previewing";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const reach: Reach = {
  url: (at) => `asset://localhost/${at}`,
  weight: () => 2_400_000,
  title: (id) => (id === "mac0-0007" ? "Informe técnico" : null),
};

const made = (content: string, how: Partial<Reach> = {}) =>
  new Editor({
    extensions: [...written(), previewing(() => ({ ...reach, ...how }))],
    content,
  });

describe("what the editor shows beside a link", () => {
  it("puts a player under a video, without touching the markdown", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)");

    const player = editor.view.dom.querySelector("video");
    expect(player).toBeTruthy();
    expect(player?.getAttribute("src")).toBe("asset://localhost/attachments/charla-a3f9.mp4");
    expect(asMarkdown(editor)).toBe("[charla](attachments/charla-a3f9.mp4)");

    editor.destroy();
  });

  it("puts a player under a sound too", () => {
    const editor = made("[nota](<attachments/nota-11bc.m4a>)");

    expect(editor.view.dom.querySelector("audio")).toBeTruthy();
    expect(editor.view.dom.querySelector("video")).toBeNull();

    editor.destroy();
  });

  it("makes a card of another file, with its kind and its weight", () => {
    const editor = made("[contrato](<attachments/contrato-91f2.pdf>)");

    const card = editor.view.dom.querySelector(".preview-file");
    expect(card?.textContent).toContain("contrato-91f2.pdf");
    expect(card?.textContent).toContain("PDF · 2.4 MB");

    editor.destroy();
  });

  it("makes a card of a document, named as the document is named now", () => {
    const editor = made("[lo que sea](tisty:doc/mac0-0007)");

    expect(editor.view.dom.querySelector(".preview-doc")?.textContent).toContain(
      "Informe técnico",
    );

    editor.destroy();
  });

  it("says a document is gone rather than showing a card that leads nowhere", () => {
    const editor = made("[Informe](tisty:doc/borrado)");

    expect(editor.view.dom.querySelector(".preview-gone")).toBeTruthy();

    editor.destroy();
  });

  it("shows nothing for an address that leaves the machine", () => {
    const editor = made("[fuera](https://ejemplo.org/charla.mp4)");

    expect(editor.view.dom.querySelector(".preview")).toBeNull();

    editor.destroy();
  });

  it("shows one player for a link whose words are split by a format", () => {
    const editor = made("[una **charla** larga](<attachments/charla-a3f9.mp4>)");

    expect(editor.view.dom.querySelectorAll("video").length).toBe(1);

    editor.destroy();
  });

  it("sits after the paragraph, never among the words being typed", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)");

    expect(editor.view.dom.querySelector(".preview")?.closest("p")).toBeNull();

    editor.destroy();
  });

  it("goes away with the link that brought it, so it can be deleted", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)");
    expect(editor.view.dom.querySelector("video")).toBeTruthy();

    editor.chain().selectAll().deleteSelection().run();

    expect(editor.view.dom.querySelector("video")).toBeNull();
    expect(asMarkdown(editor)).toBe("");

    editor.destroy();
  });

  it("keeps the preview out of what gets saved", () => {
    const editor = made("[contrato](<attachments/contrato-91f2.pdf>)");

    expect(asMarkdown(editor)).toBe("[contrato](attachments/contrato-91f2.pdf)");
    expect(asMarkdown(editor)).not.toContain("PDF");

    editor.destroy();
  });
});

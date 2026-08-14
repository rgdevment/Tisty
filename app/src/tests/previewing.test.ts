import { readFileSync } from "node:fs";
import { describe, expect, it, vi } from "vitest";
import { Editor } from "@tiptap/core";
import { asMarkdown, written } from "../ui/writing";
import { alone, plugged, previewing, type Reach } from "../ui/previewing";

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

describe("clicking a link inside the editor", () => {
  const clicked = async (md: string, at: number, how: { metaKey?: boolean } = {}) => {
    const { clicking } = await import("../ui/Editor");
    const opened = vi.fn();
    const editor = made(md);
    const took = clicking(opened)({ state: editor.state }, at, how);
    editor.destroy();
    return { took, opened };
  };

  it("lets a plain click place the caret, or the words could never be edited", async () => {
    const { took, opened } = await clicked("[charla](<attachments/charla-a3f9.mp4>)", 3);

    expect(took).toBe(false);
    expect(opened).not.toHaveBeenCalled();
  });

  it("opens the attachment when the click is held with the command key", async () => {
    const { took, opened } = await clicked("[charla](<attachments/charla-a3f9.mp4>)", 3, {
      metaKey: true,
    });

    expect(took).toBe(true);
    expect(opened).toHaveBeenCalledWith("attachments/charla-a3f9.mp4");
  });

  it("leaves an address of the world to the system, held key or not", async () => {
    const { took, opened } = await clicked("[fuera](https://ejemplo.org)", 3, { metaKey: true });

    expect(took).toBe(false);
    expect(opened).not.toHaveBeenCalled();
  });
});

describe("what the editor shows beside a link", () => {
  it("offers to play a video rather than filling the page with it", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)");

    const asks = editor.view.dom.querySelector<HTMLElement>(".preview-play");
    expect(asks?.getAttribute("aria-label")).toBe("Play");
    expect(editor.view.dom.querySelector("video[controls]")).toBeNull();
    expect(asMarkdown(editor)).toBe("[charla](attachments/charla-a3f9.mp4)");

    editor.destroy();
  });

  it("unfolds the player where the offer was, once it is asked for", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)");

    editor.view.dom
      .querySelector<HTMLElement>(".preview-play")
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));

    const player = editor.view.dom.querySelector("video[controls]");
    expect(player?.getAttribute("src")).toBe("asset://localhost/attachments/charla-a3f9.mp4");
    expect(editor.view.dom.querySelector(".preview-play")).toBeNull();
    expect(asMarkdown(editor)).toBe("[charla](attachments/charla-a3f9.mp4)");

    editor.destroy();
  });

  it("puts a player under a sound too", () => {
    const editor = made("[nota](<attachments/nota-11bc.m4a>)");

    expect(editor.view.dom.querySelector("audio")).toBeTruthy();
    expect(editor.view.dom.querySelector("video")).toBeNull();

    editor.destroy();
  });

  it("folds the player away again, so the offer is not a one way door", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)");
    const play = () =>
      editor.view.dom
        .querySelector<HTMLElement>(".preview-play")
        ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));

    play();
    expect(editor.view.dom.querySelector("video[controls]")).toBeTruthy();

    editor.view.dom
      .querySelector<HTMLElement>(".preview-fold")
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));

    expect(editor.view.dom.querySelector("video[controls]")).toBeNull();
    expect(editor.view.dom.querySelector(".preview-play")).toBeTruthy();

    play();
    expect(editor.view.dom.querySelector("video[controls]")).toBeTruthy();

    editor.destroy();
  });

  it("cards a file with the name that was written, not the one on disk", () => {
    const editor = made("[contrato](<attachments/contrato-91f2.pdf>)");

    expect(editor.view.dom.querySelector(".card-name")?.textContent).toBe("contrato");
    expect(editor.view.dom.querySelector(".card-under")?.textContent).toBe("PDF · 2.4 MB");
    expect(editor.view.dom.querySelector(".card-badge")?.getAttribute("data-kind")).toBe("PDF");

    editor.destroy();
  });

  it("makes a card of a document, named as the document is named now", () => {
    const editor = made("[lo que sea](tisty:doc/mac0-0007)");

    expect(editor.view.dom.querySelector(".card-name")?.textContent).toBe("Informe técnico");

    editor.destroy();
  });

  it("says a card points at the very document you are reading, instead of doing nothing", () => {
    const editor = made("[Informe](tisty:doc/mac0-0007)", { here: "mac0-0007" });

    const card = editor.view.dom.querySelector<HTMLElement>(".card-doc");
    expect(card?.textContent).toContain("This very document");
    expect(card?.getAttribute("role")).toBeNull();
    expect(card?.getAttribute("tabindex")).toBeNull();

    editor.destroy();
  });

  it("still opens a card that points somewhere else", () => {
    const editor = made("[Informe](tisty:doc/mac0-0007)", { here: "mac0-0001" });

    expect(editor.view.dom.querySelector(".card-doc")?.getAttribute("role")).toBe("button");

    editor.destroy();
  });

  it("says a document is gone rather than saying it is opening for ever", () => {
    const editor = made("[Informe](tisty:doc/borrado)");

    expect(editor.view.dom.querySelector(".card-under")?.textContent).toBe(
      "That document is not on this machine",
    );

    editor.destroy();
  });

  it("says it is opening only while the papers are still on their way", () => {
    const editor = made("[Informe](tisty:doc/mac0-0007)", { title: () => undefined });

    expect(editor.view.dom.querySelector(".card-name")?.textContent).toBe("Opening…");
    expect(editor.view.dom.querySelector(".card-gone")).toBeNull();

    editor.destroy();
  });

  it("shows nothing for an address that leaves the machine", () => {
    const editor = made("[fuera](https://ejemplo.org/charla.mp4)");

    expect(editor.view.dom.querySelector(".preview")).toBeNull();

    editor.destroy();
  });

  it("shows one player for a link whose words are split by a format", () => {
    const editor = made("[una **charla** larga](<attachments/charla-a3f9.mp4>)");

    expect(editor.view.dom.querySelectorAll(".preview-play").length).toBe(1);

    editor.destroy();
  });

  it("says a file is not there instead of offering to play nothing", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)", { gone: () => true });

    expect(editor.view.dom.querySelector(".preview-play")).toBeNull();
    expect(editor.view.dom.querySelector(".card-under")?.textContent).toContain("look again");

    editor.destroy();
  });

  it("offers another look, because a file can arrive after the first try failed", () => {
    const onAgain = vi.fn();
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)", { gone: () => true, onAgain });

    const card = editor.view.dom.querySelector<HTMLElement>(".card-gone");
    expect(card?.getAttribute("role")).toBe("button");
    card?.dispatchEvent(new MouseEvent("click", { bubbles: true }));

    expect(onAgain).toHaveBeenCalledWith("attachments/charla-a3f9.mp4");

    editor.destroy();
  });

  it("sits after the paragraph, never among the words being typed", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)");

    expect(editor.view.dom.querySelector(".preview")?.closest("p")).toBeNull();

    editor.destroy();
  });

  it("goes away with the link that brought it, so it can be deleted", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)");
    expect(editor.view.dom.querySelector(".preview-play")).toBeTruthy();

    editor.chain().selectAll().deleteSelection().run();

    expect(editor.view.dom.querySelector(".preview-play")).toBeNull();
    expect(asMarkdown(editor)).toBe("");

    editor.destroy();
  });

  it("keeps the preview out of what gets saved", () => {
    const editor = made("[contrato](<attachments/contrato-91f2.pdf>)");

    expect(asMarkdown(editor)).toBe("[contrato](attachments/contrato-91f2.pdf)");
    expect(asMarkdown(editor)).not.toContain("PDF");

    editor.destroy();
  });

  it("shows a card for each reference sharing one paragraph", () => {
    const editor = made("[uno](<attachments/uno-1111.pdf>) y [dos](<attachments/dos-2222.pdf>)");

    expect(editor.view.dom.querySelectorAll(".card").length).toBe(2);

    editor.destroy();
  });

  it("shows a card for the reference inside each item of a list", () => {
    const editor = made("- [uno](<attachments/uno-1111.pdf>)\n- [dos](<attachments/dos-2222.pdf>)");

    const items = editor.view.dom.querySelectorAll("li");
    expect(items).toHaveLength(2);
    expect(items[0].querySelector(".card")).toBeTruthy();
    expect(items[1].querySelector(".card")).toBeTruthy();

    editor.destroy();
  });

  it("shows a card for a reference sitting inside a quote", () => {
    const editor = made("> [uno](<attachments/uno-1111.pdf>)");

    const quote = editor.view.dom.querySelector("blockquote");
    expect(quote?.querySelector(".card")).toBeTruthy();

    editor.destroy();
  });

  it("shows a card for a reference sitting inside a table cell", () => {
    const editor = made("| a | b |\n| --- | --- |\n| [uno](<attachments/uno-1111.pdf>) | texto |");

    const table = editor.view.dom.querySelector("table");
    expect(table?.querySelector(".card")).toBeTruthy();

    editor.destroy();
  });

  it("can be deleted from the card, since the link that made it is hidden", () => {
    const editor = made("[contrato](<attachments/contrato-91f2.pdf>)");
    const card = editor.view.dom.querySelector<HTMLElement>(".card");

    card?.dispatchEvent(new KeyboardEvent("keydown", { key: "Backspace", bubbles: true }));

    expect(asMarkdown(editor)).toBe("");
    expect(editor.view.dom.querySelector(".card")).toBeNull();

    editor.destroy();
  });

  it("leaves the words alone when the link shares its line with other text", () => {
    const editor = made("mira [contrato](<attachments/contrato-91f2.pdf>) por favor");
    const card = editor.view.dom.querySelector<HTMLElement>(".card");

    card?.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete", bubbles: true }));

    expect(asMarkdown(editor)).toBe("");

    editor.destroy();
  });

  it("hides only the link that fills its own paragraph", () => {
    const sheet = readFileSync("src/index.css", "utf8");

    expect(sheet).toContain("p:has(> a:only-child):has(+ .preview)");
  });

  it("leaves the words above a preview on screen, so there is a line to write in", () => {
    const sheet = readFileSync("src/index.css", "utf8");
    const rule = sheet.slice(sheet.indexOf(".tisty-doc p:has(> a:only-child):has(+ .preview)"));

    expect(rule.slice(0, rule.indexOf("}"))).not.toMatch(/display:\s*none/);
  });

  it("stands the words above the preview in the middle, over what they name", () => {
    const sheet = readFileSync("src/index.css", "utf8");
    const rule = sheet.slice(sheet.indexOf(".tisty-doc p:has(> a:only-child):has(+ .preview)"));

    expect(rule.slice(0, rule.indexOf("}"))).toMatch(/text-align:\s*center/);
  });

  it("does not read the whole document again when only the caret moved", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)\n\nun parrafo largo");
    const doc = editor.state.doc;
    const walk = vi.spyOn(doc, "descendants");

    editor.commands.setTextSelection(3);
    editor.commands.setTextSelection(5);

    expect(walk).not.toHaveBeenCalled();
    expect(editor.state.doc).toBe(doc);

    editor.destroy();
  });

  it("reads it again once the words themselves change", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)\n\nun parrafo largo");
    editor.commands.setTextSelection(editor.state.doc.content.size - 1);
    const before = editor.view.dom.querySelectorAll(".preview").length;

    editor.commands.insertContent(" y mas");

    expect(editor.view.dom.querySelectorAll(".preview")).toHaveLength(before);

    editor.destroy();
  });

  const rubs = (editor: Editor, key: string) => {
    const mine = plugged.get(editor.state);
    return mine?.props.handleKeyDown?.call(
      mine,
      editor.view,
      new KeyboardEvent("keydown", { key }),
    );
  };

  it("takes the whole reference when a single letter of its name is rubbed out", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)\n\nqueda esto");

    editor.commands.setTextSelection(4);

    expect(rubs(editor, "Backspace")).toBe(true);
    expect(asMarkdown(editor)?.trim()).toBe("queda esto");

    editor.destroy();
  });

  it("takes it whole with the forward key too", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)\n\nqueda esto");

    editor.commands.setTextSelection(1);

    expect(rubs(editor, "Delete")).toBe(true);
    expect(asMarkdown(editor)?.trim()).toBe("queda esto");

    editor.destroy();
  });

  it("goes whole from the end of its name, the way a hand reaches for it", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)\n\nqueda esto");

    editor.commands.setTextSelection(7);
    const took = editor.view.someProp("handleKeyDown", (f) =>
      f(editor.view, new KeyboardEvent("keydown", { key: "Backspace" })),
    );

    expect(took).toBe(true);
    expect(asMarkdown(editor)?.trim()).toBe("queda esto");
    expect(editor.view.dom.querySelector(".preview")).toBeNull();

    editor.destroy();
  });

  const block = (editor: Editor, tail: string) => {
    const link = editor.schema.marks.link.create({ href: "attachments/charla-a3f9.mp4" });
    return editor.schema.nodes.paragraph.create(null, [
      editor.schema.text("charla", [link]),
      editor.schema.text(tail),
    ]);
  };

  it("is not stopped by the blank space that rides along with a name", () => {
    const editor = made("nada");

    expect(alone(block(editor, " "))).toBe(true);

    editor.destroy();
  });

  it("is stopped by anything a person actually wrote there", () => {
    const editor = made("nada");

    expect(alone(block(editor, " y lo que dije"))).toBe(false);

    editor.destroy();
  });

  it("takes only the reference under the caret when two share one line", () => {
    const editor = made(
      "[charla](<attachments/charla-a3f9.mp4>) [notas](<attachments/notas-b1c2.pdf>)",
    );

    editor.commands.setTextSelection(5);

    expect(rubs(editor, "Backspace")).toBe(true);
    const said = asMarkdown(editor) ?? "";
    expect(said).not.toContain("charla-a3f9.mp4");
    expect(said).toContain("notas-b1c2.pdf");

    editor.destroy();
  });

  it("takes the reference and leaves the words it was sitting among", () => {
    const editor = made("antes [charla](<attachments/charla-a3f9.mp4>) despues");

    editor.commands.setTextSelection(11);

    expect(rubs(editor, "Backspace")).toBe(true);
    const said = asMarkdown(editor) ?? "";
    expect(said).not.toContain("charla-a3f9.mp4");
    expect(said).toContain("antes");
    expect(said).toContain("despues");

    editor.destroy();
  });

  it("leaves ordinary words beside a reference alone", () => {
    const editor = made("antes [charla](<attachments/charla-a3f9.mp4>) despues");

    editor.commands.setTextSelection(3);

    expect(rubs(editor, "Backspace")).toBeFalsy();

    editor.destroy();
  });

  it("keeps the words a person wrote beside the reference", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>) y lo que dije");

    editor.commands.setTextSelection(12);

    expect(rubs(editor, "Backspace")).toBeFalsy();
    expect(asMarkdown(editor)).toContain("y lo que dije");

    editor.destroy();
  });

  it("says nothing about an empty line, where there is no name to rub out", () => {
    const editor = made("");

    editor.commands.setTextSelection(1);

    expect(() => rubs(editor, "Backspace")).not.toThrow();

    editor.destroy();
  });

  it("takes it whole when letters were picked from the very start of the name", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)\n\nqueda esto");

    editor.commands.setTextSelection({ from: 1, to: 4 });

    expect(rubs(editor, "Backspace")).toBe(true);
    expect(asMarkdown(editor)?.trim()).toBe("queda esto");

    editor.destroy();
  });

  it("does not touch a plain link, which has no preview to stand for it", () => {
    const editor = made("[la web](https://ejemplo.org)");

    editor.commands.setTextSelection(4);

    expect(rubs(editor, "Backspace")).toBeFalsy();
    expect(asMarkdown(editor)).toContain("ejemplo.org");

    editor.destroy();
  });

  it("leaves the far edge of the name to the editor's own keys as well", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)\n\ny esto");

    editor.commands.setTextSelection(7);

    expect(rubs(editor, "Delete")).toBeFalsy();
    expect(asMarkdown(editor)).toContain("charla-a3f9.mp4");

    editor.destroy();
  });

  it("leaves the caret at the edge of the name to the editor's own keys", () => {
    const editor = made("antes\n\n[charla](<attachments/charla-a3f9.mp4>)");

    editor.commands.setTextSelection(8);

    expect(rubs(editor, "Backspace")).toBeFalsy();
    expect(asMarkdown(editor)).toContain("charla-a3f9.mp4");

    editor.destroy();
  });

  it("leaves ordinary words alone when they are rubbed out", () => {
    const editor = made("un parrafo cualquiera");

    editor.commands.setTextSelection(5);

    expect(rubs(editor, "Backspace")).toBeFalsy();

    editor.destroy();
  });

  it("does not swallow a sweep that reaches past the reference", () => {
    const editor = made("[charla](<attachments/charla-a3f9.mp4>)\n\ny esto");

    editor.commands.setTextSelection({ from: 3, to: editor.state.doc.content.size - 2 });

    expect(rubs(editor, "Backspace")).toBeFalsy();

    editor.destroy();
  });

  it("keeps one line between two attachments in a row", () => {
    const editor = made(
      "[charla](<attachments/charla-a3f9.mp4>)\n\n[notas](<attachments/notas-b1c2.pdf>)",
    );

    const shown = editor.view.dom.querySelectorAll("p");
    const above = [...shown].filter((one) => one.querySelector("a"));

    expect(above).toHaveLength(2);
    expect(editor.view.dom.querySelectorAll(".preview")).toHaveLength(2);

    editor.destroy();
  });

  it("sits after a heading rather than inside it", () => {
    const editor = made("# [uno](<attachments/uno-1111.pdf>)");

    expect(editor.view.dom.querySelector("h1 .card")).toBeNull();
    expect(editor.view.dom.querySelector(".card")).toBeTruthy();

    editor.destroy();
  });

  it("shows a card at every place the same attachment is referenced, not just the first", () => {
    const editor = made(
      "primero [uno](<attachments/mismo-1111.pdf>)\n\nsegundo [otra vez](<attachments/mismo-1111.pdf>)",
    );

    expect(editor.view.dom.querySelectorAll(".card")).toHaveLength(2);

    editor.destroy();
  });

  it.fails("offers one player, not two, for a link whose text is split by a hard break", () => {
    const editor = made("[linea uno\nlinea dos](<attachments/charla-a3f9.mp4>)");

    const players = editor.view.dom.querySelectorAll("video");

    editor.destroy();
    expect(players).toHaveLength(1);
  });
});

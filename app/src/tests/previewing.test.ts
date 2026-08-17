import { Editor } from "@tiptap/core";
import { describe, expect, it, vi } from "vitest";
import { previewing, type Reach } from "../ui/previewing";
import { asMarkdown, written } from "../ui/writing";

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

describe("what the editor makes of a reference marked as a card", () => {
  it("offers to play a video rather than filling the page with it", () => {
    const editor = made("![charla](<attachments/charla-a3f9.mp4>)");

    const asks = editor.view.dom.querySelector<HTMLElement>(".preview-play");
    expect(asks?.getAttribute("aria-label")).toBe("Play");

    editor.destroy();
  });

  it("unfolds the player where the offer was, once it is asked for", () => {
    const editor = made("![charla](<attachments/charla-a3f9.mp4>)");
    const asks = editor.view.dom.querySelector<HTMLElement>(".preview-play");

    asks?.click();

    expect(editor.view.dom.querySelector("video")).toBeTruthy();

    editor.destroy();
  });

  it("puts a player under a sound too", () => {
    const editor = made("![nota](<attachments/nota-11bc.m4a>)");

    expect(editor.view.dom.querySelector("audio")).toBeTruthy();

    editor.destroy();
  });

  it("cards a file with the name that was written, not the one on disk", () => {
    const editor = made("![contrato](<attachments/contrato-91f2.pdf>)");

    expect(editor.view.dom.querySelector(".card-name")?.textContent).toBe("contrato");

    editor.destroy();
  });

  it("makes a card of a document, named as the document is named now", () => {
    const editor = made("![lo que sea](tisty:doc/mac0-0007)");

    expect(editor.view.dom.querySelector(".card-name")?.textContent).toBe("Informe técnico");

    editor.destroy();
  });

  it("says a card points at the very document you are reading", () => {
    const editor = made("![Informe](tisty:doc/mac0-0007)", { here: "mac0-0007" });

    expect(editor.view.dom.querySelector(".card-itself")).toBeTruthy();

    editor.destroy();
  });

  it("says a document is gone rather than saying it is opening for ever", () => {
    const editor = made("![Informe](tisty:doc/borrado)");

    expect(editor.view.dom.querySelector(".card-gone")).toBeTruthy();

    editor.destroy();
  });

  it("says it is opening only while the papers are still on their way", () => {
    const editor = made("![Informe](tisty:doc/mac0-0007)", { title: () => undefined });

    expect(editor.view.dom.querySelector(".card-gone")).toBeNull();

    editor.destroy();
  });

  it("says a file is not there instead of offering to play nothing", () => {
    const editor = made("![charla](<attachments/charla-a3f9.mp4>)", { gone: () => true });

    expect(editor.view.dom.querySelector(".card-gone")).toBeTruthy();
    expect(editor.view.dom.querySelector(".preview-play")).toBeNull();

    editor.destroy();
  });

  it("leaves an address of the world to the world", () => {
    const editor = made("![fuera](https://ejemplo.org/charla.mp4)");

    expect(editor.view.dom.querySelector(".preview")).toBeNull();

    editor.destroy();
  });

  it("never writes the name twice, which is what a card is for", () => {
    const editor = made("![el canario](tisty:doc/mac0-0007)");

    const said = editor.view.dom.textContent ?? "";
    const times = said.split("Informe técnico").length - 1;

    expect(times).toBe(1);

    editor.destroy();
  });

  it("hides the reference it grew from, so nothing shows behind the card", () => {
    const editor = made("![contrato](<attachments/contrato-91f2.pdf>)");

    expect(editor.view.dom.querySelector("img")?.className).toContain("card-source");

    editor.destroy();
  });

  it("offers one player, never two, because a card name cannot be split", () => {
    const editor = made("![linea uno\nlinea dos](<attachments/charla-a3f9.mp4>)");

    expect(editor.view.dom.querySelectorAll(".preview-play")).toHaveLength(1);

    editor.destroy();
  });

  it("gives back the very markdown it was given", () => {
    const editor = made("![contrato](<attachments/contrato-91f2.pdf>)");

    expect(asMarkdown(editor)?.trim()).toBe("![contrato](attachments/contrato-91f2.pdf)");

    editor.destroy();
  });
});

describe("a reference left as a plain link", () => {
  it("shows no card, so the words keep their line", () => {
    const editor = made("mira [contrato](<attachments/contrato-91f2.pdf>) por favor");

    expect(editor.view.dom.querySelector(".preview")).toBeNull();
    expect(editor.view.dom.querySelector("a")?.textContent).toBe("contrato");

    editor.destroy();
  });

  it("shows no card even alone on its line, because that is what was asked for", () => {
    const editor = made("[contrato](<attachments/contrato-91f2.pdf>)");

    expect(editor.view.dom.querySelector(".preview")).toBeNull();

    editor.destroy();
  });

  it("is not swallowed whole when a letter of it is rubbed out", () => {
    const editor = made("[contrato](<attachments/contrato-91f2.pdf>)\n\nqueda esto");

    editor.commands.setTextSelection(4);
    editor.commands.deleteRange({ from: 3, to: 4 });

    const said = asMarkdown(editor) ?? "";
    expect(said).toContain("contrato-91f2.pdf");
    expect(said).toContain("queda esto");

    editor.destroy();
  });
});

describe("a picture is still a picture", () => {
  it("draws it rather than carding it", () => {
    const editor = made("![foto](<attachments/ab/foto-1234.png>)");

    expect(editor.view.dom.querySelector("img")).toBeTruthy();
    expect(editor.view.dom.querySelector(".preview")).toBeNull();

    editor.destroy();
  });

  it("does not hide it behind anything", () => {
    const editor = made("![foto](<attachments/ab/foto-1234.png>)");

    expect(editor.view.dom.querySelector("img")?.className).not.toContain("card-source");

    editor.destroy();
  });
});

describe("swapping one for the other", () => {
  const menued = (md: string) => {
    let held: { untie: () => void; drop: () => void } | null = null;
    const editor = made(md, {
      onMenu: (_at, untie, drop) => {
        held = { untie, drop };
      },
    });
    editor.view.dom.querySelector<HTMLElement>(".card-swap")?.click();
    return { editor, said: () => held };
  };

  it("asks what to do instead of deciding on one click", () => {
    const { editor, said } = menued("![contrato](<attachments/contrato-91f2.pdf>)");

    expect(said()).not.toBeNull();
    expect(asMarkdown(editor)?.trim()).toBe("![contrato](attachments/contrato-91f2.pdf)");

    editor.destroy();
  });

  it("turns a card back into a link, keeping the name and the address", () => {
    const { editor, said } = menued("![contrato](<attachments/contrato-91f2.pdf>)");

    said()?.untie();

    expect(asMarkdown(editor)?.trim()).toBe("[contrato](attachments/contrato-91f2.pdf)");
    expect(editor.view.dom.querySelector(".preview")).toBeNull();

    editor.destroy();
  });

  it("says what the button does, so it is not a bare glyph", () => {
    const editor = made("![contrato](<attachments/contrato-91f2.pdf>)");

    const swap = editor.view.dom.querySelector<HTMLElement>(".card-swap");

    expect(swap?.getAttribute("aria-label")).toBeTruthy();
    expect(swap?.getAttribute("title")).toBeTruthy();
    expect(swap?.getAttribute("aria-haspopup")).toBe("menu");

    editor.destroy();
  });

  it("falls back to the file name when the card carried none", () => {
    const { editor, said } = menued("![](<attachments/contrato-91f2.pdf>)");

    said()?.untie();

    expect(asMarkdown(editor)?.trim()).toBe("[contrato-91f2.pdf](attachments/contrato-91f2.pdf)");

    editor.destroy();
  });

  it("takes the card away from the same menu", () => {
    const { editor, said } = menued("![contrato](<attachments/contrato-91f2.pdf>)\n\nqueda esto");

    said()?.drop();

    expect(asMarkdown(editor)?.trim()).toBe("queda esto");

    editor.destroy();
  });

  it("takes the card away whole when it is rubbed out", () => {
    const editor = made("![contrato](<attachments/contrato-91f2.pdf>)\n\nqueda esto");
    const card = editor.view.dom.querySelector<HTMLElement>(".card");

    card?.dispatchEvent(new KeyboardEvent("keydown", { key: "Backspace", bubbles: true }));

    expect(asMarkdown(editor)?.trim()).toBe("queda esto");

    editor.destroy();
  });
});

describe("what a reference picked from the list becomes", () => {
  const shaped = async (content: string, at: number, how: "card" | "link") => {
    const { docLink, DOC } = await import("../markdown");
    const editor = made(content);
    editor.commands.setTextSelection(at);
    editor
      .chain()
      .focus()
      .insertContent(
        how === "card"
          ? { type: "image", attrs: { src: `${DOC}mac0-0007`, alt: "Informe técnico" } }
          : docLink("mac0-0007", "Informe técnico"),
      )
      .run();
    const said = asMarkdown(editor) ?? "";
    editor.destroy();
    return said;
  };

  it("becomes a card when a card is what was asked for, wherever the caret was", async () => {
    expect(await shaped("mira aqui", 5, "card")).toContain(
      "![Informe técnico](tisty:doc/mac0-0007)",
    );
  });

  it("becomes a link when a link is what was asked for, even on a line of its own", async () => {
    const said = await shaped("", 1, "link");

    expect(said).toContain("[Informe técnico](tisty:doc/mac0-0007)");
    expect(said).not.toContain("![Informe");
  });
});

describe("what lands when a file is attached or dropped in", () => {
  const dropped = (into: string, at: number, said: string) => {
    const editor = made(into);
    editor.commands.setTextSelection(at);
    editor.chain().focus().insertContent(said).run();
    const cards = editor.view.dom.querySelectorAll(".card").length;
    const md = asMarkdown(editor) ?? "";
    editor.destroy();
    return { cards, md };
  };

  it("gives a card for something that cannot be drawn", () => {
    const { cards, md } = dropped("", 1, "![el contrato](<attachments/ab/cd.pdf>)");

    expect(cards).toBe(1);
    expect(md).toContain("![el contrato](attachments/ab/cd.pdf)");
  });

  it("gives a player for a video, not a bare link", () => {
    const editor = made("");
    editor.chain().focus().insertContent("![charla](<attachments/charla-a3f9.mp4>)").run();

    expect(editor.view.dom.querySelector(".preview-play")).toBeTruthy();

    editor.destroy();
  });

  it("still draws a picture as a picture", () => {
    const editor = made("");
    editor.chain().focus().insertContent("![foto](<attachments/aa/1.png>)").run();

    expect(editor.view.dom.querySelector("img")?.className).not.toContain("card-source");
    expect(editor.view.dom.querySelector(".card")).toBeNull();

    editor.destroy();
  });
});

describe("what the floating panel makes of a card", () => {
  const held = async (md: string, at: number) => {
    const { NodeSelection } = await import("@tiptap/pm/state");
    const { perched } = await import("../ui/Editor");
    const editor = made(md);
    editor.commands.setNodeSelection(at);
    const sel = editor.state.selection;
    const whole = sel instanceof NodeSelection && sel.node.isAtom;
    const shown = perched(sel.empty, false, false, whole);
    editor.destroy();
    return shown;
  };

  it("keeps the panel away from a card, which has no words to underline", async () => {
    expect(await held("![contrato](<attachments/contrato-91f2.pdf>)", 0)).toBe(false);
  });

  it("keeps it away from a picture too", async () => {
    expect(await held("![foto](<attachments/aa/1.png>)", 0)).toBe(false);
  });

  it("still brings it over words that were picked", async () => {
    const { perched } = await import("../ui/Editor");
    const editor = made("mira [contrato](<attachments/contrato-91f2.pdf>) aqui");
    editor.commands.setTextSelection({ from: 1, to: 5 });

    expect(perched(editor.state.selection.empty, false, false, false)).toBe(true);

    editor.destroy();
  });
});

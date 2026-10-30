import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { heads } from "tisty-editor/read";
import { describe, expect, it } from "vitest";
import { edited } from "../ui/blocking";
import { marksShown, seen } from "../ui/seeing";

const lines = (doc: string): string[] => {
  const state = EditorState.create({
    doc,
    selection: EditorSelection.cursor(0),
    extensions: [markdown({ base: markdownLanguage }), marksShown.init(() => false)],
  });
  const out: string[] = [];
  seen(state, [{ from: 0, to: doc.length }]).between(0, doc.length, (from, to, deco) => {
    const spec = deco.spec as { class?: string };
    if (from === to && spec.class) out.push(`${state.doc.lineAt(from).number}:${spec.class}`);
  });
  return out;
};

const sheet = (html: string) => {
  const page = document.implementation.createHTMLDocument("p");
  page.body.innerHTML = `<article class="page">${html}</article>`;
  return page;
};

describe("properties at the top of a note", () => {
  it("are written as they are, never read as a rule over a title", () => {
    const found = lines("---\ntitle: x\n---\n\n# Hola");
    expect(found).toEqual(["1:cm-front", "2:cm-front", "3:cm-front", "5:cm-h1"]);
    expect(lines("# Hola\n\n---\nx\n---").some((one) => one.endsWith("cm-front"))).toBe(false);
  });

  it("are opened from the panel by an entry that is no heading", () => {
    const page = sheet('<dl class="front" data-at="0-9"></dl><h1>Plan</h1><h2>Uno</h2>');
    const found = heads(page);
    expect(found.map((one) => one.level)).toEqual([0, 1, 2]);
    let opened = false;
    page.querySelector(".front")?.addEventListener("click", () => {
      opened = true;
    });
    found[0].go();
    expect(opened).toBe(true);
    expect(heads(sheet("<h1>Plan</h1>")).map((one) => one.level)).toEqual([1]);
  });

  it("leave the title look to the block that follows them", () => {
    const page = document.createElement("article");
    page.className = "page";
    const front = document.createElement("dl");
    front.className = "front";
    const title = document.createElement("h1");
    title.textContent = "Plan";
    page.append(front, title);
    document.body.append(page);

    const opened = edited(
      front,
      "---\ntitle: Plan\n---",
      () => {},
      () => {},
    );
    const host = page.querySelector(".editing");
    expect(host?.classList.contains("fronted")).toBe(true);
    expect(host?.classList.contains("titled")).toBe(false);
    opened.leave();

    page.replaceChildren(front, title);
    const named = edited(
      title,
      "# Plan",
      () => {},
      () => {},
    );
    expect(page.querySelector(".editing")?.classList.contains("titled")).toBe(true);
    named.leave();
    page.remove();
  });
});

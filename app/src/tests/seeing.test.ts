import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import {
  hosted,
  keepShown,
  marked,
  marksShown,
  revealing,
  seen,
  showMarks,
  visual,
} from "tisty-editor/edit";
import { describe, expect, it } from "vitest";
import { HOST } from "../ui/hosting";

const state = (doc: string, at = doc.length, shown = false) =>
  EditorState.create({
    doc,
    selection: EditorSelection.cursor(at),
    extensions: [
      hosted.of(HOST),
      markdown({ base: markdownLanguage }),
      marksShown.init(() => shown),
    ],
  });

const wholly = (doc: string, at: number) =>
  EditorState.create({
    doc,
    selection: EditorSelection.cursor(at),
    extensions: [
      hosted.of(HOST),
      markdown({ base: markdownLanguage }),
      marksShown.init(() => false),
      revealing.of(true),
    ],
  });

type Seen = { from: number; to: number; line?: string; widget?: string; mark?: string };

const looked = (one: EditorState): Seen[] => {
  const out: Seen[] = [];
  seen(one, [{ from: 0, to: one.doc.length }]).between(0, one.doc.length, (from, to, deco) => {
    const spec = deco.spec as { class?: string; widget?: object };
    if (from === to && spec.class) out.push({ from, to, line: spec.class });
    else if (spec.widget) out.push({ from, to, widget: spec.widget.constructor.name });
    else if (spec.class) out.push({ from, to, mark: spec.class });
    else out.push({ from, to });
  });
  return out;
};

// What a person reads in the block: the words with every hidden mark taken out.
const read = (one: EditorState): string => {
  const hidden = looked(one).filter((it) => it.from < it.to && !it.mark);
  let text = "";
  for (let at = 0; at < one.doc.length; at++) {
    if (!hidden.some((it) => at >= it.from && at < it.to)) text += one.sliceDoc(at, at + 1);
  }
  return text;
};

describe("a block written as it reads", () => {
  it("draws a title at its size and keeps its marks out of sight", () => {
    const one = state("# Plan semanal");
    expect(looked(one)).toContainEqual({ from: 0, to: 0, line: "cm-h1" });
    expect(read(one)).toBe("Plan semanal");
    expect(looked(state("#"))).toEqual([]);
    expect(read(state("# "))).toBe("");
    expect(looked(state("# "))).toContainEqual({ from: 0, to: 0, line: "cm-h1" });
  });

  it("keeps a bare address in sight inside a format, and a link's own out of it", () => {
    expect(read(state("**a http://x.y** fin", 0))).toBe("a http://x.y fin");
    expect(read(state("*http://x.y* fin", 0))).toBe("http://x.y fin");
    expect(read(state("~~http://x.y~~ fin", 0))).toBe("http://x.y fin");
    expect(read(state("**[a](http://x.y)** fin", 0))).toBe("a fin");
  });

  it("keeps a format's marks hidden wherever the caret or the selection stands", () => {
    expect(read(state("toca **ordenar** hoy", 0))).toBe("toca ordenar hoy");
    expect(read(state("toca **ordenar** hoy", 9))).toBe("toca ordenar hoy");
    expect(read(state("toca **ordenar** hoy", 16))).toBe("toca ordenar hoy");
    expect(read(state("un ***k*** ya", 10))).toBe("un k ya");
    expect(read(state("el ==dato=={green} y", 6))).toBe("el dato y");
    expect(read(state("un ~~no~~ y `si`", 0))).toBe("un no y si");
    const chosen = state("toca **ordenar** hoy").update({
      selection: { anchor: 8, head: 12 },
    }).state;
    expect(read(chosen)).toBe("toca ordenar hoy");
    const across = state(":::details A\n\nx\n\n:::").update({
      selection: { anchor: 0, head: 20 },
    }).state;
    expect(read(across)).toBe("A\n\nx\n\n");
  });

  it("opens a link where the caret stands, whose address has no other way to be written", () => {
    expect(read(state("ver [el plano](https://x.y) hoy", 7))).toBe(
      "ver [el plano](https://x.y) hoy",
    );
    expect(read(state("ver [el plano](https://x.y) hoy", 0))).toBe("ver el plano hoy");
    const chosen = state("ver [el plano](https://x.y) hoy").update({
      selection: { anchor: 6, head: 9 },
    }).state;
    expect(read(chosen)).toBe("ver el plano hoy");
  });

  it("opens a format under the caret and the selection in the whole document", () => {
    expect(read(wholly("toca **ordenar** hoy", 9))).toBe("toca **ordenar** hoy");
    expect(read(wholly("toca **ordenar** hoy", 18))).toBe("toca ordenar hoy");
    expect(read(wholly("un ***k*** ya", 10))).toBe("un ***k*** ya");
    expect(read(wholly("el ==dato=={green} y", 6))).toBe("el ==dato=={green} y");
    const chosen = wholly("toca **ordenar** hoy", 0).update({
      selection: { anchor: 8, head: 12 },
    }).state;
    expect(read(chosen)).toBe("toca **ordenar** hoy");
  });

  it("shows every mark while Markdown is switched on, sizes still drawn", () => {
    const one = state("# Plan\n\ntoca **ordenar**", 0, true);
    expect(read(one)).toBe("# Plan\n\ntoca **ordenar**");
    expect(looked(one)).toContainEqual({ from: 0, to: 0, line: "cm-h1" });
  });

  it("draws stages as boxes, the done ones dimmed, and loose items as bullets", () => {
    const one = state("- [ ] uno\n- [x] dos\n\n- tres", 0);
    const widgets = looked(one).filter((it) => it.widget);
    expect(widgets.map((it) => it.widget)).toEqual(["Check", "Check", "Bullet"]);
    expect(looked(one)).toContainEqual({ from: 10, to: 10, line: "cm-done" });
    expect(read(one)).toBe("uno\ndos\n\ntres");
  });

  it("draws a callout with its name and a quote with its rule, without their marks", () => {
    const tip = state("> [!TIP]\n> Si algo no cabe hoy", 0);
    expect(
      looked(tip)
        .filter((it) => it.line)
        .map((it) => it.line),
    ).toEqual(["cm-callout cm-callout-tip", "cm-callout cm-callout-tip"]);
    expect(looked(tip).some((it) => it.widget === "Said")).toBe(true);
    expect(read(tip)).toBe("\nSi algo no cabe hoy");
    expect(looked(state("> dicho", 0))).toContainEqual({ from: 0, to: 0, line: "cm-quote" });
  });

  it("draws a sticky note with its name in place of its head, and keeps a title in sight", () => {
    const stuck = state("> [!IDEA] 2026-11-12 09:12 · blue · right\n> Probar con harina", 60);
    expect(
      looked(stuck)
        .filter((it) => it.line)
        .map((it) => it.line),
    ).toEqual(["cm-callout cm-callout-idea", "cm-callout cm-callout-idea"]);
    expect(read(stuck)).toBe("\nProbar con harina");
    const titled = state("> [!idea] Llamar al vet\n> el lunes", 30);
    expect(read(titled)).toBe(" Llamar al vet\nel lunes");
  });

  it("shows Tisty's pens and underline without their marks, and its icons as icons", () => {
    expect(read(state("el ==informe=={green} ya", 0))).toBe("el informe ya");
    expect(read(state("un ++subrayado++ aquí", 0))).toBe("un subrayado aquí");
    expect(read(state("un ==gris=={silver} ya", 0))).toBe("un ==gris=={silver} ya");
    expect(read(state("un ==sol=={yellow} ya", 0))).toBe("un sol ya");
    const icon = looked(state("Llamar al ::star amber:: hoy", 0));
    expect(icon.filter((it) => it.widget).map((it) => it.widget)).toEqual(["Glyph"]);
    expect(looked(state("::nada:: aquí", 12)).some((it) => it.widget)).toBe(false);
  });

  it("shows a link as its words, and leaves brackets that lead nowhere alone", () => {
    const one = state("ver [el plano](https://x.y) hoy", 0);
    expect(read(one)).toBe("ver el plano hoy");
    expect(looked(one).some((it) => it.mark === "cm-link")).toBe(true);
    expect(read(state("ver [nota] hoy", 0))).toBe("ver [nota] hoy");
  });

  it("lets the caret walk into a link's words, and never hides across a line", () => {
    const view = new EditorView({
      parent: document.body,
      state: EditorState.create({
        doc: "ver [el plano](https://x.y) hoy",
        selection: EditorSelection.cursor(5),
        extensions: [hosted.of(HOST), markdown({ base: markdownLanguage }), visual],
      }),
    });
    view.dispatch({ selection: { anchor: 7 } });
    expect(view.state.selection.main.head).toBe(7);
    view.destroy();
    expect(read(state('[a](http://x "uno\ndos") y', 0))).toContain("\n");
  });

  it("wears a tag's chip while it is written, but not on a ticket, a title or code", () => {
    const tags = (doc: string) =>
      looked(state(doc, 0))
        .filter((it) => it.mark === "cm-tag")
        .map((it) => doc.slice(it.from, it.to));
    expect(tags("#Hola Mundo y #sdas")).toEqual(["#Hola", "#sdas"]);
    expect(tags("# Hola, ticket #1234, color:#fff y `#code`")).toEqual([]);
  });

  it("draws a collapsible section as its summary over an indented body", () => {
    const one = state(":::details Más\n\nadentro\n\n:::\n\nfuera", 30);
    const lines = looked(one)
      .filter((it) => it.line)
      .map((it) => `${it.from}:${it.line}`);
    expect(lines).toEqual([
      "0:cm-fold-head",
      "15:cm-fold-body",
      "16:cm-fold-body",
      "24:cm-fold-body",
      "25:cm-fold-end",
    ]);
    expect(read(one)).toBe("Más\n\nadentro\n\n\n\nfuera");
    expect(read(state(":::details Más\n\n:::", 0, true))).toBe(":::details Más\n\n:::");
    expect(looked(state(":::\n\nsolo", 0)).some((it) => it.line)).toBe(false);
  });

  it("pairs a section's fences as the page does, and opens them where the caret touches", () => {
    const nested = ":::details A\n\n:::tip\n\nx\n\n:::\n\ny\n\n:::\n\nfuera";
    expect(read(state(nested, nested.length))).toBe("A\n\n\n\nx\n\n\n\ny\n\n\n\nfuera");
    expect(read(state(":::detailsTodo\n\nx\n\n:::", 16))).toBe("Todo\n\nx\n\n");
    expect(read(state(":::details A\n\nx\n\n:::", 0))).toBe(":::details A\n\nx\n\n");
    const fenced = "```\n:::details x\n```\n\ndespués";
    expect(looked(state(fenced, fenced.length)).some((it) => it.line?.startsWith("cm-fold"))).toBe(
      false,
    );
    expect(looked(state(":::details A\nx\n\n:::", 14)).some((it) => it.widget)).toBe(false);
    expect(read(state(":::details A\n\nsin cierre", 20))).toBe(":::details A\n\nsin cierre");
    expect(read(state("## Tema\n:::details A\n\nx\n\n:::", 23))).toBe("Tema\nA\n\nx\n\n");
    expect(read(state("  :::details A\n\nx\n\n:::", 18))).toBe("  A\n\nx\n\n");
    const bodies = looked(state(nested, nested.length))
      .filter((it) => it.line === "cm-fold-body")
      .map((it) => it.from);
    expect(bodies).toHaveLength(9);
  });

  it("draws a callout written as a colon container like the one written as a quote", () => {
    const one = state(":::tip\n\nRiega las plantas.\n\n:::\n\nfuera", 36);
    const lines = looked(one).filter((it) => it.line);
    expect(lines.map((it) => it.line)).toEqual(Array(5).fill("cm-callout cm-callout-tip"));
    expect(looked(one).some((it) => it.widget === "Said")).toBe(true);
    expect(read(one)).toBe("\n\nRiega las plantas.\n\n\n\nfuera");
    expect(read(state(":::tip\n\nOjo\n\n:::", 0))).toContain(":::tip");
  });

  it("leaves code, cards and attachments exactly as written", () => {
    expect(read(state("```\n**x**\n```", 0))).toBe("```\n**x**\n```");
    expect(read(state("![Rina](tisty:doc/ab12-0003)", 0))).toBe("![Rina](tisty:doc/ab12-0003)");
  });

  it("switches the marks on and off in a live block, and ticks a stage on a press", () => {
    const view = new EditorView({
      parent: document.body,
      state: EditorState.create({
        doc: "- [ ] uno",
        extensions: [hosted.of(HOST), markdown({ base: markdownLanguage }), visual],
      }),
    });
    expect(view.dom.querySelector(".cm-check")).not.toBeNull();
    view.dom
      .querySelector(".cm-check")
      ?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    expect(view.state.sliceDoc()).toBe("- [x] uno");
    view.dispatch({ effects: showMarks.of(true) });
    expect(view.dom.querySelector(".cm-check")).toBeNull();
    expect(view.state.field(marksShown)).toBe(true);
    view.destroy();
  });
});

describe("whether a block has marks to show", () => {
  it("finds none in plain words", () => {
    expect(marked(state("Tisty guarda lo que escribes, sin más."))).toBe(false);
  });

  it("finds Markdown's own marks, wherever the caret is", () => {
    expect(marked(state("# Título", 0))).toBe(true);
    expect(marked(state("> dicho", 0))).toBe(true);
    expect(marked(state("- uno", 0))).toBe(true);
    expect(marked(state("algo **fuerte**", 0))).toBe(true);
    expect(marked(state("un `código`", 0))).toBe(true);
    expect(marked(state("un [enlace](https://tisty.app)", 0))).toBe(true);
  });

  it("finds Tisty's own marks and the properties", () => {
    expect(marked(state("==resaltado=={green}", 0))).toBe(true);
    expect(marked(state("++subrayado++", 0))).toBe(true);
    expect(marked(state("---\ntitle: Acta\n---\nhola", 0))).toBe(true);
  });
});

describe("the marks as the person left them", () => {
  it("opens the next block as the last one was left, and starts hidden on a first day", () => {
    localStorage.removeItem("tisty.marksShown");
    expect(EditorState.create({ extensions: [marksShown] }).field(marksShown)).toBe(false);
    keepShown(true);
    expect(localStorage.getItem("tisty.marksShown")).toBe("1");
    expect(EditorState.create({ extensions: [marksShown] }).field(marksShown)).toBe(true);
    keepShown(false);
    expect(localStorage.getItem("tisty.marksShown")).toBe("0");
    expect(EditorState.create({ extensions: [marksShown] }).field(marksShown)).toBe(false);
  });
});

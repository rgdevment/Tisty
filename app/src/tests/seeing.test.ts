import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { describe, expect, it } from "vitest";
import { marksShown, seen, showMarks, visual } from "../ui/seeing";

const state = (doc: string, at = doc.length, shown = false) =>
  EditorState.create({
    doc,
    selection: EditorSelection.cursor(at),
    extensions: [markdown({ base: markdownLanguage }), marksShown.init(() => shown)],
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

  it("hides the marks of a format until the caret steps into it", () => {
    expect(read(state("toca **ordenar** hoy", 0))).toBe("toca ordenar hoy");
    expect(read(state("toca **ordenar** hoy", 9))).toBe("toca **ordenar** hoy");
    expect(read(state("toca **ordenar** hoy", 16))).toBe("toca **ordenar** hoy");
    expect(read(state("toca **ordenar** hoy", 18))).toBe("toca ordenar hoy");
    expect(read(state("el ==dato=={green} y", 6))).toBe("el ==dato=={green} y");
    expect(read(state("el ==dato=={green} y", 0))).toBe("el dato y");
    expect(read(state("un ~~no~~ y `si`", 0))).toBe("un no y si");
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

  it("shows Tisty's pens and underline without their marks, and its icons as icons", () => {
    expect(read(state("el ==informe=={green} ya", 0))).toBe("el informe ya");
    expect(read(state("un ++subrayado++ aquí", 0))).toBe("un subrayado aquí");
    expect(read(state("un ==gris=={silver} ya", 0))).toBe("un ==gris=={silver} ya");
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
        extensions: [markdown({ base: markdownLanguage }), visual],
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

  it("leaves code, cards and attachments exactly as written", () => {
    expect(read(state("```\n**x**\n```", 0))).toBe("```\n**x**\n```");
    expect(read(state("![Rina](tisty:doc/ab12-0003)", 0))).toBe("![Rina](tisty:doc/ab12-0003)");
  });

  it("switches the marks on and off in a live block, and ticks a stage on a press", () => {
    const view = new EditorView({
      parent: document.body,
      state: EditorState.create({
        doc: "- [ ] uno",
        extensions: [markdown({ base: markdownLanguage }), visual],
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

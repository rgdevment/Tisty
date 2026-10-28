import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { describe, expect, it } from "vitest";
import { t } from "../locales";
import { barred, flipped, kindOf, retyped } from "../ui/barring";
import { marksShown, visual } from "../ui/seeing";
import type { Asked, Asking } from "../ui/slashing";

const view = (doc: string, from = 0, to = from) =>
  new EditorView({
    parent: document.body,
    state: EditorState.create({
      doc,
      selection: EditorSelection.single(from, to),
      extensions: [markdown({ base: markdownLanguage }), visual],
    }),
  });

describe("the bar over the block being written", () => {
  it("steps aside inside a fenced block or a $$ formula, where formatting writes nothing", () => {
    // The caret sits at the `|` of each text.
    const cases: [string, boolean][] = [
      ["```mermaid\nclassDiagram\n  class A|\n```", true],
      ["|```mermaid\ngraph TD\n```", true],
      ["```js\nlet a = 1|\n```", true],
      ["$$\nx^2 + y|^2\n$$", true],
      ["$$a + b|$$", true],
      ["texto con $x|$ en línea", false],
      ["un párrafo|", false],
      ["```\ncódigo\n```\n\nfuera|", false],
      ["$$\nx\n$$\n\ndespués|", false],
    ];
    for (const [written, hidden] of cases) {
      const at = written.indexOf("|");
      const editor = view(written.replace("|", ""), at);
      const { bar, follow } = barred(editor);
      follow(editor);
      expect(bar.hidden, written).toBe(hidden);
      editor.destroy();
    }
  });

  it("names the kind of the line the caret is on", () => {
    expect(kindOf("# Plan")).toBe("bigTitle");
    expect(kindOf("## Plan")).toBe("midTitle");
    expect(kindOf("#### Plan")).toBe("smallTitle");
    expect(kindOf("- [x] hecho")).toBe("checks");
    expect(kindOf("* uno")).toBe("bullets");
    expect(kindOf("2) dos")).toBe("numbers");
    expect(kindOf("> dicho")).toBe("quote");
    expect(kindOf("nada más")).toBe("paragraph");
    expect(kindOf("  - hija")).toBe("bullets");
  });

  it("turns every line the selection touches into the kind asked for, in place of the old one", () => {
    const editor = view("uno\n## dos\n- [ ] tres", 0, 19);
    retyped(editor, "- ");
    expect(editor.state.sliceDoc()).toBe("- uno\n- dos\n- tres");
    retyped(editor, "");
    expect(editor.state.sliceDoc()).toBe("uno\ndos\ntres");
    editor.destroy();
    const nested = view("- uno\n  - dos\nfin", 6, 14);
    retyped(nested, "1. ");
    expect(nested.state.sliceDoc()).toBe("- uno\n  1. dos\nfin");
    retyped(nested, "# ");
    expect(nested.state.sliceDoc()).toBe("- uno\n# dos\nfin");
    nested.destroy();
  });

  it("offers a table a press away, except inside a table, where it offers the table's shape", () => {
    const doc = "Leche\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n";
    const editor = view(doc, 2);
    let asked = 0;
    const { bar, follow } = barred(editor, false, undefined, () => {
      asked += 1;
    });
    const button = bar.querySelector<HTMLButtonElement>(".cm-bar-table") as HTMLButtonElement;
    expect(button.getAttribute("aria-label")).toBe(t("table"));
    expect(button.hidden).toBe(false);
    button.click();
    expect(asked).toBe(1);

    editor.dispatch({ selection: { anchor: doc.indexOf("1 |") } });
    follow(editor);
    expect(button.hidden).toBe(true);
    expect(barred(editor).bar.querySelector(".cm-bar-table")).toBeNull();
    editor.destroy();
  });

  it("switches the block's marks on and back off", () => {
    const editor = view("**hola**");
    flipped(editor);
    expect(editor.state.field(marksShown)).toBe(true);
    flipped(editor);
    expect(editor.state.field(marksShown)).toBe(false);
    editor.destroy();
  });

  it("tells under a document's block the keys it answers to, and opens the whole from the bar", () => {
    const editor = view("leche y pan");
    let asked = 0;
    const { bar, hint } = barred(editor, false, () => {
      asked += 1;
    });
    const items = Array.from(hint.querySelectorAll(".cm-hint-item")).map((one) => one.textContent);
    expect(items).toMatchInlineSnapshot(`
      [
        "Ctrl / insert",
        "Ctrl Shift M marks",
        "Ctrl E whole document",
      ]
    `);
    const wide = bar.querySelector<HTMLButtonElement>(".cm-bar-whole");
    expect(wide?.getAttribute("aria-label")).toMatchInlineSnapshot(
      `"Edit the whole document (Ctrl E)"`,
    );
    expect(wide?.getAttribute("aria-pressed")).toBe("false");
    wide?.click();
    expect(asked).toBe(1);
    editor.destroy();
  });

  it("says, with the whole document open, that the same key goes back to reading", () => {
    const editor = view("leche y pan");
    const { bar, hint } = barred(editor, true, () => {});
    const items = Array.from(hint.querySelectorAll(".cm-hint-item")).map((one) => one.textContent);
    expect(items).toMatchInlineSnapshot(`
      [
        "Ctrl / insert",
        "Ctrl Shift M marks",
        "Ctrl E back to reading",
      ]
    `);
    const wide = bar.querySelector<HTMLButtonElement>(".cm-bar-whole");
    expect(wide?.getAttribute("aria-label")).toMatchInlineSnapshot(`"Back to reading (Ctrl E)"`);
    expect(wide?.getAttribute("aria-pressed")).toBe("true");
    editor.destroy();
  });

  it("offers a task's page no whole document, neither in the bar nor among the keys", () => {
    const editor = view("leche y pan");
    const { bar, hint } = barred(editor);
    expect(bar.querySelector(".cm-bar-whole")).toBeNull();
    const items = Array.from(hint.querySelectorAll(".cm-hint-item")).map((one) => one.textContent);
    expect(items).toMatchInlineSnapshot(`
      [
        "Ctrl / insert",
        "Ctrl Shift M marks",
      ]
    `);
    editor.destroy();
  });

  it("offers the marks only where a block has some, with its key beside it", () => {
    localStorage.removeItem("tisty.marksShown");
    const plain = view("leche y pan");
    const bare = barred(plain);
    const marks = () => bare.bar.querySelector<HTMLButtonElement>(".cm-bar-marks");
    const told = () =>
      Array.from(bare.hint.querySelectorAll<HTMLElement>(".cm-hint-item")).find((one) =>
        one.textContent?.includes(t("hintMarks")),
      );
    expect(marks()?.hidden).toBe(true);
    expect(told()?.hidden).toBe(true);
    plain.dispatch({ changes: { from: 0, insert: "**" }, selection: { anchor: 2 } });
    plain.dispatch({ changes: { from: 7, insert: "**" } });
    bare.follow(plain);
    expect(marks()?.hidden).toBe(false);
    expect(told()?.hidden).toBe(false);
    plain.destroy();
  });

  it("shows the marks switch held while it is on, keeps it there with no marks left, and remembers it", () => {
    localStorage.removeItem("tisty.marksShown");
    const editor = view("**leche** y pan");
    const { bar, follow } = barred(editor);
    const marks = bar.querySelector<HTMLButtonElement>(".cm-bar-marks");
    expect(marks?.getAttribute("aria-label")).toMatchInlineSnapshot(
      `"Show every mark (Ctrl Shift M)"`,
    );
    expect(marks?.getAttribute("aria-pressed")).toBe("false");
    marks?.click();
    follow(editor);
    expect(editor.state.field(marksShown)).toBe(true);
    expect(marks?.getAttribute("aria-pressed")).toBe("true");
    expect(localStorage.getItem("tisty.marksShown")).toBe("1");
    editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "leche" } });
    follow(editor);
    expect(marks?.hidden).toBe(false);
    marks?.click();
    follow(editor);
    expect(marks?.getAttribute("aria-pressed")).toBe("false");
    expect(marks?.hidden).toBe(true);
    expect(localStorage.getItem("tisty.marksShown")).toBe("0");
    editor.destroy();
  });

  it("lays the whole document's switch before the marks, and the insert menu last but the fold", () => {
    const editor = view("**leche**");
    const { bar } = barred(editor, false, () => {});
    const order = Array.from(bar.querySelectorAll("button"))
      .map((one) => one.className)
      .filter((one) => /cm-bar-(whole|marks|more|fold)/.test(one))
      .map((one) => /cm-bar-(whole|marks|more|fold)/.exec(one)?.[1]);
    expect(order).toMatchInlineSnapshot(`
      [
        "whole",
        "marks",
        "more",
        "fold",
      ]
    `);
    editor.destroy();
  });

  it("marks words as code with Ctrl+Shift+C, since Ctrl+E opens the whole document", () => {
    const editor = view("leche y pan");
    const { bar } = barred(editor);
    expect(
      bar.querySelector<HTMLButtonElement>(".cm-bar-code")?.getAttribute("aria-label"),
    ).toMatchInlineSnapshot(`"Code (Ctrl Shift C)"`);
    editor.destroy();
  });

  it("formats, pens and changes the kind from its buttons, and says whether the words are kept", () => {
    const editor = view("leche y pan", 0, 5);
    const { bar, hint, follow, kept } = barred(editor);
    document.body.append(bar, hint);
    const press = (label: string) =>
      bar.querySelector<HTMLButtonElement>(`button[aria-label^="${label}"]`)?.click();

    press(t("bold"));
    expect(editor.state.sliceDoc()).toBe("**leche** y pan");
    editor.dispatch({ selection: { anchor: 12, head: 15 } });
    press(t("penGreen"));
    expect(editor.state.sliceDoc()).toBe("**leche** y ==pan=={green}");

    expect(bar.querySelector('[role="switch"]')).toBeNull();
    expect(hint.textContent).toContain(t("hintMarks"));

    bar.querySelector<HTMLButtonElement>(".cm-bar-kind")?.click();
    expect(bar.querySelector<HTMLElement>(".cm-bar-menu")?.hidden).toBe(false);
    bar.querySelectorAll<HTMLButtonElement>(".cm-bar-item")[1]?.click();
    follow(editor);
    expect(editor.state.sliceDoc().startsWith("# ")).toBe(true);
    expect(bar.querySelector(".cm-bar-kind")?.textContent).toContain(t("bigTitle"));
    bar.querySelector<HTMLButtonElement>(".cm-bar-kind")?.click();
    bar
      .querySelector(".cm-bar-more")
      ?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    expect(bar.querySelector<HTMLElement>(".cm-bar-menu")?.hidden).toBe(true);

    kept(false);
    expect(hint.textContent).toContain(t("saving"));
    kept(true);
    expect(hint.textContent).toContain(t("keptNow"));
    editor.destroy();
  });

  it("opens the insert menu from its button without writing a slash into the words", () => {
    const editor = view("hola");
    const { bar } = barred(editor);
    editor.dispatch({ selection: { anchor: 4 } });
    bar.querySelector<HTMLButtonElement>(".cm-bar-more")?.click();
    expect(editor.state.sliceDoc()).toBe("hola");
    editor.destroy();
  });

  it("never takes the caret out of the words when one of its buttons is pressed", () => {
    const editor = view("hola");
    const { bar } = barred(editor);
    const pressed = new MouseEvent("mousedown", { bubbles: true, cancelable: true });
    bar.querySelector("button")?.dispatchEvent(pressed);
    expect(pressed.defaultPrevented).toBe(true);
    editor.destroy();
  });
});

describe("the diagram button of the bar", () => {
  const asking = (kinds: Asked[]) => {
    const heard: [Asked, boolean][] = [];
    const told: Asking = {
      kinds,
      ask: (kind, _at, put, first) => {
        heard.push([kind, first]);
        put("```mermaid\nmindmap\n  raíz\n```");
      },
    };
    return { heard, told };
  };

  it("asks for a diagram where the caret is, leaving a selection in place", () => {
    const editor = view("# Plan\n\nleche y pan", 9, 14);
    const { heard, told } = asking(["diagram", "diagramFrom"]);
    const { bar } = barred(editor, false, undefined, undefined, () => told);

    bar.querySelector<HTMLButtonElement>(".cm-bar-diagram")?.click();

    expect(heard).toEqual([["diagram", false]]);
    expect(editor.state.sliceDoc()).toBe(
      "# Plan\n\nleche\n\n```mermaid\nmindmap\n  raíz\n```\n\ny pan",
    );
    editor.destroy();
  });

  it("is not there where diagrams cannot be asked for", () => {
    const editor = view("leche");
    const { told } = asking(["icon", "file"]);

    expect(barred(editor).bar.querySelector(".cm-bar-diagram")).toBeNull();
    expect(
      barred(editor, false, undefined, undefined, () => told).bar.querySelector(".cm-bar-diagram"),
    ).toBeNull();
    editor.destroy();
  });
});

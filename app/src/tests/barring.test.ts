import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { describe, expect, it } from "vitest";
import { t } from "../locales";
import { barred, flipped, kindOf, retyped } from "../ui/barring";
import { marksShown, visual } from "../ui/seeing";

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

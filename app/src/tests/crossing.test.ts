import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView, keymap, runScopeHandlers } from "@codemirror/view";
import { type Crossing, crossingKeys, enteredAt, hosted, visual } from "tisty-editor/edit";
import { afterEach, describe, expect, it, vi } from "vitest";
import { HOST } from "../ui/hosting";
import { joined } from "../ui/parting";

const left: (() => void)[] = [];

afterEach(() => {
  for (const one of left.splice(0)) one();
});

const onPage = (
  doc: string,
  at: number,
  around = '<p data-at="0-1">a</p><HOST/><p data-at="9-9">b</p>',
) => {
  const page = document.createElement("div");
  page.className = "page";
  page.innerHTML = around.replace("<HOST/>", '<div class="host"></div><div class="cm-hint"></div>');
  document.body.append(page);
  const host = page.querySelector(".host") as HTMLElement;
  const cross = vi.fn<(one: Crossing) => void>();
  const view = new EditorView({
    parent: host,
    state: EditorState.create({
      doc,
      selection: EditorSelection.cursor(at),
      extensions: [
        hosted.of(HOST),
        markdown({ base: markdownLanguage }),
        visual,
        keymap.of(crossingKeys(host, cross)),
      ],
    }),
  });
  left.push(() => {
    view.destroy();
    page.remove();
  });
  return { view, cross };
};

const press = (view: EditorView, key: string) =>
  runScopeHandlers(view, new KeyboardEvent("keydown", { key }), "editor");

describe("the caret at a block's edge", () => {
  it("goes down from the last line and up from the first", () => {
    const down = onPage("uno\ndos", 7);
    expect(press(down.view, "ArrowDown")).toBe(true);
    expect(down.cross.mock.calls[0]?.[0]).toMatchObject({ forward: true, join: false });
    const up = onPage("uno\ndos", 1);
    expect(press(up.view, "ArrowUp")).toBe(true);
    expect(up.cross.mock.calls[0]?.[0]).toMatchObject({ forward: false, join: false });
  });

  it("stays inside while lines are left to walk", () => {
    const { view, cross } = onPage("uno\ndos", 1);
    expect(press(view, "ArrowDown")).toBe(false);
    expect(cross).not.toHaveBeenCalled();
  });

  it("goes on past the block's last and first letter", () => {
    const on = onPage("uno", 3);
    expect(press(on.view, "ArrowRight")).toBe(true);
    expect(on.cross.mock.calls[0]?.[0]).toMatchObject({ forward: true, x: null });
    const back = onPage("uno", 0);
    expect(press(back.view, "ArrowLeft")).toBe(true);
    expect(back.cross.mock.calls[0]?.[0]).toMatchObject({ forward: false, x: null });
    const inside = onPage("uno", 1);
    expect(press(inside.view, "ArrowLeft")).toBe(false);
  });

  it("goes back from the words of a title, past its hidden mark", () => {
    const { view, cross } = onPage("# Plan", 2);
    expect(press(view, "ArrowLeft")).toBe(true);
    expect(cross).toHaveBeenCalled();
  });

  it("stays when the selection holds words or nothing written lies that way", () => {
    const chosen = onPage("uno", 3);
    chosen.view.dispatch({ selection: { anchor: 0, head: 3 } });
    expect(press(chosen.view, "ArrowRight")).toBe(false);
    const last = onPage("uno", 3, '<p data-at="0-1">a</p><HOST/><figure data-at="9-9"></figure>');
    expect(press(last.view, "ArrowDown")).toBe(false);
    expect(press(last.view, "ArrowRight")).toBe(false);
  });

  it("walks past what cannot be written in place", () => {
    const { view, cross } = onPage(
      "uno",
      3,
      '<HOST/><figure data-at="4-5"></figure><p data-at="6-7">b</p>',
    );
    expect(press(view, "ArrowDown")).toBe(true);
    expect(cross).toHaveBeenCalled();
  });
});

describe("deleting across a block's edge", () => {
  it("joins with the block before from the very start, and with the next from the end", () => {
    const back = onPage("uno", 0);
    expect(press(back.view, "Backspace")).toBe(true);
    expect(back.cross.mock.calls[0]?.[0]).toMatchObject({ forward: false, join: true });
    const on = onPage("uno", 3);
    expect(press(on.view, "Delete")).toBe(true);
    expect(on.cross.mock.calls[0]?.[0]).toMatchObject({ forward: true, join: true });
  });

  it("only leaves an empty block for its neighbour, joining nothing", () => {
    const { view, cross } = onPage("", 0);
    expect(press(view, "Backspace")).toBe(true);
    expect(cross.mock.calls[0]?.[0]).toMatchObject({ forward: false, join: false });
  });

  it("joins only with written words right beside it, and not from code or a table", () => {
    const past = onPage("uno", 0, '<figure data-at="0-1"></figure><HOST/><p data-at="9-9">b</p>');
    expect(press(past.view, "Backspace")).toBe(false);
    const code = onPage("```\nuno\n```", 0);
    expect(press(code.view, "Backspace")).toBe(false);
    const table = onPage("| a |\n| - |\n| b |", 0);
    expect(press(table.view, "Backspace")).toBe(false);
  });

  it("leaves the middle of the words to their own keys", () => {
    const { view, cross } = onPage("uno", 1);
    expect(press(view, "Backspace")).toBe(false);
    expect(press(view, "Delete")).toBe(false);
    expect(cross).not.toHaveBeenCalled();
  });

  it("joins the source, with no gap and without the mark of what follows", () => {
    expect(joined("uno\n\ndos\n", 5)).toEqual({ text: "unodos\n", at: 3 });
    expect(joined("uno\r\n\r\ndos", 7)).toEqual({ text: "unodos", at: 3 });
    expect(joined("uno\n\n## dos", 3)).toEqual({ text: "unodos", at: 3 });
    expect(joined("# uno\n\n- dos\n- tres", 5)).toEqual({ text: "# unodos\n- tres", at: 5 });
  });
});

describe("arriving at a block", () => {
  it("lands at the words or the end when no column is kept", () => {
    const title = onPage("# Plan", 0);
    expect(enteredAt(title.view, true, null)).toBe(2);
    expect(enteredAt(title.view, false, null)).toBe(6);
  });
});

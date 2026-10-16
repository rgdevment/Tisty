import { CompletionContext, type CompletionResult } from "@codemirror/autocomplete";
import { markdown } from "@codemirror/lang-markdown";
import { EditorState } from "@codemirror/state";
import { EditorView, runScopeHandlers } from "@codemirror/view";
import { describe, expect, it } from "vitest";
import { wrapped } from "../ui/formatting";
import { type Asked, holding, offered, put, SLASHED, slashing, tagging } from "../ui/slashing";

const view = (doc: string) =>
  new EditorView({
    parent: document.body,
    state: EditorState.create({ doc, extensions: [markdown(), slashing()] }),
  });

const one = (key: string) => {
  const found = SLASHED.find((each) => each.key === key);
  if (!found) throw new Error(key);
  return found;
};

const asked = (doc: string): CompletionResult | null => {
  const state = EditorState.create({ doc, extensions: [markdown()] });
  return offered()(new CompletionContext(state, doc.length, false));
};

const typed = (doc: string, key: string) => {
  const editor = view(doc);
  const slash = doc.lastIndexOf("/");
  put(editor, one(key), slash, doc.length);
  const said = { text: editor.state.sliceDoc(), at: editor.state.selection.main.head };
  editor.destroy();
  return said;
};

describe("the / menu in a block", () => {
  it("turns the line it is asked on into a title, a list or a quote, with the cursor after the mark", () => {
    expect(typed("/", "h1")).toEqual({ text: "# ", at: 2 });
    expect(typed("/tit", "h2")).toEqual({ text: "## ", at: 3 });
    expect(typed("/", "todo")).toEqual({ text: "- [ ] ", at: 6 });
    expect(typed("/", "fold")).toEqual({ text: ":::details \n\n\n\n:::", at: 11 });
    expect(typed("Leche\n/", "bullets")).toEqual({ text: "Leche\n\n- ", at: 9 });
    expect(typed("Leche\n/", "h1")).toEqual({ text: "Leche\n# ", at: 8 });
  });

  it("starts a line of its own when it is asked in the middle of one", () => {
    expect(typed("Leche /", "quote")).toEqual({ text: "Leche\n\n> ", at: 9 });
    const editor = view("hola / mundo");
    put(editor, one("h1"), 5, 6);
    expect(editor.state.sliceDoc()).toBe("hola\n# mundo");
    editor.destroy();
  });

  it("puts the words after the slash below a block, apart from it", () => {
    const editor = view("Leche /pan");
    put(editor, one("table"), 6, 7);
    expect(editor.state.sliceDoc()).toBe("Leche\n\n|  |  |\n| --- | --- |\n|  |  |\n\npan");
    editor.destroy();
  });

  it("wraps what was selected when a format is picked from the bar's menu", () => {
    const editor = view("uno dos /tres");
    holding(editor, { from: 4, to: 7, slash: 8 });
    put(editor, one("bold"), 8, 9);
    expect(editor.state.sliceDoc()).toBe("uno **dos**tres");
    editor.destroy();
  });

  it("opens a fence with the cursor inside it", () => {
    const said = typed("/", "mermaid");
    expect(said.text).toBe("```mermaid\n\n```");
    expect(said.text.slice(0, said.at)).toBe("```mermaid\n");
  });

  it("writes a callout in Tisty's own words", () => {
    expect(typed("/", "callout-tip").text).toBe("> [!TIP]\n> ");
  });

  it("leaves a blank line before a rule or a table, so the line above is never made a title", () => {
    expect(typed("Leche\n/", "rule").text).toBe("Leche\n\n---\n");
    expect(typed("Leche\n/", "table").text.startsWith("Leche\n\n|")).toBe(true);
    expect(typed("/", "rule").text).toBe("---\n");
  });

  it("puts a link or a pen where the slash was, with the cursor inside", () => {
    expect(typed("Ver /", "link")).toEqual({ text: "Ver []()", at: 5 });
    expect(typed("Ver /", "pen")).toEqual({ text: "Ver ====", at: 6 });
  });

  it("is asked for only at the start of a word, and never inside code", () => {
    expect(asked("/")?.options).toHaveLength(SLASHED.filter((one) => !("ask" in one.put)).length);
    expect(asked("ver /ti")?.from).toBe(4);
    expect(asked("a/b")).toBeNull();
    expect(asked("    /")).toBeNull();
    expect(asked("```\n/")).toBeNull();
  });

  it("wraps what is selected in a format, and leaves the cursor between the marks when nothing is", () => {
    const editor = view("leche y pan");
    editor.dispatch({ selection: { anchor: 0, head: 5 } });
    wrapped(editor, "**");
    expect(editor.state.sliceDoc()).toBe("**leche** y pan");
    editor.dispatch({ selection: { anchor: editor.state.doc.length } });
    wrapped(editor, "[", "]()");
    expect(editor.state.sliceDoc()).toBe("**leche** y pan[]()");
    expect(editor.state.selection.main.head).toBe(16);
    editor.destroy();
  });

  it("keeps the spaces a selection caught outside the marks, so the format still reads", () => {
    const editor = view("Hola mundo");
    editor.dispatch({ selection: { anchor: 0, head: 5 } });
    wrapped(editor, "==");
    expect(editor.state.sliceDoc()).toBe("==Hola== mundo");
    editor.dispatch({ selection: { anchor: 8, head: 14 } });
    wrapped(editor, "**");
    expect(editor.state.sliceDoc()).toBe("==Hola== **mundo**");
    editor.destroy();
  });

  it("takes a format off when it is pressed again, and swaps a pen for another", () => {
    const editor = view("leche ***y*** pan");
    editor.dispatch({ selection: { anchor: 9 } });
    wrapped(editor, "**");
    expect(editor.state.sliceDoc()).toBe("leche *y* pan");
    editor.dispatch({ selection: { anchor: 7 } });
    wrapped(editor, "*");
    expect(editor.state.sliceDoc()).toBe("leche y pan");

    const pen = view("un ==dato=={green} y ++otro++");
    pen.dispatch({ selection: { anchor: 6, head: 8 } });
    wrapped(pen, "==", "=={blue}");
    expect(pen.state.sliceDoc()).toBe("un ==dato=={blue} y ++otro++");
    wrapped(pen, "==", "=={blue}");
    expect(pen.state.sliceDoc()).toBe("un dato y ++otro++");
    pen.dispatch({ selection: { anchor: 14 } });
    wrapped(pen, "++");
    expect(pen.state.sliceDoc()).toBe("un dato y otro");
    pen.dispatch({ selection: { anchor: 3, head: 7 } });
    wrapped(pen, "==");
    expect(pen.state.sliceDoc()).toBe("un ==dato== y otro");
    editor.destroy();
    pen.destroy();
  });

  it("hands a page, a document or an icon to the window and writes what it brings back", () => {
    const editor = view("Ver\n/");
    const asks: Asked[] = [];
    put(editor, one("page"), 4, 5, {
      kinds: ["page"],
      ask: (kind, _, bring) => {
        asks.push(kind);
        bring("![Rina](tisty:doc/abc-0003)");
      },
    });
    expect(asks).toEqual(["page"]);
    expect(editor.state.sliceDoc()).toBe("Ver\n\n![Rina](tisty:doc/abc-0003)");
    editor.destroy();
  });

  it("offers only the pickers the window can open", () => {
    const shown = (kinds: Asked[]) => {
      const state = EditorState.create({ doc: "/", extensions: [markdown()] });
      const said = offered(() => ({ kinds, ask: () => {} }))(
        new CompletionContext(state, 1, false),
      );
      return said?.options.length ?? 0;
    };
    expect(shown(["icon", "doc"]) - shown([])).toBe(2);
  });

  it("narrows by what follows the slash, by label or by key, and says so when nothing fits", () => {
    expect(asked("/mermaid")?.options.map((one) => one.detail)).toEqual(["```mermaid"]);
    expect(asked("/zzzz")).toBeNull();
  });

  it("formats what is selected with the keys the menu names", () => {
    const editor = view("leche y pan");
    const key = (init: KeyboardEventInit, code: number) => {
      const pressed = new KeyboardEvent("keydown", { ctrlKey: true, ...init });
      Object.defineProperty(pressed, "keyCode", { value: code });
      return runScopeHandlers(editor, pressed, "editor");
    };
    editor.dispatch({ selection: { anchor: 0, head: 5 } });
    expect(key({ key: "b" }, 66)).toBe(true);
    expect(editor.state.sliceDoc()).toBe("**leche** y pan");
    editor.dispatch({ selection: { anchor: 12, head: 15 } });
    expect(key({ key: "X", shiftKey: true }, 88)).toBe(true);
    expect(editor.state.sliceDoc()).toBe("**leche** y ~~pan~~");
    editor.destroy();
  });

  it("says a card was not written when its block closed while the window was asked", () => {
    const editor = view("Ver /");
    let bring: ((text: string) => boolean) | undefined;
    put(editor, one("icon"), 4, 5, {
      kinds: ["icon"],
      ask: (_, __, given, first) => {
        expect(first).toBe(false);
        bring = given;
      },
    });
    expect(bring?.("::star::")).toBe(true);
    expect(editor.state.sliceDoc()).toBe("Ver ::star::");
    editor.destroy();
    expect(bring?.("::dog::")).toBe(false);
  });
});

describe("tags offered after a #", () => {
  const offeredTags = (doc: string) => {
    const state = EditorState.create({ doc, extensions: [markdown()] });
    const asking = { kinds: [], ask: () => {}, tags: ["compras", "casa", "trabajo"] };
    return tagging(() => asking)(new CompletionContext(state, doc.length, false));
  };

  it("offers the tags in use that fit what is typed, as a word with a space after it", () => {
    const found = offeredTags("leche #c");
    expect(found?.from).toBe(6);
    expect(found?.options.map((one) => one.label)).toEqual(["#compras", "#casa"]);
    expect(found?.options[0].apply).toBe("#compras ");
    expect(offeredTags("#tra")?.options.map((one) => one.label)).toEqual(["#trabajo"]);
  });

  it("stays shut on a lone #, a title, a word glued to the #, code, or a tag already whole", () => {
    expect(offeredTags("leche #")).toBeNull();
    expect(offeredTags("# Plan")).toBeNull();
    expect(offeredTags("color#c")).toBeNull();
    expect(offeredTags("`#c")).toBeNull();
    expect(offeredTags("#casa")).toBeNull();
    expect(offeredTags("#xyz")).toBeNull();
  });
});

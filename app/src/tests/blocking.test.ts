import { completionStatus } from "@codemirror/autocomplete";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { aimed, caretAt } from "tisty-editor/edit";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { edited } from "../ui/hosting";

const block = () => {
  const page = document.createElement("article");
  page.className = "page";
  const one = document.createElement("p");
  one.textContent = "Leche y pan.";
  page.append(one);
  document.body.append(page);
  return { page, one };
};

describe("writing a block in place", () => {
  it("hands back the words once when it is left, however often it is asked to leave", () => {
    const { page, one } = block();
    const left: string[] = [];
    const editing = edited(
      one,
      "Hola",
      () => {},
      (text) => left.push(text),
    );
    editing.view.dispatch({ changes: { from: 4, insert: "!" } });
    editing.leave();
    editing.leave();
    expect(left).toEqual(["Hola!"]);
    page.remove();
  });

  it("keeps a second Ctrl+A in the block when nothing larger can be written", () => {
    const { page, one } = block();
    const editing = edited(
      one,
      "Hola",
      () => {},
      () => {},
    );
    const press = () =>
      editing.view.contentDOM.dispatchEvent(
        new KeyboardEvent("keydown", { key: "a", ctrlKey: true, bubbles: true, cancelable: true }),
      );
    press();
    press();
    expect(editing.view.state.selection.main).toMatchObject({ from: 0, to: 4 });
    editing.leave();
    page.remove();
  });

  it("lays the keys under the block being written, not at the foot of the page", () => {
    const { page, one } = block();
    const after = document.createElement("p");
    after.textContent = "Después.";
    page.append(after);
    const editing = edited(
      one,
      "Hola",
      () => {},
      () => {},
    );
    const hint = page.querySelector(".cm-hint");
    expect(hint?.previousElementSibling?.classList.contains("editing")).toBe(true);
    expect(hint?.nextElementSibling).toBe(after);
    expect(hint?.classList.contains("under")).toBe(true);
    editing.leave();
    expect(page.querySelector(".cm-hint")).toBeNull();
    page.remove();
  });

  it("keeps files pasted from the clipboard as attachments, and leaves pasted words alone", async () => {
    const { page, one } = block();
    const pasted: string[] = [];
    let lost = 0;
    const editing = edited(
      one,
      "Ver",
      () => {},
      () => {},
      () => ({
        kinds: [],
        ask: () => {},
        pasted: (files) => {
          pasted.push(...files.map((file) => file.name));
          return Promise.resolve("![captura](captura.png)");
        },
        lost: () => {
          lost += 1;
        },
      }),
    );
    const view = editing.view;
    view.dispatch({ selection: { anchor: 3 } });
    const paste = (files: File[]) => {
      const event = new Event("paste", { bubbles: true, cancelable: true });
      Object.defineProperty(event, "clipboardData", { value: { files, getData: () => "" } });
      view.contentDOM.dispatchEvent(event);
      return event;
    };
    paste([]);
    expect(pasted).toEqual([]);
    const sheet = new Event("paste", { bubbles: true, cancelable: true });
    Object.defineProperty(sheet, "clipboardData", {
      value: { files: [new File(["x"], "image.png")], getData: () => "A1\tB1" },
    });
    view.contentDOM.dispatchEvent(sheet);
    expect(pasted).toEqual([]);
    expect(paste([new File(["x"], "captura.png")]).defaultPrevented).toBe(true);
    await vi.waitFor(() =>
      expect(view.state.sliceDoc()).toBe("VerA1\tB1\n\n![captura](captura.png)"),
    );
    expect(pasted).toEqual(["captura.png"]);
    paste([new File(["x"], "tarde.png")]);
    editing.leave();
    await vi.waitFor(() => expect(lost).toBe(1));
    page.remove();
  });

  it("redoes with Ctrl+Shift+Z as well as Ctrl+Y", () => {
    const { page, one } = block();
    const editing = edited(
      one,
      "Hola",
      () => {},
      () => {},
    );
    const view = editing.view;
    view.dispatch({ changes: { from: 4, insert: "!" }, userEvent: "input.type" });
    view.contentDOM.dispatchEvent(
      new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true, cancelable: true }),
    );
    expect(view.state.sliceDoc()).toBe("Hola");
    view.contentDOM.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "Z",
        ctrlKey: true,
        shiftKey: true,
        keyCode: 90,
        bubbles: true,
        cancelable: true,
      }),
    );
    expect(view.state.sliceDoc()).toBe("Hola!");
    editing.leave();
    page.remove();
  });

  describe("Enter while a list is offered", () => {
    const typing = (view: EditorView, text: string) => {
      const at = view.state.doc.length;
      view.dispatch({
        changes: { from: at, insert: text },
        selection: { anchor: at + text.length },
        userEvent: "input.type",
      });
    };
    const enter = (view: EditorView) =>
      view.contentDOM.dispatchEvent(
        new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true }),
      );
    // A list just opened ignores Enter for a moment, so the clock is moved past it.
    const offeredFor = async (view: EditorView) => {
      await vi.waitFor(() => expect(completionStatus(view.state)).toBe("active"));
      vi.setSystemTime(Date.now() + 1000);
    };

    beforeEach(() => vi.useFakeTimers({ toFake: ["Date"] }));
    afterEach(() => vi.useRealTimers());

    it("takes the / menu's choice, even pressed right after a letter, instead of starting a paragraph", async () => {
      const { page, one } = block();
      const editing = edited(
        one,
        "Leche",
        () => {},
        () => {},
      );
      typing(editing.view, " /quo");
      await offeredFor(editing.view);
      typing(editing.view, "t");
      enter(editing.view);
      expect(editing.view.state.doc.toString()).toBe("Leche\n\n> ");
      editing.leave();
      page.remove();
    });

    it("keeps a tag as typed and starts the next paragraph, its list open or not", async () => {
      const { page, one } = block();
      const editing = edited(
        one,
        "Comprar pan",
        () => {},
        () => {},
        () => ({ kinds: [], tags: ["casa"], ask: vi.fn() }),
      );
      typing(editing.view, " #ca");
      await offeredFor(editing.view);
      enter(editing.view);
      expect(editing.view.state.doc.toString()).toBe("Comprar pan #ca\n\n");
      editing.leave();
      page.remove();
    });
  });

  it("puts the drawn block back once it is left, so nothing is missing until the page is drawn again", () => {
    const { page, one } = block();
    const editing = edited(
      one,
      "Leche y pan.",
      () => {},
      () => {},
    );
    expect(page.contains(one)).toBe(false);
    editing.leave();
    expect(page.contains(one)).toBe(true);
    expect(page.querySelector(".editing")).toBeNull();
    page.remove();
  });

  it("leaves a block untouched when nothing was written in it, however wide its gaps", () => {
    const { page, one } = block();
    const left: string[] = [];
    const editing = edited(
      one,
      ":::details A\n\n\n\nx\n\n:::",
      () => {},
      (text) => left.push(text),
    );
    editing.leave();
    expect(left).toEqual([":::details A\n\n\n\nx\n\n:::"]);
    page.remove();
  });

  it("keeps the line endings the block was written with", () => {
    const { page, one } = block();
    const changes: string[] = [];
    const editing = edited(
      one,
      "uno\r\ndos\r\n",
      (text) => changes.push(text),
      () => {},
    );
    expect(editing.view.state.doc.lines).toBe(2);
    editing.view.dispatch({ changes: { from: 3, insert: "!" } });
    editing.flush?.();
    expect(changes).toEqual(["uno!\r\ndos"]);
    editing.leave();
    page.remove();
  });
});

describe("the steps of a block being left", () => {
  const leaving = (said: string, keys: string[]) => {
    const { page, one } = block();
    const changes: string[] = [];
    const left: string[] = [];
    const editing = edited(
      one,
      said,
      (text) => changes.push(text),
      (text) => left.push(text),
    );
    editing.view.dispatch({ selection: { anchor: editing.view.state.doc.length } });
    for (const key of keys) {
      editing.view.contentDOM.dispatchEvent(
        new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }),
      );
    }
    editing.leave();
    page.remove();
    return { changes, left };
  };

  it("drops the empty step an Enter leaves behind, and hands the tidy words back to be kept", () => {
    const { changes, left } = leaving("- [ ] Lavaloza", ["Enter", "Escape"]);
    expect(left).toEqual(["- [ ] Lavaloza"]);
    expect(changes[changes.length - 1]).toBe("- [ ] Lavaloza");
  });

  it("keeps no blank line after the last step, so the next one joins the same list", () => {
    expect(leaving("- [ ] Lavaloza", ["Enter", "Enter"]).left).toEqual(["- [ ] Lavaloza"]);
  });

  it("leaves nothing when the only step was never written", () => {
    expect(leaving("- [ ] ", ["Enter"]).left).toEqual([""]);
  });
});

describe("the caret a click leaves in a block", () => {
  it("sits after the words read before the click, past the marks the page does not show", () => {
    expect(caretAt("Leche y **pan** fresco", "Leche y pan")).toBe(13);
    expect(caretAt("# Compras del mes", "Compras")).toBe(9);
    expect(caretAt("uno, dos, uno, dos", "uno, dos, uno")).toBe(13);
  });

  it("starts at the beginning when nothing was read, and at the end when the words are not found", () => {
    expect(caretAt("Leche", "")).toBe(0);
    expect(caretAt("Leche", "§")).toBe(5);
  });
});

describe("the boxes of a block being written", () => {
  it("say which ones were written over, line by line, so a step keeps who it is", () => {
    const { page, one } = block();
    let told: [string, string][] = [];
    const editing = edited(
      one,
      "- [ ] a\n- [ ] b\n- [ ] c",
      () => {},
      (_, renamed) => {
        told = renamed;
      },
    );
    const doc = editing.view.state.doc;
    editing.view.dispatch({
      changes: [
        { from: doc.line(1).from + 6, to: doc.line(1).to, insert: "call the vet" },
        { from: doc.line(2).from + 6, to: doc.line(2).to, insert: "bring the card" },
      ],
    });
    editing.leave();

    expect(told).toEqual([
      ["a", "call the vet"],
      ["b", "bring the card"],
    ]);
    page.remove();
  });
});

describe("where the bar rests", () => {
  const resting = (doc: string, from: number, to: number) => {
    const view = new EditorView({
      parent: document.body,
      state: EditorState.create({
        doc,
        selection: EditorSelection.single(from, to),
        extensions: [markdown({ base: markdownLanguage })],
      }),
    });
    view.coordsAtPos = () => ({ top: 300, bottom: 320, left: 0, right: 0 });
    view.dom.getBoundingClientRect = () => ({ top: 100 }) as DOMRect;
    const at = aimed(view, false);
    view.destroy();
    return at;
  };

  it("goes over the words chosen, but keeps above a table whose cells are walked", () => {
    const words = "Una frase con palabras.";
    expect(resting(words, 4, 9)).toBe(300);
    expect(resting(words, 4, 4)).toBe(100);
    const table = "| a | b |\n| --- | --- |\n| uno | dos |\n";
    const cell = table.indexOf("dos");
    expect(resting(table, cell, cell + 3)).toBe(100);
  });
});

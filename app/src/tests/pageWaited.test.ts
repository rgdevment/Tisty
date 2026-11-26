import { afterEach, describe, expect, it } from "vitest";
import { applied, landedAfter, reachIn } from "../ui/pageEdits";
import { escapeTaken, keptGone } from "../ui/pageMarks";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY = "# Hola\n\nUno dos tres 👍🏽\n";
const PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-30"><span data-src="8-29">Uno dos tres 👍🏽</span></p>';
const MARKED = "# Hola\n\n我们**公园**。\n";
const MARKED_PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-27"><span data-src="8-14">我们</span><strong><span data-src="16-22">公园</span></strong><span data-src="24-27">。</span></p>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const opened = (body = BODY, placed = PLACED) => {
  let text = body;
  document.body.innerHTML = `<article class="page">${placed}</article>`;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: () => {},
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translator: { fromHtml: async () => "", fromText: async () => null },
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  return () => text;
};

const asked = (inputType: string, data: string | null = null) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(
      new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
    );

const words = () => document.querySelector('[data-src="8-29"]')?.firstChild as Text;

const chosen = (said: string) => {
  const at = words().data.indexOf(said);
  const range = document.createRange();
  range.setStart(words(), at);
  range.setEnd(words(), at + said.length);
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(range);
};

const caretAfter = (said: string) => {
  const at = words().data.indexOf(said) + said.length;
  document.getSelection()?.collapse(words(), at);
};

const left = (text: string, at: number, kind: string) => {
  const reach = (from: number) => reachIn(text, from, kind);
  const key = { back: kind.endsWith("Backward"), reach };
  return applied(text, keptGone(text, escapeTaken(text, reach(at)), key));
};

const whole = (text: string) =>
  !/[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/.test(text);

describe("how far a word's delete reaches in the text", () => {
  it("takes one word of Chinese, not the whole run", () => {
    expect(reachIn("我们今天去公园", 7, "deleteWordBackward")).toEqual({ from: 5, to: 7 });
  });

  it("takes the word with the spaces after it, and stops at a comma", () => {
    expect(reachIn("uno dos  ", 9, "deleteWordBackward")).toEqual({ from: 4, to: 9 });
    expect(reachIn("uno, dos", 8, "deleteWordBackward")).toEqual({ from: 5, to: 8 });
    expect(reachIn("uno dos", 3, "deleteWordForward")).toEqual({ from: 3, to: 7 });
  });

  it("takes a word of Chinese with the stop after it, as a Latin word goes with its full stop", () => {
    expect(reachIn("我们今天去公园。", 8, "deleteWordBackward")).toEqual({ from: 5, to: 8 });
    expect(reachIn("我们今天去公园。」 ", 10, "deleteWordBackward")).toEqual({ from: 5, to: 10 });
    expect(reachIn("Vamos al parque.", 16, "deleteWordBackward")).toEqual({ from: 9, to: 16 });
    expect(reachIn("。我们今天", 0, "deleteWordForward")).toEqual({ from: 0, to: 3 });
    expect(reachIn("公园👍", 4, "deleteWordBackward")).toEqual({ from: 2, to: 4 });
  });

  it("takes an emoji, a link or marks whole, never half their writing nor the word before", () => {
    expect(reachIn("tres 👍🏽", 9, "deleteWordBackward")).toEqual({ from: 5, to: 9 });
    expect(reachIn("ver [docs](https://x.com/a)", 27, "deleteWordBackward")).toEqual({
      from: 4,
      to: 27,
    });
    expect(reachIn("**fuerte** resto", 0, "deleteWordForward")).toEqual({ from: 0, to: 10 });
  });

  it("takes a stop alone when a mark stands between it and its word, so no mark loses its partner", () => {
    expect(left("我们**公园**。", 9, "deleteWordBackward")).toBe("我们**公园**");
    expect(left("我们~~公园~~。", 9, "deleteWordBackward")).toBe("我们~~公园~~");
    expect(left("我们==公园==。", 9, "deleteWordBackward")).toBe("我们==公园==");
    expect(left("我们`公园`。", 7, "deleteWordBackward")).toBe("我们`公园`");
    expect(left("我们[公园](https://x.com)。", 22, "deleteWordBackward")).toBe(
      "我们[公园](https://x.com)",
    );
    expect(left("。**公园**我们", 0, "deleteWordForward")).toBe("**公园**我们");
  });

  it("takes the word inside the marks beside the caret, and a format's marks when it was all they held", () => {
    expect(left("我们**公园**", 8, "deleteWordBackward")).toBe("我们");
    expect(left("我们**今天公园**", 10, "deleteWordBackward")).toBe("我们**今天**");
    expect(left("我们**公园。**", 9, "deleteWordBackward")).toBe("我们");
    expect(left("我们[公园](https://x.com)", 21, "deleteWordBackward")).toBe(
      "我们[](https://x.com)",
    );
    expect(left("**今天公园**我们", 0, "deleteWordForward")).toBe("**公园**我们");
  });

  it("reads an underscore or a plus inside a word and a lone tilde, equals or plus as words, as the page draws them", () => {
    expect(left("使用my_var", 8, "deleteWordBackward")).toBe("使用");
    expect(left("使用my_var的", 2, "deleteWordForward")).toBe("使用的");
    expect(left("答案是1+1=", 7, "deleteWordBackward")).toBe("答案是1+1");
    expect(left("中文_a_", 5, "deleteWordBackward")).toBe("中文");
    expect(left("使用__init__", 10, "deleteWordBackward")).toBe("使用");
    expect(left("使用a_b和c_d", 9, "deleteWordBackward")).toBe("使用a_b和");
    expect(left("snake_case and other_name", 25, "deleteWordBackward")).toBe("snake_case and ");
    expect(left("使用C++和C++", 9, "deleteWordBackward")).toBe("使用C++和C+");
  });

  it("reads marks with no partner on the line as words, so none is left out of reach", () => {
    expect(left("我们**", 4, "deleteWordBackward")).toBe("");
    expect(left("好的~~", 4, "deleteWordBackward")).toBe("好的~");
    expect(left("使用C++", 5, "deleteWordBackward")).toBe("使用C+");
    expect(left("见[注]", 4, "deleteWordBackward")).toBe("见[");
  });

  it("takes the spaces between the caret and the marks first, so the caret stays where it was", () => {
    expect(left("我们**公园**  ", 10, "deleteWordBackward")).toBe("我们**公园**");
    expect(left("  **今天**", 0, "deleteWordForward")).toBe("**今天**");
  });

  it("keeps a colored highlight's closing mark whole", () => {
    expect(left("我们==公园=={red}。", 14, "deleteWordBackward")).toBe("我们==公园=={red}");
    expect(left("我们==公园=={red}", 13, "deleteWordBackward")).toBe("我们");
  });

  it("reads an escaped bracket as a word, never as a link's opening", () => {
    expect(left("\\[a](u) x", 7, "deleteContentBackward")).toBe("\\[a](u x");
  });

  it("takes a link left with no words whole, as the page draws nothing of it", () => {
    expect(left("x [](u) y", 7, "deleteContentBackward")).toBe("x  y");
    expect(left("x [](u) y", 2, "deleteContentForward")).toBe("x  y");
  });

  it("takes an escaped character whole when the key steps over a closing mark to reach it", () => {
    expect(left("**a\\_**", 7, "deleteContentBackward")).toBe("**a**");
  });

  it("takes nothing past the line's words when the key steps over marks at its edge", () => {
    expect(left("- **a**\n- b", 5, "deleteContentForward")).toBe("- **a**\n- b");
    expect(left("# **T**", 4, "deleteContentBackward")).toBe("# **T**");
  });

  it("reads stars in an address or a formula as words, as the page draws them", () => {
    expect(left("<https://x.com/a*b*c>", 18, "deleteContentBackward")).toBe("<https://x.com/a**c>");
    expect(left("$x*y*z$ fin", 4, "deleteContentBackward")).toBe("$x**z$ fin");
  });

  it("reads marks between dollars or angles the page does not draw as a formula or an address as marks", () => {
    expect(left("$5 y **dos** $", 10, "deleteWordBackward")).toBe("$5 y  $");
    expect(left("a <b **c** d> e", 8, "deleteContentBackward")).toBe("a <b  d> e");
  });

  it("pairs stars as the page does where one run closes a format and opens the next", () => {
    expect(left("**a***d*", 7, "deleteContentBackward")).toBe("**a**");
    expect(left("a**b***c*", 8, "deleteContentBackward")).toBe("a**b**");
    expect(left("***a*b**", 6, "deleteContentBackward")).toBe("***a***");
  });

  it("reads an underscore left open when stars close around it as a word, as the page draws it", () => {
    expect(left("*a _b* c_", 9, "deleteContentBackward")).toBe("*a _b* c");
  });

  it("leaves underscores between letters, which the page draws, beside a letter taken", () => {
    expect(left("snake_x_case", 7, "deleteContentBackward")).toBe("snake__case");
  });

  it("takes an escaped mark with its backslash, and a mark after an escaped backslash as a mark", () => {
    expect(left("我们公园\\~", 6, "deleteWordBackward")).toBe("我们公园");
    expect(left("我们\\\\**公园**", 10, "deleteWordBackward")).toBe("我们\\\\");
  });
});

describe("the caret after an edit widened to whole characters", () => {
  it("moves back with the words taken past the end", () => {
    const wanted = { from: 5, to: 6, insert: "" };
    expect(landedAfter(wanted, { ...wanted, to: 7 }, 9)).toBe(8);
    expect(landedAfter(wanted, { ...wanted, to: 7 }, 5)).toBe(5);
    expect(landedAfter(wanted, { ...wanted, from: 4 }, 5)).toBe(4);
  });
});

describe("a key whose browser names a place other than the caret", () => {
  it("writes at the caret shown, not at a stale place before it", () => {
    const text = opened();
    caretAfter("tres");
    const stale = document.createRange();
    stale.setStart(document.querySelector("h1") as Element, 0);
    const event = new InputEvent("beforeinput", {
      inputType: "insertText",
      data: "X",
      bubbles: true,
      cancelable: true,
    });
    Object.defineProperty(event, "getTargetRanges", { value: () => [stale] });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(text()).toBe("# Hola\n\nUno dos tresX 👍🏽\n");
  });
});

describe("keys that come before the page shows the last one", () => {
  it("take a character drawn from several back whole, never half of it", () => {
    const text = opened();
    caretAfter("👍🏽");
    asked("insertText", "a");
    asked("deleteContentBackward");
    asked("deleteContentBackward");
    expect(whole(text())).toBe(true);
    expect(text()).toBe("# Hola\n\nUno dos tres \n");
  });

  it("take the words back to the line's start for a line's delete, not the letter after", () => {
    const text = opened();
    caretAfter("dos");
    asked("insertText", "a");
    asked("deleteSoftLineBackward");
    expect(text()).toBe("# Hola\n\n tres 👍🏽\n");
  });

  it("take a word back for a word's delete", () => {
    const text = opened();
    caretAfter("tres");
    asked("insertText", "x");
    asked("deleteWordBackward");
    expect(text()).toBe("# Hola\n\nUno dos  👍🏽\n");
  });

  it("take a stop back alone past the marks that close a word of Chinese, then the word with them", () => {
    const text = opened(MARKED, MARKED_PLACED);
    const stop = document.querySelector('[data-src="24-27"]')?.firstChild as Text;
    document.getSelection()?.collapse(stop, 1);
    asked("insertText", "X");
    asked("deleteWordBackward");
    asked("deleteWordBackward");
    expect(text()).toBe("# Hola\n\n我们**公园**\n");
    asked("deleteWordBackward");
    expect(text()).toBe("# Hola\n\n我们\n");
  });

  it("write where the last key left the caret, not over words chosen before it", () => {
    const text = opened();
    chosen("dos");
    asked("insertText", "X");
    asked("insertText", "Y");
    expect(text()).toBe("# Hola\n\nUno XY tres 👍🏽\n");
  });
});

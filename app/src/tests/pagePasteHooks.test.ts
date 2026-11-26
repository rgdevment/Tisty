import { afterEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { escapedMarks, TISTY_MARKDOWN, type Translator } from "../ui/clipboarding";
import { wired, worn } from "../ui/dressing";
import type { Hands } from "../ui/pageWriter";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.documentElement.classList.remove("untitled");
  document.body.innerHTML = "";
});

const settled = () => new Promise((done) => setTimeout(done, 0));

const BODY = "# T\n\nHola\n";
const PLACED =
  '<h1 id="t" data-at="0-4"><span data-src="2-3">T</span></h1><p data-at="5-9"><span data-src="5-9">Hola</span></p>';
const CODE = "# T\n\n```js\nx\n```\n";
const CODE_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<figure class="code" data-at="5-17"><pre><code class="language-js" data-src="11-13">x\n</code></pre></figure>';

const translating = () => ({
  fromHtml: vi.fn(async (_html: string) => "HTML"),
  fromText: vi.fn(async (_text: string): Promise<string | null> => null),
});

const onPage = (
  translator: Translator,
  body = BODY,
  placed = PLACED,
  more: Partial<Hands> = {},
) => {
  let text = body;
  document.body.innerHTML = `<article class="page">${placed}</article>`;
  const reach = {
    body: "",
    onDoc: () => {},
    onBegin: (at: number, seed: string, near: Element) => writing?.begin(at, seed, near),
  };
  worn(document, () => true, reach);
  wired(document, () => reach);
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: () => {},
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translator,
    ask: () => {},
    blocks: () => {},
    acts: () => {},
    ...more,
  });
  return () => text;
};

const atEnd = (back = 0) => {
  const words = Array.from(document.querySelectorAll("[data-src]")).pop()?.firstChild as Text;
  document.getSelection()?.collapse(words, words.length - back);
};

const sheet = () => document.querySelector(".page") as HTMLElement;

const keyed = (init: KeyboardEventInit) =>
  sheet().dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init }));
const plainShortcut = (init: KeyboardEventInit = { ctrlKey: true }) =>
  keyed({ code: "KeyV", key: "V", shiftKey: true, ...init });
const ordinaryShortcut = () => keyed({ code: "KeyV", key: "v", ctrlKey: true });

type Faces = { plain?: string; html?: string; own?: string };
const pasted = ({ plain = "", html = "", own = "" }: Faces) => {
  const event = new InputEvent("beforeinput", {
    inputType: "insertFromPaste",
    bubbles: true,
    cancelable: true,
  });
  Object.defineProperty(event, "getTargetRanges", { value: () => [] });
  const faces: Record<string, string> = {
    "text/plain": plain,
    "text/html": html,
    [TISTY_MARKDOWN]: own,
  };
  Object.defineProperty(event, "dataTransfer", {
    value: { getData: (kind: string) => faces[kind] ?? "" },
  });
  sheet().dispatchEvent(event);
};

const RICH =
  "==plumón=={c}\n\n:rocket: ve [la nota](https://a.b/c) y $x^2$\n\n- [ ] tarea\n- [x] hecha";

describe("Tisty's own type pasted", () => {
  it("lands the Markdown byte for byte and never calls the translator", async () => {
    const by = translating();
    const text = onPage(by);
    atEnd();
    pasted({ own: RICH, html: "<b>otra cosa</b>", plain: "otra cosa" });
    await settled();
    expect(text()).toBe(`# T\n\nHola${RICH}\n`);
    expect(by.fromHtml).not.toHaveBeenCalled();
    expect(by.fromText).not.toHaveBeenCalled();
  });

  it("wins over the plain text alone too", async () => {
    const by = translating();
    const text = onPage(by);
    atEnd();
    pasted({ own: "**x**", plain: "x" });
    await settled();
    expect(text()).toBe("# T\n\nHola**x**\n");
    expect(by.fromText).not.toHaveBeenCalled();
  });

  it("lands the own type with CRLF as LF", async () => {
    const text = onPage(translating());
    atEnd();
    pasted({ own: "a\r\nb" });
    await settled();
    expect(text()).not.toContain("\r");
  });
});

describe("paste without formatting", () => {
  it("lands the marks as letters after Ctrl+Shift+V, ignoring html and the Tisty type", async () => {
    const by = translating();
    const text = onPage(by);
    atEnd();
    plainShortcut();
    pasted({ plain: "**hola** # x", html: "<b>hola</b>", own: "==otro==" });
    await settled();
    expect(text()).toBe(`# T\n\nHola${escapedMarks("**hola** # x")}\n`);
    expect(text()).toContain("\\*\\*hola\\*\\* \\# x");
    expect(by.fromHtml).not.toHaveBeenCalled();
    expect(by.fromText).not.toHaveBeenCalled();
  });

  it("works with plain text alone", async () => {
    const text = onPage(translating());
    atEnd();
    plainShortcut();
    pasted({ plain: "**hola** # x" });
    await settled();
    expect(text()).toContain("\\*\\*hola\\*\\* \\# x");
  });

  it("works with Cmd+Shift+V", async () => {
    const text = onPage(translating());
    atEnd();
    plainShortcut({ metaKey: true });
    pasted({ plain: "**hola** # x" });
    await settled();
    expect(text()).toContain("\\*\\*hola\\*\\* \\# x");
  });

  it("is spent by one paste: the next Ctrl+V paste translates again", async () => {
    const by = translating();
    by.fromText.mockImplementation(async (plain: string) => `T(${plain})`);
    const text = onPage(by);
    atEnd();
    plainShortcut();
    pasted({ plain: "**a**" });
    await settled();
    ordinaryShortcut();
    pasted({ plain: "**b**" });
    await settled();
    expect(text()).toContain("\\*\\*a\\*\\*");
    expect(text()).toContain("T(**b**)");
    expect(by.fromText).toHaveBeenCalledTimes(1);
  });

  it("is spent even when no keydown comes between two pastes", async () => {
    const by = translating();
    const text = onPage(by);
    atEnd();
    plainShortcut();
    pasted({ plain: "*a*" });
    pasted({ plain: "*b*" });
    await settled();
    expect(by.fromText).toHaveBeenCalledTimes(1);
    expect(text()).toContain("\\*a\\*");
  });

  it("is cleared by any other key pressed before the paste", async () => {
    const by = translating();
    by.fromText.mockImplementation(async (plain: string) => `T(${plain})`);
    const text = onPage(by);
    atEnd();
    plainShortcut();
    keyed({ key: "a", code: "KeyA" });
    pasted({ plain: "**x**" });
    await settled();
    expect(text()).toContain("T(**x**)");
    expect(text()).not.toContain("\\*");
  });

  it("is not asked by Shift+V without Ctrl or Cmd", async () => {
    const by = translating();
    by.fromText.mockImplementation(async (plain: string) => `T(${plain})`);
    const text = onPage(by);
    atEnd();
    keyed({ code: "KeyV", key: "V", shiftKey: true });
    pasted({ plain: "p" });
    await settled();
    expect(text()).toContain("T(p)");
  });
});

describe("translation of what is pasted", () => {
  it("lands what fromHtml answers when the html has words", async () => {
    const by = translating();
    const text = onPage(by);
    atEnd();
    pasted({ html: "<p>hola</p>", plain: "hola" });
    await settled();
    expect(by.fromHtml).toHaveBeenCalledWith("<p>hola</p>");
    expect(text()).toBe("# T\n\nHolaHTML\n");
  });

  it("lands the plain text when fromHtml rejects", async () => {
    const by = translating();
    by.fromHtml.mockRejectedValue(new Error("no core"));
    const text = onPage(by);
    atEnd();
    pasted({ html: "<p>hola</p>", plain: "hola" });
    await settled();
    expect(text()).toBe("# T\n\nHolahola\n");
  });

  it("lands the plain text when fromHtml answers nothing", async () => {
    const by = translating();
    by.fromHtml.mockResolvedValue("");
    const text = onPage(by);
    atEnd();
    pasted({ html: "<p>hola</p>", plain: "hola" });
    await settled();
    expect(text()).toBe("# T\n\nHolahola\n");
  });

  it("ignores html with no words and goes by the plain text", async () => {
    const by = translating();
    by.fromText.mockResolvedValue("TEXT");
    const text = onPage(by);
    atEnd();
    pasted({ html: '<img src="a.png">', plain: "hola" });
    await settled();
    expect(by.fromHtml).not.toHaveBeenCalled();
    expect(text()).toBe("# T\n\nHolaTEXT\n");
  });

  it("lands what fromText answers for plain text alone", async () => {
    const by = translating();
    by.fromText.mockResolvedValue("**hola**");
    const text = onPage(by);
    atEnd();
    pasted({ plain: "hola" });
    await settled();
    expect(by.fromText).toHaveBeenCalledWith("hola");
    expect(text()).toBe("# T\n\nHola**hola**\n");
  });

  it("lands the plain text when fromText answers null", async () => {
    const text = onPage(translating());
    atEnd();
    pasted({ plain: "hola" });
    await settled();
    expect(text()).toBe("# T\n\nHolahola\n");
  });

  it("lands the plain text when fromText rejects", async () => {
    const by = translating();
    by.fromText.mockRejectedValue(new Error("no core"));
    const text = onPage(by);
    atEnd();
    pasted({ plain: "hola" });
    await settled();
    expect(text()).toBe("# T\n\nHolahola\n");
  });

  it("lands CRLF plain text with LF", async () => {
    const text = onPage(translating());
    atEnd();
    pasted({ plain: "uno\r\ndos" });
    await settled();
    expect(text()).not.toContain("\r");
    expect(text()).toContain("uno\ndos");
  });
});

describe("a paste inside a fenced code block", () => {
  const SOURCE = "- a\n**b**";
  const inside = (by: Translator) => {
    const text = onPage(by, CODE, CODE_PLACED);
    atEnd(1);
    return text;
  };

  it("lands the plain text raw with no translation and no blank lines", async () => {
    const by = translating();
    const text = inside(by);
    pasted({ plain: SOURCE });
    await settled();
    expect(text()).toBe(`# T\n\n\`\`\`js\nx${SOURCE}\n\`\`\`\n`);
    expect(by.fromHtml).not.toHaveBeenCalled();
    expect(by.fromText).not.toHaveBeenCalled();
  });

  it("ignores the html", async () => {
    const by = translating();
    const text = inside(by);
    pasted({ plain: SOURCE, html: "<b>no</b>" });
    await settled();
    expect(text()).toContain(SOURCE);
    expect(text()).not.toContain("HTML");
    expect(by.fromHtml).not.toHaveBeenCalled();
  });

  it("is not escaped after Ctrl+Shift+V", async () => {
    const text = inside(translating());
    plainShortcut();
    pasted({ plain: SOURCE });
    await settled();
    expect(text()).toContain(SOURCE);
    expect(text()).not.toContain("\\*");
  });

  it("spends the literal ask, so the next paste outside code translates", async () => {
    const by = translating();
    by.fromText.mockImplementation(async (plain: string) => `T(${plain})`);
    const text = inside(by);
    plainShortcut();
    pasted({ plain: "x" });
    await settled();
    document.body.innerHTML = "";
    writing?.stop();
    const again = onPage(by);
    atEnd();
    pasted({ plain: "**y**" });
    await settled();
    expect(text()).toContain("x");
    expect(again()).toContain("T(**y**)");
  });

  it("lands CRLF as LF", async () => {
    const text = inside(translating());
    pasted({ plain: "a\r\nb" });
    await settled();
    expect(text()).not.toContain("\r");
    expect(text()).toContain("a\nb");
  });
});

const begunEmpty = (by: Translator) => {
  const text = onPage(by, "", "");
  for (const kind of ["mousedown", "mouseup", "click"])
    document
      .querySelector(".page > h1[data-title]")
      ?.dispatchEvent(
        new MouseEvent(kind, { bubbles: true, cancelable: true, button: 0, clientY: 900 }),
      );
  return text;
};

const afterShiftEnter = (by: Translator) => {
  const text = onPage(by, "Hola\n", '<p data-at="0-5"><span data-src="0-4">Hola</span></p>');
  atEnd();
  keyed({ key: "Enter", code: "Enter", shiftKey: true });
  return text;
};

describe("a paste on a line just begun", () => {
  it("uses Tisty's own type on an empty new page", async () => {
    const by = translating();
    const text = begunEmpty(by);
    pasted({ own: "# Nota\n\n==a=={c}", html: "<b>x</b>", plain: "x" });
    await settled();
    expect(text()).toBe("# Nota\n\n==a=={c}");
    expect(by.fromHtml).not.toHaveBeenCalled();
    expect(by.fromText).not.toHaveBeenCalled();
  });

  it("uses Tisty's own type after Shift+Enter", async () => {
    const by = translating();
    const text = afterShiftEnter(by);
    pasted({ own: "==a==", plain: "x", html: "<b>x</b>" });
    await settled();
    expect(text()).toBe("Hola\\\n==a==\n");
    expect(by.fromHtml).not.toHaveBeenCalled();
  });

  it("writes the html translation on an empty new page", async () => {
    const text = begunEmpty(translating());
    pasted({ html: "<p>hola</p>", plain: "hola" });
    await settled();
    expect(text()).toBe("# HTML");
  });

  it("does nothing for an all-blank arrival", async () => {
    const by = translating();
    const text = begunEmpty(by);
    pasted({ plain: "  \n ", html: "", own: "" });
    await settled();
    expect(text()).toBe("");
    expect(by.fromText).not.toHaveBeenCalled();
  });

  it("does nothing for a blank Tisty type after Shift+Enter", async () => {
    const text = afterShiftEnter(translating());
    pasted({ own: "   " });
    await settled();
    expect(text()).toBe("Hola\n");
  });

  it("lands the literal letters after Ctrl+Shift+V on an empty new page", async () => {
    const text = begunEmpty(translating());
    plainShortcut();
    pasted({ plain: "**hola**", own: "==x==" });
    await settled();
    expect(text()).toContain("\\*\\*hola\\*\\*");
  });
});

const choose = (from: number, to: number) => {
  const words = Array.from(document.querySelectorAll("[data-src]")).pop()?.firstChild as Text;
  const range = document.createRange();
  range.setStart(words, from);
  range.setEnd(words, to);
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(range);
};

describe("the literal ask of Ctrl+Shift+V", () => {
  it("lasts only until the paste it raises: a tick later a paste is translated", async () => {
    const by = translating();
    by.fromText.mockImplementation(async (plain: string) => `T(${plain})`);
    const text = onPage(by);
    atEnd();
    plainShortcut();
    await settled();
    pasted({ plain: "**x**" });
    await settled();
    expect(text()).toContain("T(**x**)");
    expect(text()).not.toContain("\\*");
    expect(by.fromText).toHaveBeenCalledWith("**x**");
  });
});

describe("a paste that brings no words", () => {
  it("in a code block leaves the chosen code as it was with html only", async () => {
    const by = translating();
    const text = onPage(by, CODE, CODE_PLACED);
    choose(0, 1);
    pasted({ html: "<b>no</b>" });
    await settled();
    expect(text()).toBe(CODE);
    expect(by.fromHtml).not.toHaveBeenCalled();
  });

  it("in a code block still replaces the chosen code when plain text comes", async () => {
    const text = onPage(translating(), CODE, CODE_PLACED);
    choose(0, 1);
    pasted({ plain: "y" });
    await settled();
    expect(text()).toBe("# T\n\n```js\ny\n```\n");
  });

  it("outside code leaves the chosen words as they were when the translation is empty", async () => {
    const by = translating();
    by.fromHtml.mockResolvedValue("");
    const text = onPage(by);
    choose(0, 4);
    pasted({ html: "<p>hola</p>", plain: "" });
    await settled();
    expect(by.fromHtml).toHaveBeenCalled();
    expect(text()).toBe(BODY);
  });
});

describe("a paste whose landing throws", () => {
  it("says the paste was lost and leaves no unhandled rejection", async () => {
    const lost: string[] = [];
    let breaking = false;
    const rejected = vi.fn();
    process.on("unhandledRejection", rejected);
    let body = BODY;
    onPage(translating(), BODY, PLACED, {
      text: () => {
        if (breaking) throw new Error("page gone");
        return body;
      },
      write: (next) => {
        body = next;
      },
      lost: (notice) => lost.push(notice),
    });
    atEnd();
    pasted({ plain: "hola" });
    breaking = true;
    await settled();
    await settled();
    breaking = false;
    process.off("unhandledRejection", rejected);
    expect(lost).toEqual([t("pastedLost")]);
    expect(rejected).not.toHaveBeenCalled();
  });
});

describe("the literal ask when the keyboard gives no event code", () => {
  const LITERAL = "\\*\\*a\\*\\*";
  const afterKey = async (init: KeyboardEventInit) => {
    const by = translating();
    by.fromText.mockImplementation(async (plain: string) => `T(${plain})`);
    const text = onPage(by);
    atEnd();
    keyed({ code: "", ...init });
    pasted({ plain: "**a**" });
    await settled();
    return text();
  };

  it("is asked by key V with Ctrl+Shift", async () => {
    expect(await afterKey({ key: "V", ctrlKey: true, shiftKey: true })).toContain(LITERAL);
  });

  it("is asked by key v with Cmd+Shift", async () => {
    expect(await afterKey({ key: "v", metaKey: true, shiftKey: true })).toContain(LITERAL);
  });

  it("is not asked by key v with Ctrl alone", async () => {
    const said = await afterKey({ key: "v", ctrlKey: true });
    expect(said).toContain("T(**a**)");
    expect(said).not.toContain(LITERAL);
  });
});

describe("Tisty's own type pasted inside a block of code", () => {
  const FENCE = "```";
  const AFTER = `# T\n\n${FENCE}js\nx`;
  const BEFORE = `\n${FENCE}\n`;
  const inCode = (by: Translator) => {
    const text = onPage(by, CODE, CODE_PLACED);
    atEnd(1);
    return text;
  };

  it("lands the lines of a whole fenced block, blank line kept, inside the fence", async () => {
    const by = translating();
    const text = inCode(by);
    pasted({
      own: `${FENCE}js\nlet a = 1;\n\nb();\n${FENCE}`,
      plain: "let a = 1; b();",
      html: "<pre>let a = 1; b();</pre>",
    });
    await settled();
    expect(text()).toBe(`${AFTER}let a = 1;\n\nb();${BEFORE}`);
    expect(by.fromHtml).not.toHaveBeenCalled();
    expect(by.fromText).not.toHaveBeenCalled();
  });

  it("lands the plain text for own lines that are not a whole fenced block", async () => {
    const text = inCode(translating());
    pasted({ own: "a\nb", plain: "a b" });
    await settled();
    expect(text()).toBe(`${AFTER}a b${BEFORE}`);
  });

  it("brings no backslash of escaped prose into the code", async () => {
    const text = inCode(translating());
    pasted({ own: "2 \\* 3", plain: "2 * 3" });
    await settled();
    expect(text()).toBe(`${AFTER}2 * 3${BEFORE}`);
    expect(text()).not.toContain("\\");
  });

  it("lands the plain text when there is no own type", async () => {
    const text = inCode(translating());
    pasted({ plain: "- a\n**b**" });
    await settled();
    expect(text()).toBe(`${AFTER}- a\n**b**${BEFORE}`);
  });

  it("lands the lines of a whole fenced block with LF when they came with CRLF", async () => {
    const text = inCode(translating());
    pasted({ own: `${FENCE}js\r\na\r\n\r\nb\r\n${FENCE}`, plain: "a b" });
    await settled();
    expect(text()).toBe(`${AFTER}a\n\nb${BEFORE}`);
    expect(text()).not.toContain("\r");
  });
});

describe("the menu's paste without formatting", () => {
  const afterWaiting = async (ms: number) => {
    const by = translating();
    by.fromText.mockImplementation(async (plain: string) => `T(${plain})`);
    const text = onPage(by);
    atEnd();
    writing?.plainNext();
    await new Promise((done) => setTimeout(done, ms));
    pasted({ plain: "**a**" });
    await settled();
    return text();
  };

  it("makes a paste 300 ms later literal", async () => {
    expect(await afterWaiting(300)).toContain("\\*\\*a\\*\\*");
  });

  it("is over 1600 ms later", async () => {
    const said = await afterWaiting(1600);
    expect(said).toContain("T(**a**)");
    expect(said).not.toContain("\\*");
  });

  it("is spent by one paste", async () => {
    const by = translating();
    by.fromText.mockImplementation(async (plain: string) => `T(${plain})`);
    const text = onPage(by);
    atEnd();
    writing?.plainNext();
    pasted({ plain: "**a**" });
    pasted({ plain: "**b**" });
    await settled();
    expect(text()).toContain("\\*\\*a\\*\\*");
    expect(text()).toContain("T(**b**)");
  });
});

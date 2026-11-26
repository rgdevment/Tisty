import { describe, expect, it, vi } from "vitest";
import {
  arrival,
  type Clipboard,
  escapedMarks,
  evenLines,
  pastedWords,
  TISTY_MARKDOWN,
  type Translator,
} from "../ui/clipboarding";

const board = (held: Record<string, string>): Clipboard => ({
  getData: (format) => held[format] ?? "",
});

const translator = (over: Partial<Translator> = {}): Translator => ({
  fromHtml: vi.fn(async () => "from html"),
  fromText: vi.fn(async () => null),
  ...over,
});

describe("arrival", () => {
  it("takes Tisty's own words before the html and the plain text", () => {
    const data = board({
      [TISTY_MARKDOWN]: "# own",
      "text/html": "<p>html</p>",
      "text/plain": "plain",
    });
    expect(arrival(data)).toEqual({ kind: "own", text: "# own" });
  });

  it("gives the plain text as it is when the person asked for no formatting", () => {
    const data = board({
      [TISTY_MARKDOWN]: "own",
      "text/html": "<b>html</b>",
      "text/plain": "plain **text**",
    });
    expect(arrival(data, true)).toEqual({ kind: "literal", text: "plain \\*\\*text\\*\\*" });
  });

  it("gives CRLF input as LF and escaped text when asked for no formatting", () => {
    expect(arrival(board({ "text/plain": "- a\r\n**b**" }), true)).toEqual({
      kind: "literal",
      text: "\\- a\n\\*\\*b\\*\\*",
    });
  });

  it("takes the html with the plain text beside it when the html says something", () => {
    const data = board({ "text/html": "<p>hola</p>", "text/plain": "hola" });
    expect(arrival(data)).toEqual({ kind: "html", html: "<p>hola</p>", plain: "hola" });
  });

  it("evens CRLF and lone CR into LF in the own, literal, html and plain routes", () => {
    expect(arrival(board({ [TISTY_MARKDOWN]: "a\r\nb\rc" }))).toEqual({
      kind: "own",
      text: "a\nb\nc",
    });
    expect(arrival(board({ "text/plain": "a\r\nb\rc" }), true)).toEqual({
      kind: "literal",
      text: "a\nb\nc",
    });
    expect(arrival(board({ "text/html": "<p>x</p>", "text/plain": "a\r\nb\rc" }))).toEqual({
      kind: "html",
      html: "<p>x</p>",
      plain: "a\nb\nc",
    });
    expect(arrival(board({ "text/plain": "a\r\nb\rc" }))).toEqual({
      kind: "plain",
      plain: "a\nb\nc",
    });
  });

  it("goes the plain route when the html is only a picture or empty markup", () => {
    const picture = board({ "text/html": '<img src="x.png">', "text/plain": "texto" });
    expect(arrival(picture)).toEqual({ kind: "plain", plain: "texto" });
    const empty = board({ "text/html": "<div> <span>&nbsp;</span></div>", "text/plain": "texto" });
    expect(arrival(empty)).toEqual({ kind: "plain", plain: "texto" });
    const sheet = board({
      "text/html": "<html><head><style>p{color:red}</style></head><body><p></p></body></html>",
      "text/plain": "texto",
    });
    expect(arrival(sheet)).toEqual({ kind: "plain", plain: "texto" });
  });

  it("takes the html of a picture chosen alone, with no plain text or only its alt beside it", () => {
    const html = '<img src="https://ejemplo.cl/a.png" alt="Logo">';
    expect(arrival(board({ "text/html": html, "text/plain": "" }))).toEqual({
      kind: "html",
      html,
      plain: "",
    });
    expect(arrival(board({ "text/html": html, "text/plain": " Logo\n" }))).toEqual({
      kind: "html",
      html,
      plain: " Logo\n",
    });
    const pair = "<img src='https://x/a.png' alt='Uno'><img src=\"https://x/b.png\" alt=\"Dos\">";
    expect(arrival(board({ "text/html": pair, "text/plain": "Uno Dos" })).kind).toBe("html");
    expect(arrival(board({ "text/html": html, "text/plain": "Otra cosa" }))).toEqual({
      kind: "plain",
      plain: "Otra cosa",
    });
    expect(arrival(board({ "text/html": '<img alt="Logo">', "text/plain": "" }))).toEqual({
      kind: "plain",
      plain: "",
    });
  });

  it("treats a getData that throws as empty", () => {
    const throwing: Clipboard = {
      getData: () => {
        throw new Error("no such type");
      },
    };
    expect(arrival(throwing)).toEqual({ kind: "plain", plain: "" });
    expect(arrival(throwing, true)).toEqual({ kind: "literal", text: "" });
  });

  it("keeps the plain text when only the html type throws", () => {
    const data: Clipboard = {
      getData: (format) => {
        if (format === "text/html") throw new Error("boom");
        return format === "text/plain" ? "solo texto" : "";
      },
    };
    expect(arrival(data)).toEqual({ kind: "plain", plain: "solo texto" });
  });

  it("treats a missing clipboard as empty", () => {
    expect(arrival(null)).toEqual({ kind: "plain", plain: "" });
  });
});

const LITERAL = [
  "# Título",
  "- uno",
  "  - dos",
  "12. doce",
  "3) tres",
  "son 12. ok",
  "> cita",
  "| a | b |",
  "==p=={red} ++u++ ~~s~~",
  "::star:: y :::tip",
  "$x$ <b>t</b> [l](u) ![i](u)",
  "`c` &amp; \\ fin",
  "**hola** _e_",
].join("\n");

const NO_BREAK = " ";

const ESCAPED = [
  "\\# Título",
  "\\- uno",
  `${NO_BREAK.repeat(2)}\\- dos`,
  "12\\. doce",
  "3\\) tres",
  "son 12. ok",
  "\\> cita",
  "\\| a \\| b \\|",
  "\\=\\=p\\=\\={red} \\+\\+u\\+\\+ \\~\\~s\\~\\~",
  "\\::star\\:: y \\:\\::tip",
  "\\$x\\$ \\<b\\>t\\</b\\> \\[l\\](u) \\!\\[i\\](u)",
  "\\`c\\` \\&amp; \\\\ fin",
  "\\*\\*hola\\*\\* \\_e\\_",
].join("\n");

describe("escapedMarks", () => {
  it("puts a backslash before every sign of the language", () => {
    expect(escapedMarks("**hola** # x")).toBe("\\*\\*hola\\*\\* \\# x");
  });

  it("escapes a dash only when it opens a line", () => {
    expect(escapedMarks("- uno")).toBe("\\- uno");
    expect(escapedMarks("  - dos")).toBe(`${NO_BREAK.repeat(2)}\\- dos`);
    expect(escapedMarks("a - b")).toBe("a - b");
  });

  it("escapes the dot or paren after a line-opening number only", () => {
    expect(escapedMarks("12. doce")).toBe("12\\. doce");
    expect(escapedMarks("3) tres")).toBe("3\\) tres");
    expect(escapedMarks("son 12. ok")).toBe("son 12. ok");
  });

  it("keeps icons and containers as text", () => {
    expect(escapedMarks("::star::")).toBe("\\::star\\::");
    expect(escapedMarks(":::tip")).toBe("\\:\\::tip");
    expect(escapedMarks("a: b")).toBe("a: b");
  });

  it("turns any indent into no-break spaces of the same width", () => {
    const gap = " ".repeat(4);
    expect(escapedMarks("    hola")).toBe(`${gap}hola`);
    expect(escapedMarks("\tx")).toBe(`${gap}x`);
    expect(escapedMarks("    - x")).toBe(`${gap}\\- x`);
    expect(escapedMarks("   tres")).toBe(`${gap.slice(1)}tres`);
  });

  it("escapes the opening colon of an emoji code but not times or labels", () => {
    expect(escapedMarks("Listo :tada: y :x:")).toBe("Listo \\:tada: y \\:x:");
    expect(escapedMarks("10:30")).toBe("10:30");
    expect(escapedMarks("a: b")).toBe("a: b");
    expect(escapedMarks("https://x")).toBe("https\\://x");
  });

  it("escapes the colon of a link scheme and still the icon colons", () => {
    expect(escapedMarks("ver https://x.y")).toBe("ver https\\://x.y");
    expect(escapedMarks("::a::")).toBe("\\::a\\::");
  });

  it.each([
    ["==p=={red}", "\\=\\=p\\=\\={red}"],
    ["++u++", "\\+\\+u\\+\\+"],
    ["~~s~~", "\\~\\~s\\~\\~"],
    ["`c`", "\\`c\\`"],
    ["$x$", "\\$x\\$"],
    ["<b>t</b>", "\\<b\\>t\\</b\\>"],
    ["&amp;", "\\&amp;"],
    ["| a |", "\\| a \\|"],
    ["[l](u)", "\\[l\\](u)"],
    ["![i](u)", "\\!\\[i\\](u)"],
    ["\\", "\\\\"],
  ])("escapes %s", (input, expected) => {
    expect(escapedMarks(input)).toBe(expected);
  });

  it("leaves plain words untouched", () => {
    expect(escapedMarks("hola mundo, qué tal")).toBe("hola mundo, qué tal");
  });

  describe("spacing the page would collapse", () => {
    const nb = " ";

    it("keeps the columns of a run of spaces inside a line", () => {
      expect(escapedMarks(`Name${" ".repeat(14)}Id`)).toBe(`Name ${nb.repeat(13)}Id`);
      expect(escapedMarks("a  b")).toBe(`a ${nb}b`);
    });

    it("counts a tab as four columns", () => {
      expect(escapedMarks("Nombre\tEdad")).toBe(`Nombre ${nb.repeat(3)}Edad`);
    });

    it("widens a gap and a four-column indent in the same line", () => {
      expect(escapedMarks("    indented  gap")).toBe(`${nb.repeat(4)}indented ${nb}gap`);
    });

    it("leaves a single space alone", () => {
      expect(escapedMarks("a b c")).toBe("a b c");
    });

    it("leaves two spaces before the end of a line alone", () => {
      expect(escapedMarks("x  \ny")).toBe("x  \ny");
    });

    it("keeps the height of a run of blank lines as no-break space lines", () => {
      expect(escapedMarks("a\n\n\nb")).toBe(`a\n\n${nb}\nb`);
      expect(escapedMarks("a\n\n\n\nb")).toBe(`a\n\n${nb}\n${nb}\nb`);
    });

    it("counts whitespace-only lines as blank lines", () => {
      expect(escapedMarks("a\n \n\t\n\nb")).toBe(`a\n\n${nb}\n${nb}\nb`);
    });

    it("leaves a lone line of spaces blank", () => {
      expect(escapedMarks("a\n   \nb")).toBe("a\n   \nb");
    });
    it("leaves one blank line and trailing blank lines as they are", () => {
      expect(escapedMarks("a\n\nb")).toBe("a\n\nb");
      expect(escapedMarks("tail\n\n\n\n")).toBe("tail\n\n\n\n");
    });
  });

  it("escapes the shared fixture exactly", () => {
    expect(escapedMarks(LITERAL)).toBe(ESCAPED);
  });
});

describe("evenLines", () => {
  it("turns CRLF and lone CR into LF", () => {
    expect(evenLines("a\r\nb\rc\nd")).toBe("a\nb\nc\nd");
  });
});

describe("pastedWords", () => {
  it("returns the own and literal text without calling the translator", async () => {
    const by = translator();
    expect(await pastedWords({ kind: "own", text: "mio" }, by)).toBe("mio");
    expect(await pastedWords({ kind: "literal", text: "tal cual" }, by)).toBe("tal cual");
    expect(by.fromHtml).not.toHaveBeenCalled();
    expect(by.fromText).not.toHaveBeenCalled();
  });

  it("returns what the html translation says, with its lines evened", async () => {
    const by = translator({ fromHtml: vi.fn(async () => "# a\r\n\r\n- b\rc") });
    const said = await pastedWords({ kind: "html", html: "<h1>a</h1>", plain: "a" }, by);
    expect(said).toBe("# a\n\n- b\nc");
    expect(by.fromHtml).toHaveBeenCalledWith("<h1>a</h1>");
  });

  it("falls back to the plain text when fromHtml rejects", async () => {
    const by = translator({ fromHtml: vi.fn(async () => Promise.reject(new Error("no"))) });
    expect(await pastedWords({ kind: "html", html: "<p>a</p>", plain: "plano" }, by)).toBe("plano");
  });

  it("falls back to the plain text when fromHtml returns nothing", async () => {
    const by = translator({ fromHtml: vi.fn(async () => "") });
    expect(await pastedWords({ kind: "html", html: "<p>a</p>", plain: "plano" }, by)).toBe("plano");
  });

  it("returns the translated text for plain, with its lines evened", async () => {
    const by = translator({ fromText: vi.fn(async () => "```\r\nPS> dir\r\n```") });
    const said = await pastedWords({ kind: "plain", plain: "PS> dir" }, by);
    expect(said).toBe("```\nPS> dir\n```");
    expect(by.fromText).toHaveBeenCalledWith("PS> dir");
  });

  it("returns the plain text when fromText says null", async () => {
    const by = translator({ fromText: vi.fn(async () => null) });
    expect(await pastedWords({ kind: "plain", plain: "tal cual" }, by)).toBe("tal cual");
  });

  it("returns the plain text when fromText rejects", async () => {
    const by = translator({ fromText: vi.fn(async () => Promise.reject(new Error("no"))) });
    expect(await pastedWords({ kind: "plain", plain: "tal cual" }, by)).toBe("tal cual");
  });

  it("never calls fromText for an empty plain text", async () => {
    const by = translator();
    expect(await pastedWords({ kind: "plain", plain: "" }, by)).toBe("");
    expect(by.fromText).not.toHaveBeenCalled();
    expect(by.fromHtml).not.toHaveBeenCalled();
  });
});

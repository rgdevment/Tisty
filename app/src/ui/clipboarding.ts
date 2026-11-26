/** Tisty's own words exactly as written; the copy side writes them under this type too. */
export const TISTY_MARKDOWN = "text/x-tisty-markdown";

export interface Clipboard {
  readonly getData: (format: string) => string;
}

/** What a paste carries and who still has to turn it into Tisty's words. */
export type Arrival =
  | { readonly kind: "own"; readonly text: string }
  | { readonly kind: "literal"; readonly text: string }
  | { readonly kind: "html"; readonly html: string; readonly plain: string }
  | { readonly kind: "plain"; readonly plain: string };

/** The core's two translations, as the window's host offers them. */
export interface Translator {
  readonly fromHtml: (html: string) => Promise<string>;
  readonly fromText: (text: string) => Promise<string | null>;
}

/** Windows ends a line with CRLF and old Macs with CR; the page only knows LF. */
export const evenLines = (text: string): string => text.replace(/\r\n?/g, "\n");

const SIGNS = /[\\*_~=+`[\]#>|!$<&]/g;
const LINE_DASH = /^([ \t]*)-/gm;
const LINE_NUMBER = /^([ \t]*\d{1,9})([.)])/gm;
// `::`, a link's `://` and the colon that opens an emoji code such as `:tada:`.
const MARKED_COLONS = /:(?=:|\/\/)|(?<!:):(?=[\w+-]+:)/g;
const LINE_INDENT = /^[ \t]+(?=\S)/gm;
const INNER_GAP = /(?<=\S)(?:[ \t]*\t[ \t]*| {2,})(?=\S)/g;
const BLANK_RUN = /\n(?:[ \t]*\n){2,}(?=[ \t]*\S)/g;
const CODE_INDENT = 4;
const NO_BREAK = " ";

const columns = (gap: string): number =>
  [...gap].reduce((wide, one) => wide + (one === "\t" ? CODE_INDENT : 1), 0);

// The page shows a run of spaces or a tab as one space; no-break spaces keep a console's columns apart.
const widened = (gap: string): string => ` ${NO_BREAK.repeat(columns(gap) - 1)}`;

// Two blank lines read as one; each further one keeps its height as a line holding a no-break space.
const unfolded = (run: string): string =>
  `\n\n${`${NO_BREAK}\n`.repeat(run.split("\n").length - 3)}`;

// The page drops up to three leading spaces and takes four as code; no-break spaces show the gap and open nothing.
const unindented = (indent: string): string => NO_BREAK.repeat(columns(indent));

/** Text the page draws exactly as it reads: every sign the language would take for a mark goes behind a backslash. */
export const escapedMarks = (text: string): string =>
  text
    .replace(BLANK_RUN, unfolded)
    .replace(SIGNS, "\\$&")
    .replace(LINE_DASH, "$1\\-")
    .replace(LINE_NUMBER, "$1\\$2")
    .replace(MARKED_COLONS, "\\:")
    .replace(LINE_INDENT, unindented)
    .replace(INNER_GAP, widened);

// Some browsers throw on a type the clipboard does not hold rather than answer empty.
const read = (data: Clipboard | null, format: string): string => {
  try {
    return data?.getData(format) ?? "";
  } catch {
    return "";
  }
};

// A picture alone or an empty wrapper says nothing the plain text does not.
const worded = (html: string): boolean =>
  html
    .replace(/<(script|style|head)\b[\s\S]*?<\/\1\s*>/gi, "")
    .replace(/<[^>]*>/g, "")
    .replace(/&nbsp;|&#160;/gi, " ")
    .trim().length > 0;

const ALT = /<img\b[^>]*\balt\s*=\s*(?:"([^"]*)"|'([^']*)')/gi;

const squeezed = (text: string): string => text.replace(/\s+/g, " ").trim();

// A picture chosen alone on a web page brings, as plain text, nothing or only its alt; the html is all there is of it.
const pictured = (html: string, plain: string): boolean => {
  if (!/<img\b[^>]*\bsrc\s*=\s*["']?https?:/i.test(html)) return false;
  const alts = Array.from(html.matchAll(ALT), (one) => one[1] ?? one[2] ?? "").join(" ");
  return squeezed(plain) === "" || squeezed(plain) === squeezed(alts);
};

// Asking for no formatting outranks even Tisty's own words: the person wants the letters, marks shown as letters.
export const arrival = (data: Clipboard | null, literally = false): Arrival => {
  const plain = evenLines(read(data, "text/plain"));
  if (literally) return { kind: "literal", text: escapedMarks(plain) };
  const own = read(data, TISTY_MARKDOWN);
  if (own) return { kind: "own", text: evenLines(own) };
  const html = read(data, "text/html");
  if (worded(html) || pictured(html, plain)) return { kind: "html", html, plain };
  return { kind: "plain", plain };
};

/** The words a paste lands as; when the core cannot translate them, the plain text is what was meant. */
export const pastedWords = (arrived: Arrival, by: Translator): Promise<string> => {
  switch (arrived.kind) {
    case "own":
    case "literal":
      return Promise.resolve(arrived.text);
    case "html":
      return by.fromHtml(arrived.html).then(
        (said) => evenLines(said) || arrived.plain,
        () => arrived.plain,
      );
    case "plain":
      if (!arrived.plain) return Promise.resolve("");
      return by.fromText(arrived.plain).then(
        (said) => (said === null ? arrived.plain : evenLines(said)),
        () => arrived.plain,
      );
  }
};

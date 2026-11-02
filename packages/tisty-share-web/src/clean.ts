import DOMPurify from "dompurify";

// The writer's own Tisty drew the page, so it is taken as anyone's: only what Tisty draws goes through (decision 91).
const purify = DOMPurify(window);

// What the engine writes, and what it lets through of the HTML written inside the Markdown.
const TAGS = [
  "a",
  "abbr",
  "acronym",
  "article",
  "aside",
  "b",
  "bdi",
  "bdo",
  "blockquote",
  "br",
  "caption",
  "center",
  "cite",
  "code",
  "col",
  "colgroup",
  "data",
  "dd",
  "del",
  "details",
  "dfn",
  "div",
  "dl",
  "dt",
  "em",
  "figcaption",
  "figure",
  "footer",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "header",
  "hgroup",
  "hr",
  "i",
  "img",
  "ins",
  "kbd",
  "li",
  "mark",
  "nav",
  "ol",
  "p",
  "pre",
  "q",
  "rp",
  "rt",
  "rtc",
  "ruby",
  "s",
  "samp",
  "small",
  "span",
  "strike",
  "strong",
  "sub",
  "summary",
  "sup",
  "table",
  "tbody",
  "td",
  "tfoot",
  "th",
  "thead",
  "time",
  "tr",
  "tt",
  "u",
  "ul",
  "var",
  "wbr",
  // The formulas math-core writes.
  "math",
  "menclose",
  "merror",
  "mfrac",
  "mi",
  "mmultiscripts",
  "mn",
  "mo",
  "mover",
  "mpadded",
  "mphantom",
  "mprescripts",
  "mroot",
  "mrow",
  "mspace",
  "msqrt",
  "msub",
  "msubsup",
  "msup",
  "mtable",
  "mtd",
  "mtext",
  "mtr",
  "munder",
  "munderover",
];

const ATTRIBUTES = [
  "alt",
  "class",
  "colspan",
  "datetime",
  "dir",
  "height",
  "href",
  "id",
  "lang",
  "loading",
  "open",
  "rowspan",
  "scope",
  "src",
  "start",
  "style",
  "title",
  "width",
  "accent",
  "accentunder",
  "depth",
  "display",
  "displaystyle",
  "form",
  "linethickness",
  "lspace",
  "mathvariant",
  "maxsize",
  "minsize",
  "movablelimits",
  "notation",
  "rspace",
  "scriptlevel",
  "stretchy",
  "symmetric",
  "voffset",
];

// The properties Tisty and its formulas write inline; none of them can lift a part out to lay it over the page.
const STYLES = new Set([
  "color",
  "font-size",
  "font-variant-caps",
  "margin-left",
  "math-shift",
  "padding-left",
  "padding-right",
  "text-align",
  "width",
]);
// A value of words, numbers and units only: no url(), nothing that asks for anything.
const PLAIN = /^[#\w.%\s-]+$/;
const PICTURED = /^tisty:picture\/\d+$/;

const styled = (said: string): boolean =>
  said
    .split(";")
    .map((one) => one.trim())
    .filter(Boolean)
    .every((one) => {
      const [property, value] = one.split(":").map((part) => part.trim());
      return STYLES.has(property ?? "") && value !== undefined && PLAIN.test(value);
    });

// A picture is only ever one the envelope carries, so the page never asks anyone else for one.
purify.addHook("uponSanitizeAttribute", (_node, data) => {
  if (data.attrName === "style") data.keepAttr = styled(data.attrValue);
  if (data.attrName === "src") data.keepAttr = PICTURED.test(data.attrValue.trim());
});

export const cleaned = (html: string): string =>
  purify.sanitize(html, {
    ALLOWED_TAGS: TAGS,
    ALLOWED_ATTR: ATTRIBUTES,
    // DOMPurify's own shape, which also passes a value with no scheme at all, with the schemes cut to Tisty's.
    ALLOWED_URI_REGEXP: /^(?:(?:https?|mailto|tisty):|[^a-z]|[a-z+.-]+(?:[^a-z+.\-:]|$))/i,
  });

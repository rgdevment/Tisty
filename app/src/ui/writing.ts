import type { Editor as Writing } from "@tiptap/core";
import { getHTMLFromFragment, InputRule, Node, textblockTypeInputRule } from "@tiptap/core";
import { BulletList } from "@tiptap/extension-bullet-list";
import { CodeBlockLowlight } from "@tiptap/extension-code-block-lowlight";
import { Highlight } from "@tiptap/extension-highlight";
import { Image } from "@tiptap/extension-image";
import { Table, TableCell, TableHeader, TableRow } from "@tiptap/extension-table";
import { TaskItem } from "@tiptap/extension-task-item";
import { TaskList } from "@tiptap/extension-task-list";
import { Text } from "@tiptap/extension-text";
import { Fragment, type Node as ProseNode } from "@tiptap/pm/model";
import StarterKit from "@tiptap/starter-kit";
import { common, createLowlight } from "lowlight";
import markPlugin from "markdown-it-mark";
import taskLists from "markdown-it-task-lists";
import { MarkdownSerializerState } from "prosemirror-markdown";
import { Markdown } from "tiptap-markdown";
import { markup } from "../glyphs";
import { t } from "../locales";
import { spared } from "./Icons";

/// A bracket left bare closes the label early, and the reference stops naming anything.
export const labelled = (said: string): string => said.replace(/([[\]\\])/g, "\\$1");

const inked = Symbol("ink");
const peeked = Symbol("peek");

type Peek = { text: string; from: number; part: number };

type Inline = { start: number; end?: number; delimiter: string };

type Marked = { type: { name: string } };

type Scanning = {
  [inked]?: Ink;
  [peeked]?: Peek | null;
  out: string;
  delim: string;
  closed: unknown;
  inlines: Inline[];
  marks: Record<string, { open: string; expelEnclosingWhitespace?: boolean }>;
  atBlockStart: boolean;
  atBlank: () => boolean;
  ensureNewLine: () => void;
  flushClose: (size?: number) => void;
  write: (content?: string) => void;
  text: (value: string, escaped?: boolean) => void;
  esc: (value: string, atBlockStart?: boolean) => string;
  markString: (mark: Marked, open: boolean, parent: unknown, index: number) => string;
  normalizeInline: (inline: Inline) => Inline;
  render: (node: unknown, parent: unknown, index: number) => void;
};

class Ink {
  parts: string[] = [];
  offs: number[] = [];
  len = 0;
  last = -1;

  add(text: string): void {
    if (!text) return;
    this.offs.push(this.len);
    this.parts.push(text);
    this.len += text.length;
    this.last = text.charCodeAt(text.length - 1);
  }

  prior(): number {
    if (this.len < 2) return -1;
    const tail = this.parts[this.parts.length - 1];
    if (tail.length > 1) return tail.charCodeAt(tail.length - 2);
    const older = this.parts[this.parts.length - 2];
    return older.charCodeAt(older.length - 1);
  }

  priorSlashes(): number {
    let at = this.parts.length - 1;
    if (at < 0) return 0;
    let idx = this.parts[at].length - 2;
    let seen = 0;
    while (at >= 0) {
      while (idx >= 0) {
        if (this.parts[at].charCodeAt(idx) !== 92) return seen;
        seen += 1;
        idx -= 1;
      }
      at -= 1;
      if (at < 0) break;
      idx = this.parts[at].length - 1;
    }
    return seen;
  }

  escapeLast(): void {
    const at = this.parts.length - 1;
    const tail = this.parts[at];
    this.parts[at] = `${tail.slice(0, tail.length - 1)}\\!`;
    this.len += 1;
  }

  near(pos: number): Peek {
    if (!this.parts.length) return { text: "", from: 0, part: 0 };
    let low = 0;
    let high = this.offs.length - 1;
    while (low < high) {
      const mid = (low + high + 1) >> 1;
      if (this.offs[mid] <= pos) low = mid;
      else high = mid - 1;
    }
    return { text: this.parts.slice(low).join(""), from: this.offs[low], part: low };
  }

  mend(peek: Peek, text: string): void {
    this.parts.length = peek.part;
    this.offs.length = peek.part;
    this.len = peek.from;
    const older = this.parts[peek.part - 1];
    this.last = older ? older.charCodeAt(older.length - 1) : -1;
    this.add(text);
  }

  whole(): string {
    if (this.parts.length > 1) {
      const one = this.parts.join("");
      this.parts = [one];
      this.offs = [0];
    }
    return this.parts[0] ?? "";
  }
}

const scanning = MarkdownSerializerState.prototype as unknown as Scanning;

const plain = {
  atBlank: scanning.atBlank,
  ensureNewLine: scanning.ensureNewLine,
  flushClose: scanning.flushClose,
  write: scanning.write,
  text: scanning.text,
};

let scanned: Scanning | null = null;
let plainScanned: Pick<Scanning, "markString" | "normalizeInline" | "render"> | null = null;
let buffered = false;

const inkOf = (state: Scanning): Ink => (state[inked] ??= new Ink());

const swift = {
  out: {
    configurable: true,
    get(this: Scanning): string {
      const peek = this[peeked];
      return peek ? peek.text : inkOf(this).whole();
    },
    set(this: Scanning, value: string) {
      const peek = this[peeked];
      if (peek) {
        this[peeked] = null;
        inkOf(this).mend(peek, value);
        return;
      }
      const fresh = new Ink();
      fresh.add(value);
      this[inked] = fresh;
      this[peeked] = null;
      if (!scanned) learn(Object.getPrototypeOf(this) as Scanning);
    },
  } as PropertyDescriptor,

  atBlank(this: Scanning): boolean {
    const ink = inkOf(this);
    return ink.len === 0 || ink.last === 10;
  },

  ensureNewLine(this: Scanning): void {
    if (!this.atBlank()) inkOf(this).add("\n");
  },

  flushClose(this: Scanning, size = 2): void {
    if (!this.closed) return;
    const ink = inkOf(this);
    if (!this.atBlank()) ink.add("\n");
    if (size > 1) {
      let delimMin = this.delim;
      const trim = /\s+$/.exec(delimMin);
      if (trim) delimMin = delimMin.slice(0, delimMin.length - trim[0].length);
      for (let i = 1; i < size; i++) ink.add(`${delimMin}\n`);
    }
    this.closed = null;
  },

  write(this: Scanning, content?: string): void {
    this.flushClose();
    const ink = inkOf(this);
    if (this.delim && this.atBlank()) ink.add(this.delim);
    if (content) ink.add(content);
  },

  text(this: Scanning, value: string, escaped = true): void {
    const ink = inkOf(this);
    const lines = value.split("\n");
    for (let i = 0; i < lines.length; i++) {
      this.write();
      if (!escaped && lines[i][0] === "[" && ink.last === 33 && ink.priorSlashes() % 2 === 0) {
        ink.escapeLast();
      }
      ink.add(escaped ? this.esc(lines[i], this.atBlockStart) : lines[i]);
      if (i !== lines.length - 1) ink.add("\n");
    }
  },

  markString(this: Scanning, mark: Marked, open: boolean, parent: unknown, index: number): string {
    const info = this.marks[mark.type.name];
    if (info.expelEnclosingWhitespace) {
      const at = inkOf(this).len;
      if (open) this.inlines.push({ start: at, delimiter: info.open });
      else this.inlines.push({ ...(this.inlines.pop() as Inline), end: at });
    }
    return scanning.markString.call(this, mark, open, parent, index);
  },

  normalizeInline(this: Scanning, inline: Inline): Inline {
    const peek = inkOf(this).near(Math.max(0, inline.start - 1));
    this[peeked] = peek;
    const shifted = {
      ...inline,
      start: inline.start - peek.from,
      end: (inline.end ?? 0) - peek.from,
    };
    return (plainScanned as Pick<Scanning, "normalizeInline">).normalizeInline.call(this, shifted);
  },
};

function learn(proto: Scanning): void {
  if (scanned || proto === scanning) return;
  const owns = (name: string) => Object.prototype.hasOwnProperty.call(proto, name);
  if (!owns("markString") || !owns("normalizeInline") || !owns("render")) return;
  scanned = proto;
  plainScanned = {
    markString: proto.markString,
    normalizeInline: proto.normalizeInline,
    render: proto.render,
  };
  if (buffered) {
    proto.markString = swift.markString;
    proto.normalizeInline = swift.normalizeInline;
  }
}

const buffer = () => {
  if (buffered) return;
  Object.defineProperty(scanning, "out", swift.out);
  scanning.atBlank = swift.atBlank;
  scanning.ensureNewLine = swift.ensureNewLine;
  scanning.flushClose = swift.flushClose;
  scanning.write = swift.write;
  scanning.text = swift.text;
  if (scanned) {
    scanned.markString = swift.markString;
    scanned.normalizeInline = swift.normalizeInline;
  }
  buffered = true;
};

const unbuffer = () => {
  if (!buffered) return;
  delete (scanning as { out?: string }).out;
  scanning.atBlank = plain.atBlank;
  scanning.ensureNewLine = plain.ensureNewLine;
  scanning.flushClose = plain.flushClose;
  scanning.write = plain.write;
  scanning.text = plain.text;
  if (scanned && plainScanned) {
    scanned.markString = plainScanned.markString;
    scanned.normalizeInline = plainScanned.normalizeInline;
  }
  buffered = false;
};

buffer();

export const unaided = <T>(run: () => T): T => {
  unbuffer();
  try {
    return run();
  } finally {
    buffer();
  }
};

export const leaned = (): Record<string, string | null> => ({
  atBlank: String(plain.atBlank),
  ensureNewLine: String(plain.ensureNewLine),
  flushClose: String(plain.flushClose),
  write: String(plain.write),
  text: String(plain.text),
  esc: String(scanning.esc),
  baseMarkString: String(scanning.markString),
  markString: plainScanned ? String(plainScanned.markString) : null,
  normalizeInline: plainScanned ? String(plainScanned.normalizeInline) : null,
  render: plainScanned ? String(plainScanned.render) : null,
});

const Pictured = Image.extend({
  addStorage() {
    return {
      markdown: {
        serialize(
          state: { write: (text: string) => void; closeBlock: (node: unknown) => void },
          node: { attrs: Record<string, string> },
        ) {
          state.write(`![${labelled(node.attrs.alt ?? "")}](${node.attrs.src ?? ""})`);
          state.closeBlock(node);
        },
        parse: {},
      },
    };
  },
});

const Barred = Text.extend({
  addStorage() {
    return {
      markdown: {
        serialize(
          state: {
            text: (value: string, escaped?: boolean) => void;
            esc: (value: string) => string;
            inTable?: boolean;
          },
          node: { text?: string },
        ) {
          const text = node.text ?? "";
          if (!state.inTable) return state.text(text);
          state.text(state.esc(text), false);
        },
        parse: {},
      },
    };
  },
});

/// Markdown cannot say "icon", so it goes as HTML with its name inside: other readers show that.
/// This lands in the reader's own .md file, so a hand edit or a paste goes back as text, not markup.
const quoted = (value: string): string =>
  value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

const Ico = Node.create({
  name: "ico",
  inline: true,
  group: "inline",
  atom: true,
  // Selectable leaves it standing as a node selection after it lands, which stops the typing.
  selectable: false,

  addAttributes() {
    return {
      name: { default: null },
      hue: { default: null },
    };
  },

  /// A leaf with no text reads as an empty cell, and a table cell with nothing in it is dropped
  /// on save. Lending the node its own name keeps an icon-only cell from being erased.
  extendNodeSchema(extension) {
    return extension.name === "ico"
      ? { leafText: (node: { attrs: { name?: string | null } }) => spared(node.attrs.name ?? "") }
      : {};
  },

  parseHTML() {
    return [
      {
        tag: "span[data-ico]",
        getAttrs: (node) => ({
          name: (node as HTMLElement).getAttribute("data-ico"),
          hue: (node as HTMLElement).getAttribute("data-hue"),
        }),
      },
    ];
  },

  renderHTML({ node }) {
    const name = String(node.attrs.name ?? "");
    const hue = node.attrs.hue ? String(node.attrs.hue) : null;
    return [
      "span",
      {
        "data-ico": name,
        ...(hue ? { "data-hue": hue } : {}),
        class: `ico${hue ? ` ico-${hue}` : ""}`,
      },
      spared(name),
    ];
  },

  addNodeView() {
    return ({ node }) => {
      const held = document.createElement("span");
      const name = String(node.attrs.name ?? "");
      const hue = node.attrs.hue ? String(node.attrs.hue) : null;
      held.className = `ico${hue ? ` ico-${hue}` : ""}`;
      held.dataset.ico = name;
      if (hue) held.dataset.hue = hue;
      const drawn = document.createElementNS("http://www.w3.org/2000/svg", "svg");
      drawn.setAttribute("viewBox", "0 0 24 24");
      drawn.setAttribute("aria-hidden", "true");
      drawn.setAttribute("class", "glyph");
      drawn.innerHTML = markup(name) ?? "";
      held.append(drawn);
      return { dom: held };
    };
  },

  addStorage() {
    return {
      markdown: {
        serialize(
          state: { write: (value: string) => void },
          node: { attrs?: { name?: string | null; hue?: string | null } },
        ) {
          const name = node.attrs?.name ?? "";
          const hue = node.attrs?.hue;
          const spare = spared(name);
          state.write(
            `<span data-ico="${quoted(name)}"${hue ? ` data-hue="${quoted(hue)}"` : ""}>${quoted(spare)}</span>`,
          );
        },
        parse: {},
      },
    };
  },
});

export const PENS = ["yellow", "green", "blue", "pink"] as const;

export type Pen = (typeof PENS)[number];

const Lit = Highlight.configure({ multicolor: true }).extend({
  inclusive: false,

  addAttributes() {
    return {
      color: {
        default: null,
        parseHTML: (element: HTMLElement) =>
          element.getAttribute("data-pen") ?? element.getAttribute("data-color"),
        renderHTML: (attrs: { color?: string | null }) =>
          attrs.color ? { "data-pen": attrs.color } : {},
      },
    };
  },

  addStorage() {
    return {
      markdown: {
        serialize: {
          open(_state: unknown, mark: { attrs?: { color?: string | null } }) {
            const pen = mark.attrs?.color;
            return pen && pen !== "yellow" ? `<mark data-pen="${pen}">` : "==";
          },
          close(_state: unknown, mark: { attrs?: { color?: string | null } }) {
            const pen = mark.attrs?.color;
            return pen && pen !== "yellow" ? "</mark>" : "==";
          },
          mixable: true,
          expelEnclosingWhitespace: true,
        },
        parse: {
          setup(markdownit: { use: (plugin: unknown) => void }) {
            markdownit.use(markPlugin);
          },
        },
      },
    };
  },
});

export const CALLOUTS = ["note", "tip", "important", "warning", "caution"] as const;

export type Callout = (typeof CALLOUTS)[number];

const MARKED = /^\s*\[!(note|tip|important|warning|caution)\](?=\s|$)\s*/i;

const kindOf = (text: string): Callout | null => {
  const said = MARKED.exec(text);
  return said ? (said[1].toLowerCase() as Callout) : null;
};

const UNDER = /^ {0,3}(-+|=+)\s*$/;
const FENCED = /^ {0,3}(`{3,}|~{3,})/;

const deepened = (line: string): [number, string] => {
  let said = line.trim();
  let deep = 0;
  for (;;) {
    const mark = /^ {0,3}>/.exec(said);
    if (!mark) return [deep, said];
    said = said.slice(mark[0].length);
    if (said.startsWith(" ")) said = said.slice(1);
    deep += 1;
  }
};

export const loosened = (markdown: string): string => {
  const lines = markdown.replace(/^\ufeff+/, "").split("\n");
  const out: string[] = [];
  for (let at = 0; at < lines.length; at += 1) {
    out.push(lines[at]);
    const [deep, said] = deepened(lines[at]);
    if (!deep || !MARKED.test(said)) continue;
    if (at > 0 && deepened(lines[at - 1])[0] >= deep) continue;
    let ruled = false;
    for (let next = at + 1; next < lines.length && !ruled; next += 1) {
      const [under, text] = deepened(lines[next]);
      if (under !== deep || !text.trim() || FENCED.test(text)) break;
      ruled = UNDER.test(text);
    }
    if (ruled) out.push(lines[at].slice(0, lines[at].indexOf("[!")).trimEnd());
  }
  return out.join("\n");
};

const Said = Node.create({
  name: "callout",
  group: "block",
  content: "block+",
  defining: true,

  addAttributes() {
    return { kind: { default: "note" as Callout } };
  },

  addInputRules() {
    return [
      new InputRule({
        find: /^\[!(note|tip|important|warning|caution)\]\s$/i,
        handler: ({ state, range, match, chain }) => {
          const at = state.doc.resolve(range.from);
          let quoted: number | null = null;
          for (let deep = at.depth; deep > 0; deep -= 1) {
            if (at.node(deep).type.name === "blockquote") {
              quoted = at.before(deep);
              break;
            }
          }
          if (quoted === null) return null;
          const kind = match[1].toLowerCase();
          chain()
            .deleteRange(range)
            .command(({ tr }) => {
              tr.setNodeMarkup(quoted, state.schema.nodes.callout, { kind });
              return true;
            })
            .run();
        },
      }),
    ];
  },

  parseHTML() {
    return [
      {
        tag: "blockquote",
        priority: 60,
        getAttrs: (node) => {
          // The quote's own first paragraph, not any paragraph inside anything it holds.
          const first = (node as HTMLElement).firstElementChild;
          if (first?.tagName !== "P") return false;
          const walk = document.createTreeWalker(first, NodeFilter.SHOW_TEXT);
          const kind = kindOf(walk.nextNode()?.nodeValue ?? "");
          return kind ? { kind } : false;
        },
        contentElement: (node) => {
          const held = (node as HTMLElement).cloneNode(true) as HTMLElement;
          const first = held.firstElementChild;
          if (first?.tagName !== "P") return held;
          const walk = document.createTreeWalker(first, NodeFilter.SHOW_TEXT);
          const start = walk.nextNode();
          if (start) start.nodeValue = (start.nodeValue ?? "").replace(MARKED, "");
          if (!start?.nodeValue?.trim()) {
            const after = first.firstChild;
            if (after?.nodeName === "BR") after.remove();
            else if (
              start &&
              start.parentElement === first &&
              first.childNodes[1]?.nodeName === "BR"
            )
              first.childNodes[1].remove();
          }
          if (!first.textContent?.trim()) first.remove();
          return held;
        },
      },
    ];
  },

  renderHTML({ node }) {
    const kind = String(node.attrs.kind ?? "note");
    return [
      "blockquote",
      { "data-callout": kind, "data-said": t(`said${kind}` as Parameters<typeof t>[0]) },
      0,
    ];
  },

  addStorage() {
    return {
      markdown: {
        serialize(
          state: {
            write: (value: string) => void;
            wrapBlock: (delim: string, first: string | null, node: unknown, fn: () => void) => void;
            renderContent: (node: unknown) => void;
          },
          node: { attrs?: { kind?: string } },
        ) {
          const kind = String(node.attrs?.kind ?? "note").toUpperCase();
          state.wrapBlock("> ", null, node, () => {
            state.write(`[!${kind}]
`);
            state.renderContent(node);
          });
        },
        parse: {},
      },
    };
  },
});

const ALIGNED = ["left", "center", "right"] as const;

const leaning = () => ({
  textAlign: {
    default: null as string | null,
    parseHTML: (element: HTMLElement) => {
      const said = (element.style.textAlign || element.getAttribute("align") || "").toLowerCase();
      return (ALIGNED as readonly string[]).includes(said) ? said : null;
    },
    renderHTML: (attrs: { textAlign?: string | null }) =>
      attrs.textAlign ? { style: `text-align: ${attrs.textAlign}` } : {},
  },
});

const inACell = (editor: Writing): boolean => {
  const { $from } = editor.state.selection;
  for (let deep = $from.depth; deep > 0; deep -= 1) {
    const named = $from.node(deep).type.name;
    if (named === "tableCell" || named === "tableHeader") return true;
  }
  return false;
};

const barred = {
  addKeyboardShortcuts(this: { editor: Writing }) {
    return { Enter: () => inACell(this.editor) };
  },
};

const Celled = TableCell.extend({
  ...barred,
  addAttributes() {
    return { ...this.parent?.(), ...leaning() };
  },
});

const Headed = TableHeader.extend({
  ...barred,
  addAttributes() {
    return { ...this.parent?.(), ...leaning() };
  },
});

interface Marking {
  core: {
    ruler: {
      push: (name: string, rule: (state: { src: string; tokens: Tokened[] }) => boolean) => void;
    };
  };
}

interface Tokened {
  type: string;
  info: string;
  map: [number, number] | null;
  attrSet: (name: string, value: string) => void;
}

const RULED: Record<string, string> = {
  left: ":---",
  center: ":---:",
  right: "---:",
};

const STEP = 10;
const DASHES = [4, 120];

const dashed = (line: string): number[] | null => {
  const said = line.trim().replace(/^\|/, "").replace(/\|$/, "");
  if (!said.includes("-")) return null;
  return said.split("|").map((cell) => {
    const many = (cell.match(/-/g) ?? []).length;
    return many > DASHES[0] - 1 ? many * STEP : 0;
  });
};

const widths = (md: Marking) => {
  md.core.ruler.push("tistyWidths", (state) => {
    const lines = state.src.split("\n");
    let ruled: number[] | null = null;
    let column = 0;
    for (const token of state.tokens) {
      if (token.type === "table_open") ruled = dashed(lines[(token.map?.[0] ?? -1) + 1] ?? "");
      else if (token.type === "table_close") ruled = null;
      else if (token.type === "tr_open") column = 0;
      else if (token.type === "th_open" || token.type === "td_open") {
        const wide = ruled?.[column];
        if (wide) token.attrSet("colwidth", String(wide));
        column += 1;
      }
    }
    return true;
  });
};

interface Celling {
  type: { name: string };
  forEach: (fn: (kid: ProseNode) => void) => void;
  childCount: number;
  attrs: { textAlign?: string | null; colspan?: number; rowspan?: number; colwidth?: unknown };
}

interface Rowed {
  childCount: number;
  forEach: (fn: (cell: Celling, offset: number, at: number) => void) => void;
}

/// The delimiter row is where Markdown keeps a column's alignment, and tiptap-markdown writes
/// `---` for every column whatever the cells say.
const Ruled = Table.configure({ resizable: true }).extend({
  addStorage() {
    return {
      markdown: {
        serialize(
          state: {
            write: (value: string) => void;
            ensureNewLine: () => void;
            renderInline: (node: unknown) => void;
            closeBlock: (node: unknown) => void;
            text: (value: string, escaped?: boolean) => void;
            inTable?: boolean;
          },
          node: { forEach: (fn: (row: Rowed, offset: number, at: number) => void) => void },
        ) {
          // A span or a cell holding more than one block has no markdown to be written in.
          let plain = true;
          node.forEach((row) => {
            row.forEach((cell) => {
              const { colspan = 1, rowspan = 1 } = cell.attrs;
              let holds = true;
              cell.forEach((kid) => {
                if (kid.type.name !== "paragraph" && kid.type.name !== "image") holds = false;
              });
              if (colspan > 1 || rowspan > 1 || cell.childCount > 1 || !holds) {
                plain = false;
              }
            });
          });
          if (!plain) {
            const held = node as unknown as ProseNode;
            state.write(getHTMLFromFragment(Fragment.from(held), held.type.schema));
            state.closeBlock(node);
            return;
          }
          state.inTable = true;
          const said = state.text;
          state.text = (value: string, escaped = true) =>
            said.call(state, escaped ? value : value.replace(/\|/g, "\\|"), escaped);
          const leans: (string | null)[] = [];
          const wides: (number | null)[] = [];
          node.forEach((row, _at, index) => {
            state.write("| ");
            row.forEach((cell, _spot, column) => {
              if (column) state.write(" | ");
              if (!index) {
                leans[column] = cell.attrs.textAlign ?? null;
                const held = cell.attrs.colwidth;
                wides[column] = Array.isArray(held) ? (held[0] ?? null) : null;
              }
              cell.forEach((kid) => {
                if (kid.type.name === "image") {
                  const alt = labelled(String(kid.attrs.alt ?? ""));
                  const src = String(kid.attrs.src ?? "").replace(/\|/g, "\\|");
                  state.write(`![${alt.replace(/\|/g, "\\|")}](${src})`);
                } else if (kid.content.size > 0) {
                  state.renderInline(kid);
                }
              });
            });
            state.write(" |");
            state.ensureNewLine();
            if (!index) {
              const ruled = Array.from({ length: row.childCount }, (_, column) => {
                const lean = RULED[leans[column] ?? ""] ?? "---";
                const wide = wides[column];
                if (!wide) return lean;
                const many = Math.min(DASHES[1], Math.max(DASHES[0], Math.round(wide / STEP)));
                return lean.replace(/-+/, "-".repeat(many));
              }).join(" | ");
              state.write(`| ${ruled} |`);
              state.ensureNewLine();
            }
          });
          state.text = said;
          state.closeBlock(node);
          state.inTable = false;
        },
        parse: { setup: widths },
      },
    };
  },
});

export const TONGUES = [
  "bash",
  "c",
  "cpp",
  "csharp",
  "css",
  "diff",
  "go",
  "graphql",
  "ini",
  "java",
  "javascript",
  "json",
  "kotlin",
  "less",
  "lua",
  "makefile",
  "markdown",
  "objectivec",
  "perl",
  "php",
  "python",
  "r",
  "ruby",
  "rust",
  "scss",
  "shell",
  "sql",
  "swift",
  "typescript",
  "vbnet",
  "wasm",
  "xml",
  "yaml",
] as const;

// One counter for the window: mermaid resolves its id against the whole document.
let sketches = 0;

const sketchers = new Set<() => void>();
let watching: MutationObserver | null = null;

const watched = () => {
  if (watching) return;
  watching = new MutationObserver(() => {
    for (const one of sketchers) one();
  });
  watching.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme"],
  });
};

type Setting = { set: (options: { langPrefix: string }) => void };

const NAMED = /\btitle="((?:[^"\\\n]|\\.)*)"/;

const named = (md: Marking) => {
  md.core.ruler.push("tistyNamed", (state) => {
    for (const token of state.tokens) {
      const said = token.type === "fence" ? NAMED.exec(token.info ?? "") : null;
      if (!said) continue;
      token.attrSet("data-title", said[1].replace(/\\(.)/g, "$1"));
      token.info = (token.info ?? "").replace(NAMED, "").trim();
    }
    return true;
  });
};

export const DRAWN = ["mermaid", "math"];

export const KINDS = ["note", "tip", "important", "warning", "caution"] as const;

const SHORT: Record<string, string> = { mmd: "mermaid" };

const Lettered = CodeBlockLowlight.configure({ lowlight: createLowlight(common) }).extend({
  addInputRules() {
    return [
      textblockTypeInputRule({
        find: /^```(mmd)[\s\n]$/,
        type: this.type,
        getAttributes: ({ 1: said }) => ({ language: SHORT[said] }),
      }),
      ...(this.parent?.() ?? []),
    ];
  },

  addAttributes() {
    return {
      ...this.parent?.(),
      title: {
        default: null,
        parseHTML: (element: HTMLElement) =>
          element.getAttribute("data-title") ??
          element.firstElementChild?.getAttribute("data-title") ??
          null,
        renderHTML: (attrs: { title?: string | null }) =>
          attrs.title ? { "data-title": attrs.title } : {},
      },
    };
  },

  addNodeView() {
    return ({ node, editor, getPos }) => {
      const held = document.createElement("div");
      held.className = "lit";

      const bar = document.createElement("div");
      bar.className = "lit-bar";
      bar.setAttribute("contenteditable", "false");
      const picked = document.createElement("select");
      picked.className = "lit-tongue";
      picked.title = t("codeTongue");
      picked.setAttribute("aria-label", t("codeTongue"));
      const none = document.createElement("option");
      none.value = "";
      none.textContent = t("codeNoTongue");
      picked.append(none);
      for (const one of [...TONGUES, ...DRAWN]) {
        const said = document.createElement("option");
        said.value = one;
        said.textContent = one;
        picked.append(said);
      }
      const name = document.createElement("input");
      name.className = "lit-name";
      name.type = "text";
      name.placeholder = t("codeName");
      name.setAttribute("aria-label", t("codeName"));

      const said = document.createElement("span");
      said.className = "lit-said";
      const showing = (one: ProseNode) => {
        const now = String(one.attrs.language ?? "");
        const drawing = DRAWN.includes(now);
        if (drawing) said.textContent = now;
        if (document.activeElement !== name) name.value = String(one.attrs.title ?? "");
        picked.hidden = drawing;
        said.hidden = !drawing;
        if (drawing) return;
        if (now && !picked.querySelector(`option[value="${CSS.escape(now)}"]`)) {
          const own = document.createElement("option");
          own.value = now;
          own.textContent = now;
          picked.append(own);
        }
        picked.value = now;
      };
      showing(node);

      picked.addEventListener("change", () => {
        const at = getPos();
        if (at === undefined) return;
        editor
          .chain()
          .focus()
          .command(({ tr }) => {
            tr.setNodeAttribute(at, "language", picked.value || null);
            return true;
          })
          .run();
      });
      name.addEventListener("change", () => {
        const at = getPos();
        if (at === undefined) return;
        editor
          .chain()
          .command(({ tr }) => {
            tr.setNodeAttribute(at, "title", name.value.trim() || null);
            return true;
          })
          .run();
      });
      bar.append(name, said, picked);

      const drawn = document.createElement("div");
      drawn.className = "lit-drawn";
      drawn.setAttribute("contenteditable", "false");

      const body = document.createElement("div");
      body.className = "lit-body";
      const lines = document.createElement("div");
      lines.className = "lit-lines";
      lines.setAttribute("contenteditable", "false");
      lines.setAttribute("aria-hidden", "true");
      const pre = document.createElement("pre");
      const code = document.createElement("code");
      pre.append(code);
      body.append(lines, pre);
      held.append(bar, body, drawn);

      let asked = 0;
      let drew = "";

      const figured = (source: string) => {
        asked += 1;
        const mine = asked;
        if (!source.trim()) {
          drawn.replaceChildren();
          return;
        }
        import("katex")
          .then(({ default: katex }) => {
            if (mine !== asked) return;
            drawn.replaceChildren();
            katex.render(source, drawn, { displayMode: true, throwOnError: false });
          })
          .catch(() => {
            if (mine !== asked) return;
            drew = "";
            drawn.replaceChildren();
          });
      };

      const sketched = (source: string) => {
        asked += 1;
        sketches += 1;
        const mine = asked;
        const named = `said${sketches}`;
        if (!source.trim()) {
          drawn.replaceChildren();
          return;
        }
        import("mermaid")
          .then(({ default: mermaid }) => {
            const dark = document.documentElement.getAttribute("data-theme") === "dark";
            mermaid.initialize({
              startOnLoad: false,
              securityLevel: "strict",
              theme: dark ? "dark" : "default",
            });
            return mermaid.render(named, source, drawn);
          })
          .then(({ svg }) => {
            if (mine !== asked) return;
            drawn.innerHTML = svg;
          })
          .catch(() => {
            if (mine !== asked) return;
            drew = "";
            drawn.replaceChildren();
          });
      };

      const counted = (one: { textContent: string | null }) => {
        const many = Math.max(1, (one.textContent ?? "").split("\n").length);
        lines.replaceChildren();
        for (let n = 1; n <= many; n += 1) {
          const said = document.createElement("span");
          said.textContent = String(n);
          lines.append(said);
        }
      };
      counted(node);

      const sketching = (one: { attrs: Record<string, unknown>; textContent: string | null }) => {
        const tongue = String(one.attrs.language ?? "");
        if (!DRAWN.includes(tongue)) {
          asked += 1;
          drew = "";
          drawn.replaceChildren();
          return;
        }
        const source = one.textContent ?? "";
        // Every keystroke would otherwise start a render, and a failed one leaves litter behind.
        if (source === drew) return;
        drew = source;
        if (tongue === "math") figured(source);
        else sketched(source);
      };
      sketching(node);

      let mine: { attrs: Record<string, unknown>; textContent: string | null } = node;
      const again = () => {
        if (!DRAWN.includes(String(mine.attrs.language ?? ""))) return;
        drew = "";
        sketching(mine);
      };
      sketchers.add(again);
      watched();

      return {
        dom: held,
        contentDOM: code,
        ignoreMutation: (one) => one.type !== "selection" && !code.contains(one.target),
        destroy: () => {
          asked += 1;
          sketchers.delete(again);
          drawn.replaceChildren();
        },
        update: (fresh) => {
          if (fresh.type.name !== "codeBlock") return false;
          mine = fresh;
          counted(fresh);
          sketching(fresh);
          showing(fresh);
          return true;
        },
      };
    };
  },

  addStorage() {
    return {
      markdown: {
        serialize(
          state: {
            write: (value: string) => void;
            text: (value: string, escaped?: boolean) => void;
            ensureNewLine: () => void;
            closeBlock: (node: unknown) => void;
          },
          node: { attrs?: { language?: unknown; title?: unknown }; textContent: string },
        ) {
          const said = String(node.attrs?.language ?? "");
          const name = String(node.attrs?.title ?? "").replace(/([\\"])/g, "\\$1");
          const told = name ? `${said} title="${name}"` : said;
          const mark = told.includes("`") ? "~" : "`";
          const runs = node.textContent.split(new RegExp(`[^\\${mark}]+`));
          const longest = runs.reduce((most, one) => Math.max(most, one.length), 0);
          const wall = mark.repeat(Math.max(3, longest + 1));

          state.write(`${wall}${told}\n`);
          state.text(node.textContent, false);
          state.ensureNewLine();
          state.write(wall);
          state.closeBlock(node);
        },
        parse: {
          setup(this: { options: { languageClassPrefix?: string } }, md: Setting & Marking) {
            md.set({ langPrefix: this.options.languageClassPrefix ?? "language-" });
            named(md);
          },
          updateDOM(element: HTMLElement) {
            element.innerHTML = element.innerHTML.replace(/\n<\/code><\/pre>/g, "</code></pre>");
          },
        },
      },
    };
  },
});

const tasked = (item: Element): boolean => item.classList.contains("task-list-item");

const apart = (item: Element): boolean => item.getAttribute("data-apart") === "true";

interface Lined {
  type: string;
  hidden?: boolean;
  map?: [number, number] | null;
  attrSet: (name: string, value: string) => void;
}

const PLAIN = new Set([
  "paragraph_open",
  "paragraph_close",
  "inline",
  "list_item_open",
  "list_item_close",
  "bullet_list_open",
  "bullet_list_close",
  "ordered_list_open",
  "ordered_list_close",
]);

/// A blank line anywhere makes Markdown call the whole list loose, and a nested list carries its
/// own blank lines up to the one holding it. Both are read here, where the lines are still known.
/// A list holding anything but prose is left alone: written tight it would not read back the same.
const spaced = (md: {
  core: { ruler: { push: (name: string, rule: (state: { tokens: Lined[] }) => void) => void } };
}): void => {
  md.core.ruler.push("spaced", (state) => {
    const lists: { token: Lined; blank: boolean; tight: boolean }[] = [];
    const items: { token: Lined; end: number }[] = [];
    for (const token of state.tokens) {
      const list = lists[lists.length - 1];
      if (token.type.endsWith("_list_open")) {
        lists.push({ token, blank: false, tight: true });
      } else if (token.type.endsWith("_list_close")) {
        const done = lists.pop();
        if (done?.tight) done.token.attrSet("data-tight", "true");
      } else if (token.type === "list_item_open") {
        if (list?.blank) token.attrSet("data-apart", "true");
        items.push({ token, end: token.map ? token.map[0] : 0 });
      } else if (token.type === "list_item_close") {
        const held = items.pop();
        if (!held) continue;
        if (list) list.blank = held.token.map ? held.token.map[1] > held.end : false;
        const up = items[items.length - 1];
        if (up) up.end = Math.max(up.end, held.end);
      } else {
        if (token.type === "paragraph_open" && list && !token.hidden) list.tight = false;
        if (!PLAIN.has(token.type)) for (const one of lists) one.tight = false;
        if (token.map && items.length) {
          const held = items[items.length - 1];
          held.end = Math.max(held.end, token.map[1]);
        }
      }
    }
  });
};

const parted = (list: Element): void => {
  const kids = Array.from(list.childNodes).filter((one) => one.nodeType === 1) as Element[];
  const runs: { task: boolean; items: Element[] }[] = [];
  for (const one of kids) {
    const task = tasked(one);
    const last = runs[runs.length - 1];
    if (last?.task === task) last.items.push(one);
    else runs.push({ task, items: [one] });
  }
  const alone = runs.length < 2 ? runs[0] : null;
  if (alone && (!alone.task || list.tagName === "UL")) {
    if (alone.task) list.setAttribute("data-type", "taskList");
    return;
  }
  let count = Number(list.getAttribute("start") ?? 1) || 1;
  for (const run of runs) {
    const fresh = list.ownerDocument.createElement(run.task ? "ul" : list.tagName);
    if (run.task) fresh.setAttribute("data-type", "taskList");
    else if (count > 1) fresh.setAttribute("start", String(count));
    count += run.items.length;
    if (run.items.every((one, at) => at === 0 || !apart(one))) {
      fresh.setAttribute("data-tight", "true");
    }
    if (apart(run.items[0])) fresh.setAttribute("data-apart", "true");
    for (const one of run.items) fresh.append(one);
    list.before(fresh);
  }
  list.remove();
};

const LISTS = new Set(["bulletList", "taskList"]);

interface Listing {
  flushClose: (size: number) => void;
  renderList: (node: ProseNode, delim: string, first: (index: number) => string) => void;
}

function glued(
  this: { editor: Writing },
  state: Listing,
  node: ProseNode,
  parent: ProseNode | null,
  index: number,
): void {
  const before = parent && index > 0 ? parent.child(index - 1) : null;
  if (
    before &&
    LISTS.has(before.type.name) &&
    before.type.name !== node.type.name &&
    before.attrs.tight !== false &&
    node.attrs.tight !== false &&
    node.attrs.apart !== true
  ) {
    state.flushClose(1);
  }
  const marked = (
    this.editor.storage as unknown as {
      markdown: { options: { bulletListMarker?: string } };
    }
  ).markdown.options.bulletListMarker;
  state.renderList(node, "  ", () => `${marked || "-"} `);
}

const APART = {
  apart: {
    default: false,
    rendered: false,
    parseHTML: (element: HTMLElement) => element.getAttribute("data-apart") === "true",
  },
};

const Listed = BulletList.extend({
  addAttributes() {
    return { ...this.parent?.(), ...APART };
  },

  addStorage() {
    return { markdown: { serialize: glued } };
  },
});

const Tightened = TaskList.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      tight: {
        default: true,
        rendered: false,
        parseHTML: (element: HTMLElement) =>
          element.getAttribute("data-tight") === "true" || !element.querySelector("p"),
      },
      ...APART,
    };
  },

  addStorage() {
    return {
      markdown: {
        serialize: glued,
        parse: {
          setup(md: unknown) {
            (md as { use: (one: unknown) => void }).use(taskLists);
            spaced(md as Parameters<typeof spaced>[0]);
          },
          updateDOM(element: HTMLElement) {
            for (const list of [...element.querySelectorAll(".contains-task-list")]) parted(list);
          },
        },
      },
    };
  },
});

export const written = () => [
  StarterKit.configure({
    link: { openOnClick: false, autolink: true, protocols: ["tisty"] },
    text: false,
    codeBlock: false,
    bulletList: false,
  }),
  Listed,
  Barred,
  Lettered,
  Pictured,
  Ruled,
  TableRow,
  Headed,
  Celled,
  Tightened,
  Said,
  Ico,
  Lit,
  TaskItem.configure({ nested: true }),
  Markdown.configure({ html: true, linkify: true, breaks: true, transformPastedText: true }),
];

export const asMarkdown = (editor: Writing): string | null => {
  if (editor.isDestroyed) return null;
  const kept = (editor.storage as unknown as { markdown?: { getMarkdown?: () => string } })
    .markdown;
  return typeof kept?.getMarkdown === "function" ? kept.getMarkdown() : null;
};

export const bared = (markdown: string): string => {
  const fence = /^\s*(?:```|~~~)/;
  let inside = false;
  return markdown
    .split("\n")
    .map((line) => {
      if (fence.test(line)) {
        inside = !inside;
        return line;
      }
      if (inside) return line;
      return line.replace(/<\/?u\b[^>]*>/gi, "");
    })
    .join("\n");
};

export interface Head {
  key: string;
  level: number;
  text: string;
  go: () => void;
}

export const headed = (editor: Writing): Head[] => {
  if (editor.isDestroyed) return [];
  const found: Head[] = [];
  editor.state.doc.descendants((node, pos) => {
    if (node.type.name !== "heading") return true;
    const text = node.textContent.trim();
    if (text) {
      found.push({
        key: String(pos),
        level: Number(node.attrs.level ?? 1),
        text,
        go: () =>
          editor
            .chain()
            .focus()
            .setTextSelection(pos + 1)
            .scrollIntoView()
            .run(),
      });
    }
    return false;
  });
  return found;
};

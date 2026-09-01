import type { Editor as Writing } from "@tiptap/core";
import { getHTMLFromFragment, Node } from "@tiptap/core";
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
          state: { text: (value: string) => void; inTable?: boolean },
          node: { text?: string },
        ) {
          const text = node.text ?? "";
          state.text(state.inTable ? text.replace(/\|/g, "\\|") : text);
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

const MARKED = /^\s*\[!(note|tip|important|warning|caution)\]\s*/i;

const kindOf = (text: string): Callout | null => {
  const said = MARKED.exec(text);
  return said ? (said[1].toLowerCase() as Callout) : null;
};

const Said = Node.create({
  name: "callout",
  group: "block",
  content: "block+",
  defining: true,

  addAttributes() {
    return { kind: { default: "note" as Callout } };
  },

  parseHTML() {
    return [
      {
        tag: "blockquote",
        priority: 60,
        getAttrs: (node) => {
          const kind = kindOf((node as HTMLElement).textContent ?? "");
          return kind ? { kind } : false;
        },
        contentElement: (node) => {
          const held = (node as HTMLElement).cloneNode(true) as HTMLElement;
          const first = held.querySelector("p");
          if (!first) return held;
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

const Celled = TableCell.extend({
  content: "paragraph",
  addAttributes() {
    return { ...this.parent?.(), ...leaning() };
  },
});

const Headed = TableHeader.extend({
  content: "paragraph",
  addAttributes() {
    return { ...this.parent?.(), ...leaning() };
  },
});

const RULED: Record<string, string> = {
  left: ":---",
  center: ":---:",
  right: "---:",
};

interface Celling {
  firstChild: { textContent: string } | null;
  childCount: number;
  attrs: { textAlign?: string | null; colspan?: number; rowspan?: number };
}

interface Rowed {
  childCount: number;
  forEach: (fn: (cell: Celling, offset: number, at: number) => void) => void;
}

/// The delimiter row is where Markdown keeps a column's alignment, and tiptap-markdown writes
/// `---` for every column whatever the cells say.
const Ruled = Table.configure({ resizable: false }).extend({
  addStorage() {
    return {
      markdown: {
        serialize(
          state: {
            write: (value: string) => void;
            ensureNewLine: () => void;
            renderInline: (node: unknown) => void;
            closeBlock: (node: unknown) => void;
            inTable?: boolean;
          },
          node: { forEach: (fn: (row: Rowed, offset: number, at: number) => void) => void },
        ) {
          // A span or a cell holding more than one block has no markdown to be written in.
          let plain = true;
          node.forEach((row) =>
            row.forEach((cell) => {
              const { colspan = 1, rowspan = 1 } = cell.attrs;
              if (colspan > 1 || rowspan > 1 || cell.childCount > 1) plain = false;
            }),
          );
          if (!plain) {
            const held = node as unknown as ProseNode;
            state.write(getHTMLFromFragment(Fragment.from(held), held.type.schema));
            state.closeBlock(node);
            return;
          }
          state.inTable = true;
          const leans: (string | null)[] = [];
          node.forEach((row, _at, index) => {
            state.write("| ");
            row.forEach((cell, _spot, column) => {
              if (column) state.write(" | ");
              if (!index) leans[column] = cell.attrs.textAlign ?? null;
              const held = cell.firstChild;
              if (held?.textContent.trim()) state.renderInline(held);
            });
            state.write(" |");
            state.ensureNewLine();
            if (!index) {
              const ruled = Array.from({ length: row.childCount }, (_, column) =>
                RULED[leans[column] ?? ""] ? RULED[leans[column] ?? ""] : "---",
              ).join(" | ");
              state.write(`| ${ruled} |`);
              state.ensureNewLine();
            }
          });
          state.closeBlock(node);
          state.inTable = false;
        },
        parse: {},
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

const KNOWN: string[] = [...TONGUES, "mermaid"];

const Lettered = CodeBlockLowlight.configure({ lowlight: createLowlight(common) }).extend({
  addNodeView() {
    return ({ node, editor, getPos }) => {
      const held = document.createElement("div");
      held.className = "lit";

      const bar = document.createElement("div");
      bar.className = "lit-bar";
      bar.contentEditable = "false";
      const picked = document.createElement("select");
      picked.className = "lit-tongue";
      picked.title = t("codeTongue");
      picked.setAttribute("aria-label", t("codeTongue"));
      const none = document.createElement("option");
      none.value = "";
      none.textContent = t("codeNoTongue");
      picked.append(none);
      for (const one of [...TONGUES, "mermaid"]) {
        const said = document.createElement("option");
        said.value = one;
        said.textContent = one;
        picked.append(said);
      }
      const said = String(node.attrs.language ?? "");
      picked.value = KNOWN.includes(said) ? said : "";
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
      bar.append(picked);

      const drawn = document.createElement("div");
      drawn.className = "lit-drawn";
      drawn.contentEditable = "false";

      const body = document.createElement("div");
      body.className = "lit-body";
      const lines = document.createElement("div");
      lines.className = "lit-lines";
      lines.setAttribute("aria-hidden", "true");
      const pre = document.createElement("pre");
      const code = document.createElement("code");
      pre.append(code);
      body.append(lines, pre);
      held.append(bar, body, drawn);

      let asked = 0;

      const sketched = (source: string) => {
        const mine = (asked += 1);
        if (!source.trim()) {
          held.classList.remove("lit-sketched");
          drawn.replaceChildren();
          return;
        }
        import("mermaid")
          .then(({ default: mermaid }) => {
            mermaid.initialize({ startOnLoad: false, securityLevel: "strict" });
            return mermaid.render(`said${mine}`, source);
          })
          .then(({ svg }) => {
            if (mine !== asked) return;
            drawn.innerHTML = svg;
            held.classList.add("lit-sketched");
          })
          .catch(() => {
            if (mine !== asked) return;
            drawn.replaceChildren();
            held.classList.remove("lit-sketched");
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
        if (String(one.attrs.language ?? "") !== "mermaid") {
          asked += 1;
          drawn.replaceChildren();
          held.classList.remove("lit-sketched");
          return;
        }
        sketched(one.textContent ?? "");
      };
      sketching(node);

      return {
        dom: held,
        contentDOM: code,
        update: (fresh) => {
          if (fresh.type.name !== "codeBlock") return false;
          counted(fresh);
          sketching(fresh);
          const now = String(fresh.attrs.language ?? "");
          picked.value = KNOWN.includes(now) ? now : "";
          return true;
        },
      };
    };
  },
});

const Tightened = TaskList.extend({
  addAttributes() {
    return { ...this.parent?.(), tight: { default: true, rendered: false } };
  },
});

export const written = () => [
  StarterKit.configure({
    link: { openOnClick: false, autolink: true, protocols: ["tisty"] },
    text: false,
    codeBlock: false,
  }),
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
  Barred,
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

import { beforeAll, describe, expect, it } from "vitest";
import { Editor } from "@tiptap/core";
import { MarkdownSerializerState } from "prosemirror-markdown";
import { asMarkdown, leaned, unaided, written } from "../ui/writing";

type Loose = Record<string, unknown>;

const proto = MarkdownSerializerState.prototype as unknown as Loose;

const serialize = (content: string) => {
  const editor = new Editor({ extensions: written(), content });
  const out = asMarkdown(editor);
  editor.destroy();
  return out;
};

const glimpsed = (): Loose => {
  const esc = proto.esc as (...args: unknown[]) => unknown;
  let caught: Loose | null = null;
  proto.esc = function (this: Loose, ...args: unknown[]) {
    caught ??= this;
    return esc.apply(this, args);
  };
  serialize("hola **mundo ** y *algo* y ![i](a.png)");
  proto.esc = esc;
  if (!caught) throw new Error("no serializer state reached esc(): prosemirror-markdown changed under writing.ts");
  return caught;
};

const tidy = (source: string | null) => (source ?? "").replace(/\/\/[^\n]*/g, " ").replace(/\s+/g, " ").trim();

let pins: Record<string, string | null>;

beforeAll(() => {
  serialize("hola **mundo ** y *algo*");
  pins = leaned();
});

describe("the state fields the buffer in writing.ts writes through", () => {
  it("still finds out, delim, closed, inlines, esc and atBlockStart on a live serializer state", () => {
    const state = glimpsed();
    expect(typeof state.out).toBe("string");
    expect(typeof state.delim).toBe("string");
    expect("closed" in state).toBe(true);
    expect(Array.isArray(state.inlines)).toBe(true);
    expect(typeof state.atBlockStart).toBe("boolean");
    expect(typeof proto.esc).toBe("function");
    expect(typeof proto.markString).toBe("function");
  });

  it("still reaches the tiptap-markdown subclass that owns markString, normalizeInline and render", () => {
    expect(pins.markString).not.toBeNull();
    expect(pins.normalizeInline).not.toBeNull();
    expect(pins.render).not.toBeNull();
  });

  it("swaps both prototypes together, so nothing keeps reading the flat string behind the buffer", () => {
    expect(typeof Object.getOwnPropertyDescriptor(proto, "out")?.get).toBe("function");
    unaided(() => {
      expect(Object.getOwnPropertyDescriptor(proto, "out")).toBeUndefined();
      expect(String(proto.atBlank)).toContain("this.out");
    });
    expect(String(proto.atBlank)).not.toContain("/(^|\\n)$/");
  });
});

describe("the library source writing.ts copied, pinned so an upgrade cannot slip past", () => {
  it("prosemirror-markdown atBlank", () => {
    expect(tidy(pins.atBlank)).toBe("atBlank() { return /(^|\\n)$/.test(this.out); }");
  });

  it("prosemirror-markdown ensureNewLine", () => {
    expect(tidy(pins.ensureNewLine)).toBe('ensureNewLine() { if (!this.atBlank()) this.out += "\\n"; }');
  });

  it("prosemirror-markdown flushClose", () => {
    expect(tidy(pins.flushClose)).toBe(
      'flushClose(size = 2) { if (this.closed) { if (!this.atBlank()) this.out += "\\n"; if (size > 1) { let delimMin = this.delim; let trim = /\\s+$/.exec(delimMin); if (trim) delimMin = delimMin.slice(0, delimMin.length - trim[0].length); for (let i = 1; i < size; i++) this.out += delimMin + "\\n"; } this.closed = null; } }',
    );
  });

  it("prosemirror-markdown write", () => {
    expect(tidy(pins.write)).toBe(
      "write(content) { this.flushClose(); if (this.delim && this.atBlank()) this.out += this.delim; if (content) this.out += content; }",
    );
  });

  it("prosemirror-markdown text", () => {
    expect(tidy(pins.text)).toBe(
      'text(text, escape = true) { let lines = text.split("\\n"); for (let i = 0; i < lines.length; i++) { this.write(); if (!escape && lines[i][0] == "[" && /(^|[^\\\\])\\!$/.test(this.out)) this.out = this.out.slice(0, this.out.length - 1) + "\\\\!"; this.out += escape ? this.esc(lines[i], this.atBlockStart) : lines[i]; if (i != lines.length - 1) this.out += "\\n"; } }',
    );
  });

  it("prosemirror-markdown esc, which the buffered text() keeps calling", () => {
    expect(tidy(pins.esc)).toBe(
      "esc(str, startOfLine = false) { str = str.replace(/[`*\\\\~\\[\\]_]/g, (m, i) => m == \"_\" && i > 0 && i + 1 < str.length && str[i - 1].match(/\\w/) && str[i + 1].match(/\\w/) ? m : \"\\\\\" + m); if (startOfLine) str = str.replace(/^(\\+[ ]|[\\-*>])/, \"\\\\$&\").replace(/^(\\s*)(#{1,6})(\\s|$)/, '$1\\\\$2$3').replace(/^(\\s*\\d+)\\.\\s/, \"$1\\\\. \"); if (this.options.escapeExtraCharacters) str = str.replace(this.options.escapeExtraCharacters, \"\\\\$&\"); return str; }",
    );
  });

  it("prosemirror-markdown markString, the super the buffered markString stands in for", () => {
    expect(tidy(pins.baseMarkString)).toBe(
      'markString(mark, open, parent, index) { let info = this.getMark(mark.type.name); let value = open ? info.open : info.close; return typeof value == "string" ? value : value(this, mark, parent, index); }',
    );
  });

  it("tiptap-markdown markString", () => {
    expect(tidy(pins.markString)).toBe(
      "markString(mark, open, parent, index) { const info = this.marks[mark.type.name]; if (info.expelEnclosingWhitespace) { if (open) { this.inlines.push({ start: this.out.length, delimiter: info.open }); } else { const top = this.inlines.pop(); this.inlines.push({ ...top, end: this.out.length }); } } return super.markString(mark, open, parent, index); }",
    );
  });

  it("tiptap-markdown normalizeInline, where the buffer hands over a window instead of the whole string", () => {
    expect(tidy(pins.normalizeInline)).toBe(
      "normalizeInline(inline) { let { start, end } = inline; while (this.out.charAt(start).match(/\\s/)) { start++; } return { ...inline, start }; }",
    );
  });

  it("tiptap-markdown render, which must still read out once and write it back once around trimInline", () => {
    expect(tidy(pins.render)).toBe(
      "render(node, parent, index) { super.render(node, parent, index); const top = this.inlines[this.inlines.length - 1]; if (top !== null && top !== void 0 && top.start && top !== null && top !== void 0 && top.end) { const { delimiter, start, end } = this.normalizeInline(top); this.out = trimInline(this.out, delimiter, start, end); this.inlines.pop(); } }",
    );
  });
});

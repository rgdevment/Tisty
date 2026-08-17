import { describe, expect, it } from "vitest";
import { Editor } from "@tiptap/core";
import { asMarkdown, written } from "../ui/writing";
import { asked, narrowed } from "../ui/Slash";

const build = (content: string) => new Editor({ extensions: written(), content });

const markdown = (editor: Editor) => asMarkdown(editor) ?? "";

const formatted = (content: string, apply: (editor: Editor) => void) => {
  const editor = build(content);
  apply(editor);
  const out = markdown(editor);
  editor.destroy();
  return out;
};

const roundtripped = (content: string) => {
  const editor = build(content);
  const out = markdown(editor);
  editor.destroy();
  return out;
};

describe("what this suite is measuring", () => {
  it("builds its editors from the very list the window uses, not from a copy", () => {
    const names = written().map((one) => (one as { name: string }).name);

    expect(names).toContain("markdown");
    expect(names).toContain("table");
    expect(names).toContain("image");
  });
});

describe("the markdown each formatting command writes", () => {
  it("wraps the selection in ** for bold", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleBold().run())).toBe("**hello**");
  });

  it("wraps the selection in * for italic", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleItalic().run())).toBe("*hello*");
  });

  it("wraps the selection in ~~ for strikethrough", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleStrike().run())).toBe("~~hello~~");
  });

  it("wraps the selection in a single backtick for inline code", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleCode().run())).toBe("`hello`");
  });

  it("fences the paragraph off for a code block", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleCodeBlock().run())).toBe(
      "```\nhello\n```",
    );
  });

  it("prefixes the line with > for a quote", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleBlockquote().run())).toBe("> hello");
  });

  it("prefixes the line with - for a bulleted list", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleBulletList().run())).toBe("- hello");
  });

  it("prefixes the line with 1. for a numbered list", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleOrderedList().run())).toBe("1. hello");
  });

  it("prefixes the line with an empty checkbox for a task list", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleTaskList().run())).toBe("- [ ] hello");
  });

  it("marks a level 1 heading with a single #", () => {
    expect(
      formatted("hello", (e) => e.chain().focus().selectAll().toggleHeading({ level: 1 }).run()),
    ).toBe("# hello");
  });

  it("marks a level 2 heading with ##", () => {
    expect(
      formatted("hello", (e) => e.chain().focus().selectAll().toggleHeading({ level: 2 }).run()),
    ).toBe("## hello");
  });

  it("marks a level 3 heading with ###", () => {
    expect(
      formatted("hello", (e) => e.chain().focus().selectAll().toggleHeading({ level: 3 }).run()),
    ).toBe("### hello");
  });

  it("writes a standalone divider as three dashes", () => {
    expect(formatted("", (e) => e.chain().focus().setHorizontalRule().run())).toBe("---");
  });

  it("separates a divider from the text above it with a blank line", () => {
    expect(formatted("hello", (e) => e.chain().focus("end").setHorizontalRule().run())).toBe("hello\n\n---");
  });

  it("writes a fresh table as a header row, its separator, and two empty body rows", () => {
    expect(
      formatted("", (e) => e.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()),
    ).toBe("|  |  |  |\n| --- | --- | --- |\n|  |  |  |\n|  |  |  |\n");
  });

  it("writes a link as its text in brackets and the address in parentheses", () => {
    expect(
      formatted("hello", (e) => e.chain().focus().selectAll().setLink({ href: "https://example.com" }).run()),
    ).toBe("[hello](https://example.com)");
  });

  it("writes an image as its alt text and its address", () => {
    expect(
      formatted("", (e) => e.chain().focus().setImage({ src: "https://example.com/x.png", alt: "a shot" }).run()),
    ).toBe("![a shot](https://example.com/x.png)");
  });
});

describe("loading a full document back out unchanged", () => {
  const full = [
    "# Title",
    "",
    'Some **bold**, *italic*, ~~struck~~ and `code` text with a [link](https://x.example "tip").',
    "",
    "> a quote",
    "",
    "```js",
    "const x = 1;",
    "```",
    "",
    "- one",
    "- two",
    "",
    "1. first",
    "2. second",
    "",
    "---",
    "",
    "| a | b |",
    "| --- | --- |",
    "| 1 | 2 |",
    "",
  ].join("\n");

  it("comes back byte for byte the same on the very first save", () => {
    expect(roundtripped(full)).toBe(full);
  });

  it("stays exactly the same through a second load and save", () => {
    const once = roundtripped(full);
    expect(roundtripped(once)).toBe(once);
  });
});

describe("text that only looks like markdown syntax", () => {
  it("keeps ** on the wire, even though it now reads as real bold rather than literal asterisks", () => {
    expect(roundtripped("**not bold**")).toBe("**not bold**");
  });

  it("keeps ~~ on the wire, even though it now reads as a real strikethrough", () => {
    expect(roundtripped("~~not struck~~")).toBe("~~not struck~~");
  });

  it("keeps a backtick pair on the wire, even though it now reads as real inline code", () => {
    expect(roundtripped("`not code`")).toBe("`not code`");
  });

  it("leaves a pipe-separated line alone when there is no header separator to make it a table", () => {
    expect(roundtripped("a | b | c")).toBe("a | b | c");
  });

  it("keeps a leading # on the wire, even though it now reads as a real heading", () => {
    expect(roundtripped("# not a heading, just text")).toBe("# not a heading, just text");
  });

  it("keeps a standalone --- on the wire, even though it now reads as a real divider", () => {
    expect(roundtripped("---")).toBe("---");
  });
});

describe("content that gets rewritten the first time it is saved, then holds still", () => {
  it("leaves html a person wrote in their own file alone", () => {
    const first = roundtripped("<div>hi</div>");
    expect(first).not.toContain("&lt;");
    expect(roundtripped(first)).toBe(first);
  });

  it("escapes a footnote-shaped [^1] so the brackets cannot be mistaken for a link", () => {
    const first = roundtripped("footnote[^1]");
    expect(first).toBe("footnote\\[^1\\]");
    expect(roundtripped(first)).toBe(first);
  });

  it("wraps a bare autolinked URL in angle brackets", () => {
    const first = roundtripped("visit https://example.com now");
    expect(first).toBe("visit <https://example.com> now");
    expect(roundtripped(first)).toBe(first);
  });
});

describe("a task list stays as tight as it was written", () => {
  it("keeps a tight task list tight, the same way a plain bulleted list does", () => {
    expect(roundtripped("- [x] done\n- [ ] pending\n")).toBe("- [x] done\n- [ ] pending");
  });

  it("keeps it whole, which is what lets a merge treat it as one block", () => {
    const said = roundtripped("- [x] done\n- [ ] pending\n- [ ] third\n");

    expect(said).not.toMatch(/\n\n/);
    expect(roundtripped(said)).toBe(said);
  });

  it("does not have the same problem with a plain bulleted list", () => {
    expect(roundtripped("- one\n- two\n")).toBe("- one\n- two");
  });

  it("does not have the same problem with a numbered list either", () => {
    expect(roundtripped("1. one\n2. two\n")).toBe("1. one\n2. two");
  });
});

describe("known bug: a block image swallows the blank line that follows it", () => {
  it("keeps a blank line between an image and the paragraph after it", () => {
    expect(roundtripped("![alt](https://x.example/pic.png)\n\nSome text after.")).toBe(
      "![alt](https://x.example/pic.png)\n\nSome text after.",
    );
  });

  it("leaves the paragraph after an image on its own line", () => {
    expect(roundtripped("![alt](https://x.example/pic.png)\n\nSome text after.")).toBe(
      "![alt](https://x.example/pic.png)\n\nSome text after.",
    );
  });

  it("leaves a heading after an image still a heading", () => {
    expect(roundtripped("![alt](https://x.example/pic.png)\n\n# Heading after")).toContain(
      "\n\n# Heading after",
    );
  });

  it("leaves a table after an image separated from it", () => {
    expect(
      roundtripped("![alt](https://x.example/pic.png)\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n"),
    ).toContain("\n\n| a | b |");
  });

  it("keeps that table a table however many times the document is opened", () => {
    const first = roundtripped(
      "![alt](https://x.example/pic.png)\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n",
    );
    const reloaded = build(first);
    const stillHasATable = JSON.stringify(reloaded.getJSON()).includes('"type":"table"');
    reloaded.destroy();
    expect(stillHasATable).toBe(true);
  });

  it("does not happen when the image comes after the text instead of before it", () => {
    expect(roundtripped("Some text before.\n\n![alt](https://x.example/pic.png)")).toBe(
      "Some text before.\n\n![alt](https://x.example/pic.png)",
    );
  });
});

describe("what plain markdown cannot represent", () => {
  it("keeps underline, which Markdown has no syntax for, as inline html", () => {
    const editor = build("hello");
    editor.chain().focus().selectAll().toggleUnderline().run();

    expect(editor.isActive("underline")).toBe(true);
    expect(markdown(editor)).toBe("<u>hello</u>");
    editor.destroy();
  });

  it("reads that underline back as a mark instead of as letters", () => {
    const editor = build("<u>hello</u>");

    expect(editor.getHTML()).toContain("<u>hello</u>");
    expect(editor.getText()).toBe("hello");
    editor.destroy();
  });
});

describe("asked(): edge cases beyond what slash.test.tsx already covers", () => {
  it("finds the trigger after the last slash, ignoring an earlier one used as a path separator", () => {
    expect(asked("visita c:/temp y luego /tab")).toBe("tab");
  });

  it("accepts a tab, not only a space, as the whitespace before the trigger", () => {
    expect(asked("hola\t/tab")).toBe("tab");
  });

  it("keeps filtering through accented letters, which Spanish needs", () => {
    expect(asked("bloque de /cód")).toBe("cód");
    expect(asked("/tí")).toBe("tí");
    expect(asked("/viñ")).toBe("viñ");
  });

  it("stays shut on a bare double slash", () => {
    expect(asked("//")).toBeNull();
  });
});

describe("narrowed(): edge cases beyond what slash.test.tsx already covers", () => {
  const blocks = [{ label: "Table" }, { label: "Task list" }];

  it("can keep more than one block when the word matches several labels", () => {
    expect(narrowed(blocks, "ta")).toEqual(blocks);
  });

  it("returns nothing rather than throwing when there are no blocks to search", () => {
    expect(narrowed([], "tab")).toEqual([]);
  });
});

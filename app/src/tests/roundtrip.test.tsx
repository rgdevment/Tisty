import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { Link } from "@tiptap/extension-link";
import { Image } from "@tiptap/extension-image";
import { Table, TableRow, TableCell, TableHeader } from "@tiptap/extension-table";
import { TaskList } from "@tiptap/extension-task-list";
import { TaskItem } from "@tiptap/extension-task-item";
import { Markdown } from "tiptap-markdown";
import { asked, narrowed } from "../ui/Slash";

function build(content = ""): Editor {
  return new Editor({
    extensions: [
      StarterKit,
      Link.configure({ openOnClick: false, autolink: true }),
      Image,
      Table.configure({ resizable: false }),
      TableRow,
      TableHeader,
      TableCell,
      TaskList,
      TaskItem.configure({ nested: true }),
      Markdown.configure({ html: true, linkify: true, breaks: true, transformPastedText: false }),
    ],
    content,
  });
}

function markdown(editor: Editor): string {
  return (editor.storage as unknown as { markdown: { getMarkdown: () => string } }).markdown.getMarkdown();
}

function formatted(content: string, run: (editor: Editor) => void): string {
  const editor = build(content);
  run(editor);
  const out = markdown(editor);
  editor.destroy();
  return out;
}

function roundtripped(content: string): string {
  return formatted(content, () => {});
}

describe("keeping this file honest about matching Editor.tsx", () => {
  it("still configures the real editor with the same Markdown options this suite assumes", () => {
    const source = readFileSync("src/ui/Editor.tsx", "utf8");
    expect(source).toContain(
      "Markdown.configure({ html: true, linkify: true, breaks: true, transformPastedText: false })",
    );
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

describe("known bug: a task list forgets it was written tight", () => {
  it.fails("keeps a tight task list tight, the same way a plain bulleted list does", () => {
    expect(roundtripped("- [x] done\n- [ ] pending\n")).toBe("- [x] done\n- [ ] pending");
  });

  it("currently inserts a blank line between every task, even when none was there", () => {
    const first = roundtripped("- [x] done\n- [ ] pending\n");
    expect(first).toBe("- [x] done\n\n- [ ] pending");
    expect(roundtripped(first)).toBe(first);
  });

  it("does not have the same problem with a plain bulleted list", () => {
    expect(roundtripped("- one\n- two\n")).toBe("- one\n- two");
  });

  it("does not have the same problem with a numbered list either", () => {
    expect(roundtripped("1. one\n2. two\n")).toBe("1. one\n2. two");
  });
});

describe("known bug: a block image swallows the blank line that follows it", () => {
  it.fails("keeps a blank line between an image and the paragraph after it", () => {
    expect(roundtripped("![alt](https://x.example/pic.png)\n\nSome text after.")).toBe(
      "![alt](https://x.example/pic.png)\n\nSome text after.",
    );
  });

  it("currently glues the next paragraph onto the same line as the image", () => {
    expect(roundtripped("![alt](https://x.example/pic.png)\n\nSome text after.")).toBe(
      "![alt](https://x.example/pic.png)Some text after.",
    );
  });

  it("glues a heading onto the image just the same", () => {
    expect(roundtripped("![alt](https://x.example/pic.png)\n\n# Heading after")).toBe(
      "![alt](https://x.example/pic.png)# Heading after",
    );
  });

  it("glues a table onto the image with no separation at all", () => {
    expect(
      roundtripped("![alt](https://x.example/pic.png)\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n"),
    ).toBe("![alt](https://x.example/pic.png)| a | b |\n| --- | --- |\n| 1 | 2 |\n");
  });

  it("destroys the table structure entirely by the second time the document is opened", () => {
    const first = roundtripped(
      "![alt](https://x.example/pic.png)\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n",
    );
    const reloaded = build(first);
    const stillHasATable = JSON.stringify(reloaded.getJSON()).includes('"type":"table"');
    reloaded.destroy();
    expect(stillHasATable).toBe(false);
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

import { Editor } from "@tiptap/core";
import { describe, expect, it } from "vitest";
import { asked, narrowed } from "../ui/Slash";
import { asMarkdown, written } from "../ui/writing";

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
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleBold().run())).toBe(
      "**hello**",
    );
  });

  it("wraps the selection in * for italic", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleItalic().run())).toBe(
      "*hello*",
    );
  });

  it("wraps the selection in ~~ for strikethrough", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleStrike().run())).toBe(
      "~~hello~~",
    );
  });

  it("wraps the selection in a single backtick for inline code", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleCode().run())).toBe(
      "`hello`",
    );
  });

  it("fences the paragraph off for a code block", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleCodeBlock().run())).toBe(
      "```\nhello\n```",
    );
  });

  it("prefixes the line with > for a quote", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleBlockquote().run())).toBe(
      "> hello",
    );
  });

  it("prefixes the line with - for a bulleted list", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleBulletList().run())).toBe(
      "- hello",
    );
  });

  it("prefixes the line with 1. for a numbered list", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleOrderedList().run())).toBe(
      "1. hello",
    );
  });

  it("prefixes the line with an empty checkbox for a task list", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleTaskList().run())).toBe(
      "- [ ] hello",
    );
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
    expect(formatted("hello", (e) => e.chain().focus("end").setHorizontalRule().run())).toBe(
      "hello\n\n---",
    );
  });

  it("writes a fresh table as a header row, its separator, and two empty body rows", () => {
    expect(
      formatted("", (e) =>
        e.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
      ),
    ).toBe("|  |  |  |\n| --- | --- | --- |\n|  |  |  |\n|  |  |  |\n");
  });

  it("writes a link as its text in brackets and the address in parentheses", () => {
    expect(
      formatted("hello", (e) =>
        e.chain().focus().selectAll().setLink({ href: "https://example.com" }).run(),
      ),
    ).toBe("[hello](https://example.com)");
  });

  it("writes an image as its alt text and its address", () => {
    expect(
      formatted("", (e) =>
        e.chain().focus().setImage({ src: "https://example.com/x.png", alt: "a shot" }).run(),
      ),
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

describe("the highlighter", () => {
  it("writes == around the selection", () => {
    expect(formatted("hello", (e) => e.chain().focus().selectAll().toggleHighlight().run())).toBe(
      "==hello==",
    );
  });

  it("reads == back as a highlight", () => {
    expect(roundtripped("say ==this== out loud")).toBe("say ==this== out loud");
  });

  it("keeps a coloured pen as an html mark", () => {
    expect(
      formatted("hello", (e) =>
        e.chain().focus().selectAll().toggleHighlight({ color: "green" }).run(),
      ),
    ).toBe('<mark data-pen="green">hello</mark>');
  });

  it("reads a coloured pen back", () => {
    expect(roundtripped('<mark data-pen="green">hello</mark>')).toBe(
      '<mark data-pen="green">hello</mark>',
    );
  });

  it("leaves bold inside a highlight alone", () => {
    expect(roundtripped("==a **strong** word==")).toBe("==a **strong** word==");
  });
});

describe("a paragraph that was centred before Tisty stopped centring", () => {
  it("comes back as a plain paragraph, and what it points at as a plain link", () => {
    const once =
      '<p style="text-align: center">mira <a href="attachments/ab/plano-1234.pdf">el plano</a></p>';

    expect(roundtripped(once)).toBe("mira [el plano](attachments/ab/plano-1234.pdf)");
  });

  it("keeps its bold, its highlight and its icon rather than dropping them", () => {
    const once =
      '<p style="text-align: center">a <strong>strong</strong> word and <mark>that</mark></p>';

    expect(roundtripped(once)).toBe("a **strong** word and ==that==");
  });

  it("keeps a line break inside it, which the html never did", () => {
    const said = roundtripped('<p style="text-align: center">uno<br>dos</p>');

    expect(said).not.toBe("unodos");
    expect(said.replace(/\\?\n/g, "|")).toBe("uno|dos");
  });
});

describe("the guide's own markup", () => {
  it("keeps a coloured pen inside a table cell", () => {
    const row = '| <mark data-pen="blue">tomorrow 10am</mark> | a day |';
    const once = `| You write | It understands |\n| --- | --- |\n${row}`;

    expect(roundtripped(once)).toContain('<mark data-pen="blue">tomorrow 10am</mark>');
  });

  it("shows the pen as a mark when read", () => {
    const editor = build('a <mark data-pen="green">green</mark> word');
    const html = editor.getHTML();
    editor.destroy();

    expect(html).toContain('data-pen="green"');
  });
});

describe("the highlighter stops where it is told", () => {
  it("does not spread into what is typed next", () => {
    const editor = build("hello");
    editor.chain().focus().selectAll().toggleHighlight().run();
    editor.commands.setTextSelection(editor.state.doc.content.size - 1);
    editor.commands.insertContent(" world");
    const out = markdown(editor);
    editor.destroy();

    expect(out).toBe("==hello== world");
  });

  it("comes off again from inside the word, without selecting it", () => {
    const editor = build("say ==this== out loud");
    editor.commands.setTextSelection(6);
    editor.chain().focus().extendMarkRange("highlight").toggleHighlight().run();
    const out = markdown(editor);
    editor.destroy();

    expect(out).toBe("say this out loud");
  });
});

describe("a callout is a quote GitHub reads, and comes back as it went", () => {
  const KINDS = ["NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"];

  it.each(KINDS)("keeps a %s exactly as written", (kind) => {
    const was = `> [!${kind}]\n> Cuidado con esto.`;
    expect(roundtripped(was)).toBe(was);
  });

  it("does not escape the marker, which is what broke it before", () => {
    expect(roundtripped("> [!WARNING]\n> Algo.")).not.toContain("\\[");
  });

  it("does not put a hard break after the marker", () => {
    expect(roundtripped("> [!NOTE]\n> Algo.")).not.toContain("\\\n");
  });

  it("leaves a quote with no marker a quote", () => {
    const was = "> Una cita normal, sin marcador.";
    expect(roundtripped(was)).toBe(was);
  });

  it("keeps what the callout holds, marks and all", () => {
    const was = "> [!TIP]\n> Con **negrita** y `código`.";
    expect(roundtripped(was)).toBe(was);
  });

  it("keeps more than one paragraph inside", () => {
    const was = "> [!CAUTION]\n> Primera.\n>\n> Segunda.";
    expect(roundtripped(was)).toBe(was);
  });

  it("reads the marker whatever case it is written in", () => {
    expect(roundtripped("> [!warning]\n> Algo.")).toBe("> [!WARNING]\n> Algo.");
  });

  it("is a fixed point: writing it twice changes nothing", () => {
    const once = roundtripped("> [!NOTE]\n> Algo.");
    expect(roundtripped(once)).toBe(once);
  });
});

describe("a table keeps the alignment its columns were given", () => {
  it("keeps a column leaning right", () => {
    expect(roundtripped("| a | b |\n| --- | ---: |\n| 1 | 2 |")).toBe(
      "| a | b |\n| --- | ---: |\n| 1 | 2 |\n",
    );
  });

  it("keeps left, centre and right at once", () => {
    expect(roundtripped("| a | b | c |\n| :--- | :---: | ---: |\n| 1 | 2 | 3 |")).toBe(
      "| a | b | c |\n| :--- | :---: | ---: |\n| 1 | 2 | 3 |\n",
    );
  });

  it("leaves a table with no alignment plain", () => {
    expect(roundtripped("| a | b |\n| --- | --- |\n| 1 | 2 |")).toBe(
      "| a | b |\n| --- | --- |\n| 1 | 2 |\n",
    );
  });

  it("is a fixed point: writing it twice changes nothing", () => {
    const once = roundtripped("| a | b |\n| :---: | ---: |\n| 1 | 2 |");
    expect(roundtripped(once)).toBe(once);
  });
});

describe("a code block keeps its language, now that it can be given one", () => {
  it.each(["rust", "js", "python", "sql", "mermaid"])("keeps %s on the fence", (tongue) => {
    const was = `\`\`\`${tongue}\nalgo\n\`\`\``;
    expect(roundtripped(was)).toBe(was);
  });

  it("leaves a fence with no language alone", () => {
    const was = "```\nsin lenguaje\n```";
    expect(roundtripped(was)).toBe(was);
  });

  it("keeps what is inside untouched, blank lines and all", () => {
    const was = "```mermaid\ngraph TD;\n\nA[Inicio] --> B{Sigue?}\n```";
    expect(roundtripped(was)).toBe(was);
  });

  it("is a fixed point", () => {
    const once = roundtripped("```rust\nfn main() {}\n```");
    expect(roundtripped(once)).toBe(once);
  });
});

describe("a mermaid diagram is a code block, so the file never learns about it", () => {
  it("comes back with its fence and its language", () => {
    const was = "```mermaid\ngraph TD;\nA[Inicio] --> B{Sigue?}\n```";
    expect(roundtripped(was)).toBe(was);
  });

  it("keeps the blank lines inside, which mermaid reads as separators", () => {
    const was = "```mermaid\nsequenceDiagram\n\nA->>B: hola\n```";
    expect(roundtripped(was)).toBe(was);
  });

  it("keeps a diagram that does not parse, because the file is not ours to fix", () => {
    const was = "```mermaid\nesto no compila {{{\n```";
    expect(roundtripped(was)).toBe(was);
  });
});

describe("typing a callout by hand, which is how one actually gets written", () => {
  const typed = (editor: Editor, text: string) => {
    for (const one of text) {
      const { from, to } = editor.state.selection;
      const took = editor.view.someProp("handleTextInput", (fn) =>
        fn(editor.view, from, to, one, () => editor.state.tr),
      );
      if (!took) editor.view.dispatch(editor.state.tr.insertText(one, from, to));
    }
  };

  const written_ = (text: string) => {
    const editor = build("");
    editor.chain().focus().toggleBlockquote().run();
    typed(editor, text);
    const out = markdown(editor);
    editor.destroy();
    return out;
  };

  it.each(["NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"])("makes a %s as it is typed", (k) => {
    expect(written_(`[!${k}] Cuidado`)).toBe(`> [!${k}]\n> Cuidado`);
  });

  it("does not escape the marker, which is what typing used to do", () => {
    expect(written_("[!WARNING] Algo")).not.toContain("\\[");
  });

  it("reads it typed in lower case", () => {
    expect(written_("[!tip] Algo")).toBe("> [!TIP]\n> Algo");
  });

  it("leaves a marker it does not know as the text it is", () => {
    expect(written_("[!raro] Algo")).toContain("raro");
  });

  it("leaves a plain quote alone", () => {
    expect(written_("Una cita normal")).toBe("> Una cita normal");
  });
});

describe("a table holds what markdown can hold, and keeps the rest whole", () => {
  it("keeps a picture in a cell instead of emptying it", () => {
    const was =
      "| build | cover |\n| --- | --- |\n| ![b](https://x/b.svg) | ![c](https://x/c.svg) |\n";
    expect(roundtripped(was)).toBe(was);
  });

  it("is a fixed point with a picture in a cell", () => {
    const once = roundtripped("| a |\n| --- |\n| ![b](https://x/b.svg) |");
    expect(roundtripped(once)).toBe(once);
  });
});

describe("a callout is the quote's own first line, never one it holds", () => {
  it("leaves a marker inside a list inside a quote as the text it is", () => {
    const was = "> - [!NOTE] item\n>\n> - dos";
    expect(roundtripped(was)).toContain("item");
    expect(roundtripped(roundtripped(was))).toBe(roundtripped(was));
  });

  it("leaves a nested callout at the depth it was written", () => {
    const was = "> > [!NOTE]\n> > dentro";
    expect(roundtripped(was)).toBe(was);
  });

  it("does not empty a nested quote", () => {
    const was = "> > [!NOTE]\n>\n> texto";
    expect(roundtripped(was)).toBe(was);
  });
});

describe("a cell holds what markdown can hold, and says so when it cannot", () => {
  const inACell = (kid: unknown) => ({
    type: "doc",
    content: [
      {
        type: "table",
        content: [
          {
            type: "tableRow",
            content: [
              {
                type: "tableHeader",
                content: [{ type: "paragraph", content: [{ type: "text", text: "a" }] }],
              },
            ],
          },
          { type: "tableRow", content: [{ type: "tableCell", content: [kid] }] },
        ],
      },
    ],
  });

  const written_ = (doc: unknown) => {
    const editor = new Editor({ extensions: written(), content: doc as never });
    const out = markdown(editor);
    editor.destroy();
    return out;
  };

  it("writes a picture in a cell as markdown", () => {
    const out = written_(inACell({ type: "image", attrs: { src: "shot.png", alt: "foto" } }));
    expect(out).toContain("![foto](shot.png)");
    expect(out).not.toContain("<table");
  });

  it.each([
    ["a rule", { type: "horizontalRule" }],
    ["a heading", { type: "heading", attrs: { level: 2 }, content: [{ type: "text", text: "t" }] }],
    ["a code block", { type: "codeBlock", content: [{ type: "text", text: "a | b" }] }],
    [
      "a quote",
      {
        type: "blockquote",
        content: [{ type: "paragraph", content: [{ type: "text", text: "c" }] }],
      },
    ],
  ])("keeps %s as html rather than losing it", (_name, kid) => {
    const out = written_(inACell(kid));
    expect(out).toContain("<table");
    expect(written_(new Editor({ extensions: written(), content: out }).state.doc.toJSON())).toBe(
      out,
    );
  });
});

describe("Enter is only held back inside a table cell", () => {
  const pressed = (content: string, at: number) => {
    const el = document.createElement("div");
    document.body.append(el);
    const editor = new Editor({ extensions: written(), content, element: el });
    editor.commands.setTextSelection(at);
    editor.view.dom.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true }),
    );
    const out = markdown(editor);
    editor.destroy();
    el.remove();
    return out;
  };

  it("splits a paragraph", () => {
    expect(pressed("hola mundo", 5)).toContain("\n\n");
  });

  it("splits a list item", () => {
    expect(pressed("- uno", 5)).toBe("- un\n- o");
  });

  it("splits a quote", () => {
    expect(pressed("> una cita", 6)).toContain(">\n>");
  });

  it("does nothing in a table cell, where markdown has no second paragraph", () => {
    const was = "| a | b |\n| --- | --- |\n| uno | dos |";
    expect(pressed(was, 14)).toBe(`${was}\n`);
  });
});

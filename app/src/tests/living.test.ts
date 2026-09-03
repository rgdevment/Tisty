import { describe, expect, it, vi } from "vitest";
import { frail } from "../frail";
import { SHAPES } from "../ui/Beside";
import { DRAWN, KINDS, loosened } from "../ui/writing";
import { inCell, opened } from "./mounted";

const sketch = vi.hoisted(() => ({ calls: [] as unknown[][] }));

vi.mock("mermaid", () => ({
  default: {
    initialize: () => {},
    render: (...args: unknown[]) => {
      sketch.calls.push(args);
      return Promise.resolve({ svg: "<svg data-drawn=\"yes\"></svg>" });
    },
  },
}));

describe("what only a mounted editor can be asked", () => {
  it("draws the code block's own frame, which no unmounted editor builds", () => {
    const one = opened("```rust\nfn main() {}\n```");
    expect(one.dom.querySelector(".lit")).toBeTruthy();
    expect(one.dom.querySelector(".lit pre code")).toBeTruthy();
    one.shut();
  });

  it("numbers as many lines as the code has, and keeps the numbers out of the text", () => {
    const one = opened("```js\nuno\ndos\ntres\n```");
    const lines = one.dom.querySelectorAll(".lit-lines span");
    expect(lines).toHaveLength(3);
    expect([...lines].map((said) => said.textContent)).toEqual(["1", "2", "3"]);
    expect(one.dom.querySelector(".lit pre code")?.textContent).not.toContain("1");
    one.shut();
  });

  it("counts the lines again when the code grows", () => {
    const one = opened("```js\nuno\n```");
    expect(one.dom.querySelectorAll(".lit-lines span")).toHaveLength(1);
    one.at(4);
    one.wrote("\ndos\ntres");
    expect(one.dom.querySelectorAll(".lit-lines span")).toHaveLength(3);
    one.shut();
  });

  it("offers the language the fence was written with, among the rest", () => {
    const one = opened("```rust\nfn main() {}\n```");
    const picked = one.dom.querySelector<HTMLSelectElement>(".lit-tongue");
    expect(picked?.value).toBe("rust");
    expect([...(picked?.options ?? [])].map((said) => said.value)).toContain("mermaid");
    one.shut();
  });

  it("shows a fence with no language as plain, not as a language it is not", () => {
    const one = opened("```\nalgo\n```");
    expect(one.dom.querySelector<HTMLSelectElement>(".lit-tongue")?.value).toBe("");
    one.shut();
  });

  it("shows the language the file names, even one it cannot colour", () => {
    const one = opened("```brainfuck\n+++\n```");
    expect(one.dom.querySelector<HTMLSelectElement>(".lit-tongue")?.value).toBe("brainfuck");
    expect(one.markdown()).toBe("```brainfuck\n+++\n```");
    one.shut();
  });

  it.each(["js", "ts", "py", "sh", "yml", "rs", "md"])(
    "shows %s as the fence wrote it, not as no language at all",
    (tongue) => {
      const one = opened(`\`\`\`${tongue}\nx\n\`\`\``);
      expect(one.dom.querySelector<HTMLSelectElement>(".lit-tongue")?.value).toBe(tongue);
      expect(one.markdown()).toBe(`\`\`\`${tongue}\nx\n\`\`\``);
      one.shut();
    },
  );

  it.each([
    ['```ts title="core.ts"', "core.ts", "ts"],
    ['```mermaid title="El plano"', "El plano", "mermaid"],
    ['``` title="sin lenguaje"', "sin lenguaje", null],
  ])("keeps the name a fence carries in %s", (fence, name, tongue) => {
    const was = `${fence}\nalgo\n\`\`\``;
    const one = opened(was);

    expect(one.editor.state.doc.firstChild?.attrs.title).toBe(name);
    expect(one.editor.state.doc.firstChild?.attrs.language).toBe(tongue);
    expect(one.markdown()).toBe(was);
    one.shut();
  });

  it("shows the name in the bar and leaves it empty when there is none", () => {
    const one = opened('```ts title="core.ts"\nalgo\n```');
    expect(one.dom.querySelector<HTMLInputElement>(".lit-name")?.value).toBe("core.ts");
    one.shut();

    const two = opened("```ts\nalgo\n```");
    expect(two.dom.querySelector<HTMLInputElement>(".lit-name")?.value).toBe("");
    expect(two.markdown()).toBe("```ts\nalgo\n```");
    two.shut();
  });

  it.each([
    ['dijo "hola"', '```ts title="dijo \\"hola\\""\nx\n```'],
    ["con | tubo", '```ts title="con | tubo"\nx\n```'],
    ["con = igual", '```ts title="con = igual"\nx\n```'],
  ])("keeps a name that says %s exactly as it was written", (name, was) => {
    const one = opened(was);

    expect(one.editor.state.doc.firstChild?.attrs.title).toBe(name);
    expect(one.markdown()).toBe(was);
    one.shut();
  });

  it("draws a formula from a block that says it holds one", async () => {
    const one = opened("```math\nE = mc^2\n```");
    await vi.waitFor(() => expect(one.dom.querySelector(".lit-drawn .katex")).toBeTruthy(), {
      timeout: 5000,
    });

    expect(one.markdown()).toBe("```math\nE = mc^2\n```");
    one.shut();
  });

  it("draws a diagram without handing mermaid the box it will be shown in", async () => {
    sketch.calls.length = 0;
    const one = opened("```mermaid\ngraph TD\n  A --> B\n```");
    await vi.waitFor(() => expect(sketch.calls).toHaveLength(1), { timeout: 5000 });

    expect(sketch.calls[0]).toHaveLength(2);
    one.shut();
  });

  it.each(["mermaid", "math"])("names %s rather than offering to change it", (tongue) => {
    const one = opened(`\`\`\`${tongue}\nx\n\`\`\``);

    expect(one.dom.querySelector<HTMLSelectElement>(".lit-tongue")?.hidden).toBe(true);
    expect(one.dom.querySelector<HTMLElement>(".lit-said")?.textContent).toBe(tongue);
    one.shut();
  });

  it("leaves a formula it cannot read as the text somebody wrote", async () => {
    const one = opened("```math\n\\frac{sin cerrar\n```");
    await new Promise((go) => setTimeout(go, 300));

    expect(one.markdown()).toBe("```math\n\\frac{sin cerrar\n```");
    one.shut();
  });

  it("keeps the line numbers out of what a person can type in", () => {
    const one = opened("```js\nuno\ndos\n```");
    const lines = one.dom.querySelector<HTMLElement>(".lit-lines");

    expect(lines?.getAttribute("contenteditable")).toBe("false");
    expect(one.editor.state.doc.textContent).toBe("uno\ndos");
    one.shut();
  });

  it("names what it draws instead of offering to make it another language", () => {
    const one = opened("```mermaid\ngraph TD;\nA --> B;\n```");
    const picked = one.dom.querySelector<HTMLSelectElement>(".lit-tongue");
    const said = one.dom.querySelector<HTMLElement>(".lit-said");

    expect(picked?.hidden).toBe(true);
    expect(said?.hidden).toBe(false);
    expect(said?.textContent).toBe("mermaid");
    one.shut();
  });

  it("offers the language again on a block that draws nothing", () => {
    const one = opened("```js\nconst x = 1;\n```");

    expect(one.dom.querySelector<HTMLSelectElement>(".lit-tongue")?.hidden).toBe(false);
    expect(one.dom.querySelector<HTMLElement>(".lit-said")?.hidden).toBe(true);
    one.shut();
  });

  it("lets go of a diagram it drew, so turning the theme never wakes a dead block", async () => {
    const one = opened("```mermaid\ngraph TD;\nA --> B;\n```");
    one.shut();

    document.documentElement.setAttribute("data-theme", "dark");
    await Promise.resolve();
    document.documentElement.removeAttribute("data-theme");

    expect(true).toBe(true);
  });

  it("keeps the diagram's frame out of the way when there is no diagram", () => {
    const one = opened("```js\nconst x = 1;\n```");
    expect(one.dom.querySelector(".lit-drawn")?.childElementCount).toBe(0);
    one.shut();
  });
});

describe("a fence line that is not a fence is not taken for one", () => {
  const BT = String.fromCharCode(96);

  it("says a backtick fence whose name holds a backtick will not survive", () => {
    const was = `${BT.repeat(3)}js title="a${BT}b"
algo
${BT.repeat(3)}`;

    expect(frail(was)).toContain("frailFence");
    const one = opened(was);
    expect(one.markdown()).not.toBe(was);
    one.shut();
  });

  it("leaves the same name alone on a tilde fence, which markdown allows", () => {
    const was = `~~~js title="a${BT}b"
algo
~~~`;

    expect(frail(was)).toEqual([]);
    const one = opened(was);
    expect(one.markdown()).toBe(was);
    one.shut();
  });

  it("says nothing about a fence with no backtick in what it says", () => {
    const was = `${BT.repeat(3)}js title="normal"
algo
${BT.repeat(3)}`;

    expect(frail(was)).toEqual([]);
    const one = opened(was);
    expect(one.markdown()).toBe(was);
    one.shut();
  });
});

describe("a cell keeps the bar somebody escaped into it", () => {
  const BAR = String.fromCharCode(92);

  it("reads it as one cell and writes it back the same", () => {
    const was = `| a | b |\n| --- | --- |\n| uno ${BAR}| dos | tres |`;
    const one = opened(was);

    expect(one.editor.state.doc.firstChild?.child(1).child(0).textContent).toBe("uno | dos");
    expect(one.markdown()).toBe(`${was}\n`);
    one.shut();
  });

  it("gains no backslash however many times it is saved", () => {
    let body = `| a | b |\n| --- | --- |\n| uno ${BAR}| dos | tres |`;
    for (let round = 0; round < 3; round += 1) {
      const one = opened(body);
      body = one.markdown();
      one.shut();
    }

    expect(body).toBe(`| a | b |\n| --- | --- |\n| uno ${BAR}| dos | tres |\n`);
  });

  it("still escapes what markdown would otherwise read as marks", () => {
    const one = opened("| a |\n| --- |\n| con \\*asterisco\\* |");

    expect(one.editor.state.doc.firstChild?.child(1).child(0).textContent).toBe("con *asterisco*");
    one.shut();
  });
});

describe("a list that mixes bullets and tasks is parted, not haunted", () => {
  const kindsOf = (one: ReturnType<typeof opened>) => {
    const said: string[] = [];
    one.editor.state.doc.forEach((node) => {
      said.push(node.type.name);
    });
    return said;
  };

  it.each([
    ["- uno\n- [x] hecho", ["bulletList", "taskList"]],
    ["- [x] hecho\n- uno", ["taskList", "bulletList"]],
    ["- uno\n- [x] hecho\n- dos", ["bulletList", "taskList", "bulletList"]],
    ["- [ ] a\n- [x] b", ["taskList"]],
    ["- uno\n- dos", ["bulletList"]],
  ])("reads %j as the blocks it really is", (was, want) => {
    const one = opened(was);
    expect(kindsOf(one)).toEqual(want);
    one.shut();
  });

  it("settles instead of growing a new empty task on every save", () => {
    let body = "- uno\n\n\n- [x] hecho";
    const seen: string[] = [];
    for (let round = 0; round < 3; round += 1) {
      const one = opened(body);
      body = one.markdown();
      one.shut();
      seen.push(body);
    }

    expect(seen[0]).toBe(seen[1]);
    expect(seen[1]).toBe(seen[2]);
    expect(body).not.toContain("- [ ] \n");
  });
});

describe("a pen and an aside keep what they say in the file", () => {
  it.each(["green", "blue", "pink"])("writes a %s pen as markdown the editor reads back", (pen) => {
    const one = opened("");
    one.editor.chain().focus().insertContent("resaltado").run();
    one.editor
      .chain()
      .focus()
      .setTextSelection({ from: 1, to: 10 })
      .setHighlight({ color: pen })
      .run();
    const out = one.markdown();
    one.shut();

    expect(out).toContain(`data-pen="${pen}"`);
    expect(frail(out)).toEqual([]);
    expect(opened(out).editor.state.doc.textContent).toBe("resaltado");
  });

  it("writes the yellow pen as plain markdown, with no html at all", () => {
    const one = opened("");
    one.editor.chain().focus().insertContent("resaltado").run();
    one.editor.chain().focus().setTextSelection({ from: 1, to: 10 }).toggleHighlight().run();

    expect(one.markdown()).toBe("==resaltado==");
    one.shut();
  });

  it.each(["tip", "important", "warning", "caution"])("turns an aside into a %s", (kind) => {
    const one = opened("> [!NOTE]\n> algo");
    one.editor.chain().focus().updateAttributes("callout", { kind }).run();

    expect(one.markdown()).toBe(`> [!${kind.toUpperCase()}]\n> algo`);
    expect(frail(one.markdown())).toEqual([]);
    one.shut();
  });
});

describe("a block that draws is offered where every other block is", () => {
  it.each(DRAWN)("has %s in the column beside the document", (tongue) => {
    expect(SHAPES).toContain(tongue);
  });

  it.each(KINDS)("shapes an aside that is a %s, rather than inserting one", (kind) => {
    expect(SHAPES).toContain(`callout-${kind}`);
  });

  it("takes ```mmd and writes the fence out as mermaid, which github draws", () => {
    const one = opened("");
    one.typed("```mmd ");

    expect(one.editor.state.doc.firstChild?.attrs.language).toBe("mermaid");
    expect(one.markdown()).toBe("```mermaid\n```");
    one.shut();
  });

  it("needs no short way to say math, which is already short", () => {
    const one = opened("");
    one.typed("```math ");

    expect(one.editor.state.doc.firstChild?.attrs.language).toBe("math");
    one.shut();
  });
});

describe("a table markdown can hold is never written as html", () => {
  it("keeps the table when the header row is taken away", () => {
    const one = opened("| a | b |\n| --- | --- |\n| uno | dos |\n| tres | cuatro |");
    inCell(one, 0, 0);
    one.editor.chain().focus().deleteRow().run();

    expect(one.markdown()).toBe("| uno | dos |\n| --- | --- |\n| tres | cuatro |\n");
    expect(frail(one.markdown())).toEqual([]);
    one.shut();
  });

  it("carries a column's width in how long its rule is drawn", () => {
    const one = opened("| a | b |\n| -------- | --- |\n| uno | dos |");
    const head = one.editor.state.doc.firstChild?.firstChild;

    expect(head?.child(0).attrs.colwidth).toEqual([80]);
    expect(head?.child(1).attrs.colwidth).toBeNull();
    expect(one.markdown()).toBe("| a | b |\n| -------- | --- |\n| uno | dos |\n");
    one.shut();
  });

  it("leaves a plain rule plain, so a table nobody sized never changes", () => {
    const was = "| a | b |\n| --- | --- |\n| uno | dos |";
    const one = opened(was);

    expect(one.editor.state.doc.firstChild?.firstChild?.child(0).attrs.colwidth).toBeNull();
    expect(one.markdown()).toBe(`${was}\n`);
    one.shut();
  });

  it("keeps the width and the leaning in the same rule", () => {
    const one = opened("| a | b |\n| :------: | ---: |\n| uno | dos |");

    expect(one.markdown()).toBe("| a | b |\n| :------: | ---: |\n| uno | dos |\n");
    one.shut();
  });

  it("keeps a table somebody built with no header at all", () => {
    const one = opened("");
    one.editor.chain().focus().insertTable({ rows: 2, cols: 2, withHeaderRow: false }).run();

    expect(one.markdown()).not.toContain("<table");
    expect(frail(one.markdown())).toEqual([]);
    one.shut();
  });
});

describe("the keys a person actually presses", () => {
  it("splits a paragraph on Enter", () => {
    const one = opened("hola mundo");
    one.at(5);
    one.pressed("Enter");
    expect(one.markdown()).toContain("\n\n");
    one.shut();
  });

  it("splits a list item on Enter", () => {
    const one = opened("- uno");
    one.at(5);
    one.pressed("Enter");
    expect(one.markdown()).toBe("- un\n- o");
    one.shut();
  });

  it("splits a quote on Enter", () => {
    const one = opened("> una cita");
    one.at(6);
    one.pressed("Enter");
    expect(one.markdown()).toContain(">\n>");
    one.shut();
  });

  it("splits a callout on Enter, which is a quote like any other", () => {
    const one = opened("> [!NOTE]\n> una nota");
    one.at(6);
    one.pressed("Enter");
    expect(one.markdown()).toContain("> [!NOTE]");
    expect(one.markdown()).toContain(">\n>");
    one.shut();
  });

  it("holds Enter back in a table cell, where markdown has no second paragraph", () => {
    const was = "| a | b |\n| --- | --- |\n| uno | dos |";
    const one = opened(was);
    inCell(one, 1, 0);
    one.pressed("Enter");
    expect(one.markdown()).toBe(`${was}\n`);
    one.shut();
  });

  it("still writes a line inside a code block on Enter", () => {
    const one = opened("```js\nuno\n```");
    one.at(4);
    one.pressed("Enter");
    expect(one.editor.state.doc.textContent).toBe("uno\n");
    one.wrote("dos");
    expect(one.markdown()).toBe("```js\nuno\ndos\n```");
    one.shut();
  });

  it("turns a quote into a callout as the marker is typed", () => {
    const one = opened("");
    one.editor.chain().focus().toggleBlockquote().run();
    one.typed("[!WARNING] Cuidado");
    expect(one.markdown()).toBe("> [!WARNING]\n> Cuidado");
    one.shut();
  });

  it("leaves a marker it does not know as the text a person typed", () => {
    const one = opened("");
    one.editor.chain().focus().toggleBlockquote().run();
    one.typed("[!raro] algo");
    expect(one.markdown()).toContain("raro");
    expect(one.markdown()).not.toContain("[!raro]\n");
    one.shut();
  });
});

describe("a rule under the marker does not swallow the callout", () => {
  it("keeps the callout when the rule is the whole body", () => {
    const one = opened("> [!NOTE]\n> ---");
    expect(one.editor.state.doc.firstChild?.type.name).toBe("callout");
    expect(one.markdown()).toBe("> [!NOTE]\n> ---");
    one.shut();
  });

  it("keeps the marker and the words when a rule follows them", () => {
    const one = opened("> [!WARNING]\n> algo importante\n> ---");
    expect(one.editor.state.doc.firstChild?.attrs.kind).toBe("warning");
    expect(one.markdown()).toBe("> [!WARNING]\n> ## algo importante");
    one.shut();
  });

  it("underlines only the words the rule stood under, however many they were", () => {
    const one = opened("> [!WARNING]\n> uno\n> dos\n> ---");
    expect(one.editor.state.doc.firstChild?.attrs.kind).toBe("warning");
    expect(one.markdown()).toBe("> [!WARNING]\n> ## uno\\\n> dos");
    one.shut();
  });

  it("keeps it when the marker shares its line with the words", () => {
    const one = opened("> [!TIP] algo\n> ---");
    expect(one.editor.state.doc.firstChild?.attrs.kind).toBe("tip");
    expect(one.markdown()).toBe("> [!TIP]\n> algo\n>\n> ---");
    one.shut();
  });

  it("keeps it when the rule underneath is the other kind", () => {
    const one = opened("> [!TIP]\n> ===");
    expect(one.editor.state.doc.firstChild?.type.name).toBe("callout");
    expect(one.markdown()).toBe("> [!TIP]\n> ===");
    one.shut();
  });

  it("leaves a quote that is not a callout to commonmark", () => {
    const one = opened("> normal\n> ---");
    expect(one.markdown()).toBe("> ## normal");
    one.shut();
  });

  it("adds nothing to a callout with no rule under it", () => {
    const was = "> [!NOTE]\n> una nota";
    expect(opened(was).markdown()).toBe(was);
  });

  it("keeps a callout nested in a list where it was indented", () => {
    const one = opened("- uno\n\n  > [!NOTE]\n  > algo\n  > ---");
    expect(one.markdown()).toContain("- uno");
    expect(one.markdown()).toContain("[!NOTE]");
    expect(one.markdown()).toContain("algo");
    one.shut();
  });

  it("keeps a callout quoted inside another quote", () => {
    const one = opened("> > [!NOTE]\n> > ---");
    expect(one.markdown()).toContain("[!NOTE]");
    one.shut();
  });

  it.each([
    ["a rule that is code inside a fence", "> [!NOTE]\n> ```\n> ---\n> ```"],
    ["dashes indented far enough to be code", "> [!NOTE]\n>     ---"],
    ["dashes a blank line already parted", "> [!NOTE]\n>\n> ---"],
    ["the dashes a table draws", "> [!NOTE]\n> | a | b |\n> | --- | --- |\n> | 1 | 2 |"],
    ["dashes with spaces between them", "> [!NOTE]\n> - - -"],
    ["a marker nobody knows", "> [!RARO]\n> ---"],
  ])("touches nothing for %s", (_what, was) => {
    expect(loosened(was)).toBe(was);
  });

  it("does not walk the whole document once per marker", () => {
    const deep = Array.from({ length: 2000 }, (_, at) => `${">".repeat(at + 1)} [!NOTE]`);
    const at = performance.now();
    loosened(deep.join("\n"));
    expect(performance.now() - at).toBeLessThan(3000);
  });
});

describe("a key bound inside one node stays inside it", () => {
  const splits: [string, string, string][] = [
    ["a heading", "## titulo", "## tit\n\n## ulo"],
    ["a bullet", "- uno dos", "- uno\n-  dos"],
    ["a number", "1. uno dos", "1. uno\n2.  dos"],
    ["a task", "- [ ] uno dos", "- [ ] uno\n- [ ]  dos"],
    ["a quote", "> uno dos", "> uno\n>\n>  dos"],
    ["a callout", "> [!TIP]\n> uno dos", "> [!TIP]\n> uno\n>\n>  dos"],
    ["a paragraph", "uno dos", "uno\n\n dos"],
  ];

  it.each(splits)(
    "splits %s on Enter, as every block did before a node bound a key",
    (_what, was, then) => {
      const one = opened(was);
      let at = -1;
      one.editor.state.doc.descendants((node, pos) => {
        if (at < 0 && node.isText) at = pos + 3;
        return at < 0;
      });
      one.at(at);
      one.pressed("Enter");
      expect(one.markdown()).toBe(then);
      one.shut();
    },
  );
});

describe("a list that mixes bullets and tasks reads back as it was written", () => {
  const thrice = (source: string): string[] => {
    const seen: string[] = [];
    let held = source;
    for (let round = 0; round < 3; round += 1) {
      const one = opened(held);
      held = one.markdown();
      one.shut();
      seen.push(held);
    }
    return seen;
  };

  it.each([
    ["- [ ] cero\n- uno\n- dos", "- [ ] cero\n- uno\n- dos"],
    ["- [x] a\n- [x] b\n\n- c", "- [x] a\n- [x] b\n\n- c"],
    ["- a\n- b\n\n- [x] c", "- a\n- b\n\n- [x] c"],
    ["- [x] a\n\n- [x] b\n\n- c", "- [x] a\n\n- [x] b\n\n- c"],
    ["- a\n- [x] b\n\n- c\n- d", "- a\n- [x] b\n\n- c\n- d"],
    ["- a\n  - [x] b\n  - c", "- a\n  - [x] b\n  - c"],
    ["- [ ] a\n  - b\n\n  - c\n- [ ] d", "- [ ] a\n  - b\n\n  - c\n- [ ] d"],
    ["- uno\n- dos\n- [x] tres", "- uno\n- dos\n- [x] tres"],
    ["- uno\n- [x] dos\n- tres", "- uno\n- [x] dos\n- tres"],
    ["- [ ] a\n\n- b\n\n- c", "- [ ] a\n\n- b\n\n- c"],
  ])("leaves %j alone, save after save", (was, want) => {
    const seen = thrice(was);
    expect(seen[0]).toBe(want);
    expect(seen[1]).toBe(want);
    expect(seen[2]).toBe(want);
  });

  it("keeps counting where the numbers left off", () => {
    const seen = thrice("1. uno\n2. [x] dos\n3. tres");

    expect(seen[0]).toBe("1. uno\n\n- [x] dos\n\n3. tres");
    expect(seen[2]).toBe(seen[0]);
  });

  it("reads a numbered list of tasks as tasks, with nothing left empty", () => {
    const seen = thrice("1. [ ] uno\n2. [x] dos");

    expect(seen[0]).toBe("- [ ] uno\n- [x] dos");
    expect(seen[2]).toBe(seen[0]);
  });
});

describe("a document another editor left a byte order mark in", () => {
  const BOM = "﻿";

  it.each([
    ["# Hola\n\ntexto", "a heading"],
    ["```js\nalgo\n```", "a fence"],
    ["- uno\n- dos", "a list"],
  ])("opens %j as %s, not as escaped text", (was) => {
    const one = opened(BOM + was);
    const out = one.markdown();
    one.shut();

    expect(out).toBe(was);
  });
});

describe("a bar inside a link, a code span or a picture in a cell", () => {
  const BAR = String.fromCharCode(92);
  const head = "| a | b | c |";
  const ruled = "| --- | --- | --- |";

  const sided = (held: unknown) => ({
    type: "doc",
    content: [
      {
        type: "table",
        content: [
          {
            type: "tableRow",
            content: ["a", "b", "c"].map((text) => ({
              type: "tableHeader",
              content: [{ type: "paragraph", content: [{ type: "text", text }] }],
            })),
          },
          {
            type: "tableRow",
            content: [
              {
                type: "tableCell",
                content: [{ type: "paragraph", content: [{ type: "text", text: "left" }] }],
              },
              held,
              {
                type: "tableCell",
                content: [{ type: "paragraph", content: [{ type: "text", text: "right" }] }],
              },
            ],
          },
        ],
      },
    ],
  });

  const inked = (kind: string, text: string, attrs: Record<string, unknown>) => ({
    type: "tableCell",
    content: [
      {
        type: "paragraph",
        content: [{ type: "text", text, marks: [{ type: kind, attrs }] }],
      },
    ],
  });

  const written = (held: unknown): string => {
    const one = opened("");
    one.editor.commands.setContent(sided(held) as never);
    const out = one.markdown();
    one.shut();
    return out;
  };

  const cells = (body: string): string[] => {
    const one = opened(body);
    const said: string[] = [];
    one.editor.state.doc.firstChild?.child(1).forEach((cell) => {
      said.push(cell.textContent);
    });
    one.shut();
    return said;
  };

  it.each([
    ["a code span", inked("code", "a|b", {}), `\`a${BAR}|b\``],
    ["a link", inked("link", "ver", { href: "http://x.dev/a|b" }), `[ver](http://x.dev/a${BAR}|b)`],
    [
      "a picture",
      {
        type: "tableCell",
        content: [{ type: "image", attrs: { src: "http://x.dev/a|b.png", alt: "f" } }],
      },
      `![f](http://x.dev/a${BAR}|b.png)`,
    ],
  ])("writes %s with the bar escaped, so the row keeps its cells", (_what, held, said) => {
    const body = written(held);

    expect(body).toBe(`${head}\n${ruled}\n| left | ${said} | right |\n`);
    expect(cells(body)).toHaveLength(3);
    expect(cells(body)[2]).toBe("right");
  });
});

describe("a list item that opens on a block is caught before it is opened", () => {
  const TAB = String.fromCharCode(9);

  it.each([
    ["a quote", "- > una cita"],
    ["a numbered quote", "1. > una cita"],
    ["a fence", "- ```js\n  algo\n  ```"],
    ["a heading", "- # un titulo"],
    ["a picture", "- ![foto](x.png)"],
    ["a list", "- - anidada"],
    ["a rule", "- ***"],
    ["indented code", "-      codigo"],
    ["a table", "- | a |\n  | --- |\n  | uno |"],
    ["a heading with no words", "- #"],
    ["a quote behind a tab", `-${TAB}> una cita`],
    ["code behind two tabs", `-${TAB}${TAB}codigo`],
    ["a rule of dashes", "* ---"],
  ])("says %s cannot be kept, and it truly cannot", (_what, was) => {
    const one = opened(was);
    const out = one.markdown();
    one.shut();

    expect(frail(was)).toContain("frailBlocked");
    expect(out).not.toBe(`${was}\n`);
  });

  it.each([
    ["a hash that is no heading", "- #hashtag"],
    ["a row that is no table", "- | a | b |"],
    ["bold that is no rule", "- ***fuerte*** y nada mas"],
    ["a task", "- [ ] una tarea"],
    ["a quote below the first line", "- uno\n\n  > la cita"],
    ["what a fence holds", "```\n- > dentro de una valla\n```"],
    ["a line that is a rule, not a list", "- ---"],
    ["a rule written with spaces", "- - -"],
    ["seven hashes, which name nothing", "- #######"],
    ["a tab before plain words", `-${TAB}texto normal`],
  ])("leaves %s alone", (_what, was) => {
    expect(frail(was)).not.toContain("frailBlocked");
  });
});

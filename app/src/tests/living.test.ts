import { describe, expect, it } from "vitest";
import { inCell, opened } from "./mounted";

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

  it("keeps the diagram's frame out of the way when there is no diagram", () => {
    const one = opened("```js\nconst x = 1;\n```");
    expect(one.dom.querySelector(".lit-drawn")?.childElementCount).toBe(0);
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

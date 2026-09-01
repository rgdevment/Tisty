import { describe, expect, it } from "vitest";
import { loosened } from "../ui/writing";
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
    expect(one.markdown()).toContain("[!WARNING]");
    expect(one.markdown()).toContain("algo importante");
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

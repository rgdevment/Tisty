import { describe, expect, it } from "vitest";
import { frail } from "../frail";

describe("what a document brings that the editor cannot keep", () => {
  it("says nothing about a document made of what the editor writes", () => {
    const kept = [
      "# Título",
      "",
      "Un párrafo con **negrita**, *cursiva* y `código`.",
      "",
      "- una lista",
      "- con dos cosas",
      "",
      "| a | b |",
      "| --- | --- |",
      "| uno | dos |",
      "",
      "> una cita",
      "",
      "[un enlace](https://ejemplo.org) y ![una imagen](<attachments/ab/x.png>)",
    ].join("\n");

    expect(frail(kept)).toEqual([]);
  });

  it("sees the front matter, which grows a backslash on every save", () => {
    expect(frail("---\ntitle: algo\n---\n\n# Hola")).toContain("frailFront");
  });

  it("sees a block of html, which loses its tag and keeps only the words", () => {
    expect(frail("Hola\n\n<details>\n<summary>ver</summary>\nel detalle\n</details>")).toContain(
      "frailHtml",
    );
    expect(frail('<div class="warn">Cuidado</div>')).toContain("frailHtml");
  });

  it("sees a footnote, which comes back with its brackets escaped", () => {
    expect(frail("una nota[^1]\n\n[^1]: el pie")).toContain("frailNotes");
  });

  it("sees a link by reference, whose definition is thrown away", () => {
    expect(frail("mira [esto][uno]\n\n[uno]: https://ejemplo.org")).toContain("frailRefs");
  });

  it("leaves an aligned table alone, now that the alignment comes back", () => {
    expect(frail("| tarea | horas |\n| :--- | ---: |\n| una | 3 |")).toEqual([]);
  });

  it("leaves alone what a person wrote inside a code fence", () => {
    const fenced = ["Mira esto:", "", "```html", '<div class="warn">x</div>', "```"].join("\n");

    expect(frail(fenced)).toEqual([]);
  });

  it("leaves alone html written as indented code", () => {
    expect(frail("Ejemplo:\n\n    <div>x</div>\n")).toEqual([]);
  });

  it("does not mistake a link whose words begin with a caret for a note", () => {
    expect(frail("una nota[^1](uno) y otra[^2](dos)")).toEqual([]);
    expect(frail("mira [^arriba][uno]")).toEqual([]);
  });

  it("still sees a note that is a note", () => {
    expect(frail("una nota[^1] y su pie\n\n[^1]: el pie")).toContain("frailNotes");
  });

  it("does not mistake two horizontal rules for a front matter", () => {
    expect(frail("# Uno\n\n---\n\ntexto\n\n---\n\nmas")).toEqual([]);
    expect(frail("# Uno\n\n---\n\ntexto")).toEqual([]);
  });

  it("still sees a front matter that a rule follows further down", () => {
    expect(frail("---\ntitle: x\n---\n\n# Uno\n\n---\n\n# Dos")).toContain("frailFront");
  });

  it("leaves alone brackets that were escaped, which are text and not a note", () => {
    expect(frail("una nota\\[^1\\]\n\n\\[^1\\]: el pie")).toEqual([]);
  });

  it("names every kind it finds, not only the first", () => {
    const messy = "---\na: b\n---\n\n<div>x</div>\n\nnota[^1]\n\n[^1]: pie\n\n[uno]: https://x.dev";

    expect(frail(messy)).toEqual(["frailFront", "frailHtml", "frailNotes", "frailRefs"]);
  });

  it("sees html that plays inline in the middle of a sentence, not only html standing on its own", () => {
    expect(frail('mira este video <video src="clip.mp4"></video> antes de seguir')).toContain(
      "frailHtml",
    );
    expect(frail('escucha <audio src="clip.mp3"></audio> esto')).toContain("frailHtml");
    expect(frail('ver <iframe src="https://x.example"></iframe> aqui')).toContain("frailHtml");
  });

  it.each([
    ["section", "<section>contenido</section>"],
    ["article", "<article>cuerpo</article>"],
    ["aside", "<aside>nota</aside>"],
    ["figure", '<figure><img src="a.png"><figcaption>pie</figcaption></figure>'],
    ["form", "<form><input></form>"],
  ])("sees a block of %s, one more tag the editor does not keep", (_name, html) => {
    expect(frail(html)).toContain("frailHtml");
  });

  it("does not warn about <u>, the one raw tag Tisty keeps on purpose", () => {
    expect(frail("un <u>subrayado</u> normal")).toEqual([]);
    expect(frail("- **negrita** y <u>subrayado</u>\n- otro")).toEqual([]);
  });
});

describe("a fence is a fence however it is written", () => {
  it("leaves html inside a tilde fence alone", () => {
    expect(frail("~~~html\n<div>x</div>\n~~~")).toEqual([]);
  });

  it("leaves html inside an indented fence alone", () => {
    expect(frail("- Ejemplo:\n\n  ```html\n  <div>x</div>\n  ```")).toEqual([]);
  });

  it("leaves html inside a fence within a callout alone", () => {
    expect(frail("> [!NOTE]\n> ```html\n> <div>x</div>\n> ```")).toEqual([]);
  });

  it("still sees html that is not fenced at all", () => {
    expect(frail("# t\n\n<div>x</div>")).toContain("frailHtml");
  });
});

describe("a fence ends where the quote that opened it ends", () => {
  it("still sees html past a fence left open inside a quote", () => {
    expect(frail('> ```\n> code\n\n<div class="warn">Cuidado</div>')).toContain("frailHtml");
  });

  it("still sees a footnote past a fence left open inside a quote", () => {
    expect(frail("> ```\n> code\n\nuna nota[^1]\n\n[^1]: el pie")).toContain("frailNotes");
  });

  it("reads a fence nobody closed as code to the end, which is what markdown says", () => {
    expect(frail("~~~\nno cierra\n\nmira [esto][uno]\n\n[uno]: https://x.dev")).toEqual([]);
  });

  it("is not fooled by a fence marker quoted inside a code block", () => {
    expect(frail("```text\n> ```\n```\n\n<div>real</div>")).toContain("frailHtml");
  });

  it("is not closed by a marker shorter than the one that opened it", () => {
    expect(frail("````\ncode\n```\naun es codigo <div>x</div>\n````\n")).toEqual([]);
    expect(frail("````\ncode\n```\n````\n\n<div>real</div>")).toContain("frailHtml");
  });
});

describe("what the agent is allowed to send opens for editing", () => {
  it("leaves a tag named inside a code span alone", () => {
    expect(frail("# Guia\n\nEscribe `<div>` para abrir un bloque.")).toEqual([]);
  });

  it("leaves a footnote shape named inside a code span alone", () => {
    expect(frail("# Guia\n\nvea `[^1]` en el codigo")).toEqual([]);
  });

  it("leaves html written as tab-indented code alone", () => {
    expect(frail("Ejemplo:\n\n\t<div>x</div>")).toEqual([]);
  });

  it("tells an entity from an ampersand that only looks like one", () => {
    expect(frail("&am;")).toContain("frailEntities");
    expect(frail("&abcdefghijk;")).toContain("frailEntities");
    expect(frail("&a;")).not.toContain("frailEntities");
    expect(frail("&abcdefghijkl;")).not.toContain("frailEntities");
    expect(frail("&no es;")).not.toContain("frailEntities");
    expect(frail("uno & dos")).not.toContain("frailEntities");
  });

  it("sees a pair of dollars that opens maths and not one that was escaped", () => {
    expect(frail("$$x$$")).toContain("frailMaths");
    expect(frail("a $$x$$ b")).toContain("frailMaths");
    expect(frail("a \\$$x b")).not.toContain("frailMaths");
    expect(frail("cuesta 5$ y no 7$")).not.toContain("frailMaths");
    expect(frail("a \\$$x b$$y")).toContain("frailMaths");
  });

  it("counts the gap after a bullet and calls it a block from five spaces on", () => {
    expect(frail("-     texto")).toContain("frailBlocked");
    expect(frail("-    texto")).not.toContain("frailBlocked");
  });

  it("sees each kind of block a list item can be hiding", () => {
    expect(frail("- > una cita")).toContain("frailBlocked");
    expect(frail("- ```\n- x")).toContain("frailBlocked");
    expect(frail("- ![una imagen](x.png)")).toContain("frailBlocked");
    expect(frail("- - una sublista")).toContain("frailBlocked");
    expect(frail("- # un titulo")).toContain("frailBlocked");
    expect(frail("- | a | b |\n  | --- | --- |")).toContain("frailBlocked");
    expect(frail("- | a | b |\n  texto")).not.toContain("frailBlocked");
    expect(frail("- texto llano")).not.toContain("frailBlocked");
  });

  it("sees a reference definition, wherever its target was written", () => {
    expect(frail("[uno]: https://ejemplo.org")).toContain("frailRefs");
    expect(frail("[uno]:\n  https://ejemplo.org")).toContain("frailRefs");
    expect(frail("[uno]:")).not.toContain("frailRefs");
    expect(frail("[^1]: una nota")).not.toContain("frailRefs");
    expect(frail("[uno] no es una definicion")).not.toContain("frailRefs");
  });

  it("sees a footnote and not one whose bracket was escaped or left open", () => {
    expect(frail("un texto[^1] con nota")).toContain("frailNotes");
    expect(frail("un texto\\[^1] sin nota")).not.toContain("frailNotes");
    expect(frail("un texto[^1 sin cerrar")).not.toContain("frailNotes");
  });

  it("keeps every tag the editor itself writes", () => {
    expect(frail("un <u>subrayado</u>")).not.toContain("frailHtml");
    expect(frail("una <mark>marca</mark>")).not.toContain("frailHtml");
    expect(frail('una <mark data-pen="rojo">marca</mark>')).not.toContain("frailHtml");
    expect(frail('un <span data-ico="check"></span>')).not.toContain("frailHtml");
    expect(frail('un <span data-ico="check" data-hue="verde"></span>')).not.toContain(
      "frailHtml",
    );
    expect(frail('un <span data-ico="🎉"></span>')).not.toContain("frailHtml");
  });

  it("flags a tag the editor would lose, however close it looks to one it keeps", () => {
    expect(frail("un <div>bloque</div>")).toContain("frailHtml");
    expect(frail('un <span data-otro="x"></span>')).toContain("frailHtml");
    expect(frail('una <mark data-pen="rojo2">marca</mark>')).toContain("frailHtml");
    expect(frail('un <span data-ico="check" data-hue="verde2"></span>')).toContain("frailHtml");
  });

  it("leaves an autolink and a target written between angles alone", () => {
    expect(frail("ver <https://ejemplo.org>")).not.toContain("frailHtml");
    expect(frail("escribe a <alguien@ejemplo.org>")).not.toContain("frailHtml");
    expect(frail("[un enlace](<attachments/ab/x.png>)")).not.toContain("frailHtml");
  });

  it("only calls the line under a table a table rule", () => {
    expect(frail("- | a | b |\n  | --- | --- |")).toContain("frailBlocked");
    expect(frail("- | a | b |\n  ---")).not.toContain("frailBlocked");
    expect(frail("- | a | b |\n  | x | y |")).not.toContain("frailBlocked");
    expect(frail("- | a | b |\n  | :-: |")).toContain("frailBlocked");
  });
});

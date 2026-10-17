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

  it("lets a footnote through, now that the language keeps it", () => {
    expect(frail("una nota[^1]\n\n[^1]: el pie")).toEqual([]);
  });

  it("takes the standard forms the Tisty language adopts as they come", () => {
    expect(frail("mira [esto][uno]\n\n[uno]: https://ejemplo.org")).toEqual([]);
    expect(frail("$$\nx^2\n$$\n\ny $a$ b")).toEqual([]);
    expect(frail("```rust title=main.rs\nfn main() {}\n```")).toEqual([]);
    expect(frail("- > una cita\n- ```\n  x\n  ```")).toEqual([]);
    expect(frail("un <!-- aparte --> y &copy; 2026")).toEqual([]);
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
    expect(frail("una nota[^1] y su pie\n\n[^1]: el pie")).toEqual([]);
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

    expect(frail(messy)).toEqual(["frailFront", "frailHtml"]);
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
    expect(frail("> ```\n> code\n\nuna nota[^1]\n\n[^1]: el pie")).toEqual([]);
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

  it("takes a footnote, which the Tisty language keeps", () => {
    expect(frail("un texto[^1] con nota")).toEqual([]);
  });

  it("keeps every tag the editor itself writes", () => {
    expect(frail("un <u>subrayado</u>")).not.toContain("frailHtml");
    expect(frail("una <mark>marca</mark>")).not.toContain("frailHtml");
    expect(frail('una <mark data-pen="red">marca</mark>')).not.toContain("frailHtml");
    expect(frail('un <span data-ico="dog"></span>')).not.toContain("frailHtml");
    expect(frail('un <span data-ico="dog" data-hue="green"></span>')).not.toContain("frailHtml");
    expect(frail('un <span data-ico="🎉"></span>')).not.toContain("frailHtml");
  });

  it("flags a tag the editor would lose, however close it looks to one it keeps", () => {
    expect(frail("un <div>bloque</div>")).toContain("frailHtml");
    expect(frail('un <span data-otro="x"></span>')).toContain("frailHtml");
    expect(frail('una <mark data-pen="rojo">marca</mark>')).toContain("frailHtml");
    expect(frail('un <span data-ico="nada-que-ver"></span>')).toContain("frailHtml");
    expect(frail('un <span data-ico="dog" data-hue="verde"></span>')).toContain("frailHtml");
  });

  it("leaves an autolink and a target written between angles alone", () => {
    expect(frail("ver <https://ejemplo.org>")).not.toContain("frailHtml");
    expect(frail("escribe a <alguien@ejemplo.org>")).not.toContain("frailHtml");
    expect(frail("[un enlace](<attachments/ab/x.png>)")).not.toContain("frailHtml");
  });
});

import { describe, expect, it } from "vitest";
import { translated } from "../ui/translating";

describe("a block written in Tisty's own words", () => {
  it("shows an underline and the pens without their markup, and writes them back as they were", () => {
    const stored =
      'Hoy <u>sí</u>: ==ojo==, <mark data-pen="green">verde</mark> y <mark data-pen="teal">agua</mark>.\n';
    const one = translated(stored);
    expect(one.shown).toBe("Hoy ++sí++: ==ojo==, ==verde=={green} y ==agua=={teal}.\n");
    expect(one.stored(one.shown)).toBe(stored);
  });

  it("writes what is typed in Tisty's words the way the file keeps them", () => {
    const one = translated("Nada.\n");
    expect(one.stored("Un ++subrayado++ y ==algo=={pink} y ==esto=={yellow}.")).toBe(
      'Un <u>subrayado</u> y <mark data-pen="pink">algo</mark> y ==esto==.',
    );
  });

  it("reads every spelling of the yellow pen as the one Tisty writes", () => {
    const one = translated('<mark>uno</mark> y <mark data-pen="yellow">dos</mark>\n');
    expect(one.shown).toBe("==uno== y ==dos==\n");
    expect(one.stored(one.shown)).toBe("==uno== y ==dos==\n");
  });

  it("leaves code as it was written, in a fence and in a span", () => {
    const stored = "Mira `<u>x</u>` aquí.\n\n```html\n<u>y</u>\n```\n";
    const one = translated(stored);
    expect(one.shown).toBe(stored);
    expect(one.stored("Mira `++x++` aquí.")).toBe("Mira `++x++` aquí.");
  });

  it("never reads a plus inside a word as an underline", () => {
    const one = translated("Nada.\n");
    expect(one.stored("C++ y C++ son lo mismo; a++b++ también.")).toBe(
      "C++ y C++ son lo mismo; a++b++ también.",
    );
  });

  it("keeps a pen it does not know as it was written", () => {
    const stored = '<mark data-pen="gold">oro</mark> y ==plata=={silver}\n';
    const one = translated(stored);
    expect(one.shown).toBe(stored);
    expect(one.stored(one.shown)).toBe(stored);
  });

  it("shows an icon by its name and colour, and writes it back as the file keeps it", () => {
    const stored =
      '<span data-ico="dog" data-hue="orange">:dog:</span> Rina y <span data-ico="star">:star:</span>\n';
    const one = translated(stored);
    expect(one.shown).toBe("::dog orange:: Rina y ::star::\n");
    expect(one.stored(one.shown)).toBe(stored);
    expect(one.stored("::bone amber:: y ::nope:: y std::vector::size")).toBe(
      '<span data-ico="bone" data-hue="amber">:bone:</span> y ::nope:: y std::vector::size',
    );
    expect(one.stored("::bone amber::::dog::")).toBe(
      '<span data-ico="bone" data-hue="amber">:bone:</span><span data-ico="dog">:dog:</span>',
    );
  });

  it("calls an attachment by its name and puts its place in the store back on the way out", () => {
    const stored =
      'Mira ![Los cuadrantes](<attachments/6f/prioridades-4d6ec1f6.png>) y ![Paseos](attachments/2d/paseos-78e2bf0f.html "file")\n';
    const one = translated(stored);
    expect(one.shown).toBe(
      'Mira ![Los cuadrantes](prioridades.png) y ![Paseos](paseos.html "file")\n',
    );
    expect(one.stored(one.shown)).toBe(stored);
    expect(one.stored("Solo ![Otra vez](prioridades.png)")).toBe(
      "Solo ![Otra vez](<attachments/6f/prioridades-4d6ec1f6.png>)",
    );
  });

  it("keeps two attachments a block would call by the same name as they were written", () => {
    const stored =
      "![a](<attachments/11/foto-11111111.png>) ![b](<attachments/22/foto-22222222.png>)\n";
    expect(translated(stored).shown).toBe(stored);
  });

  it("shows a block as it was written when its words would not come back the same", () => {
    const stored = "Ya decía ++esto++ sin subrayar.\n";
    const one = translated(stored);
    expect(one.shown).toBe(stored);
    expect(one.stored("Ya decía ++esto++ sin subrayar")).toBe("Ya decía ++esto++ sin subrayar");
  });
});

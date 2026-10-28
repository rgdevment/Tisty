import { describe, expect, it } from "vitest";
import { holds, peeled, toggled } from "../ui/peeled";

// One cell per line: as written, the words the field shows, then what new words are written back as.
const CELLS = `
==mañana 10am=={blue} | mañana 10am | ==el lunes=={blue}
==hoy== | hoy | ==el lunes==
**Total** | Total | **el lunes**
*nota* | nota | *el lunes*
_nota_ | nota | _el lunes_
++subrayado++ | subrayado | ++el lunes++
~~tachado~~ | tachado | ~~el lunes~~
\`code\` | code | \`el lunes\`
**==ambos==** | ambos | **==el lunes==**
==**ambos**=={pink} | ambos | ==**el lunes**=={pink}
**hoy** y ==mañana== | **hoy** y ==mañana== | el lunes
**uno** **dos** | **uno** **dos** | el lunes
==a== b == c== | ==a== b == c== | el lunes
C++ y C++ | C++ y C++ | el lunes
** | ** | el lunes
palabras | palabras | el lunes
`;

describe("a cell opened to be written", () => {
  it("shows the words a whole mark wraps and wraps new words the same way", () => {
    for (const line of CELLS.trim().split("\n")) {
      const [written, words, back] = line.split(" | ");
      const cell = peeled(written);
      expect(cell.words, line).toBe(words);
      expect(cell.wrap("el lunes"), line).toBe(back);
    }
  });

  it("writes an emptied cell empty, without walls around nothing", () => {
    expect(peeled("==hoy=={blue}").wrap("  ")).toBe("");
  });

  it("writes plain the words that would tear their mark, keeping the marks they do not touch", () => {
    expect(peeled("==hoy=={blue}").wrap("a == b")).toBe("a == b");
    expect(peeled("_nota_").wrap("snake_case")).toBe("snake_case");
    expect(peeled("**==hoy==**").wrap("a == b")).toBe("**a == b**");
  });
});

describe("a cell's marks put on and taken off from the bar", () => {
  // One press per line: the cell as written, the mark pressed, then the cell written with "hoy".
  const PRESSES = `
hoy | ** ** | **hoy**
**hoy** | ** ** | hoy
**hoy** | * * | ***hoy***
_hoy_ | * * | hoy
==hoy== | == =={blue} | ==hoy=={blue}
==hoy=={blue} | == =={blue} | hoy
**==hoy==** | ~~ ~~ | **==~~hoy~~==**
`;

  it("puts a mark on, takes the same kind off, and turns a pen another colour", () => {
    for (const line of PRESSES.trim().split("\n")) {
      const [written, pressed, back] = line.split(" | ");
      const [open, close] = pressed.split(" ");
      const cell = peeled(written);
      const now = toggled(cell.marks, open, close);
      expect(cell.wrap("hoy", now), line).toBe(back);
      expect(holds(now, open, close), line).toBe(back !== "hoy" && back.includes(close));
    }
  });
});

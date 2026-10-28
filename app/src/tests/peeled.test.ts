import { describe, expect, it } from "vitest";
import { peeled } from "../ui/peeled";

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

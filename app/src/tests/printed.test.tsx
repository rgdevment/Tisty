import { renderToBuffer } from "@react-pdf/renderer";
import { describe, expect, it } from "vitest";
import type { Run, Shape } from "../ui/paper";
import { Papered, sliced } from "../ui/paper";

const STAMP = /\/CreationDate \([^)]*\)/g;

const book = async (sheets: Shape[][]): Promise<string> => {
  const pdf = await renderToBuffer(<Papered sheets={sheets} leaf="a4" />);
  return pdf.toString("latin1").replace(STAMP, "");
};

const para = (text: string): Shape => ({ kind: "para", runs: [{ text }] });

describe("a rule inside a callout", () => {
  const said = (inner: Shape[]): Shape[][] => [[{ kind: "said", said: "note", inner }]];

  it("is drawn, where a page break would make no sense", async () => {
    const ruled = await book(said([para("uno"), { kind: "rule" }, para("dos")]));
    const plain = await book(said([para("uno"), para("dos")]));
    expect(ruled).not.toBe(plain);
  }, 30000);
});

describe("a code block", () => {
  const code = (lines: Run[][]): Shape[][] => [[{ kind: "code", deep: 0, lines }]];

  it("reaches the page with the colour the editor gives it", async () => {
    const lit = await book(code([[{ text: "const", hue: "#7a44b8" }, { text: " uno = 1" }]]));
    const flat = await book(code([[{ text: "const uno = 1" }]]));
    expect(lit).not.toBe(flat);
  }, 30000);
});

describe("cutting a line into what fits", () => {
  const parts: Run[] = [
    { text: "const ", hue: "#7a44b8" },
    { text: "uno" },
    { text: " = 1", hue: "#b35c00" },
  ];

  it("keeps every letter, in order, however it is cut", () => {
    const whole = parts.map((one) => one.text).join("");
    for (let at = 0; at <= whole.length; at += 1) {
      const head = sliced(parts, 0, at);
      const tail = sliced(parts, at, whole.length);
      expect(
        head
          .concat(tail)
          .map((one) => one.text)
          .join(""),
      ).toBe(whole);
    }
  });

  it("carries the colour of the piece it came from", () => {
    expect(sliced(parts, 2, 8)).toEqual([{ text: "nst ", hue: "#7a44b8" }, { text: "un" }]);
  });

  it("gives nothing back for a cut outside the line", () => {
    expect(sliced(parts, 40, 60)).toEqual([]);
  });
});

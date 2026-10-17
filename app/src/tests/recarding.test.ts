import { describe, expect, it } from "vitest";
import { filedAs, reshaped } from "../ui/recarding";

const text =
  '# Plan\n\nVer ![Acta](acta-0006bf09.pdf) aquí.\n\n![Paseos](paseos-0006bf10.html "file")\n\nFin.';
const span = (said: string): [number, number] => {
  const at = text.indexOf(said);
  return [at, at + said.length];
};

describe("changing a card by its own line of Markdown", () => {
  it("turns a card into a link, leaving the words around it alone", () => {
    const at = span("Ver ![Acta](acta-0006bf09.pdf) aquí.");
    expect(reshaped(text, at, "acta-0006bf09.pdf", "link")).toBe(
      text.replace("![Acta](acta-0006bf09.pdf)", "[Acta](acta-0006bf09.pdf)"),
    );
  });

  it("removes a card, and the paragraph with it when nothing else was there", () => {
    const at = span('![Paseos](paseos-0006bf10.html "file")');
    expect(reshaped(text, at, "paseos-0006bf10.html", "remove")).toBe(
      "# Plan\n\nVer ![Acta](acta-0006bf09.pdf) aquí.\n\nFin.",
    );
  });

  it("switches a page between drawn live and kept as a file", () => {
    const at = span('![Paseos](paseos-0006bf10.html "file")');
    expect(filedAs(text, at, "paseos-0006bf10.html")).toBe(true);
    const live = reshaped(text, at, "paseos-0006bf10.html", "live") ?? "";
    expect(live).toContain("![Paseos](paseos-0006bf10.html)\n");
    const back = live.indexOf("![Paseos]");
    const line: [number, number] = [back, back + "![Paseos](paseos-0006bf10.html)".length];
    expect(filedAs(live, line, "paseos-0006bf10.html")).toBe(false);
    expect(reshaped(live, line, "paseos-0006bf10.html", "file")).toBe(text);
  });

  it("keeps a card's own title through every change but the file one", () => {
    const titled = '![Acta](acta.pdf "Q3 plan")';
    expect(reshaped(titled, [0, titled.length], "acta.pdf", "link")).toBe(
      '[Acta](acta.pdf "Q3 plan")',
    );
    expect(reshaped(titled, [0, titled.length], "acta.pdf", "live")).toBe(titled);
  });

  it("takes away the blank line a removed card leaves, with Windows line ends too", () => {
    const crlf = "Antes.\r\n\r\n![Acta](acta.pdf)\r\n\r\nDespués.";
    const at = crlf.indexOf("![");
    expect(reshaped(crlf, [at, at + "![Acta](acta.pdf)".length], "acta.pdf", "remove")).toBe(
      "Antes.\r\n\r\nDespués.",
    );
  });

  it("changes nothing when the card is not in that block", () => {
    expect(reshaped(text, span("Fin."), "acta-0006bf09.pdf", "link")).toBeNull();
  });
});

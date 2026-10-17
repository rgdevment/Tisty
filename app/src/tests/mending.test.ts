import { describe, expect, it, vi } from "vitest";
import { fill, t } from "../locales";
import { mendable, mended, told } from "../ui/mending";

const bold = { said: "<b>x</b>", fixed: "**x**" };

describe("a block outside the Tisty language on the page", () => {
  it("is told under itself once, naming what is wrong, and its fix asks for that block alone", () => {
    document.body.innerHTML =
      '<article class="page"><p data-at="0-5">Bien</p><p data-at="6-20">Mal <b>x</b></p></article>';
    const mend = vi.fn();
    const one = { from: 6, to: 20, fixed: "Mal **x**", pieces: [bold] };

    mendable(document, [one], mend);
    mendable(document, [one], mend);

    const said = document.querySelectorAll(".mend-said");
    expect(said).toHaveLength(1);
    expect(said[0].previousElementSibling?.getAttribute("data-at")).toBe("6-20");
    const codes = Array.from(said[0].querySelectorAll("code")).map((code) => code.textContent);
    expect(codes).toEqual(["<b>x</b>", "**x**"]);
    expect(said[0].textContent).toContain(t("unspokenIs"));
    said[0].querySelector("button")?.click();
    expect(mend).toHaveBeenCalledWith(one);
  });

  it("says nothing of a block the page no longer draws", () => {
    document.body.innerHTML = '<article class="page"><p data-at="0-5">Bien</p></article>';
    mendable(document, [{ from: 9, to: 12, fixed: "x", pieces: [] }], vi.fn());
    expect(document.querySelector(".mend-said")).toBeNull();
  });
});

describe("what is told of the pieces", () => {
  it("counts several and lists them, and falls back to plain words when none could be named", () => {
    const many = told([
      { from: 0, to: 1, fixed: "", pieces: [bold, { said: "<u>y</u>", fixed: "++y++" }] },
    ]);
    expect(many.parts).toEqual([{ words: fill("unspokenMany", "2") }]);
    expect(many.all).toBe("<b>x</b> → **x**\n<u>y</u> → ++y++");

    expect(told([{ from: 0, to: 1, fixed: "", pieces: [] }]).parts).toEqual([
      { words: t("unspoken") },
    ]);
  });

  it("names a piece that only goes away without saying how it is written, and cuts a long one", () => {
    const long = "x".repeat(80);
    const { parts } = told([{ from: 0, to: 1, fixed: "", pieces: [{ said: long, fixed: "" }] }]);
    expect(parts).toHaveLength(2);
    expect(parts[0].code).toHaveLength(60);
    expect(parts[0].code?.endsWith("…")).toBe(true);
  });
});

describe("mending a text", () => {
  it("puts every fix where its bytes were, past letters wider than one", () => {
    const text = "él <u>a</u> y ñ <b>b</b>";
    const bytes = (part: string) => new TextEncoder().encode(part).length;
    const first = { from: bytes("él "), to: bytes("él <u>a</u>"), fixed: "++a++", pieces: [] };
    const second = { from: bytes("él <u>a</u> y ñ "), to: bytes(text), fixed: "**b**", pieces: [] };

    expect(mended(text, [first, second])).toBe("él ++a++ y ñ **b**");
  });
});

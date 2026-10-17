import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { mendable, mended } from "../ui/mending";

describe("a block outside the Tisty language on the page", () => {
  it("is told under itself once, and its fix asks for that block alone", () => {
    document.body.innerHTML =
      '<article class="page"><p data-at="0-5">Bien</p><p data-at="6-20">Mal <b>x</b></p></article>';
    const mend = vi.fn();
    const one = { from: 6, to: 20, fixed: "Mal **x**" };

    mendable(document, [one], mend);
    mendable(document, [one], mend);

    const said = document.querySelectorAll(".mend-said");
    expect(said).toHaveLength(1);
    expect(said[0].previousElementSibling?.getAttribute("data-at")).toBe("6-20");
    expect(said[0].textContent).toContain(t("unspoken"));
    said[0].querySelector("button")?.click();
    expect(mend).toHaveBeenCalledWith(one);
  });

  it("says nothing of a block the page no longer draws", () => {
    document.body.innerHTML = '<article class="page"><p data-at="0-5">Bien</p></article>';
    mendable(document, [{ from: 9, to: 12, fixed: "x" }], vi.fn());
    expect(document.querySelector(".mend-said")).toBeNull();
  });
});

describe("mending a text", () => {
  it("puts every fix where its bytes were, past letters wider than one", () => {
    const text = "él <u>a</u> y ñ <b>b</b>";
    const bytes = (part: string) => new TextEncoder().encode(part).length;
    const first = { from: bytes("él "), to: bytes("él <u>a</u>"), fixed: "++a++" };
    const second = { from: bytes("él <u>a</u> y ñ "), to: bytes(text), fixed: "**b**" };

    expect(mended(text, [first, second])).toBe("él ++a++ y ñ **b**");
  });
});

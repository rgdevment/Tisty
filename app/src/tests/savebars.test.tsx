import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { fill, t } from "../locales";
import { ClashBar, clashSaid } from "../ui/SaveBars";

describe("what the clash bar says", () => {
  it("counts the places when only the words clashed", () => {
    expect(clashSaid(1)).toBe(t("clashTornOne"));
    expect(clashSaid(3)).toBe(fill("clashTorn", "3"));
    expect(clashSaid(null)).toBe(t("clashWhole"));
    expect(clashSaid()).toBe(t("documentMoved"));
  });

  it("says only the properties clashed when no place of the words did", () => {
    expect(clashSaid(0, true)).toBe(t("clashFront"));
    expect(clashSaid(undefined, true)).toBe(t("clashFront"));
  });

  it("adds that the properties clash too when places of the words did as well", () => {
    expect(clashSaid(1, true)).toBe(`${t("clashTornOne")} ${t("clashFrontToo")}`);
    expect(clashSaid(2, true)).toBe(`${fill("clashTorn", "2")} ${t("clashFrontToo")}`);
  });

  it("does not add it when mine puts the whole page back anyway", () => {
    expect(clashSaid(null, true)).toBe(t("clashWhole"));
  });

  it("is drawn in the bar", () => {
    render(<ClashBar style={{}} torn={1} front onMine={() => {}} onTheirs={() => {}} />);

    expect(screen.getByText(`${t("clashTornOne")} ${t("clashFrontToo")}`)).toBeTruthy();
  });
});

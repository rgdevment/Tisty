import { shell } from "tisty-reader/page";
import { describe, expect, it } from "vitest";
import { went } from "../ui/paging";
import { staged } from "../ui/staging";

const page = (html: string): Document => new DOMParser().parseFromString(shell(html), "text/html");

describe("a stage named in a document", () => {
  it("shows the words of that stage and leads to it, and keeps a number with no stage as it is", () => {
    const drawn = page(
      '<ul data-at="0-40"><li class="task" data-at="0-12"><span class="check"></span>Weigh her</li><li class="task" data-at="13-40"><span class="check"></span>Ask about the tick tablet</li></ul><p data-at="41-70">Then <span class="ref" data-ref="#2">#2</span> and <span class="ref" data-ref="#9">#9</span> and <span class="ref" data-ref="Rina">Rina</span></p>',
    );
    staged(drawn);
    const [second, missing, named] = Array.from(drawn.querySelectorAll<HTMLElement>(".ref"));
    expect(second.textContent).toBe("Ask about the tick tablet");
    expect(second.title).toBe("#2");
    expect(went(second, false)).toEqual({ kind: "here", id: "stage--2" });
    expect(drawn.getElementById("stage--2")?.textContent).toContain("tick tablet");
    expect(missing.textContent).toBe("#9");
    expect(went(missing, false)).toBeNull();
    expect(named.textContent).toBe("Rina");
    expect(went(named, false)).toBeNull();
  });

  it("cuts a long stage short in the chip", () => {
    const long = "a".repeat(80);
    const drawn = page(
      `<ul data-at="0-90"><li class="task" data-at="0-90">${long}</li></ul><p data-at="91-100"><span class="ref" data-ref="#1">#1</span></p>`,
    );
    staged(drawn);
    const chip = drawn.querySelector<HTMLElement>(".ref");
    expect(chip?.textContent).toHaveLength(60);
    expect(chip?.textContent?.endsWith("…")).toBe(true);
  });
});

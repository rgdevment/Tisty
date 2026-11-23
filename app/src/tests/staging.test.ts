import { staged } from "tisty-editor/read";
import { describe, expect, it } from "vitest";
import { framed } from "../ui/framed";
import { went } from "../ui/paging";

const page = (html: string): Document => new DOMParser().parseFromString(framed(html), "text/html");

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

  it("shows a stage's words without the letter that shows nothing beside its pieces", () => {
    const drawn = page(
      '<ul data-at="0-40"><li class="task" data-at="0-40"><span class="check"></span>Weigh <span class="tag" data-src="8-12">#her</span>\u200B<ul data-at="13-40"><li data-at="13-40">twice</li></ul></li></ul><p data-at="41-50"><span class="ref" data-ref="#1">#1</span></p>',
    );
    staged(drawn);
    expect(drawn.querySelector(".ref")?.textContent).toBe("Weigh #hertwice");
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

  it("counts the stages again when one comes before them, and writes nothing when none moved", () => {
    const drawn = page(
      '<ul data-at="0-40"><li class="task" data-at="0-12">Weigh her</li><li class="task" id="mine" data-at="13-20">Feed her</li></ul><p data-at="41-70"><span class="ref" data-ref="#1">#1</span></p>',
    );
    staged(drawn);
    const list = drawn.querySelector("ul") as Element;
    list.insertAdjacentHTML("afterbegin", '<li class="task" data-at="0-5">Wake her</li>');
    staged(drawn);
    const [first, second, third] = Array.from(drawn.querySelectorAll("li"), (one) => one.id);
    expect([first, second, third]).toEqual(["stage--1", "stage--2", "mine"]);
    expect(drawn.querySelector(".ref")?.textContent).toBe("Wake her");
    const watch = new MutationObserver(() => {});
    watch.observe(drawn.documentElement, { subtree: true, childList: true, attributes: true });
    staged(drawn);
    expect(watch.takeRecords()).toEqual([]);
    watch.disconnect();
  });
});

import { describe, expect, it } from "vitest";
import { type PartCarded, partCarded, recarded } from "../ui/partcarding";

const paged = () => {
  const page = document.implementation.createHTMLDocument("");
  page.body.innerHTML =
    '<article class="page"><p><a class="card" data-task="01P">Comprar pan</a></p></article>';
  return page;
};

describe("a part's card", () => {
  it("is drawn waiting, then takes what the part says in place once it is known", () => {
    const page = paged();
    partCarded(page, (): PartCarded => ({ kind: "waiting" }));
    const first = page.querySelector<HTMLElement>(".is-part[data-task='01P']");
    expect(first?.querySelector(".card-name")?.textContent).toBe("Comprar pan");

    recarded(page, (): PartCarded => ({ kind: "open", title: "Pan integral", done: 1, steps: 3 }));
    const now = page.querySelectorAll<HTMLElement>(".is-part[data-task='01P']");
    expect(now).toHaveLength(1);
    expect(now[0].classList.contains("is-open")).toBe(true);
    expect(now[0].querySelector(".card-name")?.textContent).toBe("Pan integral");

    recarded(page, (): PartCarded => ({ kind: "loose" }));
    expect(page.querySelector(".is-part .card-name")?.textContent).toBe("Comprar pan");
  });

  it("keeps the keys on a card that is filled in place", () => {
    document.body.innerHTML = paged().body.innerHTML;
    partCarded(document, (): PartCarded => ({ kind: "waiting" }));
    document.querySelector<HTMLElement>(".is-part")?.focus();
    expect(document.activeElement?.classList.contains("is-waiting")).toBe(true);
    recarded(
      document,
      (): PartCarded => ({ kind: "open", title: "Pan integral", done: 0, steps: 0 }),
    );
    expect(document.activeElement?.classList.contains("is-open")).toBe(true);
    document.body.innerHTML = "";
  });
});

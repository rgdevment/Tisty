import { celled, hueOf, sayingDays } from "tisty-editor/read";
import { describe, expect, it } from "vitest";
import { went } from "../ui/paging";
import { SAY } from "../ui/saying";
import { tabled } from "../ui/tabled";

const HUES = ["red", "orange", "amber", "green", "teal", "blue"];

const TABLE = `<div class="table" data-at="0-99"><table><thead><tr><th scope="col">Servicio</th><th scope="col" data-type="date">Cobro</th><th scope="col" data-type="rating">Nota</th><th scope="col" data-type="check">Activa</th><th scope="col" data-type="select">Categoría</th><th scope="col" data-type="tags">Etiquetas</th><th scope="col" data-type="percent">Avance</th></tr></thead><tbody>
<tr><td>Spotify</td><td data-key="2026-11-14">2026-11-14</td><td data-key="3">★★★☆☆</td><td data-key="1">✓</td><td data-key="música">Música</td><td data-key="web, docs">web, docs</td><td data-key="45">45%</td></tr>
<tr><td>Raro</td><td data-amiss>pronto</td><td data-amiss>9</td><td data-amiss>quizás</td><td></td><td></td><td data-amiss>mucho</td></tr>
</tbody></table></div>`;

const dressed = (): Document => {
  const page = document.implementation.createHTMLDocument("");
  page.body.innerHTML = `<article class="page">${TABLE}</article>`;
  tabled(page);
  celled(page, { today: "2026-10-27", locale: "es", hues: HUES, live: true, say: SAY });
  return page;
};

const cells = (page: Document, row: number) =>
  Array.from(page.querySelectorAll("tbody > tr")[row].querySelectorAll<HTMLElement>("td"));

describe("a typed value as it is drawn", () => {
  it("gives an option the same hue every time, from its words alone", () => {
    expect(hueOf("Música", HUES)).toBe(hueOf("música", HUES));
    expect(HUES).toContain(hueOf("Video", HUES));
    expect(hueOf("x", [])).toBe("gray");
  });

  // A day, today, then how the reader's language says it and how far it is.
  const DAYS = `
2026-11-14 | 2026-10-27 | es => 14 nov 2026 | dentro de 18 días
2026-10-28 | 2026-10-27 | es => 28 oct 2026 | mañana
2026-10-27 | 2026-10-27 | en => Oct 27, 2026 | today
2026-10-24 | 2026-10-27 | en => Oct 24, 2026 | 3 days ago
`;

  it("says a day in the reader's language and how far it is from today", () => {
    for (const line of DAYS.trim().split("\n")) {
      const [asked, said] = line.split("=>").map((one) => one.trim());
      const [iso, today, locale] = asked.split("|").map((one) => one.trim());
      const { day, when } = sayingDays(locale)(iso, today);
      expect(`${day.replace(".", "")} | ${when}`, line).toBe(said);
    }
  });

  it("draws each type as it is used, and leaves a value written amiss as it is", () => {
    const page = dressed();
    const [name, date, rating, check, option, tags, percent] = cells(page, 0);
    expect(name.querySelector(".row-grip")).not.toBeNull();
    expect(date.querySelector(".table-when")?.textContent).toBe("dentro de 18 días");
    const stars = Array.from(rating.querySelectorAll<HTMLElement>(".table-star"));
    expect(stars.map((one) => one.getAttribute("aria-pressed"))).toEqual([
      "true",
      "true",
      "true",
      "false",
      "false",
    ]);
    expect(stars.map((one) => one.dataset.to)).toEqual(["1", "2", "0", "4", "5"]);
    expect(stars[0].getAttribute("aria-label")).toBe("1 of 5");
    const toggle = check.querySelector<HTMLElement>(".table-switch");
    expect(toggle?.getAttribute("role")).toBe("switch");
    expect(toggle?.getAttribute("aria-checked")).toBe("true");
    expect(toggle?.dataset.to).toBe("0");
    expect(
      option.querySelector<HTMLElement>(".table-option")?.style.getPropertyValue("--hue"),
    ).toBe(`var(--tisty-hue-${hueOf("Música", HUES)})`);
    expect(
      Array.from(tags.querySelectorAll(".table-option")).map((one) => one.textContent),
    ).toEqual(["web", "docs"]);
    expect(percent.style.getPropertyValue("--fill")).toBe("45%");

    const amiss = cells(page, 1);
    expect(amiss.slice(1, 4).map((one) => one.textContent)).toEqual(["pronto", "9", "quizás"]);
    expect(amiss[6].classList.contains("table-percent")).toBe(false);
  });

  it("is drawn once, and a star or the switch is a press the window hears", () => {
    const page = dressed();
    celled(page, { today: "2026-10-27", locale: "es", hues: HUES, live: true, say: SAY });
    expect(cells(page, 0)[2].querySelectorAll(".table-star")).toHaveLength(5);
    const star = cells(page, 0)[2].querySelectorAll<HTMLElement>(".table-star")[3];
    expect(went(star, false)).toMatchObject({
      kind: "table",
      act: "star",
      row: 0,
      column: 2,
      to: 4,
    });
    const toggle = cells(page, 0)[3].querySelector<HTMLElement>(".table-switch") as HTMLElement;
    expect(went(toggle, false)).toMatchObject({
      kind: "table",
      act: "toggle",
      row: 0,
      column: 3,
      to: 0,
    });
  });
});

describe("a typed value with nothing in it, or a page only read", () => {
  const EMPTY = TABLE.replace(
    "<tr><td>Raro</td><td data-amiss>pronto</td><td data-amiss>9</td><td data-amiss>quizás</td>",
    "<tr><td>Nuevo</td><td></td><td></td><td></td>",
  );

  it("sets an empty rating or yes-or-no with one press too, as none and as no", () => {
    const page = document.implementation.createHTMLDocument("");
    page.body.innerHTML = `<article class="page">${EMPTY}</article>`;
    tabled(page);
    celled(page, { today: "2026-10-27", locale: "es", hues: HUES, live: true, say: SAY });
    const [, , rating, check] = cells(page, 1);
    const stars = Array.from(rating.querySelectorAll<HTMLElement>(".table-star"));
    expect(stars.map((one) => one.getAttribute("aria-pressed"))).toEqual(Array(5).fill("false"));
    expect(stars.map((one) => one.dataset.to)).toEqual(["1", "2", "3", "4", "5"]);
    const toggle = check.querySelector<HTMLElement>(".table-switch");
    expect(toggle?.getAttribute("aria-checked")).toBe("false");
    expect(toggle?.dataset.to).toBe("1");
  });

  it("shows days and options the same way on a page only read, with nothing to press", () => {
    const page = document.implementation.createHTMLDocument("");
    page.body.innerHTML = `<article class="page">${TABLE}</article>`;
    celled(page, { today: "2026-10-27", locale: "es", hues: HUES, live: false, say: SAY });
    const [, date, rating, check, option] = cells(page, 0);
    expect(date.querySelector(".table-when")?.textContent).toBe("dentro de 18 días");
    expect(option.querySelector(".table-option")).not.toBeNull();
    expect(rating.querySelector("button")).toBeNull();
    expect(rating.textContent).toBe("★★★☆☆");
    expect(check.querySelector("button")).toBeNull();
  });
});

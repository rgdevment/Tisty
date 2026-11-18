import { describe, expect, it, vi } from "vitest";
import {
  type CardAsked,
  cardKey,
  cardsIn,
  cardsOn,
  cardsPlaced,
  taskCardAt,
  unfolded,
} from "../src/placing";

// A page of the test's own window, since a key is held only where a window shows the page.
const page = (html: string): Document => {
  document.body.innerHTML = `<article class="page">${html}</article>`;
  return document;
};

const doc = (id: string, words: string): CardAsked => ({ kind: "doc", id, words });
const task = (id: string, words: string): CardAsked => ({ kind: "task", id, words });

const OPEN =
  '<span class="doc-card is-task is-open" tabindex="0" role="link" data-task="T1"><span class="card-name">Vet</span>' +
  '<span class="card-meta"><button type="button" data-asked="taskSteps" data-task="T1" class="chip card-unfold" aria-expanded="false">0 de 1</button></span>' +
  '<span class="card-steps"><span class="card-step"><button type="button" data-asked="taskStep" data-task="T1" class="card-step-box" data-step="s1"></button></span></span></span>';

describe("the cards a sheet lays", () => {
  it("names every card the core wrote, in the order written, as the page will read them", () => {
    const html =
      '<p><a class="card" data-doc="f2">Huerta &amp; &quot;riego&quot; &lt;3&gt;</a></p>' +
      '<p>Ver <a class="card" data-task="T1">la tarea</a> hoy</p>';
    expect(cardsIn(html)).toEqual([doc("f2", 'Huerta & "riego" <3>'), task("T1", "la tarea")]);
    expect(cardsIn("<p>nada</p>")).toEqual([]);
  });

  it("lays a card alone in its paragraph and leaves one inside a sentence as a word of it", () => {
    const drawn = page(
      '<p><a class="card" data-doc="f2">Huerta</a></p><p>Ver <a class="card" data-doc="f2">Huerta</a> hoy</p>' +
        '<p data-at="9"> <a class="card" data-task="T1">Vet</a> </p>',
    );
    const cards = new Map([
      [
        cardKey(doc("f2", "Huerta")),
        '<span class="doc-card is-doc" role="link" tabindex="0" data-doc="f2">Huerta</span>',
      ],
    ]);

    const missing = cardsPlaced(drawn, cards);

    expect(missing).toEqual([task("T1", "Vet")]);
    const held = drawn.querySelector<HTMLElement>(".page > p[data-card]");
    expect(held?.dataset.card).toBe(cardKey(doc("f2", "Huerta")));
    expect(held?.querySelector(".doc-card")?.textContent).toBe("Huerta");
    expect(drawn.querySelectorAll(".doc-card")).toHaveLength(1);
    expect(drawn.querySelector('p[data-at="9"] a.card')).not.toBeNull();
    expect(cardsOn(drawn)).toEqual([doc("f2", "Huerta"), doc("f2", "Huerta"), task("T1", "Vet")]);
  });

  it("lays again only the cards whose drawing changed, keeping their steps unfolded and the key pressed", () => {
    const drawn = page(
      '<p><a class="card" data-task="T1">Vet</a></p><p><a class="card" data-doc="f2">Huerta</a></p>',
    );
    const key = cardKey(task("T1", "Vet"));
    const other = cardKey(doc("f2", "Huerta"));
    const was = new Map([
      [key, OPEN],
      [other, '<span class="doc-card is-doc">Huerta</span>'],
    ]);
    cardsPlaced(drawn, was);
    const huerta = drawn.querySelector(".is-doc");
    unfolded(drawn, 0);
    const box = drawn.querySelector<HTMLElement>("[data-step='s1']");
    box?.focus();
    const days = vi.fn();

    const now = new Map([...was, [key, OPEN.replace("0 de 1", "1 de 1")]]);
    cardsPlaced(drawn, now, was, days);

    const card = drawn.querySelector<HTMLElement>(".is-task");
    expect(card?.classList.contains("unfolded")).toBe(true);
    expect(card?.querySelector(".card-unfold")?.textContent).toBe("1 de 1");
    expect(card?.querySelector(".card-unfold")?.getAttribute("aria-expanded")).toBe("true");
    expect(drawn.activeElement).toBe(card?.querySelector("[data-step='s1']"));
    expect(drawn.querySelector(".is-doc")).toBe(huerta);
    expect(days).toHaveBeenCalledTimes(1);
    expect(days).toHaveBeenCalledWith(card);
  });

  it("keeps the focus on a card drawn again when the card itself held it", () => {
    const drawn = page('<p><a class="card" data-task="T1">Vet</a></p>');
    const key = cardKey(task("T1", "Vet"));
    const was = new Map([[key, OPEN]]);
    cardsPlaced(drawn, was);
    drawn.querySelector<HTMLElement>(".is-task")?.focus();

    cardsPlaced(drawn, new Map([[key, OPEN.replace("Vet", "Veterinario")]]), was);

    expect(drawn.activeElement).toBe(drawn.querySelector(".is-task"));
    expect(drawn.activeElement?.querySelector(".card-name")?.textContent).toBe("Veterinario");
  });

  it("opens a card from the keyboard, as a link does, and only from the card itself", () => {
    const drawn = page('<p><a class="card" data-task="T1">Vet</a></p>');
    cardsPlaced(drawn, new Map([[cardKey(task("T1", "Vet")), OPEN]]));
    const card = drawn.querySelector<HTMLElement>(".is-task");
    const pressed = vi.fn();
    card?.addEventListener("click", pressed);

    for (const key of ["Enter", " ", "a"])
      card?.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));
    card
      ?.querySelector("[data-step='s1']")
      ?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

    expect(pressed).toHaveBeenCalledTimes(2);
  });

  it("lays a card in a page drawn inside a frame, with its days and its keys", () => {
    const frame = document.createElement("iframe");
    document.body.innerHTML = "";
    document.body.append(frame);
    const inside = frame.contentDocument as Document;
    inside.body.innerHTML =
      '<article class="page"><p><a class="card" data-task="T1">Vet</a></p></article>';
    const days = vi.fn();

    cardsPlaced(inside, new Map([[cardKey(task("T1", "Vet")), OPEN]]), undefined, days);

    const card = inside.querySelector<HTMLElement>(".is-task");
    expect(days).toHaveBeenCalledWith(card);
    const pressed = vi.fn();
    card?.addEventListener("click", pressed);
    card?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    expect(pressed).toHaveBeenCalledTimes(1);
  });

  it("finds which task card a press came from, and unfolds that one alone", () => {
    const drawn = page(
      '<p><a class="card" data-task="T1">Uno</a></p><p><a class="card" data-task="T2">Dos</a></p>',
    );
    cardsPlaced(
      drawn,
      new Map([
        [cardKey(task("T1", "Uno")), OPEN],
        [cardKey(task("T2", "Dos")), OPEN.replace(/T1/g, "T2")],
      ]),
    );
    const second = drawn.querySelectorAll(".card-unfold")[1];

    expect(taskCardAt(second)).toBe(1);
    unfolded(drawn, 1);
    unfolded(drawn, 5);

    const cards = drawn.querySelectorAll(".is-task");
    expect(cards[0].classList.contains("unfolded")).toBe(false);
    expect(cards[1].classList.contains("unfolded")).toBe(true);
  });
});

import { describe, expect, it } from "vitest";
import { recarded, type TaskCarded, taskCarded, unfolded } from "../src/taskCards";
import type { ReadWord } from "../src/words";

const say = (word: ReadWord, ...filled: string[]) => [word, ...filled].join(" ");
const as = (seen: TaskCarded, acting = false) => ({ task: () => seen, say, acting });

const OPEN: TaskCarded = {
  kind: "open",
  title: "Pan integral",
  about: "Casa · mañana",
  steps: [
    { id: "s1", html: "Comprar <strong>harina</strong>", done: true },
    { id: "s2", html: "Amasar", done: false },
  ],
  parts: { closed: 1, all: 3 },
};

const paged = () => {
  const page = document.implementation.createHTMLDocument("");
  page.body.innerHTML =
    '<article class="page"><p><a class="card" data-task="01P">Comprar pan</a></p></article>';
  return page;
};

describe("a cited task's card", () => {
  it("is drawn waiting, then takes what the task says in place once it is known", () => {
    const page = paged();
    taskCarded(page, as({ kind: "waiting" }));
    const first = page.querySelector<HTMLElement>(".is-task[data-task='01P']");
    expect(first?.querySelector(".card-name")?.textContent).toBe("Comprar pan");

    recarded(page, as(OPEN));
    const now = page.querySelectorAll<HTMLElement>(".is-task[data-task='01P']");
    expect(now).toHaveLength(1);
    expect(now[0].classList.contains("is-open")).toBe(true);
    expect(now[0].querySelector(".card-name")?.textContent).toBe("Pan integral");
    expect(now[0].querySelector(".card-blurb")?.textContent).toBe("Casa · mañana");
    expect(now[0].querySelector(".card-unfold")?.textContent).toBe("partCardSteps 1 2");
    expect(now[0].querySelector(".card-meta")?.textContent).toContain("taskCardParts 1 3");
    expect(now[0].querySelector(".card-step-words strong")?.textContent).toBe("harina");
  });

  it("keeps its words and says it is not here when the task is gone, opening nothing", () => {
    const page = paged();
    taskCarded(page, as({ kind: "gone" }, true));
    const box = page.querySelector<HTMLElement>(".is-task.is-gone[data-task='01P']");
    expect(box?.querySelector(".card-name")?.textContent).toBe("Comprar pan");
    expect(box?.querySelector(".card-blurb")?.textContent).toBe("taskCardAway");
    expect(box?.getAttribute("role")).toBeNull();
    expect(box?.querySelector("button")).toBeNull();

    recarded(page, as(OPEN));
    expect(page.querySelector(".is-task .card-name")?.textContent).toBe("Pan integral");
  });

  it("offers the check and the boxes only on an open task, and only to a host that acts", () => {
    const page = paged();
    taskCarded(page, as(OPEN));
    expect(page.querySelector("[data-asked='taskDone']")).toBeNull();
    expect(page.querySelector("button.card-step-box")).toBeNull();

    recarded(page, as(OPEN, true));
    expect(page.querySelector("[data-asked='taskDone']")?.getAttribute("data-task")).toBe("01P");
    const boxes = page.querySelectorAll<HTMLElement>("button.card-step-box");
    expect(Array.from(boxes, (one) => one.dataset.step)).toEqual(["s1", "s2"]);
    expect(boxes[0].getAttribute("aria-checked")).toBe("true");

    const closed: TaskCarded = { ...OPEN, kind: "closed", dropped: false };
    recarded(page, as(closed, true));
    expect(page.querySelector("[data-asked='taskDone']")).toBeNull();
    expect(page.querySelector("button.card-step-box")).toBeNull();
    expect(page.querySelector(".chip.here")?.textContent).toBe("partCardDone");
  });

  it("keeps its steps unfolded and its keys when it is filled in place", () => {
    document.body.innerHTML = paged().body.innerHTML;
    taskCarded(document, as(OPEN));
    unfolded(document, 0);
    document.querySelector<HTMLElement>(".is-task")?.focus();
    recarded(document, as({ ...OPEN, title: "Pan de molde" }));
    const box = document.querySelector<HTMLElement>(".is-task");
    expect(box?.classList.contains("unfolded")).toBe(true);
    expect(box?.querySelector(".card-unfold")?.getAttribute("aria-expanded")).toBe("true");
    expect(document.activeElement).toBe(box);
    document.body.innerHTML = "";
  });

  it("leaves a key pressed on its own buttons to them, not to the card", () => {
    document.body.innerHTML = paged().body.innerHTML;
    taskCarded(document, as(OPEN, true));
    const box = document.querySelector<HTMLElement>(".is-task");
    let opened = 0;
    box?.addEventListener("click", (event) => {
      if (event.target === box) opened += 1;
    });
    const check = document.querySelector<HTMLElement>("[data-asked='taskDone']");
    check?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    expect(opened).toBe(0);
    box?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    expect(opened).toBe(1);
    document.body.innerHTML = "";
  });
});

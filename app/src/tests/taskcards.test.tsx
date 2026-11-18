import { CompletionContext } from "@codemirror/autocomplete";
import { markdown } from "@codemirror/lang-markdown";
import { EditorState } from "@codemirror/state";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { hosted, offered } from "tisty-editor/edit";
import { type CardAsked, cardKey } from "tisty-editor/read";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Citing } from "../citing";
import { dateOf, whenLabel } from "../format";
import { fill, t } from "../locales";
import CitedBy from "../ui/CitedBy";
import { answered, placedCards, type Reaching } from "../ui/dressing";
import { HOST } from "../ui/hosting";
import { cardDays, drawnFor } from "../ui/liveCards";
import { went } from "../ui/paging";
import TaskPick from "../ui/TaskPick";

const bus = vi.hoisted(() => ({
  citing: null as Citing[] | null,
  searched: [] as string[],
  found: [] as { id: string; title: string; status: string }[],
  drawn: [] as unknown[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: { query?: string; asked?: CardAsked[] }) => {
    if (cmd === "doc_cited_by") return Promise.resolve(bus.citing);
    if (cmd === "cards_drawn") {
      bus.drawn.push(args);
      return Promise.resolve(args?.asked?.map((one) => `<span>${one.id}</span>`));
    }
    if (cmd === "search") {
      bus.searched.push(args?.query ?? "");
      return Promise.resolve({ tasks: bus.found, papers: [], total: bus.found.length });
    }
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

// As the core draws an open task's card for a window that can mark it.
const OPEN =
  '<span class="doc-card is-task is-open" tabindex="0" role="link" data-task="01K"><span class="card-tile">' +
  '<button type="button" data-asked="taskDone" data-task="01K" aria-label="Marcarla hecha" class="card-check" title="Marcarla hecha"><svg class="glyph" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="10" /></svg></button></span>' +
  '<span class="card-name">Cambiar los neumáticos</span><span class="card-blurb">Auto · vence <time datetime="2099-01-05T00:00:00" data-say="when"></time></span>' +
  '<span class="card-meta"><button type="button" data-asked="taskSteps" data-task="01K" aria-label="0 de 1 pasos" class="chip card-unfold" aria-expanded="false">0 de 1 pasos</button><span class="card-go">›</span></span>' +
  '<span class="card-steps"><span class="card-step"><button type="button" data-asked="taskStep" data-task="01K" aria-label="Pedir hora" class="card-step-box" data-step="s1" role="checkbox" aria-checked="false"></button><span class="card-step-words">Pedir hora</span></span></span></span>';
const GONE =
  '<span class="doc-card is-task is-gone" data-task="01K"><span class="card-tile"></span><span class="card-name">Neumáticos</span>' +
  '<span class="card-blurb">Esa tarea no está en esta máquina</span><span class="card-meta"></span></span>';
const TYRES: CardAsked = { kind: "task", id: "01K", words: "Neumáticos" };
const LINKED =
  '<article class="page"><p data-at="0-30"><a class="card" data-task="01K">Neumáticos</a></p></article>';

describe("the cards the window asks the core for", () => {
  it("asks once for each card it shows, and keeps each drawing by what was asked", async () => {
    bus.drawn = [];
    const auto: CardAsked = { kind: "doc", id: "auto-0001", words: "Auto" };
    const seen = { here: "01W", acting: true, more: false, examples: false };
    const drawn = await drawnFor([TYRES, auto, TYRES], { tasks: true }, seen);
    expect(bus.drawn).toEqual([{ asked: [TYRES], seen }]);
    expect([...drawn]).toEqual([[cardKey(TYRES), "<span>01K</span>"]]);
    expect(await drawnFor([auto], undefined, seen)).toEqual(new Map());
    expect(await drawnFor([auto], { tasks: true }, seen)).toEqual(new Map());
    expect(bus.drawn).toHaveLength(1);
  });

  it("words a card's days in the window's own calendar", () => {
    const box = document.createElement("span");
    box.innerHTML =
      '<time datetime="2099-01-05T00:00:00" data-say="when" data-timed></time>' +
      '<time datetime="2026-03-15T12:00:00Z" data-say="day"></time><time datetime="nunca" data-say="day"></time>';
    cardDays(box);
    const [due, closed, never] = box.querySelectorAll("time");
    const at = "2099-01-05T00:00:00";
    expect(due.textContent).toBe(whenLabel({ at, tz: "", floating: true, has_time: true }));
    expect(closed.textContent).toBe(dateOf(Date.parse("2026-03-15T12:00:00Z") / 1000));
    expect(never.textContent).toBe("");
  });
});

const page = (drawn: string) => {
  const one = document.implementation.createHTMLDocument("");
  one.body.innerHTML = LINKED;
  const cards = new Map([[cardKey(TYRES), drawn]]);
  const reach = { body: "", onDoc: () => {}, drawnCards: cards, cardDays };
  placedCards(one, reach as unknown as Reaching);
  return one;
};

describe("a press on a cited task's card", () => {
  it("finishes the task, ticks a step or unfolds the steps, and never opens the block for writing", () => {
    const sheet = page(OPEN);
    const done = sheet.querySelector("[data-asked='taskDone']") as Element;
    const step = sheet.querySelector("[data-asked='taskStep']") as Element;
    const unfold = sheet.querySelector("[data-asked='taskSteps']") as Element;
    expect(went(done, false)).toEqual({
      kind: "taskDone",
      id: "01K",
      title: "Cambiar los neumáticos",
    });
    expect(went(step, false)).toEqual({ kind: "taskStep", id: "01K", step: "s1", done: true });
    expect(went(unfold, false)).toEqual({ kind: "taskSteps", nth: 0 });
    expect(went(sheet.querySelector(".card-name") as Element, false)).toEqual({
      kind: "task",
      id: "01K",
    });

    const onTaskDone = vi.fn();
    const onTaskStep = vi.fn();
    const reach = { body: "", onDoc: () => {}, onTaskDone, onTaskStep } as unknown as Reaching;
    answered(sheet, went(done, false), reach);
    answered(sheet, went(step, false), reach);
    answered(sheet, went(unfold, false), reach);
    expect(onTaskDone).toHaveBeenCalledWith("01K", "Cambiar los neumáticos");
    expect(onTaskStep).toHaveBeenCalledWith("01K", "s1", true);
    expect(sheet.querySelector(".is-task")?.classList.contains("unfolded")).toBe(true);
    expect(sheet.querySelector(".is-task time")?.textContent).not.toBe("");
  });

  it("is laid from what the core drew, and asked for while the core has not drawn it", () => {
    const sheet = document.implementation.createHTMLDocument("");
    sheet.body.innerHTML = LINKED;
    const onCardsMissing = vi.fn();
    const reach = { body: "", onDoc: () => {}, drawnCards: new Map(), onCardsMissing };
    placedCards(sheet, reach as unknown as Reaching);
    expect(onCardsMissing).toHaveBeenCalledWith([TYRES]);
    expect(sheet.querySelector("a.card[data-task]")).not.toBeNull();
    placedCards(sheet, { body: "", onDoc: () => {}, onCardsMissing } as unknown as Reaching);
    expect(onCardsMissing).toHaveBeenCalledTimes(1);
  });

  it("opens nothing on a task that is not here", () => {
    const sheet = page(GONE);
    const name = sheet.querySelector(".card-name") as Element;
    expect(went(name, false)).not.toMatchObject({ kind: "task" });
  });

  it("offers to take a page out only from a page's own card", () => {
    const sheet = document.implementation.createHTMLDocument("");
    const more = (file: string) =>
      `<button type="button" class="card-more" data-asked="more" data-target="tisty:doc/${file}">⋯</button>`;
    sheet.body.innerHTML =
      `<article class="page"><p data-at="0-20"><span class="doc-card is-page">${more("cap1")}</span></p>` +
      `<p data-at="21-40"><span class="doc-card is-doc">${more("otro")}</span></p></article>`;
    const [leaf, other] = sheet.querySelectorAll("[data-asked='more']");
    expect(went(leaf, false)).toMatchObject({
      kind: "more",
      target: "tisty:doc/cap1",
      paged: true,
    });
    expect(went(other, false)).toMatchObject({ kind: "more", paged: false });
  });
});

describe("the menu that writes a card", () => {
  it("offers a task only where the window can find one", () => {
    const shown = (kinds: ("task" | "doc")[]) => {
      const state = EditorState.create({ doc: "/", extensions: [hosted.of(HOST), markdown()] });
      const said = offered(() => ({ kinds, ask: () => {} }))(
        new CompletionContext(state, 1, false),
      );
      return said?.options.map((one) => one.label) ?? [];
    };
    expect(shown(["task"])).toContain(t("insertTask"));
    expect(shown([])).not.toContain(t("insertTask"));
  });
});

describe("finding a task to cite", () => {
  beforeEach(() => {
    bus.searched = [];
    bus.found = [];
  });

  it("asks once the words rest, among open and archived tasks, and writes the one picked", async () => {
    bus.found = [
      { id: "01A", title: "Cambiar los neumáticos", status: "open" },
      { id: "01B", title: "Cambiar el aceite", status: "done" },
    ];
    const onPick = vi.fn();
    render(<TaskPick onPick={onPick} />);
    expect(screen.getByText(t("pickATaskHint"))).toBeTruthy();

    await userEvent.type(screen.getByRole("combobox"), "camb");
    await waitFor(() => expect(screen.getByText("Cambiar el aceite")).toBeTruthy());
    expect(bus.searched).toEqual(["camb"]);
    expect(screen.getByText(t("paletteArchived"))).toBeTruthy();

    await userEvent.click(screen.getByText("Cambiar el aceite"));
    expect(onPick).toHaveBeenCalledWith(expect.objectContaining({ id: "01B" }));
  });

  it("keeps the last answer in sight while the next words are asked", async () => {
    bus.found = [{ id: "01A", title: "Cambiar los neumáticos", status: "open" }];
    render(<TaskPick onPick={() => {}} />);
    await userEvent.type(screen.getByRole("combobox"), "camb");
    await waitFor(() => expect(screen.getByText("Cambiar los neumáticos")).toBeTruthy());
    await userEvent.type(screen.getByRole("combobox"), "i");
    expect(screen.getByText("Cambiar los neumáticos")).toBeTruthy();
    expect(screen.queryByText(t("opening"))).toBeNull();
  });

  it("does not ask again when the window around it is drawn again", async () => {
    const { rerender } = render(<TaskPick onPick={() => {}} onError={() => {}} />);
    await userEvent.type(screen.getByRole("combobox"), "aceite");
    await waitFor(() => expect(bus.searched).toEqual(["aceite"]));
    rerender(<TaskPick onPick={() => {}} onError={() => {}} />);
    await new Promise((done) => setTimeout(done, 250));
    expect(bus.searched).toEqual(["aceite"]);
  });
});

describe("who cites a task, beside its page", () => {
  it("lists the documents and tasks that cite it, leaving out the whole above it", async () => {
    bus.citing = [
      {
        id: "dev_a-0002",
        kind: "doc",
        title: "Auto",
        card: true,
        state: "open",
        under: [{ id: "taller", title: "Taller" }],
      },
      { id: "01W", kind: "task", title: "Viaje al sur", card: true, state: "open" },
      { id: "01T", kind: "task", title: "Revisión anual", card: false, state: "done" },
    ];
    const onDoc = vi.fn();
    const onTask = vi.fn();
    render(<CitedBy task="01K" whole="01W" onDoc={onDoc} onTask={onTask} />);

    await waitFor(() => expect(screen.getByText("Auto")).toBeTruthy());
    expect(screen.queryByText("Viaje al sur")).toBeNull();
    expect(screen.getByText(t("citedBy"))).toBeTruthy();
    expect(screen.getByText(fill("citedUnder", "Taller"))).toBeTruthy();

    await userEvent.click(screen.getByText("Auto"));
    await userEvent.click(screen.getByText("Revisión anual"));
    expect(onDoc).toHaveBeenCalledWith("dev_a-0002");
    expect(onTask).toHaveBeenCalledWith("01T");
  });

  it("shows the first twelve and the rest behind one press", async () => {
    bus.citing = Array.from({ length: 20 }, (_, n) => ({
      id: `dev_a-${String(n + 10).padStart(4, "0")}`,
      kind: "doc" as const,
      title: `Documento ${n}`,
      card: true,
      state: "open",
    }));
    render(<CitedBy task="01K" />);
    await waitFor(() => expect(screen.getByText("Documento 0")).toBeTruthy());
    expect(screen.queryByText("Documento 12")).toBeNull();

    await userEvent.click(screen.getByText(fill("citedSeeAll", "20")));
    expect(screen.getByText("Documento 19")).toBeTruthy();
  });

  it("stays out of the way while nobody cites it or the base is still being read", () => {
    bus.citing = null;
    const { container } = render(<CitedBy task="01K" />);
    expect(container.textContent).toBe("");
  });
});

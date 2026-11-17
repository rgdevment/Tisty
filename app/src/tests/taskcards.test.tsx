import { CompletionContext } from "@codemirror/autocomplete";
import { markdown } from "@codemirror/lang-markdown";
import { EditorState } from "@codemirror/state";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { hosted, offered } from "tisty-editor/edit";
import { taskCarded } from "tisty-editor/read";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Cited, Citing } from "../citing";
import { fill, t } from "../locales";
import CitedBy from "../ui/CitedBy";
import { answered, citedCards, type Reaching } from "../ui/dressing";
import { HOST } from "../ui/hosting";
import { went } from "../ui/paging";
import { SAY } from "../ui/saying";
import TaskPick from "../ui/TaskPick";
import { cardOf, citedIn } from "../ui/taskCards";

const bus = vi.hoisted(() => ({
  citing: null as Citing[] | null,
  searched: [] as string[],
  found: [] as { id: string; title: string; status: string }[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: { query?: string }) => {
    if (cmd === "doc_cited_by") return Promise.resolve(bus.citing);
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

const TYRES: Cited = {
  id: "01K",
  title: "Cambiar los neumáticos",
  status: "open",
  list: "Auto",
  steps: [{ id: "s1", html: "Pedir hora", done: false }],
  part_of: "01W",
};

describe("what a cited task's card is told", () => {
  it("names the ids a text cites once each, as the core writes them", () => {
    expect(citedIn("![a](tisty:task/01kx) y [b](tisty:task/01KX) ![c](tisty:task/01AB)")).toEqual([
      "01AB",
      "01KX",
    ]);
  });

  it("says its list and its days while open, and its closing once archived", () => {
    const open = cardOf({
      ...TYRES,
      deadline: { at: "2099-01-05T00:00:00", tz: "UTC", floating: true, has_time: false },
    });
    expect(open.kind).toBe("open");
    expect(open.kind === "open" && open.about?.startsWith("Auto · ")).toBe(true);
    expect(cardOf(TYRES, "01W")).toMatchObject({ kind: "open", about: undefined });

    const closed = cardOf({ ...TYRES, status: "dropped", completed_at: "2026-03-15T12:00:00Z" });
    expect(closed).toMatchObject({ kind: "closed", dropped: true });
    expect(closed.kind === "closed" && closed.about).toContain("Auto · ");
    expect(cardOf(undefined)).toEqual({ kind: "gone" });
  });
});

const page = (acting: boolean, seen = cardOf(TYRES)) => {
  const one = document.implementation.createHTMLDocument("");
  one.body.innerHTML =
    '<article class="page"><p data-at="0-30"><a class="card" data-task="01K">Neumáticos</a></p></article>';
  taskCarded(one, { task: () => seen, say: SAY, acting });
  return one;
};

describe("a press on a cited task's card", () => {
  it("finishes the task, ticks a step or unfolds the steps, and never opens the block for writing", () => {
    const sheet = page(true);
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
  });

  it("is drawn by the window itself, with its controls only when it can finish tasks", () => {
    const sheet = document.implementation.createHTMLDocument("");
    sheet.body.innerHTML =
      '<article class="page"><p data-at="0-30"><a class="card" data-task="01K">Neumáticos</a></p></article>';
    const reach = { body: "", onDoc: () => {}, cited: () => cardOf(TYRES) } as unknown as Reaching;
    citedCards(sheet, reach);
    expect(sheet.querySelector(".is-task .card-name")?.textContent).toBe("Cambiar los neumáticos");
    expect(sheet.querySelector("[data-asked='taskDone']")).toBeNull();
  });

  it("opens nothing on a task that is not here", () => {
    const sheet = page(true, { kind: "gone" });
    const name = sheet.querySelector(".card-name") as Element;
    expect(went(name, false)).not.toMatchObject({ kind: "task" });
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

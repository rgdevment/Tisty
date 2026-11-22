import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Neuron } from "../brainsty";
import type { Snapshot, Task } from "../core";
import { adopt, t } from "../locales";
import { openedHere } from "../ui/BrainstyView";
import SideBrainsty from "../ui/SideBrainsty";

Object.defineProperty(window, "innerWidth", { configurable: true, value: 1440 });

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  task: (): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return answer(cmd);
  },
  convertFileSrc: (path: string) => `asset://localhost/${path}`,
}));

const heard = vi.hoisted(() => ({
  on: new Map<string, (event: { payload: unknown }) => void>(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, handler: (event: { payload: unknown }) => void) => {
    heard.on.set(name, handler);
    return Promise.resolve(() => heard.on.delete(name));
  },
  emit: () => Promise.resolve(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null),
  save: () => Promise.resolve(null),
  ask: () => Promise.resolve(true),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({ onDragDropEvent: () => Promise.resolve(() => {}) }),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    minimize: () => Promise.resolve(),
    toggleMaximize: () => Promise.resolve(),
    close: () => Promise.resolve(),
  }),
}));

const sigma = vi.hoisted(() => ({
  turns: [] as unknown[],
  killed: 0,
  stopped: 0,
  hands: null as null | { pick: (node: number) => void; open: (node: number) => void },
}));

vi.mock("../ui/brainsty/drawing", () => ({
  drawNetwork: () => ({
    place: () => {},
    retheme: () => {},
    zoom: () => 0.5,
    toScreen: (point: [number, number]) => point,
    nodeOnScreen: () => undefined,
    onFrame: () => () => {},
    beforeFrame: () => () => {},
    light: () => {},
    hide: () => {},
    beside: () => [],
    heed: (hands: typeof sigma.hands) => {
      sigma.hands = hands;
      return () => {
        sigma.hands = null;
      };
    },
    focus: () => {},
    turn: (turn: unknown) => sigma.turns.push(turn),
    turned: () => false,
    kill: () => sigma.killed++,
  }),
}));

vi.mock("../ui/brainsty/layingOut", () => ({
  layOut: () => () => sigma.stopped++,
}));

const bread: Task = {
  id: "01A",
  title: "buy bread",
  status: "open",
  priority: "unset",
  order: "a0",
  steps: [],
  log: [],
  volume: {},
};

const shot: Snapshot = {
  tasks: [bread],
  total: 1,
  soonest: {},
  ahead: [],
  routines: [],
  lists: [],
  tags: [],
  counts: {},
  locale: "en",
  agents: {},
};

function answer(cmd: string): Promise<unknown> {
  switch (cmd) {
    case "settle_in":
      return Promise.resolve({ ran: false, brought: false, agrees: true });
    case "docs":
      return Promise.resolve({ folders: [], docs: [] });
    case "sync_state":
      return Promise.resolve({ asked: true });
    case "snapshot":
      return Promise.resolve(shot);
    case "settings":
      return Promise.resolve({ quiet: [], holds: {}, shares: false, onlySharedAbove: 0 });
    case "owed":
      return Promise.resolve([]);
    case "task_of":
      return ipc.task();
    case "brain_network":
      return Promise.resolve({
        nodes: [{ id: "01A", kind: "task", title: "buy bread", state: "open" }],
        edges: [],
      });
    case "help_pages":
      return Promise.resolve([]);
    case "help_reference_here":
    case "help_reference":
      return Promise.resolve("");
    default:
      return Promise.resolve(null);
  }
}

const asked = () => ipc.calls.filter((one) => one.cmd === "brain_network");

const panel = () => screen.queryByRole("region", { name: t("brainsty") });

const opened = async () => {
  const user = userEvent.setup();
  render(<App />);
  await screen.findByText("buy bread");
  return user;
};

const neuron = (kind: Neuron["kind"], id = "x", title = "x"): Neuron => ({ id, kind, title });

const hands = () => ({
  setChosen: vi.fn(),
  choose: vi.fn(),
  opening: vi.fn(),
  setError: vi.fn(),
});

beforeEach(() => {
  localStorage.clear();
  ipc.calls = [];
  ipc.task = () => Promise.resolve(bread);
  heard.on.clear();
  Object.assign(sigma, { killed: 0, stopped: 0, hands: null, turns: [] });
});

describe("Brainsty from the main window", () => {
  it("opens the network in the content panel, with the sidebar still there", async () => {
    const user = await opened();

    await user.click(screen.getByRole("button", { name: t("brainsty") }));

    expect(panel()).toBeTruthy();
    expect(await screen.findByText("1 nodes · 0 connections")).toBeTruthy();
    expect(asked()).toEqual([{ cmd: "brain_network", args: { near: null } }]);
    expect(screen.getByRole("button", { name: t("brainsty") }).getAttribute("aria-current")).toBe(
      "page",
    );
    expect(screen.queryByText("buy bread")).toBeNull();
  });

  it("opens it near the task open beside the list, and leaves no task open beside it", async () => {
    const user = await opened();
    await user.click(screen.getByText("buy bread"));
    await screen.findByRole("button", { name: t("expand") });

    await user.click(screen.getByRole("button", { name: t("brainsty") }));

    expect(asked()[0].args).toEqual({ near: "01A" });
    expect(screen.queryByRole("button", { name: t("expand") })).toBeNull();
  });

  it("hands the keys to the network once it is drawn", async () => {
    const user = await opened();
    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    const network = await screen.findByRole("figure", { name: t("brainstyNetwork") });

    await waitFor(() => expect(document.activeElement).toBe(network));
    await user.keyboard("/");

    expect(document.activeElement).toBe(
      screen.getByRole("searchbox", { name: t("brainstySearch") }),
    );
  });

  it("stays as it is when Brainsty is pressed again", async () => {
    const user = await opened();

    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await screen.findByText("1 nodes · 0 connections");
    await user.click(screen.getByRole("button", { name: t("brainsty") }));

    expect(asked()).toHaveLength(1);
  });

  it("goes back to where it was with Escape, and stops everything it ran", async () => {
    const user = await opened();
    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await waitFor(() => expect(sigma.hands).not.toBeNull());

    await user.keyboard("{Escape}");

    expect(panel()).toBeNull();
    expect(await screen.findByText("buy bread")).toBeTruthy();
    expect(sigma.killed).toBe(1);
    expect(sigma.stopped).toBe(1);
  });

  it("brings back the task that was open beside the list with Escape", async () => {
    const user = await opened();
    await user.click(screen.getByText("buy bread"));
    await screen.findByRole("button", { name: t("expand") });
    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await waitFor(() => expect(sigma.hands).not.toBeNull());

    await user.keyboard("{Escape}");

    expect(panel()).toBeNull();
    expect(await screen.findByRole("button", { name: t("expand") })).toBeTruthy();
    await act(() => new Promise((done) => setTimeout(done, 50)));
    expect(screen.getByRole("button", { name: t("expand") })).toBeTruthy();
  });

  it("opens no task on Escape when none was open beside the list", async () => {
    const user = await opened();
    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await waitFor(() => expect(sigma.hands).not.toBeNull());

    await user.keyboard("{Escape}");

    expect(await screen.findByText("buy bread")).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("expand") })).toBeNull();
  });

  it("names only what the network draws, in English and in Spanish", () => {
    try {
      for (const tongue of ["en", "es"]) {
        adopt(tongue);
        expect(t("brainstyNetwork")).not.toMatch(/folder|list|carpeta|lista/i);
      }
      expect(t("brainstyNetwork")).toBe(
        "Red de tus documentos, páginas, tareas, etiquetas y archivos",
      );
    } finally {
      adopt();
    }
  });

  it("leaves Escape typed in the sidebar to the sidebar", async () => {
    const user = await opened();
    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await waitFor(() => expect(sigma.hands).not.toBeNull());

    await user.click(screen.getByLabelText(t("sideSearch")));
    await user.keyboard("{Escape}");

    expect(panel()).toBeTruthy();
    expect(sigma.killed).toBe(0);
  });

  it("opens what the network asks for in the same window", async () => {
    const user = await opened();
    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await waitFor(() => expect(sigma.hands).not.toBeNull());

    act(() => sigma.hands?.open(0));

    expect(await screen.findByRole("button", { name: t("expand") })).toBeTruthy();
    expect(panel()).toBeNull();
    await waitFor(() => expect(sigma.killed).toBe(1));
  });
});

describe("Brainsty on the folded sidebar", () => {
  it("shows when the network is the view in sight", () => {
    render(<SideBrainsty rail on onOpen={() => {}} />);

    expect(screen.getByRole("button", { name: t("brainsty") }).getAttribute("aria-current")).toBe(
      "page",
    );
  });
});

describe("what the network asks the main window to open", () => {
  it("opens a document or a page among the documents", () => {
    const doc = hands();
    openedHere(neuron("doc", "notes.md"), doc);
    openedHere(neuron("page", "notes-0002.md"), doc);

    expect(doc.setChosen.mock.calls).toEqual([
      [{ named: "docs", doc: "notes.md" }],
      [{ named: "docs", doc: "notes-0002.md" }],
    ]);
  });

  it("opens a tag among the tags by its name", () => {
    const tag = hands();
    openedHere(neuron("tag", "#home", "home"), tag);

    expect(tag.choose).toHaveBeenCalledWith({ named: "tags", tags: ["home"] });
  });

  it("opens a task where it lives, and says why when it cannot be read", async () => {
    const task = hands();
    openedHere(neuron("task", "01A"), task);
    await waitFor(() => expect(task.opening).toHaveBeenCalledWith(bread));
    expect(task.setChosen).toHaveBeenCalledWith({ named: "tasks", slice: "all" });

    ipc.task = () => Promise.reject({ code: "noSuchTask" });
    const gone = hands();
    openedHere(neuron("task", "01Z"), gone);
    await waitFor(() => expect(gone.setError).toHaveBeenCalled());
    expect(gone.opening).not.toHaveBeenCalled();
  });

  it("leaves a file where it is", () => {
    const file = hands();
    openedHere(neuron("file", "attachments/rina.jpg"), file);

    expect(
      [file.setChosen, file.choose, file.opening, file.setError].flatMap((one) => one.mock.calls),
    ).toEqual([]);
  });
});

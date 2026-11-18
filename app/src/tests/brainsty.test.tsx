import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Network } from "../brainsty";
import type { Snapshot, Task } from "../core";
import { t } from "../locales";

Object.defineProperty(window, "innerWidth", { configurable: true, value: 1440 });

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  network: (): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return answer(cmd);
  },
}));

const sigma = vi.hoisted(() => ({
  centred: [] as number[],
  placed: 0,
  rethemed: 0,
  killed: 0,
  stopped: 0,
  step: null as
    | null
    | ((step: {
        x: Float32Array;
        y: Float32Array;
        done: boolean;
        iterations: number;
        ms: number;
      }) => void),
}));

vi.mock("../ui/brainsty/drawing", () => ({
  drawNetwork: () => ({
    place: () => sigma.placed++,
    retheme: () => sigma.rethemed++,
    centre: (on: number) => sigma.centred.push(on),
    kill: () => sigma.killed++,
  }),
}));

vi.mock("../ui/brainsty/layingOut", () => ({
  layOut: (_: unknown, onStep: typeof sigma.step) => {
    sigma.step = onStep;
    return () => sigma.stopped++;
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
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
    case "brain_network":
      return ipc.network();
    case "help_pages":
      return Promise.resolve([]);
    case "help_reference_here":
    case "help_reference":
      return Promise.resolve("");
    default:
      return Promise.resolve(null);
  }
}

const small: Network = {
  nodes: [
    { id: "notes.md", kind: "doc", title: "Notes", state: "open" },
    { id: "01A", kind: "task", title: "buy bread", state: "open" },
    { id: "#home", kind: "tag", title: "home" },
  ],
  edges: [
    [0, 1, 0],
    [1, 2, 3],
  ],
  near: 1,
};

const asked = () => ipc.calls.filter((one) => one.cmd === "brain_network");

const opened = async () => {
  const user = userEvent.setup();
  render(<App />);
  await screen.findByText("buy bread");
  return user;
};

beforeEach(() => {
  localStorage.clear();
  ipc.calls = [];
  ipc.network = () => Promise.resolve(small);
  Object.assign(sigma, { centred: [], placed: 0, rethemed: 0, killed: 0, stopped: 0, step: null });
});

describe("the Brainsty section", () => {
  it("opens over the whole window from the sidebar, counts the network and closes with Escape", async () => {
    const user = await opened();

    await user.click(screen.getByRole("button", { name: t("brainsty") }));

    await screen.findByRole("dialog", { name: t("brainsty") });
    expect(await screen.findByText("3 nodes · 2 connections")).toBeTruthy();
    expect(await screen.findByRole("figure", { name: t("brainstyNetwork") })).toBeTruthy();
    expect(asked()).toEqual([{ cmd: "brain_network", args: { near: null } }]);

    await user.keyboard("{Escape}");

    expect(screen.queryByRole("dialog", { name: t("brainsty") })).toBeNull();
    expect(screen.getByText("buy bread")).toBeTruthy();
  });

  it("starts on the task open beside the list and leaves it open on the way back", async () => {
    const user = await opened();
    await user.click(screen.getByText("buy bread"));
    await screen.findByRole("button", { name: t("expand") });

    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await screen.findByText("3 nodes · 2 connections");
    await user.click(screen.getByRole("button", { name: new RegExp(`^${t("goBack")}`) }));

    expect(asked()[0].args).toEqual({ near: "01A" });
    expect(screen.queryByRole("dialog", { name: t("brainsty") })).toBeNull();
    expect(screen.getByRole("button", { name: t("expand") })).toBeTruthy();

    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await screen.findByText("3 nodes · 2 connections");
    await user.keyboard("{Escape}");

    expect(screen.queryByRole("dialog", { name: t("brainsty") })).toBeNull();
    expect(screen.getByRole("button", { name: t("expand") })).toBeTruthy();
  });

  it("leaves Escape to a dialog that opens above it", async () => {
    const user = await opened();
    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await screen.findByText("3 nodes · 2 connections");
    const above = document.createElement("div");
    above.setAttribute("role", "dialog");
    above.setAttribute("aria-modal", "true");
    document.body.append(above);

    await user.keyboard("{Escape}");
    expect(screen.getByRole("dialog", { name: t("brainsty") })).toBeTruthy();

    above.remove();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog", { name: t("brainsty") })).toBeNull();
  });

  it("keeps the window's own shortcuts from acting on the view it covers", async () => {
    const user = await opened();
    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await screen.findByText("3 nodes · 2 connections");

    await user.keyboard("{Control>}b{/Control}");
    await user.keyboard("{Control>}k{/Control}");
    await user.keyboard("{Escape}");

    expect(document.getElementById("side-finder")).not.toBeNull();
    expect(document.activeElement?.id).not.toBe("side-finder");
  });

  it("says so while the base is still being built", async () => {
    ipc.network = () => Promise.resolve(null);
    const user = await opened();

    await user.click(screen.getByRole("button", { name: t("brainsty") }));

    expect(await screen.findByText(t("brainstyNotYet"))).toBeTruthy();
  });

  it("says there is nothing to draw yet when nothing is in sight", async () => {
    ipc.network = () => Promise.resolve({ nodes: [], edges: [] });
    const user = await opened();

    await user.click(screen.getByRole("button", { name: t("brainsty") }));

    expect(await screen.findByText(t("brainstyEmpty"))).toBeTruthy();
  });

  it("tells what the cap left out", async () => {
    ipc.network = () => Promise.resolve({ ...small, left_out: { nodes: 1200, edges: 34 } });
    const user = await opened();

    await user.click(screen.getByRole("button", { name: t("brainsty") }));

    const out = await screen.findByText("1,200 nodes and 34 connections left out");
    expect(out.getAttribute("title")).toBe(t("brainstyLeftOutWhy"));
  });

  it("shows why when the network could not be read", async () => {
    ipc.network = () => Promise.reject({ code: "internal" });
    const user = await opened();

    await user.click(screen.getByRole("button", { name: t("brainsty") }));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).not.toBe("");
  });

  it("is gone once something else is chosen beneath it", async () => {
    const user = await opened();
    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await screen.findByText("3 nodes · 2 connections");

    await user.keyboard("{F1}");

    await waitFor(() => expect(screen.queryByRole("dialog", { name: t("brainsty") })).toBeNull());
  });

  it("draws the network centred on where it opens, following the layout until the person moves it", async () => {
    const user = await opened();
    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    const canvas = await screen.findByRole("figure", { name: t("brainstyNetwork") });
    const spots = { x: new Float32Array(3), y: new Float32Array(3) };

    sigma.step?.({ ...spots, done: false, iterations: 10, ms: 5 });
    await waitFor(() => expect(sigma.centred).toEqual([1, 1]));
    canvas.dispatchEvent(new Event("pointerdown"));
    sigma.step?.({ ...spots, done: true, iterations: 20, ms: 12.4 });
    await new Promise((frame) => requestAnimationFrame(frame));

    expect(sigma.centred).toEqual([1, 1]);
    expect(sigma.placed).toBe(2);
    expect(canvas.dataset.laidMs).toBe("12");
    expect(canvas.dataset.iterations).toBe("20");
    expect(canvas.dataset.drawnMs).toMatch(/^\d+$/);
    expect(performance.getEntriesByName("brainsty-opened")).toHaveLength(0);
  });

  it("redraws in the other theme and lets go of the drawing and the layout on closing", async () => {
    const user = await opened();
    await user.click(screen.getByRole("button", { name: t("brainsty") }));
    await screen.findByRole("figure", { name: t("brainstyNetwork") });

    document.documentElement.setAttribute("data-theme", "dark");
    await waitFor(() => expect(sigma.rethemed).toBe(1));
    document.documentElement.removeAttribute("data-theme");
    await user.keyboard("{Escape}");

    expect(sigma.killed).toBe(1);
    expect(sigma.stopped).toBe(1);
  });
});

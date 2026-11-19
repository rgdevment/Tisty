import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import kept from "../../src-tauri/src/brainsty_window.rs?raw";
import BrainstyWindow from "../BrainstyWindow";
import type { Group, Network } from "../brainsty";
import { adopt, t } from "../locales";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  network: (): Promise<unknown> => Promise.resolve(null),
  groups: [] as unknown[],
  glance: "",
  open: (): Promise<unknown> => Promise.resolve(null),
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

const shell = vi.hoisted(() => ({ shown: 0, closed: 0 }));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    show: () => {
      shell.shown++;
      return Promise.resolve();
    },
    close: () => {
      shell.closed++;
      return Promise.resolve();
    },
  }),
}));

const sigma = vi.hoisted(() => ({
  centred: [] as number[],
  placed: 0,
  rethemed: 0,
  killed: 0,
  stopped: 0,
  focused: [] as number[],
  lit: [] as (number[] | null)[],
  hidden: [] as boolean[][],
  hands: null as null | {
    pick: (node: number) => void;
    open: (node: number) => void;
    clear: () => void;
  },
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
    zoom: () => 0.5,
    toScreen: (point: [number, number]) => point,
    toGraph: (point: [number, number]) => point,
    nodeOnScreen: () => undefined,
    onFrame: () => () => {},
    light: (nodes: ReadonlySet<number> | null) => sigma.lit.push(nodes ? [...nodes] : null),
    hide: (hidden: boolean[]) => sigma.hidden.push(hidden),
    beside: (node: number) => (node === 0 ? [1] : []),
    heed: (hands: typeof sigma.hands) => {
      sigma.hands = hands;
      return () => {
        sigma.hands = null;
      };
    },
    focus: (node: number) => sigma.focused.push(node),
    kill: () => sigma.killed++,
  }),
}));

vi.mock("../ui/brainsty/layingOut", () => ({
  layOut: (_: unknown, onStep: typeof sigma.step) => {
    sigma.step = onStep;
    return () => sigma.stopped++;
  },
}));

function answer(cmd: string): Promise<unknown> {
  switch (cmd) {
    case "brain_network":
      return ipc.network();
    case "brainsty_groups":
      return Promise.resolve(ipc.groups);
    case "brainsty_open":
      return ipc.open();
    case "doc_glance":
      return Promise.resolve({ title: "Notes", heading: null, html: ipc.glance, more: false });
    case "served":
      return Promise.resolve("C:/home/data/attachments/rina.jpg");
    case "doc_drawn":
      return Promise.resolve(
        new TextEncoder().encode('<p><a class="card" data-task="01A">buy bread</a></p>').buffer,
      );
    case "cards_drawn":
      return Promise.resolve(['<span class="doc-card is-task">buy bread</span>']);
    default:
      return Promise.resolve(null);
  }
}

const small: Network = {
  nodes: [
    { id: "notes.md", kind: "doc", title: "Notes", group: "f1", state: "open" },
    { id: "01A", kind: "task", title: "buy bread", state: "open" },
    { id: "#home", kind: "tag", title: "home" },
  ],
  edges: [
    [0, 1, 0],
    [1, 2, 3],
  ],
  near: 1,
};

const guessing: Network = {
  ...small,
  nodes: [
    ...small.nodes,
    { id: "plans.md", kind: "doc", title: "Plans", state: "open" },
    { id: "01B", kind: "task", title: "buy milk", state: "open" },
  ],
  alike: [
    [1, 4, 600, "buy groceries"],
    [3, 1, 900, "plans bread"],
    [3, 4, 500, "bread"],
  ],
};

const PROJECTS: Group = { id: "f1", name: "Proyectos", color: "teal" };

const last = <T,>(all: T[]): T | undefined => all[all.length - 1];

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const opened = async (search = "?locale=en&near=01A") => {
  window.history.replaceState(null, "", `/${search}`);
  const user = userEvent.setup();
  const shown = render(<BrainstyWindow />);
  return { user, shown };
};

const drawnNetwork = async (search?: string) => {
  const { user, shown } = await opened(search);
  const network = await screen.findByRole("figure", { name: t("brainstyNetwork") });
  await waitFor(() => expect(sigma.hands).not.toBeNull());
  return { user, shown, network };
};

const allowed = [
  ...(kept.match(/const ASKED[^=]*= \[([^\]]*)\]/)?.[1] ?? "").matchAll(/"(\w+)"/g),
].map(([, name]) => name);

beforeEach(() => {
  ipc.calls = [];
  ipc.network = () => Promise.resolve(small);
  ipc.groups = [PROJECTS];
  ipc.open = () => Promise.resolve(null);
  ipc.glance = '<h2 id="bread" data-at="3">Bread</h2><p>About bread</p>';
  heard.on.clear();
  shell.shown = 0;
  shell.closed = 0;
  Object.assign(sigma, {
    centred: [],
    placed: 0,
    rethemed: 0,
    killed: 0,
    stopped: 0,
    step: null,
    focused: [],
    lit: [],
    hidden: [],
    hands: null,
  });
});

afterEach(() => {
  adopt("en");
  window.history.replaceState(null, "", "/");
});

describe("Brainsty's own window", () => {
  it("shows itself, counts the network near what it was opened on and closes with Escape", async () => {
    const { user } = await opened();

    expect(await screen.findByText("3 nodes · 2 connections")).toBeTruthy();
    expect(shell.shown).toBe(1);
    expect(asked("brain_network")).toEqual([{ cmd: "brain_network", args: { near: "01A" } }]);
    expect(asked("brainsty_groups")).toHaveLength(1);

    await user.keyboard("{Escape}");

    expect(shell.closed).toBe(1);
  });

  it("opens on what was last touched when nothing is open beside it", async () => {
    await opened("?locale=en");

    await screen.findByText("3 nodes · 2 connections");
    expect(asked("brain_network")[0].args).toEqual({ near: null });
  });

  it("speaks the language it was opened in, and the app's when it changes", async () => {
    await drawnNetwork("?locale=es");
    expect(screen.getByText(t("brainstyKeys")).textContent).toMatch(/Esc/);
    expect(document.documentElement.lang).toBe("es");
    const spanish = t("brainstyKeys");

    act(() => heard.on.get("spoken")?.({ payload: "en" }));

    expect(screen.getByText(t("brainstyKeys")).textContent).not.toBe(spanish);
    expect(document.documentElement.lang).toBe("en");
    act(() => heard.on.get("spoken")?.({ payload: null }));
    expect(document.documentElement.lang).toBe(navigator.language);
  });

  it("stops hearing the app's language once it is gone", async () => {
    const { shown } = await drawnNetwork();
    expect(heard.on.has("spoken")).toBe(true);

    shown.unmount();
    await waitFor(() => expect(heard.on.has("spoken")).toBe(false));
  });

  it("says so while the base is still being built", async () => {
    ipc.network = () => Promise.resolve(null);
    await opened();

    expect(await screen.findByText(t("brainstyNotYet"))).toBeTruthy();
  });

  it("says there is nothing to draw yet when nothing is in sight", async () => {
    ipc.network = () => Promise.resolve({ nodes: [], edges: [] });
    await opened();

    expect(await screen.findByText(t("brainstyEmpty"))).toBeTruthy();
  });

  it("tells what the cap left out", async () => {
    ipc.network = () => Promise.resolve({ ...small, left_out: { nodes: 1200, edges: 34 } });
    await opened();

    const out = await screen.findByText("1,200 nodes and 34 connections left out");
    expect(out.getAttribute("title")).toBe(t("brainstyLeftOutWhy"));
  });

  it("shows why when the network could not be read", async () => {
    ipc.network = () => Promise.reject({ code: "internal" });
    await opened();

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).not.toBe("");
  });

  it("draws the network centred on where it opens, following the layout until the person moves it", async () => {
    const { network } = await drawnNetwork();
    const spots = { x: new Float32Array(3), y: new Float32Array(3) };

    sigma.step?.({ ...spots, done: false, iterations: 10, ms: 5 });
    await waitFor(() => expect(sigma.centred).toEqual([1, 1]));
    network.dispatchEvent(new Event("pointerdown"));
    sigma.step?.({ ...spots, done: true, iterations: 20, ms: 12.4 });
    await new Promise((frame) => requestAnimationFrame(frame));

    expect(sigma.centred).toEqual([1, 1]);
    expect(sigma.placed).toBe(2);
    expect(network.dataset.laidMs).toBe("12");
    expect(network.dataset.iterations).toBe("20");
    expect(network.dataset.drawnMs).toMatch(/^\d+$/);
    expect(performance.getEntriesByName("brainsty-opened")).toHaveLength(0);
  });

  it("redraws in the other theme and lets go of the drawing and the layout once it is gone", async () => {
    const { shown } = await drawnNetwork();

    document.documentElement.setAttribute("data-theme", "dark");
    await waitFor(() => expect(sigma.rethemed).toBe(1));
    document.documentElement.removeAttribute("data-theme");
    shown.unmount();

    expect(sigma.killed).toBe(1);
    expect(sigma.stopped).toBe(1);
  });

  it("previews a picked document in its folder, and asks the main window to open it", async () => {
    const { user } = await drawnNetwork();

    act(() => sigma.hands?.pick(0));

    const preview = await screen.findByRole("complementary", { name: "Notes" });
    expect(preview.textContent).toContain("Proyectos");
    expect(preview.textContent).toContain("1 connections");
    expect(await screen.findByText("About bread")).toBeTruthy();
    expect(preview.querySelector("[id], [data-at]")).toBeNull();
    expect(last(sigma.lit)).toEqual([0, 1]);
    await user.click(screen.getByRole("button", { name: t("brainstyOpen") }));

    expect(asked("brainsty_open")).toEqual([
      {
        cmd: "brainsty_open",
        args: { neuron: { kind: "doc", id: "notes.md", title: "Notes" } },
      },
    ]);
    expect(shell.closed).toBe(0);
  });

  it("says why when the main window could not be asked to open it", async () => {
    const { user } = await drawnNetwork();
    const refused = ipc.calls.length;
    ipc.open = () => Promise.reject({ code: "internal" });

    act(() => sigma.hands?.pick(0));
    await user.click(await screen.findByRole("button", { name: t("brainstyOpen") }));

    expect((await screen.findByRole("alert")).textContent).not.toBe("");
    expect(ipc.calls.length).toBeGreaterThan(refused);
    ipc.open = () => Promise.resolve(null);
    await user.click(screen.getByRole("button", { name: t("brainstyOpen") }));
    await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
  });

  it("keeps its page when a link in a preview is pressed", async () => {
    ipc.glance = '<p>See <a href="https://example.com/far">the far site</a></p>';
    const { user } = await drawnNetwork();
    act(() => sigma.hands?.pick(0));
    const link = await screen.findByText("the far site");
    const went = vi.fn((e: Event) => e.defaultPrevented);
    document.addEventListener("click", went);

    document.addEventListener("auxclick", went);
    await user.click(link);
    await user.pointer({ keys: "[MouseMiddle]", target: link });

    document.removeEventListener("click", went);
    document.removeEventListener("auxclick", went);
    expect(went).toHaveBeenCalledTimes(2);
    expect(went.mock.results.every((one) => one.value === true)).toBe(true);
  });

  it("asks the main window to open a task on a double click", async () => {
    await drawnNetwork();

    act(() => sigma.hands?.open(1));

    await waitFor(() =>
      expect(asked("brainsty_open")[0]?.args).toEqual({
        neuron: { kind: "task", id: "01A", title: "buy bread" },
      }),
    );
  });

  it("lets go of the search, then the picked node, then the window with each Escape", async () => {
    const { user } = await drawnNetwork();

    await user.type(screen.getByRole("searchbox", { name: t("brainstySearch") }), "bread");
    await user.click(await screen.findByRole("button", { name: /buy bread/ }));
    expect(sigma.focused).toEqual([1]);
    await waitFor(() =>
      expect(ipc.calls).toContainEqual({
        cmd: "doc_drawn",
        args: { body: "![buy bread](tisty:task/01A)\n" },
      }),
    );
    const frame = await waitFor(() => {
      const found = document.querySelector<HTMLIFrameElement>("aside iframe");
      expect(found?.srcdoc).toContain("<article");
      return found as HTMLIFrameElement;
    });
    const card = frame.contentDocument as Document;
    card.open();
    card.write(frame.srcdoc);
    card.close();
    frame.dispatchEvent(new Event("load"));
    await waitFor(() => expect(card.documentElement.classList).toContain("glanced"));
    expect(card.documentElement.classList).toContain("untitled");

    await user.type(screen.getByRole("searchbox", { name: t("brainstySearch") }), "zzz");
    expect(await screen.findByText(t("brainstyNothingFound"))).toBeTruthy();
    await user.keyboard("{Escape}");
    expect(screen.queryByText(t("brainstyNothingFound"))).toBeNull();
    expect(screen.getByRole("complementary", { name: "buy bread" })).toBeTruthy();
    act(() => {
      card.body.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    });
    expect(screen.queryByRole("complementary", { name: "buy bread" })).toBeNull();
    expect(shell.closed).toBe(0);
    await user.keyboard("{Escape}");
    expect(shell.closed).toBe(1);
  });

  it("filters the network by kind, and shows everything again", async () => {
    const { user } = await drawnNetwork();

    act(() => sigma.hands?.pick(1));
    expect(await screen.findByRole("complementary", { name: "buy bread" })).toBeTruthy();
    await user.click(screen.getByRole("button", { name: t("brainstyFilters") }));
    await user.click(screen.getByRole("button", { name: t("brainstyKind_task") }));
    expect(last(sigma.hidden)).toEqual([false, true, false]);
    expect(screen.queryByRole("complementary", { name: "buy bread" })).toBeNull();
    await user.selectOptions(screen.getByRole("combobox"), "#home");
    expect(last(sigma.hidden)).toEqual([true, true, false]);
    await user.click(screen.getByRole("button", { name: t("brainstyEverything") }));
    expect(last(sigma.hidden)).toEqual([false, false, false]);
  });

  it("filters the network by state and period", async () => {
    const today = new Date().toISOString().slice(0, 10);
    ipc.network = () =>
      Promise.resolve({
        ...small,
        nodes: [
          { ...small.nodes[0], when: today },
          { ...small.nodes[1], state: "done", when: "2020-01-01" },
          small.nodes[2],
        ],
      });
    const { user } = await drawnNetwork();
    await user.click(screen.getByRole("button", { name: t("brainstyFilters") }));

    await user.click(screen.getByRole("button", { name: t("brainstyStanding_done") }));
    expect(last(sigma.hidden)).toEqual([false, true, false]);
    await user.click(screen.getByRole("button", { name: t("brainstyEverything") }));
    expect(last(sigma.hidden)).toEqual([false, false, false]);
    await user.click(screen.getByRole("button", { name: t("brainstyPeriod_7") }));
    expect(last(sigma.hidden)).toEqual([false, true, false]);
    await user.click(screen.getByRole("button", { name: t("brainstyPeriod_0") }));
    expect(last(sigma.hidden)).toEqual([false, false, false]);
  });

  it("brings the first find close with Enter in the search, and opens it with Enter", async () => {
    const { user, network } = await drawnNetwork();

    await user.type(screen.getByRole("searchbox", { name: t("brainstySearch") }), "bread{Enter}");

    expect(sigma.focused).toEqual([1]);
    expect(screen.getByRole<HTMLInputElement>("searchbox").value).toBe("");
    expect(await screen.findByRole("complementary", { name: "buy bread" })).toBeTruthy();
    expect(document.activeElement).toBe(network);
    await user.keyboard("{Enter}");
    await waitFor(() => expect(asked("brainsty_open")).toHaveLength(1));
  });

  it("hands the keys back to the network when Escape clears the search", async () => {
    const { user, network } = await drawnNetwork();

    await user.type(screen.getByRole("searchbox", { name: t("brainstySearch") }), "bre");
    await user.keyboard("{Escape}");

    expect(screen.getByRole<HTMLInputElement>("searchbox").value).toBe("");
    expect(document.activeElement).toBe(network);
    expect(shell.closed).toBe(0);
  });

  it("reaches the network and its controls with Tab, in the order they stand", async () => {
    const { user, network } = await drawnNetwork();

    await user.tab();
    expect(document.activeElement).toBe(network);
    await user.tab();
    expect(document.activeElement).toBe(screen.getByRole("searchbox"));
    await user.tab();
    expect(document.activeElement).toBe(screen.getByRole("button", { name: t("brainstyFilters") }));
  });

  it("shows the pictures a document's preview holds", async () => {
    ipc.glance = '<p>Rina</p><img src="attachments/rina.jpg" alt="Rina on the hill">';
    await drawnNetwork();

    act(() => sigma.hands?.pick(0));

    const preview = await screen.findByRole("complementary", { name: "Notes" });
    await waitFor(() =>
      expect(preview.querySelector("img")?.getAttribute("src")).toBe(
        "asset://localhost/C:/home/data/attachments/rina.jpg",
      ),
    );
    expect(ipc.calls).toContainEqual({
      cmd: "served",
      args: { reference: "attachments/rina.jpg" },
    });
  });

  it("jumps to the search with /", async () => {
    const { user } = await drawnNetwork();

    await user.keyboard("/");

    expect(document.activeElement).toBe(
      screen.getByRole("searchbox", { name: t("brainstySearch") }),
    );
  });

  it("leaves an empty search with Escape before the picked node and the window", async () => {
    const { user, network } = await drawnNetwork();
    await user.keyboard("/");
    await user.keyboard("{Escape}");
    expect(document.activeElement).toBe(network);
    expect(shell.closed).toBe(0);

    act(() => sigma.hands?.pick(0));
    await screen.findByRole("complementary", { name: "Notes" });
    await user.keyboard("/");
    await user.keyboard("{Escape}");

    expect(document.activeElement).toBe(network);
    expect(screen.getByRole("complementary", { name: "Notes" })).toBeTruthy();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("complementary", { name: "Notes" })).toBeNull();
    expect(shell.closed).toBe(0);
    await user.keyboard("{Escape}");
    expect(shell.closed).toBe(1);
  });

  it("asks only for the commands its window is allowed", async () => {
    ipc.glance = '<p>Rina</p><img src="attachments/rina.jpg" alt="Rina on the hill">';
    const { user } = await drawnNetwork();
    act(() => sigma.hands?.pick(0));
    await screen.findByRole("complementary", { name: "Notes" });
    await waitFor(() => expect(asked("served")).toHaveLength(1));
    await user.click(screen.getByRole("button", { name: t("brainstyOpen") }));
    act(() => sigma.hands?.pick(1));
    await waitFor(() => expect(asked("cards_drawn").length).toBeGreaterThan(0));

    expect(allowed).toContain("brain_network");
    const used = [...new Set(ipc.calls.map((one) => one.cmd))];
    expect(used.filter((one) => !allowed.includes(one))).toEqual([]);
  });

  it("lists what Tisty guesses beside a picked node, strongest first, with why, and the filter takes them away", async () => {
    ipc.network = () => Promise.resolve(guessing);
    const { user } = await drawnNetwork();

    act(() => sigma.hands?.pick(1));

    const preview = await screen.findByRole("complementary", { name: "buy bread" });
    const alike = within(preview).getByRole("region", { name: t("brainstyAlike") });
    const named = within(alike)
      .getAllByRole("button")
      .map((one) => one.textContent);
    expect(named).toEqual(["Plans", "buy milk"]);
    expect(alike.textContent).toContain("Tisty guesses: they share plans and bread");
    expect(alike.textContent).toContain("Tisty guesses: they share buy and groceries");
    expect([...alike.querySelectorAll("em")].map((one) => one.textContent)).toEqual([
      "plans",
      "bread",
      "buy",
      "groceries",
    ]);
    expect(last(sigma.lit)).toEqual([1, 3, 4]);

    await user.click(within(alike).getByRole("button", { name: "Plans" }));
    expect(await screen.findByRole("complementary", { name: "Plans" })).toBeTruthy();
    expect(last(sigma.focused)).toBe(3);

    act(() => sigma.hands?.pick(1));
    await user.click(screen.getByRole("button", { name: t("brainstyFilters") }));
    await user.click(screen.getByRole("button", { name: t("brainstyAlike") }));
    const plain = await screen.findByRole("complementary", { name: "buy bread" });
    expect(within(plain).queryByRole("region", { name: t("brainstyAlike") })).toBeNull();
    expect(last(sigma.lit)).toEqual([1]);
    expect(screen.getByRole("button", { name: t("brainstyEverything") })).toBeTruthy();
  });

  it("says why in Spanish too", async () => {
    ipc.network = () => Promise.resolve(guessing);
    await drawnNetwork("?locale=es&near=01A");

    act(() => sigma.hands?.pick(4));

    const alike = await screen.findByRole("region", { name: "Parecidos" });
    expect(alike.textContent).toContain("Tisty lo adivina: comparten buy y groceries");
  });

  it("leaves out a guess whose other end the filters hide", async () => {
    ipc.network = () => Promise.resolve(guessing);
    const { user } = await drawnNetwork();
    act(() => sigma.hands?.pick(1));
    await screen.findByRole("region", { name: t("brainstyAlike") });

    await user.click(screen.getByRole("button", { name: t("brainstyFilters") }));
    await user.click(screen.getByRole("button", { name: t("brainstyKind_doc") }));

    const alike = await screen.findByRole("region", { name: t("brainstyAlike") });
    expect(
      within(alike)
        .getAllByRole("button")
        .map((one) => one.textContent),
    ).toEqual(["buy milk"]);
  });
});

import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, onTestFinished, vi } from "vitest";
import type { Group, Network, Neuron } from "../brainsty";
import { adopt, t } from "../locales";
import BrainstyView from "../ui/BrainstyView";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  network: (): Promise<unknown> => Promise.resolve(null),
  card: null as unknown,
  file: null as unknown,
  opening: (): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return answer(cmd);
  },
  convertFileSrc: (path: string) => `asset://localhost/${path}`,
}));

const clip = vi.hoisted(() => ({ taken: [] as string[] }));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: (text: string) => {
    clip.taken.push(text);
    return Promise.resolve();
  },
}));

const ways = vi.hoisted(() => ({ opened: [] as unknown[], left: 0 }));

const sigma = vi.hoisted(() => ({
  zoom: 0.5,
  turns: [] as unknown[],
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
    zoom: () => sigma.zoom,
    toScreen: (point: [number, number]) => point,
    toGraph: (point: [number, number]) => point,
    nodeOnScreen: () => undefined,
    onFrame: () => () => {},
    beforeFrame: () => () => {},
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
    turn: (turn: unknown) => sigma.turns.push(turn),
    turned: () => false,
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
    case "brain_card":
      return Promise.resolve(ipc.card);
    case "brain_file":
      return Promise.resolve(ipc.file);
    case "served":
      return Promise.resolve("C:/kept/map.png");
    case "opened":
      return ipc.opening();
    case "doc_address":
      return Promise.resolve("tisty://doc/Proyectos/Notes?id=notes.md");
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

const MAP = "attachments/map-1a2b3c4d.png";

const filed: Network = {
  ...small,
  nodes: [...small.nodes, { id: MAP, kind: "file", title: "map-1a2b3c4d.png" }],
  edges: [...small.edges, [0, 3, 4]],
};

const PROJECTS: Group = { id: "f1", name: "Proyectos", color: "teal" };

const last = <T,>(all: T[]): T | undefined => all[all.length - 1];

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const opened = async (near: string | null = "01A") => {
  const user = userEvent.setup();
  const shown = render(
    <BrainstyView
      near={near ?? undefined}
      groups={[PROJECTS]}
      onOpen={(one: Neuron) => ways.opened.push(one)}
      onLeave={() => ways.left++}
    />,
  );
  return { user, shown };
};

const drawnNetwork = async () => {
  const { user, shown } = await opened();
  const network = await screen.findByRole("figure", { name: t("brainstyNetwork") });
  await waitFor(() => expect(sigma.hands).not.toBeNull());
  return { user, shown, network };
};

beforeEach(() => {
  ipc.calls = [];
  ipc.network = () => Promise.resolve(small);
  ipc.file = null;
  ipc.opening = () => Promise.resolve(null);
  ipc.card = {
    id: "notes.md",
    kind: "doc",
    title: "Notes",
    state: "open",
    bytes: 2400,
    words: 380,
    wrote: "2026-11-18T15:00:00Z",
    lang: "en",
    book: { id: "plans.md", kind: "doc", title: "Plans" },
    tags: ["home"],
    cited_by: [{ id: "01A", kind: "task", title: "buy bread" }],
  };
  clip.taken = [];
  ways.opened = [];
  ways.left = 0;
  Object.assign(sigma, {
    zoom: 0.5,
    placed: 0,
    rethemed: 0,
    killed: 0,
    turns: [],
    stopped: 0,
    step: null,
    focused: [],
    lit: [],
    hidden: [],
    hands: null,
  });
});

describe("Brainsty in the content panel", () => {
  it("counts the network near what it was opened on and leaves with Escape", async () => {
    const { user } = await opened();

    expect(await screen.findByText("3 nodes · 2 connections")).toBeTruthy();
    expect(asked("brain_network")).toEqual([{ cmd: "brain_network", args: { near: "01A" } }]);

    await user.keyboard("{Escape}");

    expect(ways.left).toBe(1);
  });

  it("redraws when the panel changes width while the window does not", async () => {
    const seen: { told?: () => void; gone: boolean } = { gone: false };
    vi.stubGlobal(
      "ResizeObserver",
      class {
        constructor(told: () => void) {
          seen.told = told;
        }
        observe() {}
        disconnect() {
          seen.gone = true;
        }
      },
    );
    const resized = vi.fn();
    window.addEventListener("resize", resized);
    const { shown } = await opened();

    seen.told?.();
    seen.told?.();
    await new Promise((frame) => requestAnimationFrame(frame));
    shown.unmount();

    window.removeEventListener("resize", resized);
    vi.unstubAllGlobals();
    expect(resized).toHaveBeenCalledTimes(1);
    expect(seen.gone).toBe(true);
  });

  it("leaves Escape to what sits over it", async () => {
    await opened();
    await screen.findByText("3 nodes · 2 connections");
    const shut = (e: KeyboardEvent) => e.preventDefault();
    document.addEventListener("keydown", shut);

    document.body.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }),
    );

    document.removeEventListener("keydown", shut);
    expect(ways.left).toBe(0);
  });

  it("opens on what was last touched when nothing is open beside it", async () => {
    await opened(null);

    await screen.findByText("3 nodes · 2 connections");
    expect(asked("brain_network")[0].args).toEqual({ near: null });
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

  it("opens on the whole network, following the layout without bringing the camera close", async () => {
    const { network } = await drawnNetwork();
    const spots = { x: new Float32Array(3), y: new Float32Array(3) };

    sigma.step?.({ ...spots, done: false, iterations: 10, ms: 5 });
    sigma.step?.({ ...spots, done: true, iterations: 20, ms: 12.4 });
    await new Promise((frame) => requestAnimationFrame(frame));

    expect(sigma.focused).toEqual([]);
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

  it("shows a picked document's card in its folder, and opens it", async () => {
    const { user } = await drawnNetwork();

    act(() => sigma.hands?.pick(0));

    const preview = await screen.findByRole("complementary", { name: "Notes" });
    expect(preview.textContent).toContain("Proyectos");
    expect(preview.textContent).toContain("1 connections");
    expect(await within(preview).findByText(t("brainstyFactWrote"))).toBeTruthy();
    expect(within(preview).getByText("2.4 kB · 380 words")).toBeTruthy();
    expect(within(preview).getByText("EN")).toBeTruthy();
    expect(within(preview).getByText("#home")).toBeTruthy();
    expect(within(preview).getByText("Plans")).toBeTruthy();
    expect(
      await within(preview).findByText("tisty://doc/Proyectos/Notes?id=notes.md"),
    ).toBeTruthy();
    await user.click(within(preview).getByRole("button", { name: t("copyLink") }));
    expect(await within(preview).findByRole("button", { name: t("linkCopied") })).toBeTruthy();
    expect(clip.taken).toEqual(["tisty://doc/Proyectos/Notes?id=notes.md"]);
    const ties = within(preview).getByRole("region", { name: t("brainstyTiesTitle") });
    expect(ties.textContent).toContain(t("brainstyTies_cite"));
    expect(ipc.calls).toContainEqual({ cmd: "brain_card", args: { id: "notes.md" } });
    expect(last(sigma.lit)).toEqual([0, 1]);

    await user.click(within(preview).getByRole("button", { name: "buy bread" }));
    expect(await screen.findByRole("complementary", { name: "buy bread" })).toBeTruthy();
    act(() => sigma.hands?.pick(0));
    await user.click(await screen.findByRole("button", { name: t("brainstyOpen") }));

    expect(ways.opened).toEqual([small.nodes[0]]);
    expect(ways.left).toBe(0);
  });

  it("tells a task's state, days and steps, and nothing it does not know", async () => {
    ipc.card = {
      id: "01A",
      kind: "task",
      title: "buy bread",
      state: "open",
      standing: "late",
      date: "2026-11-18",
      steps: [2, 5],
      source: "sereno#12",
      part_of: { id: "01Z", kind: "task", title: "the party" },
      parts: [{ id: "01Y", kind: "task", title: "the cake" }],
    };
    await drawnNetwork();

    act(() => sigma.hands?.pick(1));

    const preview = await screen.findByRole("complementary", { name: "buy bread" });
    await within(preview).findByText(t("brainstyFactState"));
    expect(
      within(preview).getByText(`${t("brainstyStanding_live")} · ${t("brainstyDue_late")}`),
    ).toBeTruthy();
    expect(within(preview).getByText("2 of 5 done")).toBeTruthy();
    expect(within(preview).getByText("sereno#12")).toBeTruthy();
    expect(within(preview).getByText("the party")).toBeTruthy();
    expect(await within(preview).findByText("tisty://task?id=01A")).toBeTruthy();
    expect(within(preview).getByText(/18/)).toBeTruthy();
    expect(within(preview).queryByText(t("brainstyFactDeadline"))).toBeNull();
    expect(within(preview).queryByText(t("brainstyPointedHere"))).toBeNull();
  });

  it("asks no card for what has none, such as a tag", async () => {
    await drawnNetwork();

    act(() => sigma.hands?.pick(2));

    const preview = await screen.findByRole("complementary", { name: "home" });
    expect(preview.textContent).toContain(t("brainstyTies_tag"));
    expect(ipc.calls.some((one) => one.cmd === "brain_card")).toBe(false);
    expect(within(preview).queryByText(t("brainstyFactLink"))).toBeNull();
  });

  it("says what a link is for, to the pointer and on a press", async () => {
    const { user } = await drawnNetwork();

    act(() => sigma.hands?.pick(0));

    const preview = await screen.findByRole("complementary", { name: "Notes" });
    const ask = await within(preview).findByRole("button", { name: t("brainstyLinkAsk") });
    expect(ask.getAttribute("title")).toBe(t("brainstyLinkWhat"));
    expect(ask.getAttribute("aria-expanded")).toBe("false");
    expect(within(preview).queryByText(t("brainstyLinkWhat"))).toBeNull();
    await user.click(ask);
    expect(within(preview).getByText(t("brainstyLinkWhat"))).toBeTruthy();
    expect(ask.getAttribute("aria-expanded")).toBe("true");
  });

  it("shows a file's picture, kind, size and day, who uses it, and opens it or shows it in its folder", async () => {
    ipc.network = () => Promise.resolve(filed);
    ipc.file = { bytes: 2400, changed: "2026-11-18T15:00:00Z" };
    const { user } = await drawnNetwork();

    act(() => sigma.hands?.pick(3));

    const preview = await screen.findByRole("complementary", { name: "map-1a2b3c4d.png" });
    expect(await within(preview).findByText("Image · PNG")).toBeTruthy();
    expect(within(preview).getByText("2.4 kB")).toBeTruthy();
    expect(within(preview).getByText(t("brainstyFactChanged"))).toBeTruthy();
    await waitFor(() =>
      expect(preview.querySelector("img")?.getAttribute("src")).toBe(
        "asset://localhost/C:/kept/map.png",
      ),
    );
    expect(within(preview).queryByRole("region", { name: t("brainstyTiesTitle") })).toBeNull();
    expect(asked("brain_file")).toEqual([{ cmd: "brain_file", args: { id: MAP } }]);

    await user.click(within(preview).getByRole("button", { name: t("brainstyOpen") }));
    await user.click(within(preview).getByRole("button", { name: t("brainstyShowInFolder") }));
    expect(asked("opened")).toEqual([{ cmd: "opened", args: { reference: MAP } }]);
    expect(asked("shown_in_folder")).toEqual([
      { cmd: "shown_in_folder", args: { reference: MAP } },
    ]);
    expect(ways.opened).toEqual([]);

    const used = within(preview).getByRole("region", { name: t("brainstyUsedIn") });
    await user.click(within(used).getByRole("button", { name: "Notes" }));
    expect(await screen.findByRole("complementary", { name: "Notes" })).toBeTruthy();
  });

  it("says no more than the name of a file nothing in sight uses, and says so when one cannot be opened", async () => {
    ipc.network = () => Promise.resolve(filed);
    const { user } = await drawnNetwork();

    act(() => sigma.hands?.pick(3));
    const preview = await screen.findByRole("complementary", { name: "map-1a2b3c4d.png" });
    await waitFor(() => expect(asked("brain_file")).toHaveLength(1));
    expect(within(preview).queryByRole("button", { name: t("brainstyOpen") })).toBeNull();
    expect(within(preview).queryByText(t("brainstyFactSize"))).toBeNull();
    expect(preview.querySelector("img")).toBeNull();
    expect(asked("served")).toEqual([]);

    ipc.file = { bytes: 10 };
    ipc.opening = () => Promise.reject({ code: "cannotOpen", about: MAP });
    act(() => sigma.hands?.pick(0));
    act(() => sigma.hands?.pick(3));
    const again = await screen.findByRole("complementary", { name: "map-1a2b3c4d.png" });
    await user.click(await within(again).findByRole("button", { name: t("brainstyOpen") }));
    expect(await within(again).findByRole("alert")).toBeTruthy();
    expect(within(again).queryByText(t("brainstyFactChanged"))).toBeNull();
  });

  it("opens a task on a double click", async () => {
    await drawnNetwork();

    act(() => sigma.hands?.open(1));

    expect(ways.opened).toEqual([small.nodes[1]]);
  });

  it("lets go of the search, then the picked node, then the view with each Escape", async () => {
    const { user } = await drawnNetwork();

    await user.type(screen.getByRole("searchbox", { name: t("brainstySearch") }), "bread");
    await user.click(await screen.findByRole("button", { name: /buy bread/ }));
    expect(sigma.focused).toEqual([1]);
    expect(await screen.findByRole("complementary", { name: "buy bread" })).toBeTruthy();

    await user.type(screen.getByRole("searchbox", { name: t("brainstySearch") }), "zzz");
    expect(await screen.findByText(t("brainstyNothingFound"))).toBeTruthy();
    await user.keyboard("{Escape}");
    expect(screen.queryByText(t("brainstyNothingFound"))).toBeNull();
    expect(screen.getByRole("complementary", { name: "buy bread" })).toBeTruthy();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("complementary", { name: "buy bread" })).toBeNull();
    expect(ways.left).toBe(0);
    await user.keyboard("{Escape}");
    expect(ways.left).toBe(1);
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
    expect(ways.opened).toEqual([small.nodes[1]]);
  });

  it("hands the keys back to the network when Escape clears the search", async () => {
    const { user, network } = await drawnNetwork();

    await user.type(screen.getByRole("searchbox", { name: t("brainstySearch") }), "bre");
    await user.keyboard("{Escape}");

    expect(screen.getByRole<HTMLInputElement>("searchbox").value).toBe("");
    expect(document.activeElement).toBe(network);
    expect(ways.left).toBe(0);
  });

  it("takes the focus once drawn, and reaches its controls with Tab, in the order they stand", async () => {
    const { user, network } = await drawnNetwork();

    await waitFor(() => expect(document.activeElement).toBe(network));
    await user.tab();
    expect(document.activeElement).toBe(screen.getByRole("searchbox"));
    await user.tab();
    expect(document.activeElement).toBe(screen.getByRole("button", { name: t("brainstyFilters") }));
  });

  it("comes alive once laid out, and the legend turns the travellers and the turning apart, for this computer", async () => {
    localStorage.clear();
    const { user, shown } = await drawnNetwork();
    const spots = { x: new Float32Array(3), y: new Float32Array(3) };

    sigma.step?.({ ...spots, done: true, iterations: 20, ms: 12 });
    await waitFor(() => expect(sigma.turns.length).toBeGreaterThan(0));
    expect(screen.queryByRole("switch")).toBeNull();
    await user.click(screen.getByRole("button", { name: t("brainstyLegend") }));
    const turning = screen.getByRole("switch", { name: t("brainstyTurning") });
    const travellers = screen.getByRole("switch", { name: t("brainstyTravellers") });
    expect(turning.getAttribute("aria-checked")).toBe("true");
    expect(travellers.getAttribute("aria-checked")).toBe("true");
    await user.click(turning);

    expect(sigma.turns[sigma.turns.length - 1]).toBeNull();
    expect(localStorage.getItem("tisty.brainstySpin")).toBe("off");
    expect(localStorage.getItem("tisty.brainstyTravellers")).toBeNull();
    await user.click(travellers);
    expect(localStorage.getItem("tisty.brainstyTravellers")).toBe("off");
    shown.unmount();

    await drawnNetwork();
    expect(
      screen.getByRole("switch", { name: t("brainstyTurning") }).getAttribute("aria-checked"),
    ).toBe("false");
    await user.click(screen.getByRole("switch", { name: t("brainstyTurning") }));
    await user.click(screen.getByRole("switch", { name: t("brainstyTravellers") }));
    expect(localStorage.getItem("tisty.brainstySpin")).toBeNull();
    expect(localStorage.getItem("tisty.brainstyTravellers")).toBeNull();
  });

  it("is drawn and laid out once while the folders and lists change, and takes a new colour as it is", async () => {
    const view = (groups: Group[]) => (
      <BrainstyView near="01A" groups={groups} onOpen={() => {}} onLeave={() => {}} />
    );
    const { rerender } = render(view([PROJECTS]));
    await waitFor(() => expect(sigma.hands).not.toBeNull());
    const rethemed = sigma.rethemed;

    rerender(view([{ ...PROJECTS }]));
    expect(sigma.rethemed).toBe(rethemed);
    rerender(
      view([
        { ...PROJECTS, color: "amber" },
        { id: "l1", name: "Casa" },
      ]),
    );

    expect(sigma.rethemed).toBe(rethemed + 1);
    expect(sigma.killed).toBe(0);
    expect(sigma.stopped).toBe(0);
  });

  it("keeps the travellers and the turning off when the view is drawn anew", async () => {
    onTestFinished(() => localStorage.clear());
    localStorage.setItem("tisty.brainstyTravellers", "off");
    localStorage.setItem("tisty.brainstySpin", "off");
    sigma.zoom = 0;
    ipc.network = () => Promise.resolve({ ...small });
    const view = (near: string) => (
      <BrainstyView near={near} groups={[PROJECTS]} onOpen={() => {}} onLeave={() => {}} />
    );
    const laid = () =>
      sigma.step?.({
        x: new Float32Array(3),
        y: new Float32Array(3),
        done: true,
        iterations: 20,
        ms: 12,
      });
    const frames = async () => {
      for (let n = 0; n < 5; n++) await new Promise((frame) => requestAnimationFrame(frame));
    };
    const { rerender } = render(view("01A"));
    await waitFor(() => expect(sigma.hands).not.toBeNull());
    laid();
    await frames();

    rerender(view("notes.md"));
    await waitFor(() => expect(sigma.killed).toBe(1));
    laid();
    await frames();

    expect(sigma.turns.filter((one) => one !== null)).toEqual([]);
  });

  it("leaves the agents' files out until asked, and remembers it on this computer", async () => {
    onTestFinished(() => localStorage.clear());
    ipc.network = () =>
      Promise.resolve({
        ...small,
        nodes: [...small.nodes, { id: "CLAUDE.md", kind: "doc", title: "CLAUDE", group: "a1" }],
      });
    const theirs: Group[] = [PROJECTS, { id: "a1", name: "AGENTS", agents: true }];
    const user = userEvent.setup();
    const view = <BrainstyView near="01A" groups={theirs} onOpen={() => {}} onLeave={() => {}} />;
    const shown = render(view);
    await waitFor(() => expect(sigma.hands).not.toBeNull());

    await waitFor(() => expect(last(sigma.hidden)).toEqual([false, false, false, true]));
    await user.click(screen.getByRole("button", { name: t("brainstyFilters") }));
    const agents = screen.getByRole("button", { name: t("brainstyShowAgents") });
    expect(agents.getAttribute("aria-pressed")).toBe("false");
    await user.click(agents);
    expect(last(sigma.hidden)).toEqual([false, false, false, false]);
    expect(localStorage.getItem("tisty.brainstyAgents")).toBe("shown");
    shown.unmount();

    sigma.hidden = [];
    render(view);
    await waitFor(() => expect(sigma.hidden.length).toBeGreaterThan(0));
    expect(last(sigma.hidden)).toEqual([false, false, false, false]);
  });

  it("offers no AGENTS filter when the network holds none of their files", async () => {
    const { user } = await drawnNetwork();
    await user.click(screen.getByRole("button", { name: t("brainstyFilters") }));
    expect(screen.queryByRole("button", { name: t("brainstyShowAgents") })).toBeNull();
  });

  it("starts with travellers and turning on, even when the computer asks for less motion", async () => {
    localStorage.clear();
    const asked = vi.spyOn(window, "matchMedia").mockReturnValue({
      matches: true,
    } as MediaQueryList);
    await drawnNetwork();

    await userEvent.setup().click(screen.getByRole("button", { name: t("brainstyLegend") }));
    for (const name of [t("brainstyTravellers"), t("brainstyTurning")])
      expect(screen.getByRole("switch", { name }).getAttribute("aria-checked")).toBe("true");
    asked.mockRestore();
  });

  it("shows what it costs a frame with Ctrl+Shift+M, and hides it again", async () => {
    const { user } = await drawnNetwork();

    await user.keyboard("{Control>}{Shift>}M{/Shift}{/Control}");
    expect(screen.getByText(/ms a frame/)).toBeTruthy();
    await user.keyboard("{Control>}{Shift>}M{/Shift}{/Control}");
    expect(screen.queryByText(/ms a frame/)).toBeNull();
  });

  it("jumps to the search with /", async () => {
    const { user } = await drawnNetwork();

    await user.keyboard("/");

    expect(document.activeElement).toBe(
      screen.getByRole("searchbox", { name: t("brainstySearch") }),
    );
  });

  it("leaves an empty search with Escape before the picked node and the view", async () => {
    const { user, network } = await drawnNetwork();
    await user.keyboard("/");
    await user.keyboard("{Escape}");
    expect(document.activeElement).toBe(network);
    expect(ways.left).toBe(0);

    act(() => sigma.hands?.pick(0));
    await screen.findByRole("complementary", { name: "Notes" });
    await user.keyboard("/");
    await user.keyboard("{Escape}");

    expect(document.activeElement).toBe(network);
    expect(screen.getByRole("complementary", { name: "Notes" })).toBeTruthy();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("complementary", { name: "Notes" })).toBeNull();
    expect(ways.left).toBe(0);
    await user.keyboard("{Escape}");
    expect(ways.left).toBe(1);
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
    expect(alike.textContent).toContain("Tisty thinks they are alike: they share plans and bread");
    expect(alike.textContent).toContain(
      "Tisty thinks they are alike: they share buy and groceries",
    );
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
    adopt("es");
    try {
      await drawnNetwork();

      act(() => sigma.hands?.pick(4));

      const alike = await screen.findByRole("region", { name: "Parecidos" });
      expect(alike.textContent).toContain("Tisty cree que se parecen: comparten buy y groceries");
    } finally {
      adopt("en");
    }
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

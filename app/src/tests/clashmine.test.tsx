import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { keepDraft, letDraftGo } from "../drafts";
import { setAside } from "../keptAside";
import { fill, t } from "../locales";
import { cut } from "../saving";
import Docs from "../ui/Docs";

const KEY = "tisty.clashing";
const FILE = "a3f1-0001";
const OPENED = "# Compras\n\nleche";
const MINE = "# Compras\n\nleche y pan";

type Answer = { torn: number | null } | { body: string; print: string; title: string } | null;

const store = vi.hoisted(() => ({
  disk: "",
  writes: [] as { body: string; since: unknown }[],
  merges: [] as { base: unknown; mine: string; prefer: unknown }[],
  moved: false,
  torn: 1 as number | null,
  weave: null as null | ((mine: string) => Promise<unknown>),
  woven: false,
}));

vi.mock("../desk", async (real) => ({
  ...(await real<typeof import("../desk")>()),
  useDesk: () => ({ papersChanged: () => {} }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: () => Promise.resolve(true),
  save: () => Promise.resolve(null),
  open: () => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", async () => ({
  invoke: (await import("./opened")).opening((cmd: string, args?: Record<string, unknown>) => {
    switch (cmd) {
      case "doc_read":
        return Promise.resolve(store.disk);
      case "doc_write": {
        const body = String(args?.body);
        store.writes.push({ body, since: args?.since });
        if (store.moved || args?.since === "") return Promise.reject({ code: "documentMoved" });
        store.disk = body;
        return Promise.resolve({ id: String(args?.id), title: "Compras", print: `print:${body}` });
      }
      case "doc_merge": {
        const mine = String(args?.mine);
        store.merges.push({ base: args?.base, mine, prefer: args?.prefer });
        if (store.weave) return store.weave(mine);
        if (args?.prefer !== "mine" && !store.woven) return Promise.resolve({ torn: store.torn });
        store.disk = `${store.disk}\n\nagente`;
        return Promise.resolve({
          body: store.disk,
          print: `print:${store.disk}`,
          title: "Compras",
        });
      }
      default:
        return Promise.resolve(null);
    }
  }),
}));

vi.mock("../ui/Sheet", () => ({
  default: ({
    value,
    onWrite,
    onShaped,
  }: {
    value: string;
    onWrite: (text: string) => void;
    onShaped?: (text: string) => void;
  }) => {
    onShaped?.(value);
    return <textarea aria-label="editor" value={value} onChange={(e) => onWrite(e.target.value)} />;
  },
}));

const known: Filed[] = [
  { id: "01F", file: FILE, title: "Compras", folder: null, archived: false, away: false },
];

// The draft a closed window left behind: stored, with nothing of the old run in memory.
const restart = (draft?: { words: string; base: string }) =>
  draft ? keepDraft(FILE, draft) : letDraftGo(FILE);

const draftStored = (): { words: string; base: string } | undefined =>
  JSON.parse(localStorage.getItem(KEY) ?? "{}")[FILE];

beforeEach(() => {
  cut(FILE);
  restart();
  store.disk = OPENED;
  store.writes = [];
  store.merges = [];
  store.moved = false;
  store.torn = 1;
  store.weave = null;
  store.woven = false;
});

const open = () => render(<Docs open={FILE} known={known} onKept={vi.fn()} onError={vi.fn()} />);

const type = (words: string) =>
  fireEvent.change(screen.getByLabelText("editor"), { target: { value: words } });

const clash = async (words = MINE) => {
  open();
  await screen.findByLabelText("editor");
  store.moved = true;
  type(words);
  await act(() => vi.advanceTimersByTimeAsync(800));
};

describe("the bar over a clash", () => {
  it("says how many places the two touched alike", async () => {
    store.torn = 3;
    await clash();

    expect(await screen.findByText(fill("clashTorn", "3"))).toBeTruthy();
  });

  it("says it is one place when it is one", async () => {
    store.torn = 1;
    await clash();

    expect(await screen.findByText(t("clashTornOne"))).toBeTruthy();
  });

  it("says the whole page will be put back when nothing finer can stand", async () => {
    store.torn = null;
    await clash();

    expect(await screen.findByText(t("clashWhole"))).toBeTruthy();
  });

  it("weaves against what it read first, and keeps no words written over the disk", async () => {
    await clash();
    await screen.findByText(t("clashTornOne"));

    expect(store.merges[0]).toEqual({ base: OPENED, mine: MINE, prefer: undefined });
    expect(store.disk).toBe(OPENED);
  });
});

describe("saving mine anyway", () => {
  it("asks the merge to prefer mine over the base it read, never a plain write", async () => {
    await clash();
    await screen.findByText(t("clashTornOne"));
    store.writes = [];
    store.merges = [];

    await userEvent.click(screen.getByRole("button", { name: t("clashSave") }));

    await waitFor(() => expect(store.merges).toHaveLength(1));
    expect(store.merges[0]).toEqual({ base: OPENED, mine: MINE, prefer: "mine" });
    expect(store.writes).toHaveLength(0);
    await waitFor(() => expect(screen.queryByText(t("clashTornOne"))).toBeNull());
  });

  it("loses no words typed while the merge is out", async () => {
    await clash();
    await screen.findByText(t("clashTornOne"));
    let land: (answer: Answer) => void = () => {};
    store.weave = () =>
      new Promise((done) => {
        land = done;
      });

    await userEvent.click(screen.getByRole("button", { name: t("clashSave") }));
    await waitFor(() => expect(store.merges[store.merges.length - 1]?.prefer).toBe("mine"));
    const later = `${MINE} y huevos`;
    type(later);
    await act(async () => {
      land({ body: `${MINE}\n\nagente`, print: "print:woven", title: "Compras" });
    });

    expect((screen.getByLabelText("editor") as HTMLTextAreaElement).value).toBe(later);
    store.weave = null;
    store.moved = false;
    store.woven = true;
    await act(() => vi.advanceTimersByTimeAsync(1500));
    const sentLater = [
      ...store.writes.map((one) => one.body),
      ...store.merges.map((one) => one.mine),
    ];
    expect(sentLater).toContain(later);
  });
});

describe("the draft a clash keeps", () => {
  it("is stored with the words and the body they were typed over", async () => {
    await clash();
    await screen.findByText(t("clashTornOne"));

    expect(draftStored()).toEqual({ words: MINE, base: OPENED });
  });

  it("goes once a save lands", async () => {
    await clash();
    await screen.findByText(t("clashTornOne"));

    await userEvent.click(screen.getByRole("button", { name: t("clashSave") }));

    await waitFor(() => expect(draftStored()).toBeUndefined());
  });

  it("goes when the person reads the page again", async () => {
    await clash();
    await screen.findByText(t("clashTornOne"));
    expect(draftStored()).toBeDefined();

    await userEvent.click(screen.getByRole("button", { name: t("clashLook") }));

    await waitFor(() => expect(draftStored()).toBeUndefined());
  });

  it("after a restart opens through a plain write that is refused, then a merge over the draft's base", async () => {
    restart({ words: MINE, base: OPENED });
    store.disk = `${OPENED}\n\nagente`;
    store.woven = true;
    open();

    await waitFor(() => expect(store.merges.length).toBeGreaterThan(0), { timeout: 4000 });

    expect(store.writes[0]).toEqual({ body: MINE, since: "" });
    expect(store.merges[0]).toEqual({ base: OPENED, mine: MINE, prefer: undefined });
    await waitFor(() => expect(draftStored()).toBeUndefined());
  });

  it("never writes a restarted draft over the disk", async () => {
    restart({ words: MINE, base: OPENED });
    store.disk = `${OPENED}\n\nagente`;
    store.torn = 2;
    open();

    expect(await screen.findByText(fill("clashTorn", "2"), {}, { timeout: 4000 })).toBeTruthy();

    expect(store.writes.every((one) => one.since === "")).toBe(true);
    expect(store.disk).toBe(`${OPENED}\n\nagente`);
    expect(draftStored()).toEqual({ words: MINE, base: OPENED });
  });
});

describe("the bars during a clash", () => {
  it("never shows the clash bar and the words kept aside together", async () => {
    setAside(FILE, "# Compras\n\nlo de antes");
    open();
    await screen.findByLabelText("editor");
    expect(await screen.findByText(t("asideKept"))).toBeTruthy();

    store.moved = true;
    type(MINE);
    await act(() => vi.advanceTimersByTimeAsync(800));

    await screen.findByText(t("clashTornOne"));
    expect(screen.queryByText(t("asideKept"))).toBeNull();
  });

  it("keeps the very same bar while refused autosaves tell the same clash again", async () => {
    await clash();
    const bar = (await screen.findByText(t("clashTornOne"))).closest("[role=status]");
    expect(bar).not.toBeNull();

    for (const more of ["a", "b", "c"]) {
      type(`${MINE} ${more}`);
      await act(() => vi.advanceTimersByTimeAsync(800));
      expect(screen.getByText(t("clashTornOne")).closest("[role=status]")).toBe(bar);
    }

    expect(store.merges.length).toBeGreaterThanOrEqual(4);
    expect(bar?.isConnected).toBe(true);
  });

  it("only changes the count when the count changes", async () => {
    await clash();
    const bar = (await screen.findByText(t("clashTornOne"))).closest("[role=status]");

    store.torn = 2;
    type(`${MINE} otra`);
    await act(() => vi.advanceTimersByTimeAsync(800));

    expect(await screen.findByText(fill("clashTorn", "2"))).toBeTruthy();
    expect(screen.queryByText(t("clashTornOne"))).toBeNull();
    expect(screen.getAllByRole("status")).toHaveLength(1);
    expect(bar).not.toBeNull();
  });
});

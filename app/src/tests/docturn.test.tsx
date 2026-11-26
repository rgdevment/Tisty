import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { keepDraft, letDraftGo } from "../drafts";
import { t } from "../locales";
import { cut } from "../saving";
import Docs from "../ui/Docs";

const KEY = "tisty.clashing";
const A = "a3f1-0001";
const B = "a3f1-0002";
const OPENED = "# Compras\n\nleche";
const MINE = "# Compras\n\nleche y pan";

type Call = { cmd: string; args: Record<string, unknown> };

const store = vi.hoisted(() => ({
  disk: {} as Record<string, string>,
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  heldFor: {} as Record<string, number>,
  lateFor: 0,
  stallWrites: false,
  stalled: [] as ((problem: unknown) => void)[],
  mergeThrows: null as null | { code: string },
  moved: false,
  front: false,
  troubles: [] as string[],
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

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown> = {}) => {
    store.calls.push({ cmd, args });
    const id = String(args.id);
    switch (cmd) {
      case "doc_opened":
      case "doc_read": {
        const body = store.disk[id] ?? "";
        return Promise.resolve(cmd === "doc_read" ? body : { body, print: `print:${body}` });
      }
      case "doc_write": {
        if (store.lateFor > 0) {
          store.lateFor -= 1;
          return Promise.reject({ code: "saveLate", name: id });
        }
        if (store.stallWrites)
          return new Promise((_, failed) => {
            store.stalled.push(failed);
          });
        if ((store.heldFor[id] ?? 0) > 0) {
          store.heldFor[id] -= 1;
          return Promise.reject({ code: "documentHeldOpen" });
        }
        if (store.moved || args.since === "") return Promise.reject({ code: "documentMoved" });
        store.disk[id] = String(args.body);
        return Promise.resolve({ id, title: "Compras", print: `print:${args.body}` });
      }
      case "doc_merge": {
        if (store.mergeThrows) return Promise.reject(store.mergeThrows);
        if ((store.heldFor[id] ?? 0) > 0) {
          store.heldFor[id] -= 1;
          return Promise.reject({ code: "documentHeldOpen" });
        }
        if (args.prefer !== "mine") return Promise.resolve({ torn: 1, front: store.front });
        store.disk[id] = String(args.mine);
        return Promise.resolve({ body: args.mine, print: `print:${args.mine}`, title: "Compras" });
      }
      case "note_trouble":
        store.troubles.push(String(args.code));
        return Promise.resolve(null);
      default:
        return Promise.resolve(null);
    }
  },
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
  { id: "01A", file: A, title: "Compras", folder: null, archived: false, away: false },
  { id: "01B", file: B, title: "Viaje", folder: null, archived: false, away: false },
];

const called = (cmd: string): Call[] => store.calls.filter((one) => one.cmd === cmd);

const draftStored = (id: string): { words: string; base: string } | undefined =>
  JSON.parse(localStorage.getItem(KEY) ?? "{}")[id];

beforeEach(() => {
  for (const id of [A, B]) {
    cut(id);
    letDraftGo(id);
  }
  localStorage.clear();
  store.disk = { [A]: OPENED, [B]: OPENED };
  store.calls = [];
  store.heldFor = {};
  store.lateFor = 0;
  store.stallWrites = false;
  store.stalled = [];
  store.mergeThrows = null;
  store.moved = false;
  store.front = false;
  store.troubles = [];
});

afterEach(async () => {
  cleanup();
  for (const id of [A, B]) cut(id);
  await vi.advanceTimersByTimeAsync(60_000);
});

const open = (file = A) => {
  const onError = vi.fn();
  const props = { known, onKept: vi.fn(), onError };
  const view = render(<Docs open={file} {...props} />);
  return { onError, to: (next: string) => view.rerender(<Docs open={next} {...props} />) };
};

const type = (words: string) =>
  fireEvent.change(screen.getByLabelText("editor"), { target: { value: words } });

const settle = (ms = 800) => act(() => vi.advanceTimersByTimeAsync(ms));

describe("a draft kept across a reload", () => {
  it("takes this window's turn on the file when it is opened", async () => {
    keepDraft(A, { words: MINE, base: OPENED });

    open();

    await waitFor(() => expect(called("doc_opened")).toHaveLength(1));
    expect(called("doc_opened")[0].args.turn).toEqual({
      hand: expect.any(String),
      n: expect.any(Number),
    });
    expect((screen.getByLabelText("editor") as HTMLTextAreaElement).value).toBe(MINE);
  });
});

describe("a save the core calls late", () => {
  it("takes the turn again and sends the newest words once more, and they land", async () => {
    open();
    await screen.findByLabelText("editor");
    store.lateFor = 1;

    type(MINE);
    await settle();

    await waitFor(() => expect(store.disk[A]).toBe(MINE));
    const order = store.calls
      .map((one) => one.cmd)
      .filter((cmd) => cmd.startsWith("doc_") && cmd !== "doc_read");
    expect(order).toEqual(["doc_opened", "doc_write", "doc_opened", "doc_write"]);
    expect(called("doc_opened")[1].args.turn).toBeDefined();
    expect(called("doc_write")[1].args.body).toBe(MINE);
  });

  it("tells it once when the core says late again, and keeps the words on the screen", async () => {
    const { onError } = open();
    await screen.findByLabelText("editor");
    store.lateFor = 99;

    type(MINE);
    await settle();

    await waitFor(() => expect(onError).toHaveBeenCalledWith(t("saveTurnLost")));
    expect(called("doc_opened")).toHaveLength(2);
    expect(called("doc_write")).toHaveLength(2);
    expect(store.disk[A]).toBe(OPENED);
    expect(store.troubles).toContain("saveLate");
    expect((screen.getByLabelText("editor") as HTMLTextAreaElement).value).toBe(MINE);
  });

  it("stays silent when the window cut the save itself", async () => {
    const { onError } = open();
    await screen.findByLabelText("editor");
    store.stallWrites = true;
    type(MINE);
    await settle();
    expect(store.stalled).toHaveLength(1);

    cut(A);
    store.stalled[0]({ code: "internal" });
    await settle(2000);

    expect(onError).not.toHaveBeenCalled();
    expect(called("doc_opened")).toHaveLength(1);
    expect(screen.queryByText(t("saveTurnLost"))).toBeNull();
  });
});

describe("two documents held open", () => {
  it("keeps retrying the one still held when another saves", async () => {
    const { to } = open(B);
    await screen.findByLabelText("editor");
    store.heldFor[B] = 3;
    type("# Viaje\n\nmaleta");
    await settle();
    await settle(1100);
    await waitFor(() => expect(called("doc_write")).toHaveLength(2));

    to(A);
    await waitFor(() =>
      expect((screen.getByLabelText("editor") as HTMLTextAreaElement).value).toBe(OPENED),
    );
    type(MINE);
    await settle();
    await waitFor(() => expect(store.disk[A]).toBe(MINE));
    await settle(10_000);

    expect(store.disk[B]).toBe("# Viaje\n\nmaleta");
    const sentB = called("doc_write").filter((one) => one.args.id === B);
    expect(sentB).toHaveLength(4);
  });
});

describe("the bar over a clash that reaches the properties", () => {
  it("says they clash too, beside the places of the words", async () => {
    store.front = true;
    open();
    await screen.findByLabelText("editor");
    store.moved = true;

    type(MINE);
    await settle();

    expect(await screen.findByText(`${t("clashTornOne")} ${t("clashFrontToo")}`)).toBeTruthy();
  });
});

describe("mine chosen while the file is held open", () => {
  it("shows the held-open bar, not the clash, and the retry still prefers mine", async () => {
    open();
    await screen.findByLabelText("editor");
    store.moved = true;
    type(MINE);
    await settle();
    await screen.findByText(t("clashTornOne"));
    store.heldFor[A] = 1;

    await userEvent.click(screen.getByRole("button", { name: t("clashSave") }));

    expect(await screen.findByText(t("documentHeldOpen"))).toBeTruthy();
    expect(screen.queryByText(t("clashTornOne"))).toBeNull();
    await settle(2000);
    const merges = called("doc_merge");
    expect(merges[merges.length - 1]?.args.prefer).toBe("mine");
    expect(merges.filter((one) => one.args.prefer === "mine").length).toBeGreaterThanOrEqual(2);
    expect(store.disk[A]).toBe(MINE);
    expect(screen.queryByText(t("documentHeldOpen"))).toBeNull();
  });
});

describe("the draft across the ways a save can end", () => {
  const clashFirst = async () => {
    open();
    await screen.findByLabelText("editor");
    store.moved = true;
    type(MINE);
    await settle();
  };

  it("is kept when the merge itself is refused as held open", async () => {
    store.mergeThrows = { code: "documentHeldOpen" };

    await clashFirst();

    await waitFor(() => expect(called("doc_merge")).not.toHaveLength(0));
    expect(draftStored(A)).toEqual({ words: MINE, base: OPENED });
  });

  it("is kept when the merge is refused as moved", async () => {
    store.mergeThrows = { code: "documentMoved" };

    await clashFirst();

    await waitFor(() => expect(called("doc_merge")).not.toHaveLength(0));
    expect(draftStored(A)).toEqual({ words: MINE, base: OPENED });
  });

  it("is dropped when the words are refused for good", async () => {
    keepDraft(A, { words: MINE, base: OPENED });
    open();
    await screen.findByLabelText("editor");
    expect(draftStored(A)).toBeDefined();
    store.stallWrites = true;

    type(`${MINE} y huevos`);
    await settle();
    store.stalled[0]({ code: "documentTooLong" });
    await settle(100);

    await waitFor(() => expect(draftStored(A)).toBeUndefined());
  });
});

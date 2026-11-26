import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { t } from "../locales";
import { cut, PATIENCE } from "../saving";
import Docs from "../ui/Docs";

const store = vi.hoisted(() => ({
  bodies: {} as Record<string, string>,
  writes: [] as { id: string; body: string; at: number }[],
  heldFor: 0,
  stalled: false,
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

vi.mock("@tauri-apps/api/core", async () => ({
  invoke: (await import("./opened")).opening((cmd: string, args?: Record<string, unknown>) => {
    switch (cmd) {
      case "doc_read":
        return Promise.resolve(store.bodies[String(args?.id)] ?? "");
      case "doc_write": {
        const id = String(args?.id);
        const body = String(args?.body);
        store.writes.push({ id, body, at: Date.now() });
        if (store.stalled) return new Promise(() => {});
        if (store.heldFor > 0) {
          store.heldFor -= 1;
          return Promise.reject({ code: "documentHeldOpen" });
        }
        store.bodies[id] = body;
        return Promise.resolve({ id, title: "Compras" });
      }
      case "note_trouble":
        store.troubles.push(String(args?.code));
        return Promise.resolve(null);
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

let FILE = "a3f1-0001";
let known: Filed[] = [];
let fileNo = 0;

beforeEach(() => {
  fileNo += 1;
  FILE = `a3f1-${String(fileNo).padStart(4, "0")}`;
  known = [
    {
      id: `01F${fileNo}`,
      file: FILE,
      title: "Compras",
      folder: null,
      archived: false,
      away: false,
    },
  ];
  cut(FILE);
  store.bodies = { [FILE]: "# Compras\n\nleche" };
  store.writes = [];
  store.heldFor = 0;
  store.stalled = false;
  store.troubles = [];
});

// A retry armed by a save still in flight when the view unmounts must not leak into the next test.
afterEach(async () => {
  cleanup();
  cut(FILE);
  await vi.advanceTimersByTimeAsync(60_000);
});

const open = () => {
  const onError = vi.fn();
  render(<Docs open={FILE} known={known} onKept={vi.fn()} onError={onError} />);
  return onError;
};

const type = (words: string) =>
  fireEvent.change(screen.getByLabelText("editor"), { target: { value: words } });

describe("typing without a pause", () => {
  it("keeps sending saves at least every two seconds", async () => {
    open();
    await screen.findByLabelText("editor");
    const began = Date.now();

    for (let spent = 0, n = 0; spent < 12_000; spent += 35, n++) {
      type(`# Compras\n\nleche ${n}`);
      await act(() => vi.advanceTimersByTimeAsync(35));
    }

    const at = store.writes.map((one) => one.at - began);
    expect(at.length).toBeGreaterThanOrEqual(5);
    expect(at[0]).toBeLessThanOrEqual(2100);
    for (let i = 1; i < at.length; i++) expect(at[i] - at[i - 1]).toBeLessThanOrEqual(2100);
  });

  it("sends the last words once the typing stops", async () => {
    open();
    await screen.findByLabelText("editor");

    type("# Compras\n\nleche y pan");
    await act(() => vi.advanceTimersByTimeAsync(800));

    await waitFor(() => expect(store.bodies[FILE]).toBe("# Compras\n\nleche y pan"));
    expect(store.writes).toHaveLength(1);
  });
});

describe("a document another program holds open", () => {
  it("says so plainly, never the os error, and keeps the words", async () => {
    store.heldFor = 99;
    const onError = open();
    await screen.findByLabelText("editor");

    type("# Compras\n\nleche y pan");
    await act(() => vi.advanceTimersByTimeAsync(800));

    expect(await screen.findByText(t("documentHeldOpen"))).toBeTruthy();
    expect(document.body.textContent).not.toMatch(/os error/i);
    expect(onError).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: t("saveHeldTryNow") })).toBeTruthy();
    expect((screen.getByLabelText("editor") as HTMLTextAreaElement).value).toBe(
      "# Compras\n\nleche y pan",
    );
  });

  it("tries again on its own after 1s, 2s, 4s and, once the write passes, clears the bar", async () => {
    store.heldFor = 3;
    open();
    await screen.findByLabelText("editor");

    type("# Compras\n\nleche y pan");
    await act(() => vi.advanceTimersByTimeAsync(800));
    await screen.findByText(t("documentHeldOpen"));
    await act(() => vi.advanceTimersByTimeAsync(10_000));

    expect(store.writes).toHaveLength(4);
    const gaps = store.writes.slice(1).map((one, i) => one.at - store.writes[i].at);
    expect(gaps[0]).toBeGreaterThanOrEqual(1000);
    expect(gaps[0]).toBeLessThan(1300);
    expect(gaps[1]).toBeGreaterThanOrEqual(2000);
    expect(gaps[1]).toBeLessThan(2300);
    expect(gaps[2]).toBeGreaterThanOrEqual(4000);
    expect(gaps[2]).toBeLessThan(4300);
    expect(store.bodies[FILE]).toBe("# Compras\n\nleche y pan");
    expect(screen.queryByText(t("documentHeldOpen"))).toBeNull();
    await act(() => vi.advanceTimersByTimeAsync(60_000));
    expect(store.writes).toHaveLength(4);
  });

  it("resends at once on Try now", async () => {
    store.heldFor = 1;
    open();
    await screen.findByLabelText("editor");
    type("# Compras\n\nleche y pan");
    await act(() => vi.advanceTimersByTimeAsync(800));
    await screen.findByText(t("documentHeldOpen"));
    expect(store.writes).toHaveLength(1);

    await userEvent.click(screen.getByRole("button", { name: t("saveHeldTryNow") }));

    await waitFor(() => expect(store.bodies[FILE]).toBe("# Compras\n\nleche y pan"));
    await waitFor(() => expect(screen.queryByText(t("documentHeldOpen"))).toBeNull());
  });
});

describe("a save that stalls", () => {
  it("still shows its own bar, not the held-open one", async () => {
    open();
    await screen.findByLabelText("editor");
    store.stalled = true;

    type("# Compras\n\nleche y pan");
    await act(() => vi.advanceTimersByTimeAsync(800));
    await act(() => vi.advanceTimersByTimeAsync(PATIENCE));

    expect(await screen.findByText(t("saveStuck"))).toBeTruthy();
    expect(screen.queryByText(t("documentHeldOpen"))).toBeNull();
    expect(screen.getByRole("button", { name: t("tryAgain") })).toBeTruthy();
    expect(store.troubles).toContain("saveStuck");
  });
});

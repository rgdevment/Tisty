import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { cut } from "../saving";
import Docs from "../ui/Docs";

const store = vi.hoisted(() => ({
  bodies: {} as Record<string, string>,
  writes: [] as { id: string; body: string; at: number }[],
  heldFor: 0,
  stalled: false,
  latency: 120,
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
        // A save takes a moment, so the next keys land while it is out.
        return new Promise((done) =>
          setTimeout(() => {
            store.bodies[id] = body;
            done({ id, title: body.split("\n")[0].replace("# ", "") });
          }, store.latency),
        );
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
  store.latency = 120;
  store.troubles = [];
});

afterEach(async () => {
  cleanup();
  cut(FILE);
  await vi.advanceTimersByTimeAsync(60_000);
});

const type = (words: string) =>
  fireEvent.change(screen.getByLabelText("editor"), { target: { value: words } });

const typeNonstop = async (heading: (n: number) => string, ms = 12_000) => {
  for (let spent = 0, n = 0; spent < ms; spent += 35, n++) {
    type(`${heading(n)}\n\nleche ${n}`);
    await act(() => vi.advanceTimersByTimeAsync(35));
  }
};

describe("telling the parent a save landed", () => {
  it("waits for the pause while the person keeps typing", async () => {
    const onKept = vi.fn();
    render(<Docs open={FILE} known={known} onKept={onKept} onError={vi.fn()} />);
    await screen.findByLabelText("editor");

    await typeNonstop(() => "# Compras");

    expect(store.writes.length).toBeGreaterThanOrEqual(5);
    expect(onKept.mock.calls.length).toBeLessThanOrEqual(2);
    expect(onKept.mock.calls.length).toBeLessThan(store.writes.length);
  });

  it("waits for the pause even when each save lands before the next key", async () => {
    store.latency = 5;
    const onKept = vi.fn();
    render(<Docs open={FILE} known={known} onKept={onKept} onError={vi.fn()} />);
    await screen.findByLabelText("editor");

    await typeNonstop(() => "# Compras");

    expect(store.writes.length).toBeGreaterThanOrEqual(5);
    expect(onKept.mock.calls.length).toBeLessThanOrEqual(1);
    await act(() => vi.advanceTimersByTimeAsync(1_600));
    expect(onKept).toHaveBeenLastCalledWith({ id: FILE, title: "Compras" });
  });

  it("tells the final title once the typing stops", async () => {
    const onKept = vi.fn();
    render(<Docs open={FILE} known={known} onKept={onKept} onError={vi.fn()} />);
    await screen.findByLabelText("editor");
    await typeNonstop(() => "# Compras");

    type("# Compras\n\nleche y pan");
    await act(() => vi.advanceTimersByTimeAsync(800));

    await waitFor(() => expect(store.bodies[FILE]).toBe("# Compras\n\nleche y pan"));
    expect(onKept).toHaveBeenLastCalledWith({ id: FILE, title: "Compras" });
  });

  it("tells a changed title at once even while typing", async () => {
    const onKept = vi.fn();
    render(<Docs open={FILE} known={known} onKept={onKept} onError={vi.fn()} />);
    await screen.findByLabelText("editor");

    await typeNonstop((n) => (n < 100 ? "# Compras" : "# Mercado"), 12_000);
    const before = onKept.mock.calls.length;
    await typeNonstop(() => "# Mercado", 5_000);

    expect(onKept.mock.calls.some(([doc]) => doc.title === "Mercado")).toBe(true);
    expect(onKept.mock.calls.length).toBeGreaterThanOrEqual(before);
    expect(onKept.mock.calls.length).toBeLessThan(store.writes.length);
  });
});

import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Docs from "../ui/Docs";
import type { Filed } from "../core";

const store = vi.hoisted(() => ({
  bodies: {} as Record<string, string>,
  writes: [] as { id: string; body: string }[],
  delays: [] as number[],
}));

const parting = vi.hoisted(() => ({ fire: null as null | (() => void) }));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, run: () => void) => {
    if (name === "parting") parting.fire = run;
    return Promise.resolve(() => {
      if (parting.fire === run) parting.fire = null;
    });
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    switch (cmd) {
      case "doc_read":
        return Promise.resolve(store.bodies[String(args?.id)] ?? "");
      case "doc_write": {
        const id = String(args?.id);
        const body = String(args?.body);
        store.writes.push({ id, body });
        const title = body
          .split("\n")[0]
          .replace(/^#+\s*/, "")
          .trim();
        const waits = store.delays.shift() ?? 0;
        return new Promise((go) =>
          setTimeout(() => {
            store.bodies[id] = body;
            go({ id, title });
          }, waits),
        );
      }
      default:
        return Promise.resolve(null);
    }
  },
}));

vi.mock("../ui/Editor", () => ({
  default: ({ value, onWrite }: { value: string; onWrite: (text: string) => void }) => (
    <textarea aria-label="editor" value={value} onChange={(e) => onWrite(e.target.value)} />
  ),
}));

const known: Filed[] = [
  { id: "01F", file: "a3f1-0001", title: "Compras", folder: null , archived: false },
  { id: "01G", file: "a3f1-0002", title: "Notas", folder: "01H" , archived: false },
];

describe("the document being written", () => {
  beforeEach(() => {
    store.bodies = { "a3f1-0001": "# Compras\n\nleche", "a3f1-0002": "# Notas" };
    store.writes = [];
    store.delays = [];
  });

  const show = (open?: string, onKept = vi.fn()) =>
    render(<Docs open={open} known={known} onKept={onKept} onError={vi.fn()} />);

  it("finishes writing before the app is allowed to leave", async () => {
    store.delays = [120];
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    const editor = await screen.findByLabelText("editor");
    await userEvent.clear(editor);
    await userEvent.type(editor, "sin guardar todavía");

    await waitFor(() => expect(parting.fire).toBeTruthy());
    parting.fire?.();

    await waitFor(() => expect(store.bodies["a3f1-0001"]).toBe("sin guardar todavía"));
    await waitFor(() => expect(store.writes.some((one) => one.id === "parted")).toBe(false));
  });

  it("keeps the last keystroke when a slow save lands after it", async () => {
    store.delays = [250];
    const { rerender } = render(
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />,
    );
    const editor = await screen.findByLabelText("editor");

    await userEvent.clear(editor);
    await userEvent.type(editor, "uno");
    await new Promise((go) => setTimeout(go, 720));
    await userEvent.type(editor, " dos");
    await new Promise((go) => setTimeout(go, 300));

    rerender(<Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} />);

    await waitFor(() => expect(store.bodies["a3f1-0001"]).toBe("uno dos"));
  });

  it("never lands two saves of one document out of order", async () => {
    store.delays = [250, 10];
    show("a3f1-0001");
    const editor = await screen.findByLabelText("editor");

    await userEvent.clear(editor);
    await userEvent.type(editor, "uno");
    await new Promise((go) => setTimeout(go, 720));
    await userEvent.type(editor, " dos");
    await new Promise((go) => setTimeout(go, 750));

    await waitFor(() => expect(store.writes.length).toBeGreaterThan(1));
    await new Promise((go) => setTimeout(go, 400));
    expect(store.bodies["a3f1-0001"]).toBe("uno dos");
  });

  it("closes a document the sidebar stopped pointing at, and stops writing to it", async () => {
    const { rerender } = render(
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />,
    );
    const editor = await screen.findByLabelText("editor");
    await userEvent.type(editor, "!");

    rerender(<Docs known={known} onKept={vi.fn()} onError={vi.fn()} />);

    await waitFor(() => expect(screen.queryByLabelText("editor")).toBeNull());
    await new Promise((go) => setTimeout(go, 900));
    expect(store.writes.filter((one) => one.id === "a3f1-0001")).toHaveLength(0);
  });

  it("lets go of a document that is no longer there", async () => {
    const { rerender } = render(
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />,
    );
    await screen.findByLabelText("editor");

    rerender(
      <Docs
        open="a3f1-0001"
        known={known.filter((one) => one.file !== "a3f1-0001")}
        onKept={vi.fn()}
        onError={vi.fn()}
      />,
    );

    await waitFor(() => expect(screen.queryByLabelText("editor")).toBeNull());
  });

  it("opens the file a registry entry points at, not the entry itself", async () => {
    show("a3f1-0002");

    const editor = await screen.findByLabelText("editor");
    expect((editor as HTMLTextAreaElement).value).toBe("# Notas");
  });

  it("opens the one the sidebar asked for", async () => {
    show("a3f1-0001");

    const editor = await screen.findByLabelText("editor");
    expect((editor as HTMLTextAreaElement).value).toBe("# Compras\n\nleche");
  });

  it("waits to be asked, rather than opening something on its own", async () => {
    show();

    expect(await screen.findByText(/Pick a document/)).toBeTruthy();
    expect(screen.queryByLabelText("editor")).toBeNull();
  });

  it("keeps what was written once the typing settles", async () => {
    show("a3f1-0001");
    const editor = await screen.findByLabelText("editor");

    await userEvent.clear(editor);
    await userEvent.type(editor, "# Compras del mes");

    await waitFor(() => expect(store.writes.length).toBeGreaterThan(0), { timeout: 3000 });
    expect(store.writes[store.writes.length - 1]?.body).toBe("# Compras del mes");
  });

  it("hands the new title back, so the sidebar can follow", async () => {
    const kept = vi.fn();
    show("a3f1-0001", kept);
    const editor = await screen.findByLabelText("editor");

    await userEvent.clear(editor);
    await userEvent.type(editor, "# Otra cosa");

    await waitFor(() => expect(kept).toHaveBeenCalled(), { timeout: 3000 });
    expect(kept.mock.calls[kept.mock.calls.length - 1][0]).toEqual({
      id: "a3f1-0001",
      title: "Otra cosa",
    });
  });

  it("writes what was pending before opening another one", async () => {
    const view = show("a3f1-0001");
    const editor = await screen.findByLabelText("editor");
    await userEvent.clear(editor);
    await userEvent.type(editor, "a medias");

    view.rerender(<Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} />);

    await waitFor(() => expect(store.bodies["a3f1-0001"]).toBe("a medias"), { timeout: 3000 });
  });
});

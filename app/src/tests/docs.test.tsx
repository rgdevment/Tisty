import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Docs from "../ui/Docs";
import type { Doc } from "../core";

const store = vi.hoisted(() => ({
  bodies: {} as Record<string, string>,
  writes: [] as { id: string; body: string }[],
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
        store.bodies[id] = body;
        const title = body
          .split("\n")[0]
          .replace(/^#+\s*/, "")
          .trim();
        return Promise.resolve({ id, title });
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

const known: Doc[] = [
  { id: "a3f1-0001", title: "Compras" },
  { id: "a3f1-0002", title: "Notas" },
];

describe("the document being written", () => {
  beforeEach(() => {
    store.bodies = { "a3f1-0001": "# Compras\n\nleche", "a3f1-0002": "# Notas" };
    store.writes = [];
  });

  const show = (open?: string, onKept = vi.fn()) =>
    render(<Docs open={open} known={known} onKept={onKept} onError={vi.fn()} />);

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

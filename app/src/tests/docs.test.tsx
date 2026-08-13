import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Docs from "../ui/Docs";

const store = vi.hoisted(() => ({
  docs: [] as { id: string; title: string }[],
  bodies: {} as Record<string, string>,
  writes: [] as { id: string; body: string }[],
  made: 0,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    switch (cmd) {
      case "docs":
        return Promise.resolve(store.docs);
      case "doc_read":
        return Promise.resolve(store.bodies[String(args?.id)] ?? "");
      case "doc_write": {
        const id = String(args?.id);
        const body = String(args?.body);
        store.writes.push({ id, body });
        store.bodies[id] = body;
        const title = body.split("\n")[0].replace(/^#+\s*/, "").trim();
        store.docs = store.docs.map((one) => (one.id === id ? { id, title } : one));
        return Promise.resolve({ id, title });
      }
      case "doc_new": {
        store.made += 1;
        const made = { id: `a3f1-000${store.made}`, title: "" };
        store.docs = [...store.docs, made];
        return Promise.resolve(made);
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

describe("the documents view", () => {
  beforeEach(() => {
    store.docs = [
      { id: "a3f1-0001", title: "Compras" },
      { id: "a3f1-0002", title: "" },
    ];
    store.bodies = { "a3f1-0001": "# Compras\n\nleche", "a3f1-0002": "" };
    store.writes = [];
    store.made = 0;
    vi.useRealTimers();
  });

  const show = () => render(<Docs onKnown={vi.fn()} onError={vi.fn()} />);

  it("lists what is there, by title", async () => {
    show();

    expect(await screen.findByRole("button", { name: "Compras" })).toBeTruthy();
  });

  it("names a document that has nothing written in it yet", async () => {
    show();

    expect(await screen.findByRole("button", { name: "Untitled" })).toBeTruthy();
  });

  it("opens what you pick", async () => {
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Compras" }));

    const editor = await screen.findByLabelText("editor");
    expect((editor as HTMLTextAreaElement).value).toBe("# Compras\n\nleche");
  });

  it("says nothing is open until you pick one", async () => {
    show();

    expect(await screen.findByText(/Pick a document/)).toBeTruthy();
    expect(screen.queryByLabelText("editor")).toBeNull();
  });

  it("keeps what was typed, once the typing settles", async () => {
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Compras" }));
    const editor = await screen.findByLabelText("editor");

    await userEvent.clear(editor);
    await userEvent.type(editor, "# Compras del mes");

    await waitFor(() => expect(store.writes.length).toBeGreaterThan(0), { timeout: 3000 });
    expect(store.writes[store.writes.length - 1]?.body).toBe("# Compras del mes");
  });

  it("takes the new title into the list without asking the disk again", async () => {
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Compras" }));
    const editor = await screen.findByLabelText("editor");

    await userEvent.clear(editor);
    await userEvent.type(editor, "# Otra cosa");

    expect(await screen.findByRole("button", { name: "Otra cosa" })).toBeTruthy();
  });

  it("makes a new one and opens it empty", async () => {
    show();
    await userEvent.click(await screen.findByRole("button", { name: "New document" }));

    const editor = await screen.findByLabelText("editor");
    expect((editor as HTMLTextAreaElement).value).toBe("");
    expect(store.made).toBe(1);
  });
});

import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { settled } from "../saving";
import Docs from "../ui/Docs";

const store = vi.hoisted(() => ({
  bodies: {} as Record<string, string>,
  writes: [] as { id: string; body: string; anyway: boolean }[],
  clash: false,
  delays: [] as number[],
  reads: 0,
  converted: [] as { id: string; was: string }[],
  mute: false,
  shape: null as string | null,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    switch (cmd) {
      case "doc_read":
        store.reads += 1;
        return Promise.resolve(store.bodies[String(args?.id)] ?? "");
      case "convert_paper": {
        const id = String(args?.id);
        store.converted.push({ id, was: store.bodies[id] });
        store.bodies[id] = String(args?.body);
        return Promise.resolve(null);
      }
      case "doc_write": {
        const id = String(args?.id);
        const body = String(args?.body);
        const anyway = args?.anyway === true;
        store.writes.push({ id, body, anyway });
        if (store.clash && !anyway) return Promise.reject({ code: "documentMoved" });
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
  default: ({
    value,
    onWrite,
    onShaped,
    reading,
  }: {
    value: string;
    onWrite: (text: string) => void;
    onShaped?: (text: string) => void;
    reading?: boolean;
  }) => {
    if (!store.mute)
      onShaped?.(store.shape ?? value.replace(/<[^>]+>/g, "").replace(/\n{3,}/g, "\n\n"));
    return (
      <textarea
        aria-label="editor"
        readOnly={reading}
        value={value}
        onChange={(e) => onWrite(e.target.value)}
      />
    );
  },
}));

const known: Filed[] = [
  { id: "01F", file: "a3f1-0001", title: "Compras", folder: null, archived: false },
  { id: "01G", file: "a3f1-0002", title: "Notas", folder: "01H", archived: false },
];

describe("the document being written", () => {
  beforeEach(() => {
    store.bodies = { "a3f1-0001": "# Compras\n\nleche", "a3f1-0002": "# Notas" };
    store.writes = [];
    store.reads = 0;
    store.delays = [];
    store.converted = [];
    store.mute = false;
    store.shape = null;
    store.clash = false;
  });

  const show = (open?: string, onKept = vi.fn()) =>
    render(<Docs open={open} known={known} onKept={onKept} onError={vi.fn()} />);

  it("says so when something wrote in the document while it was open", async () => {
    const props = { open: "a3f1-0001", known, onKept: vi.fn(), onError: vi.fn() };
    const { rerender } = render(<Docs {...props} fresh={0} />);
    await waitFor(() => screen.getByLabelText("editor"));

    store.bodies["a3f1-0001"] = "# Acta\n\nlo que dejo el asistente.";
    rerender(<Docs {...props} fresh={1} />);

    await waitFor(() =>
      expect(screen.getByText(/wrote in this document, and what you are reading/i)).toBeTruthy(),
    );
    await userEvent.click(screen.getByRole("button", { name: /got it/i }));
    expect(screen.queryByText(/wrote in this document, and what you are reading/i)).toBeNull();
  });

  it("says nothing when the poll finds only what the person themselves saved", async () => {
    const props = { open: "a3f1-0001", known, onKept: vi.fn(), onError: vi.fn() };
    const { rerender } = render(<Docs {...props} fresh={0} />);
    const editor = await waitFor(() => screen.getByLabelText("editor"));

    await userEvent.click(editor);
    await userEvent.keyboard(" y algo mas");
    await waitFor(() => expect(store.writes.length).toBeGreaterThan(0));

    rerender(<Docs {...props} fresh={1} />);

    await waitFor(() => expect(store.reads).toBeGreaterThan(1));
    expect(screen.queryByText(/wrote in this document, and what you are reading/i)).toBeNull();
  });

  it("says nothing when the document was stirred but reads the same", async () => {
    const props = { open: "a3f1-0001", known, onKept: vi.fn(), onError: vi.fn() };
    const { rerender } = render(<Docs {...props} fresh={0} />);
    await waitFor(() => screen.getByLabelText("editor"));

    rerender(<Docs {...props} fresh={1} />);

    await waitFor(() => expect(store.reads).toBeGreaterThan(1));
    expect(screen.queryByText(/wrote in this document, and what you are reading/i)).toBeNull();
  });

  it("does not read the document again when the parent renders with the same words", async () => {
    store.delays = [];
    const props = { open: "a3f1-0001", known, onKept: vi.fn(), onError: vi.fn() };
    const { rerender } = render(<Docs {...props} />);
    await waitFor(() => expect(store.reads).toBeGreaterThan(0));
    await waitFor(() => screen.getByLabelText("editor"));
    const reads = store.reads;

    for (let turn = 0; turn < 5; turn += 1) rerender(<Docs {...props} />);

    expect(store.reads).toBe(reads);
  });

  it("says when a document carries more attachments than it can draw briskly", async () => {
    const { MANY } = await import("../previews");
    store.bodies["a3f1-0001"] = Array.from(
      { length: MANY + 1 },
      (_, i) => `[uno ${i}](<attachments/ab/uno${i}.pdf>)`,
    ).join("\n\n");
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);

    await waitFor(() => screen.getByText(/adjuntos a la vista|attachments in view/));
  });

  it("says nothing about a document that carries only a few", async () => {
    const { MANY } = await import("../previews");
    store.bodies["a3f1-0001"] = Array.from(
      { length: MANY },
      (_, i) => `[uno ${i}](<attachments/ab/uno${i}.pdf>)`,
    ).join("\n\n");
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);

    await waitFor(() => screen.getByLabelText("editor"));

    expect(screen.queryByText(/adjuntos a la vista|attachments in view/)).toBeNull();
  });

  it("says at the foot what it cannot keep, without standing in the way", async () => {
    store.bodies["a3f1-0001"] = "# Compras\n\n<details>\n<summary>ver</summary>\nalgo\n</details>";
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);

    await screen.findByText(/needs to convert|necesita convertir/i);

    expect(screen.queryByRole("dialog")).toBeNull();
    expect(screen.getByText(/needs to convert|necesita convertir/i)).toBeTruthy();
    expect(
      screen.getByRole("button", { name: /Try converting it|Intentar convertirlo/ }),
    ).toBeTruthy();
  });

  it("keeps what it was before rewriting it, and stops asking", async () => {
    store.bodies["a3f1-0001"] = "# Compras\n\n<div>algo</div>";
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/needs to convert|necesita convertir/i);

    await userEvent.click(
      screen.getByRole("button", { name: /Try converting it|Intentar convertirlo/ }),
    );

    await waitFor(() => expect(store.converted).toHaveLength(1));
    expect(store.converted[0].was).toBe("# Compras\n\n<div>algo</div>");
    expect(store.bodies["a3f1-0001"]).toBe("# Compras\n\nalgo");
    await waitFor(() =>
      expect(screen.queryByText(/needs to convert|necesita convertir/i)).toBeNull(),
    );
    expect(screen.getByLabelText("editor").hasAttribute("readonly")).toBe(false);
  });

  it("does not ask again after coming back to a document it converted", async () => {
    store.bodies["a3f1-0001"] = "# Compras\n\n<div>algo</div>";
    const props = { known, onKept: vi.fn(), onError: vi.fn() };
    const { rerender } = render(<Docs open="a3f1-0001" {...props} />);
    await screen.findByText(/needs to convert|necesita convertir/i);
    await userEvent.click(
      screen.getByRole("button", { name: /Try converting it|Intentar convertirlo/ }),
    );
    await waitFor(() => expect(store.converted).toHaveLength(1));

    rerender(<Docs open="a3f1-0002" {...props} />);
    await waitFor(() => expect(store.reads).toBeGreaterThan(1));
    rerender(<Docs open="a3f1-0001" {...props} />);
    await screen.findByLabelText("editor");

    expect(screen.queryByText(/needs to convert|necesita convertir/i)).toBeNull();
  });

  it("says it could not be converted instead of asking again forever", async () => {
    store.bodies["a3f1-0001"] = "# Compras\n\n<table><tr><td>a</td></tr></table>";
    store.shape = "# Compras\n\n<table><tr><td>a</td></tr></table>";
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/needs to convert|necesita convertir/i);

    await userEvent.click(
      screen.getByRole("button", { name: /Try converting it|Intentar convertirlo/ }),
    );

    await waitFor(() => expect(screen.getByText(/could not be converted|No se pudo convertir/i)));
    expect(
      screen.queryByRole("button", { name: /Try converting it|Intentar convertirlo/ }),
    ).toBeNull();
  });

  it("rewrites nothing when the editor never said what it would become", async () => {
    store.mute = true;
    store.bodies["a3f1-0001"] = "# Compras\n\n<div>algo</div>";
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/needs to convert|necesita convertir/i);

    await userEvent.click(
      screen.getByRole("button", { name: /Try converting it|Intentar convertirlo/ }),
    );

    expect(store.converted).toHaveLength(0);
    expect(store.bodies["a3f1-0001"]).toBe("# Compras\n\n<div>algo</div>");
  });

  it("writes nothing at all while it is only being read", async () => {
    store.bodies["a3f1-0001"] = "# Compras\n\n<div>algo</div>";
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);

    const editor = await screen.findByLabelText("editor");
    await userEvent.type(editor, "esto no debe guardarse");
    await settled();

    expect(store.writes).toHaveLength(0);
  });

  it("says nothing when the document is made of what the editor writes", async () => {
    store.bodies["a3f1-0001"] = "# Compras\n\nleche y pan";
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await waitFor(() => screen.getByLabelText("editor"));

    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("opens the other document when a reference asks for it", async () => {
    store.bodies = {
      "a3f1-0001": "# Compras\n\nver [Notas](tisty:doc/a3f1-0002)",
      "a3f1-0002": "# Notas\n\nlo otro",
    };
    const props = { known, onKept: vi.fn(), onError: vi.fn() };
    const { rerender } = render(<Docs open="a3f1-0001" {...props} />);
    await waitFor(() =>
      expect((screen.getByLabelText("editor") as HTMLTextAreaElement).value).toContain("Compras"),
    );

    rerender(<Docs open="a3f1-0002" {...props} />);

    await waitFor(() =>
      expect((screen.getByLabelText("editor") as HTMLTextAreaElement).value).toContain("Notas"),
    );
  });

  it("says which document is on screen, not which one was asked for", async () => {
    const onShown = vi.fn();
    store.delays = [];
    const { rerender } = render(
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} onShown={onShown} />,
    );
    await waitFor(() => expect(onShown).toHaveBeenCalledWith("a3f1-0001"));

    onShown.mockClear();
    rerender(
      <Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} onShown={onShown} />,
    );

    expect(onShown).not.toHaveBeenCalledWith("a3f1-0002");
    await waitFor(() => expect(onShown).toHaveBeenCalledWith("a3f1-0002"));
  });

  it("finishes writing before the app is allowed to leave", async () => {
    store.delays = [120];
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    const editor = await screen.findByLabelText("editor");
    await userEvent.clear(editor);
    await userEvent.type(editor, "sin guardar todavía");

    await settled();

    expect(store.bodies["a3f1-0001"]).toBe("sin guardar todavía");
  });

  it("waits for a document left behind, not only the one on screen", async () => {
    store.delays = [400, 10];
    const { rerender } = render(
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />,
    );
    const first = await screen.findByLabelText("editor");
    await userEvent.clear(first);
    await userEvent.type(first, "lo de A");

    rerender(<Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await waitFor(() =>
      expect((screen.getByLabelText("editor") as HTMLTextAreaElement).value).toBe("# Notas"),
    );
    await userEvent.clear(screen.getByLabelText("editor"));
    await userEvent.type(screen.getByLabelText("editor"), "lo de B");

    await settled();

    expect(store.bodies["a3f1-0002"]).toBe("lo de B");
    expect(store.bodies["a3f1-0001"]).toBe("lo de A");
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

describe("a document that moved on disk while it was open", () => {
  beforeEach(() => {
    store.bodies = { "a3f1-0001": "# Compras\n\nleche" };
    store.writes = [];
    store.reads = 0;
    store.delays = [];
    store.mute = false;
    store.shape = null;
  });

  it("reads it again when something wrote beside the window", async () => {
    const props = { open: "a3f1-0001", known, onKept: vi.fn(), onError: vi.fn(), fresh: 0 };
    const { rerender } = render(<Docs {...props} />);
    await waitFor(() => expect(screen.getByLabelText("editor")).toHaveProperty("value"));

    store.bodies["a3f1-0001"] = "# Compras\n\nleche\n\npan";
    rerender(<Docs {...props} fresh={1} />);

    await waitFor(() =>
      expect(screen.getByLabelText<HTMLTextAreaElement>("editor").value).toContain("pan"),
    );
  });

  it("does not throw away what is still being typed", async () => {
    const props = { open: "a3f1-0001", known, onKept: vi.fn(), onError: vi.fn(), fresh: 0 };
    const { rerender } = render(<Docs {...props} />);
    const editor = await screen.findByLabelText<HTMLTextAreaElement>("editor");
    await userEvent.type(editor, " y huevos");

    store.bodies["a3f1-0001"] = "# Compras\n\nleche\n\npan";
    rerender(<Docs {...props} fresh={1} />);

    await waitFor(() =>
      expect(screen.getByLabelText<HTMLTextAreaElement>("editor").value).toContain("y huevos"),
    );
    await settled();
  });

  it("does not write over what arrived while it was being typed in", async () => {
    store.clash = true;
    const onError = vi.fn();
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={onError} />);
    const editor = await screen.findByLabelText("editor");
    await userEvent.type(editor, " y pan");

    await settled();

    await waitFor(() => screen.getByText(/mientras lo tenías abierto|while you had it open/));
    expect(store.bodies["a3f1-0001"]).not.toContain("y pan");
    expect(onError).not.toHaveBeenCalled();
  });

  it("writes it whole when the person says theirs is the one that stands", async () => {
    store.clash = true;
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    const editor = await screen.findByLabelText("editor");
    await userEvent.type(editor, " y pan");
    await settled();
    await waitFor(() => screen.getByText(/mientras lo tenías abierto|while you had it open/));

    await userEvent.click(screen.getByText(/de todos modos|anyway/));
    await settled();

    expect(store.writes[store.writes.length - 1]?.anyway).toBe(true);
    expect(store.bodies["a3f1-0001"]).toContain("y pan");
    expect(screen.queryByText(/mientras lo tenías abierto|while you had it open/)).toBeNull();
  });
});

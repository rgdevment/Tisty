import { render, screen, waitFor } from "@testing-library/react";
import { Editor } from "@tiptap/core";
import { describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import Docs from "../ui/Docs";
import { previewing, type Reach } from "../ui/previewing";
import { written } from "../ui/writing";

const store = vi.hoisted(() => ({ wrote: [] as string[], ordered: [] as string[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "doc_read") {
      return Promise.resolve("<div>lo que el editor no sabe guardar</div>\n\n# Frágil\n");
    }
    if (cmd === "doc_write") {
      store.wrote.push(String(args?.id));
      return Promise.resolve({ id: String(args?.id), title: "Frágil" });
    }
    if (cmd === "doc_order") {
      store.ordered.push(String(args?.id));
      return Promise.resolve(false);
    }
    return Promise.resolve(null);
  },
}));

vi.mock("../ui/Editor", () => ({
  default: ({ value, reading }: { value: string; reading?: boolean }) => (
    <textarea aria-label="editor" readOnly={reading} value={value} />
  ),
}));

const known: Filed[] = [
  { id: "01A", file: "a3f1-0001", title: "Frágil", folder: null, archived: false },
  {
    id: "01B",
    file: "a3f1-0002",
    title: "Una página",
    folder: null,
    archived: false,
    pageOf: "01A",
  },
];

describe("a document the editor cannot write back", () => {
  it("does not say it takes files, so none is copied in before it is refused", async () => {
    const { CATCHES } = await import("../dropped");
    const { default: Written } =
      await vi.importActual<typeof import("../ui/Editor")>("../ui/Editor");

    const { container } = render(<Written value="# Frágil" reading onWrite={vi.fn()} />);
    await waitFor(() => expect(container.querySelector(".tisty-doc")).toBeTruthy());

    expect(container.querySelector(".tisty-doc")?.hasAttribute(CATCHES)).toBe(false);
  });

  it("does not settle where its pages sit, which would be writing to it", async () => {
    store.ordered = [];
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);

    await waitFor(() => expect(screen.getByLabelText("editor")).toBeTruthy());
    await new Promise((go) => setTimeout(go, 30));

    expect(store.ordered).toEqual([]);
  });

  it("offers nothing on a card, and the key that removes one does nothing", () => {
    const reach: Reach = { url: () => null, weight: () => null, title: () => "Una página" };
    const editor = new Editor({
      extensions: [...written(), previewing(() => reach)],
      content: "![Una página](tisty:doc/a3f1-0002)",
      editable: false,
    });
    const card = editor.view.dom.querySelector<HTMLElement>(".card");
    const was = editor.getHTML();

    expect(card?.querySelector(".card-swap")).toBeNull();
    card?.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete", bubbles: true }));

    expect(editor.getHTML()).toBe(was);
    editor.destroy();
  });
});

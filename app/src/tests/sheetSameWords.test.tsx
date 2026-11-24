import { render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Sheet from "../ui/Sheet";
import { near } from "./nearing";

const ipc = vi.hoisted(() => ({
  drawn: {} as Record<string, string>,
  near: null as unknown,
  asked: [] as string[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: { body?: string } = {}) => {
    if (cmd === "doc_drawn" || cmd === "doc_placed") {
      const body = args.body ?? "";
      ipc.asked.push(body);
      return Promise.resolve(new TextEncoder().encode(ipc.drawn[body] ?? "").buffer);
    }
    if (cmd === "doc_placed_near") return Promise.resolve(ipc.near);
    if (cmd === "mends_in") return Promise.resolve([]);
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
}));

const NOTES = "# Notas\n";
const TITLED = "# a";
const UNTOUCHED = "Uno\n";
const TOUCHED = "Unos\n";
const TOUCHED_DRAWN = '<p data-at="0-4"><span data-src="0-4">Unos</span></p>';
const TOLD_MS = 600;

const loaded = async (container: HTMLElement) => {
  const frame = container.querySelector("iframe");
  await waitFor(() => expect(frame?.srcdoc ?? "").toContain("<article"));
  const page = frame?.contentDocument as Document;
  page.open();
  page.write(frame?.srcdoc ?? "");
  page.close();
  frame?.dispatchEvent(new Event("load"));
  return page;
};

const sheetOf = (page: Document) => page.querySelector<HTMLElement>(".page") as HTMLElement;

const typed = (page: Document, data: string) =>
  sheetOf(page).dispatchEvent(
    new InputEvent("beforeinput", {
      inputType: "insertText",
      data,
      bubbles: true,
      cancelable: true,
    }),
  );

const caretAt = (page: Document, at: number) =>
  page.getSelection()?.collapse(page.querySelector("[data-src]")?.firstChild as Text, at);

const writable = (page: Document) =>
  waitFor(() => expect(sheetOf(page).getAttribute("contenteditable")).toBe("true"));

const waitsAtTitle = async (page: Document) => {
  await writable(page);
  await waitFor(() => {
    const slot = page.querySelector("h1[data-title][data-ghost]");
    expect(slot).not.toBeNull();
    expect(page.getSelection()?.anchorNode).toBe(slot);
  });
};

const opened = async (file: string, value: string) => {
  const writes: [string, string][] = [];
  const sheet = (file: string, value: string) => (
    <Sheet file={file} value={value} label={file} onWrite={(text) => writes.push([file, text])} />
  );
  const view = render(sheet(file, value));
  await waitFor(() => expect(view.container.querySelector("iframe")).not.toBeNull());
  const page = await loaded(view.container);
  return { page, writes, show: (to: string, words: string) => view.rerender(sheet(to, words)) };
};

beforeEach(() => {
  ipc.near = null;
  ipc.asked = [];
  ipc.drawn = {
    "": "",
    [NOTES]: '<h1 data-at="0-7"><span data-src="2-7">Notas</span></h1>',
    [TITLED]: '<h1 data-at="0-3"><span data-src="2-3">a</span></h1>',
    [UNTOUCHED]: '<p data-at="0-3"><span data-src="0-3">Uno</span></p>',
    [TOUCHED]: TOUCHED_DRAWN,
  };
});

describe("a document with the same words as the one before it", () => {
  it("is written on when both are empty", async () => {
    const { page, writes, show } = await opened("a3f1-0001", "");
    await waitsAtTitle(page);
    show("a3f1-0002", "");
    await waitsAtTitle(page);
    typed(page, "a");
    await waitFor(() => expect(writes).toEqual([["a3f1-0002", TITLED]]));
  });

  it("is written on when it opens empty after one with words", async () => {
    const { page, writes, show } = await opened("a3f1-0001", NOTES);
    await waitFor(() => expect(sheetOf(page).textContent).toBe("Notas"));
    show("a3f1-0002", "");
    await waitsAtTitle(page);
    typed(page, "a");
    await waitFor(() => expect(writes).toEqual([["a3f1-0002", TITLED]]));
  });

  it("waits at its title when the empty one before was seen as Markdown", async () => {
    const writes: string[] = [];
    const sheet = (file: string, markdown: boolean) => (
      <Sheet
        file={file}
        value=""
        label={file}
        view={{ markdown, onTurn: () => {} }}
        onWrite={(text) => writes.push(text)}
      />
    );
    const view = render(sheet("a3f1-0001", false));
    await waitFor(() => expect(view.container.querySelector("iframe")).not.toBeNull());
    const page = await loaded(view.container);
    await waitsAtTitle(page);
    view.rerender(sheet("a3f1-0001", true));
    await waitFor(() => expect(page.querySelector(".cm-editor")).not.toBeNull());
    view.rerender(sheet("a3f1-0002", false));
    await new Promise((done) => setTimeout(done, TOLD_MS));
    await waitsAtTitle(page);
    typed(page, "a");
    await waitFor(() => expect(writes).toEqual([TITLED]));
  });

  it("is written on when the one before was written into those words", async () => {
    const { page, writes, show } = await opened("a3f1-0001", "");
    await waitsAtTitle(page);
    typed(page, "a");
    await waitFor(() => expect(writes).toEqual([["a3f1-0001", TITLED]]));
    await waitFor(() => expect(sheetOf(page).textContent).toBe("a"));
    show("a3f1-0002", TITLED);
    await writable(page);
    caretAt(page, 1);
    typed(page, "b");
    await waitFor(() => expect(writes[1]).toEqual(["a3f1-0002", "# ab"]));
  });

  it("shows its own page when another came between, though written on in place before", async () => {
    ipc.near = near({ spans: [[0, 4]], from: 0, to: 1, html: TOUCHED_DRAWN });
    const { page, writes, show } = await opened("a3f1-0001", UNTOUCHED);
    await writable(page);
    caretAt(page, 3);
    typed(page, "s");
    await waitFor(() => expect(writes).toEqual([["a3f1-0001", TOUCHED]]));
    await new Promise((done) => setTimeout(done, TOLD_MS));
    expect(sheetOf(page).textContent).toBe("Unos");
    expect(ipc.asked).not.toContain(TOUCHED);
    show("a3f1-0002", NOTES);
    await waitFor(() => expect(sheetOf(page).textContent).toBe("Notas"));
    show("a3f1-0003", TOUCHED);
    await waitFor(() => expect(sheetOf(page).textContent).toBe("Unos"));
  });
});

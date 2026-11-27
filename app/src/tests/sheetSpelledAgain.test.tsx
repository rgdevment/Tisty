import { render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Sheet from "../ui/Sheet";

const ipc = vi.hoisted(() => ({ kept: true, learned: [] as string[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: { word?: string }) => {
    if (cmd === "doc_drawn" || cmd === "doc_placed") {
      return Promise.resolve(new TextEncoder().encode(PLACED).buffer);
    }
    if (cmd === "word_spelled") return Promise.resolve({ wrong: true, guesses: [] });
    if (cmd === "word_learned") {
      ipc.learned.push(args.word ?? "");
      return Promise.resolve(ipc.kept);
    }
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
}));

const PLACED = '<p data-at="0-7"><span data-src="0-6">prueva</span></p>';

const loaded = async (frame: HTMLIFrameElement) => {
  await waitFor(() => expect(frame.srcdoc).toContain("<article"));
  const page = frame.contentDocument as Document;
  page.open();
  page.write(frame.srcdoc);
  page.close();
  frame.dispatchEvent(new Event("load"));
  return page;
};

const learnedFromTheMenu = async () => {
  const view = render(<Sheet value="prueva" label="Prueba" onWrite={() => {}} wholeable />);
  const frame = await waitFor(() => {
    const found = view.container.querySelector("iframe");
    expect(found).toBeTruthy();
    return found as HTMLIFrameElement;
  });
  const page = await loaded(frame);
  const sheet = page.querySelector(".page") as HTMLElement;
  await waitFor(() => expect(sheet.getAttribute("contenteditable")).toBe("true"));
  const words = page.querySelector("[data-src]") as HTMLElement;
  const range = page.createRange();
  range.setStart(words.firstChild as Node, 2);
  range.setEnd(words.firstChild as Node, 2);
  page.caretRangeFromPoint = () => range;
  const frames: FrameRequestCallback[] = [];
  vi.spyOn(page.defaultView as Window, "requestAnimationFrame").mockImplementation((frame) =>
    frames.push(frame),
  );
  words.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true }));
  const learn = await waitFor(() => {
    const found = Array.from(document.querySelectorAll<HTMLElement>('[role="menuitem"]')).find(
      (one) => one.textContent?.includes("to the dictionary"),
    );
    expect(found).toBeTruthy();
    return found as HTMLElement;
  });
  learn.click();
  await waitFor(() => expect(ipc.learned).toEqual(["prueva"]));
  return { sheet, frames };
};

beforeEach(() => {
  ipc.kept = true;
  ipc.learned = [];
});

describe("a word added to the dictionary from the page menu", () => {
  it("turns the page's spellcheck off and back on at the next frame once it is kept", async () => {
    const { sheet, frames } = await learnedFromTheMenu();

    await waitFor(() => expect(sheet.spellcheck).toBe(false));
    expect(frames).toHaveLength(1);
    frames[0](0);
    expect(sheet.spellcheck).toBe(true);
  });

  it("leaves the page's spellcheck alone when the system does not keep the word", async () => {
    ipc.kept = false;
    const { sheet, frames } = await learnedFromTheMenu();
    await new Promise((settled) => setTimeout(settled, 50));

    expect(frames).toHaveLength(0);
    expect(sheet.spellcheck).not.toBe(false);
  });
});

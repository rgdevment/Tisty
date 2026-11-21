import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Sheet from "../ui/Sheet";

const ipc = vi.hoisted(() => ({ fails: false, drawn: "", asked: 0, holding: false }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "doc_drawn" || cmd === "doc_placed") {
      ipc.asked += 1;
      if (ipc.holding) return new Promise(() => {});
      if (ipc.fails) return Promise.reject(new TypeError("the drawing never came back"));
      return Promise.resolve(new TextEncoder().encode(ipc.drawn).buffer);
    }
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
}));

const BODY = "# Compras\n\nleche\n";
const DRAWN =
  '<h1 data-at="0-10"><span data-src="2-9">Compras</span></h1>' +
  '<p data-at="11-17"><span data-src="11-16">leche</span></p>';

const loaded = async (frame: HTMLIFrameElement) => {
  await waitFor(() => expect(frame.srcdoc).toContain("<article"));
  const page = frame.contentDocument as Document;
  page.open();
  page.write(frame.srcdoc);
  page.close();
  frame.dispatchEvent(new Event("load"));
  return page;
};

const framed = (container: HTMLElement) =>
  waitFor(() => {
    const found = container.querySelector("iframe");
    expect(found).toBeTruthy();
    return found as HTMLIFrameElement;
  });

const typedAtTheMargin = (page: Document) => {
  const sheet = page.querySelector(".page") as HTMLElement;
  page.getSelection()?.collapse(sheet, 0);
  sheet.dispatchEvent(
    new InputEvent("beforeinput", {
      inputType: "insertText",
      data: "x",
      bubbles: true,
      cancelable: true,
    }),
  );
};

beforeEach(() => {
  ipc.fails = false;
  ipc.drawn = DRAWN;
  ipc.asked = 0;
  ipc.holding = false;
});

describe("a page the core could not draw", () => {
  it("shows the words as written and takes no key into the document", async () => {
    ipc.fails = true;
    const writes: string[] = [];
    const view = render(
      <Sheet value={BODY} label="Compras" onWrite={(text) => writes.push(text)} />,
    );
    const page = await loaded(await framed(view.container));
    await waitFor(() => expect(page.querySelector(".page pre")).toBeTruthy());

    typedAtTheMargin(page);
    await new Promise((settled) => setTimeout(settled, 50));

    expect(writes).toEqual([]);
    expect(page.querySelector(".page")?.getAttribute("contenteditable")).not.toBe("true");
  });

  it("says it could not be drawn, with why, and draws it again when asked", async () => {
    ipc.fails = true;
    const view = render(<Sheet value={BODY} label="Compras" onWrite={() => {}} />);
    const page = await loaded(await framed(view.container));
    const told = await screen.findByRole("alert");
    expect(told.textContent).toContain("the drawing never came back");

    ipc.fails = false;
    await userEvent.click(screen.getByRole("button", { name: t("tryAgain") }));

    await waitFor(() => expect(page.querySelector('.page [data-src="11-16"]')).toBeTruthy());
    await waitFor(() =>
      expect(page.querySelector(".page")?.getAttribute("contenteditable")).toBe("true"),
    );
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("says it is drawing again, not that it failed, while newer words are on their way", async () => {
    ipc.fails = true;
    const sheet = (value: string) => <Sheet value={value} label="Compras" onWrite={() => {}} />;
    const view = render(sheet(BODY));
    await loaded(await framed(view.container));
    await screen.findByRole("alert");

    ipc.holding = true;
    view.rerender(sheet(`${BODY}\npan\n`));

    expect(await screen.findByText(t("pageDrawingAgain"))).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("stops taking keys when a later drawing of a page being written fails", async () => {
    const writes: string[] = [];
    const sheet = (value: string) => (
      <Sheet value={value} label="Compras" onWrite={(text) => writes.push(text)} />
    );
    const view = render(sheet(BODY));
    const page = await loaded(await framed(view.container));
    await waitFor(() =>
      expect(page.querySelector(".page")?.getAttribute("contenteditable")).toBe("true"),
    );

    ipc.fails = true;
    view.rerender(sheet(`${BODY}\npan\n`));
    await waitFor(() => expect(page.querySelector(".page pre")).toBeTruthy());
    typedAtTheMargin(page);
    await new Promise((settled) => setTimeout(settled, 50));

    expect(writes).toEqual([]);
    expect(page.querySelector(".page")?.getAttribute("contenteditable")).not.toBe("true");
  });
});

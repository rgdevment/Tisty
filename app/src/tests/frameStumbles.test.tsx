import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Standing from "../Standing";
import Page from "../ui/Page";

const ipc = vi.hoisted(() => ({
  drawn: "",
  breaks: [] as { kind: string; said: string | null; stopped: boolean }[],
  wearing: false,
  fitting: false,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: { kind: string; said: string | null; stopped: boolean }) => {
    if (cmd === "note_break" && args)
      ipc.breaks.push({ kind: args.kind, said: args.said, stopped: args.stopped });
    if (cmd === "doc_drawn") return Promise.resolve(new TextEncoder().encode(ipc.drawn).buffer);
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
}));

vi.mock("../ui/dressing", async (real) => {
  const own = await real<typeof import("../ui/dressing")>();
  return {
    ...own,
    worn: (...asked: Parameters<typeof own.worn>) => {
      if (ipc.wearing) throw new TypeError("a widget could not be dressed");
      return own.worn(...asked);
    },
    fitted: (...asked: Parameters<typeof own.fitted>) => {
      if (ipc.fitting) throw new RangeError("the sheet could not be fitted");
      return own.fitted(...asked);
    },
  };
});

const FIRST = '<p data-at="0-5">Uno</p>';
const NEXT = '<p data-at="0-5">Dos</p>';

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

const page = (body: string, shown: (Window | null)[], wide = 900) => (
  <Standing>
    <Page body={body} label="Notas" wide={wide} onDoc={() => {}} onShown={(at) => shown.push(at)} />
    <p>al lado</p>
  </Standing>
);

beforeEach(() => {
  ipc.drawn = FIRST;
  ipc.breaks = [];
  ipc.wearing = false;
  ipc.fitting = false;
  vi.spyOn(console, "error").mockImplementation(() => {});
});

describe("a page whose wiring stumbles", () => {
  it("is still handed over and shown, says what broke, and takes its next drawing", async () => {
    ipc.wearing = true;
    const shown: (Window | null)[] = [];
    const view = render(page("Uno", shown));
    const frame = await framed(view.container);
    const doc = await loaded(frame);

    await waitFor(() => expect(shown).toContain(frame.contentWindow));
    expect(frame.classList.contains("invisible")).toBe(false);
    expect(doc.querySelector(".page")?.hasAttribute("inert")).toBe(false);
    expect(ipc.breaks).toContainEqual({
      kind: "TypeError",
      said: "a widget could not be dressed",
      stopped: false,
    });

    ipc.wearing = false;
    ipc.drawn = NEXT;
    view.rerender(page("Dos", shown));
    await waitFor(() => expect(doc.querySelector(".page")?.textContent).toContain("Dos"));
  });

  it("keeps the window standing when a later drawing cannot be dressed", async () => {
    const shown: (Window | null)[] = [];
    const view = render(page("Uno", shown));
    const doc = await loaded(await framed(view.container));
    await waitFor(() => expect(doc.querySelector(".page")?.textContent).toContain("Uno"));

    ipc.wearing = true;
    ipc.drawn = NEXT;
    view.rerender(page("Dos", shown));

    await waitFor(() => expect(doc.querySelector(".page")?.textContent).toContain("Dos"));
    expect(screen.queryByText(t("windowFell"))).toBeNull();
    expect(ipc.breaks).toContainEqual({
      kind: "TypeError",
      said: "a widget could not be dressed",
      stopped: false,
    });
  });
});

describe("Backspace on a page, outside its words", () => {
  it("never walks the window back to an earlier page", async () => {
    const shown: (Window | null)[] = [];
    const view = render(page("Uno", shown));
    const doc = await loaded(await framed(view.container));
    await waitFor(() => expect(shown).toContain(doc.defaultView));
    const button = doc.createElement("button");
    doc.body.append(button);
    const pressed = new KeyboardEvent("keydown", {
      key: "Backspace",
      bubbles: true,
      cancelable: true,
    });
    button.dispatchEvent(pressed);
    expect(pressed.defaultPrevented).toBe(true);
  });
});

describe("a page that stops drawing", () => {
  it("falls alone, with why, and is drawn again when asked", async () => {
    const shown: (Window | null)[] = [];
    const view = render(page("Uno", shown));
    await loaded(await framed(view.container));

    ipc.fitting = true;
    view.rerender(page("Uno", shown, 1200));

    const told = await screen.findByRole("alert");
    expect(told.textContent).toContain("the sheet could not be fitted");
    expect(screen.queryByText(t("windowFell"))).toBeNull();
    expect(screen.getByText("al lado")).toBeTruthy();
    expect(ipc.breaks).toContainEqual({
      kind: "RangeError",
      said: "the sheet could not be fitted",
      stopped: false,
    });

    ipc.fitting = false;
    await userEvent.click(screen.getByRole("button", { name: t("tryAgain") }));

    const again = await framed(view.container);
    const doc = await loaded(again);
    await waitFor(() => expect(doc.querySelector(".page")?.textContent).toContain("Uno"));
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("tries again by itself for other words under the same title", async () => {
    const shown: (Window | null)[] = [];
    const view = render(page("Uno", shown));
    await loaded(await framed(view.container));
    ipc.fitting = true;
    view.rerender(page("Uno", shown, 1200));
    await screen.findByRole("alert");

    ipc.fitting = false;
    ipc.drawn = NEXT;
    view.rerender(page("Dos", shown, 1200));

    const doc = await loaded(await framed(view.container));
    await waitFor(() => expect(doc.querySelector(".page")?.textContent).toContain("Dos"));
    expect(screen.queryByRole("alert")).toBeNull();
  });
});

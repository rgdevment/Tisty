import { Editor } from "@tiptap/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { AS_FILE, asPaged, previewing, type Reach } from "../ui/previewing";
import { asMarkdown, written } from "../ui/writing";

const ipc = vi.hoisted(() => ({ calls: [] as { cmd: string; args?: Record<string, unknown> }[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args });
    return Promise.resolve(cmd === "widget_lend_kept" ? "p1" : null);
  },
  convertFileSrc: (path: string, scheme: string) => `http://${scheme}.localhost/${path}`,
}));

beforeEach(() => {
  ipc.calls = [];
});

const PAGE = "attachments/ab/informe-12345678.html";

const reach: Reach = {
  url: (at) => `asset://localhost/${at}`,
  weight: () => 2_400,
  title: () => null,
};

const made = (content: string, how: Partial<Reach> = {}) => {
  const element = document.createElement("div");
  document.body.append(element);
  return new Editor({
    element,
    extensions: [...written(), previewing(() => ({ ...reach, onMenu: () => {}, ...how }))],
    content,
  });
};

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

describe("a page attached to a document", () => {
  it("is drawn as a widget rather than a card", async () => {
    const editor = made(`![informe](<${PAGE}>)`);
    const page = editor.view.dom.querySelector(".widget-page");

    expect(page).toBeTruthy();
    expect(editor.view.dom.querySelector(".preview.card")).toBeNull();
    await vi.waitFor(() => expect(page?.querySelector("iframe")).toBeTruthy());
    const frame = page?.querySelector("iframe");
    expect(frame?.getAttribute("sandbox")).toBe("allow-scripts");
    expect(frame?.src).toBe("http://widget.localhost/p1");
    expect(asked("widget_lend_kept")[0].args).toEqual({ reference: PAGE });
    editor.destroy();
  });

  it("goes back to a card on asking, and the file remembers it", async () => {
    const editor = made(`![informe](<${PAGE}>)`);
    await vi.waitFor(() =>
      expect(editor.view.dom.querySelector(".widget-page iframe")).toBeTruthy(),
    );

    editor.view.dom.querySelector<HTMLButtonElement>(".widget-page-shelve")?.click();

    expect(editor.view.dom.querySelector(".widget-page")).toBeNull();
    expect(editor.view.dom.querySelector(".preview.card")).toBeTruthy();
    expect(asMarkdown(editor)).toBe(`![informe](${PAGE} "${AS_FILE}")`);
    expect(asked("widget_take_back")[0].args).toEqual({ id: "p1" });
    editor.destroy();
  });

  it("reads a page marked as a file as a card, and offers to draw it again", () => {
    const menu = vi.fn();
    const editor = made(`![informe](${PAGE} "file")`, { onMenu: menu });

    expect(editor.view.dom.querySelector(".widget-page")).toBeNull();
    editor.view.dom.querySelector<HTMLButtonElement>(".card-swap")?.click();
    const live = menu.mock.calls[0]?.[5] as (() => void) | undefined;
    expect(live).toBeTypeOf("function");

    live?.();

    expect(editor.view.dom.querySelector(".widget-page")).toBeTruthy();
    expect(asMarkdown(editor)).toBe(`![informe](${PAGE})`);
    editor.destroy();
  });

  it("offers no way back from the widget to someone who can only read", () => {
    const editor = made(`![informe](<${PAGE}>)`, { onMenu: undefined });

    expect(editor.view.dom.querySelector(".widget-page")).toBeTruthy();
    expect(editor.view.dom.querySelector(".widget-page-shelve")).toBeNull();
    editor.destroy();
  });

  it("offers no widget for a card that is not a page", () => {
    const menu = vi.fn();
    const editor = made("![acta](<attachments/cd/acta-12345678.pdf>)", { onMenu: menu });

    editor.view.dom.querySelector<HTMLButtonElement>(".card-swap")?.click();

    expect(menu.mock.calls[0]?.[5]).toBeUndefined();
    editor.destroy();
  });

  it("knows a page by where it lives and what it ends in", () => {
    const image = (src: string, title: string | null = null) =>
      ({ type: { name: "image" }, attrs: { src, title, alt: "x" } }) as never;

    expect(asPaged(image(PAGE))).not.toBeNull();
    expect(asPaged(image("attachments/ab/viejo-12345678.HTM"))).not.toBeNull();
    expect(asPaged(image(PAGE, AS_FILE))).toBeNull();
    expect(asPaged(image("attachments/ab/foto-12345678.png"))).toBeNull();
    expect(asPaged(image("https://example.com/a.html"))).toBeNull();
    expect(asPaged(image("docs/a.html"))).toBeNull();
  });

  it("names the way back in the window's language", () => {
    expect(t("showAsAttachment")).toBeTruthy();
    expect(t("showAsWidget")).toBeTruthy();
  });
});

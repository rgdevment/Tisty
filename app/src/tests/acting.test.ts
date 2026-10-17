import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Showing } from "../core";
import { t } from "../locales";
import { type Acts, choicesFor, darkNow, keysFor, PLAIN, ZOOMS } from "../ui/viewer/acting";

const ipc = vi.hoisted(() => ({ calls: [] as { cmd: string; args?: Record<string, unknown> }[] }));
const shown = vi.hoisted(() => ({ revealed: [] as string[], copied: [] as string[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args });
    if (cmd === "doc_plain") return Promise.resolve(`standard ${args?.body}`);
    return Promise.resolve(cmd === "doc_copy" ? { id: "f2", title: "Copia" } : null);
  },
}));
vi.mock("@tauri-apps/plugin-opener", () => ({
  revealItemInDir: (at: string) => {
    shown.revealed.push(at);
    return Promise.resolve();
  },
}));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: (text: string) => {
    shown.copied.push(text);
    return Promise.resolve();
  },
}));

const file: Showing = {
  kind: "file",
  name: "a.md",
  place: "Descargas",
  path: "C:\\a.md",
  body: "# A",
};
const doc: Showing = {
  kind: "doc",
  paper: { id: "D1", file: "f1", title: "Notas", place: "" },
  body: "# Notas",
};
const widget: Showing = {
  kind: "widget",
  lent: { source: "<p>x</p>" },
  from: { id: "", file: "f1", title: "Notas", place: "" },
};

const acts = (on: Showing | null): Acts & Record<string, ReturnType<typeof vi.fn>> => {
  const spy = () => vi.fn();
  return {
    shown: on,
    body: on && "body" in on ? on.body : "",
    dark: null,
    actual: false,
    fail: spy(),
    look: spy(),
    find: spy(),
    zoom: spy(),
    side: spy(),
    raw: spy(),
    preview: spy(),
    keep: spy(),
    html: spy(),
    word: spy(),
    quietly: spy(),
    again: spy(),
    code: spy(),
    shade: spy(),
    fit: spy(),
    preset: spy(),
    close: spy(),
  } as unknown as Acts & Record<string, ReturnType<typeof vi.fn>>;
};

const settled = () => new Promise((done) => setTimeout(done, 0));

beforeEach(() => {
  ipc.calls = [];
  shown.revealed = [];
  shown.copied = [];
  document.documentElement.removeAttribute("data-theme");
});

const picked = (all: ReturnType<typeof choicesFor>, label: string) => {
  const one = all.find((choice) => choice.label.endsWith(label));
  one?.onPick?.();
  return one;
};

describe("the viewer's menu", () => {
  it("offers a file from outside its reading tools, its ways out as standard Markdown, a web page or Word, a quiet import and its folder", async () => {
    const a = acts(file);
    const all = choicesFor(a);

    picked(all, t("viewFind"));
    picked(all, t("viewBigger"));
    picked(all, t("viewSmaller"));
    picked(all, t("viewCopyMd"));
    picked(all, t("seePdf"));
    picked(all, t("toPdf"));
    picked(all, t("toHtml"));
    picked(all, t("toWord"));
    picked(all, t("viewImportQuiet"));
    picked(all, t("viewReveal"));
    await settled();

    expect(a.find).toHaveBeenCalled();
    expect(vi.mocked(a.zoom).mock.calls).toEqual([[1], [-1]]);
    expect(a.preview).toHaveBeenCalled();
    expect(a.keep).toHaveBeenCalled();
    expect(a.html).toHaveBeenCalled();
    expect(a.word).toHaveBeenCalled();
    expect(a.quietly).toHaveBeenCalled();
    expect(shown.copied).toEqual(["standard # A"]);
    expect(shown.revealed).toEqual(["C:\\a.md"]);
    expect(all.some((one) => one.label === t("viewDuplicate"))).toBe(false);
  });

  it("offers a Tisty document a copy of itself, opened in the main window", async () => {
    const all = choicesFor(acts(doc));

    picked(all, t("viewDuplicate"));
    await settled();

    expect(ipc.calls.map((one) => one.cmd)).toEqual(["doc_copy", "view_go"]);
    expect(ipc.calls[1].args).toEqual({ file: "f2" });
    expect(all.some((one) => one.label === t("viewImportQuiet"))).toBe(false);
  });

  it("offers a file that could not be read only its folder and another try", () => {
    const a = acts({
      kind: "missed",
      name: "a.md",
      place: "Descargas",
      path: "C:\\a.md",
      refusal: { code: "cannotRead" },
    });
    const all = choicesFor(a);

    picked(all, t("viewRetry"));

    expect(all.map((one) => one.key)).toEqual(["reveal", "retry"]);
    expect(a.look).toHaveBeenCalled();
  });

  it("offers a widget its code, its document and how it is drawn", async () => {
    const a = acts(widget);
    const all = choicesFor(a);

    picked(all, t("viewReload"));
    picked(all, t("viewCode"));
    picked(all, t("viewInDocGo"));
    picked(all, t("viewDark"));
    picked(all, t("viewActual"));
    await settled();

    expect(a.again).toHaveBeenCalled();
    expect(a.code).toHaveBeenCalled();
    expect(a.shade).toHaveBeenCalled();
    expect(a.fit).toHaveBeenCalled();
    expect(ipc.calls).toContainEqual({ cmd: "view_go", args: { file: "f1" } });
  });

  it("ticks the dark widget by the theme it is really drawn in", () => {
    document.documentElement.setAttribute("data-theme", "dark");

    const ticked = choicesFor(acts(widget)).find((one) => one.key === "dark");

    expect(darkNow()).toBe(true);
    expect(ticked?.label.startsWith("✓")).toBe(true);
  });

  it("offers no code nor document for a page drawn from an attachment", () => {
    const all = choicesFor(
      acts({ kind: "widget", lent: { reference: "attachments/a.html" }, from: null }),
    );

    expect(all.find((one) => one.key === "code")?.off).toBe(true);
    expect(all.find((one) => one.key === "doc")?.off).toBe(true);
  });
});

const key = (key: string, shiftKey = false) => new KeyboardEvent("keydown", { key, shiftKey });

describe("the viewer's keys", () => {
  it("reads, finds, zooms and prints a document", () => {
    const a = acts(file);
    for (const one of ["f", "\\", "p", "=", "+", "-", "0", "w", "r"]) keysFor(a, key(one))?.();
    keysFor(a, key("M", true))?.();

    expect(a.find).toHaveBeenCalled();
    expect(a.side).toHaveBeenCalled();
    expect(a.preview).toHaveBeenCalled();
    expect(vi.mocked(a.zoom).mock.calls).toEqual([[1], [1], [-1], [0]]);
    expect(a.raw).toHaveBeenCalled();
    expect(a.close).toHaveBeenCalled();
    expect(keysFor(a, key("m"))).toBeUndefined();
  });

  it("picks a widget's width by number and its true size by zero", () => {
    const a = acts(widget);
    keysFor(a, key("2"))?.();
    keysFor(a, key("0"))?.();

    expect(a.preset).toHaveBeenCalledWith("390");
    expect(a.fit).toHaveBeenCalled();
    expect(keysFor(a, key("f"))).toBeUndefined();
  });

  it("starts the text at its plain size", () => {
    expect(ZOOMS[PLAIN]).toBe(1);
  });
});

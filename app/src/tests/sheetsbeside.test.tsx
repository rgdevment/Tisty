import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView, keymap, runScopeHandlers } from "@codemirror/view";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Beside from "../ui/Beside";
import { blockKeys } from "../ui/barring";
import { sheeting } from "../ui/sheeting";
import { sizeOf } from "../ui/sheets";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const panel = (sheets: boolean, onSheets = vi.fn(), onLandscape = vi.fn()) =>
  render(
    <Beside
      title="Notas"
      paper="dev_a-0001"
      body="# Notas"
      tags={[]}
      kept={0}
      heads={[]}
      pages={[]}
      leaf="a4"
      onLeaf={() => {}}
      sheets={sheets}
      landscape={false}
      onSheets={onSheets}
      onLandscape={onLandscape}
      making={false}
      onCopy={() => {}}
      onTakeOut={() => {}}
      onShut={() => {}}
      onApart={() => {}}
    />,
  );

describe("the sheets a document is read on", () => {
  it("turns them on from a quiet switch and offers the orientation only once they are on", async () => {
    const onSheets = vi.fn();
    const off = panel(false, onSheets);
    expect(screen.queryByRole("button", { name: t("sheetsAcross") })).toBeNull();
    await userEvent.click(screen.getByRole("switch", { name: t("sheetsShown") }));
    expect(onSheets).toHaveBeenCalledWith(true);
    off.unmount();

    const onLandscape = vi.fn();
    panel(true, vi.fn(), onLandscape);
    await userEvent.click(screen.getByRole("button", { name: t("sheetsAcross") }));
    expect(onLandscape).toHaveBeenCalledWith(true);
  });

  it("lays the sheets, marks a manual break, and lets go of them when the paper asks for none", () => {
    document.body.innerHTML = '<article class="page"><p>Uno</p><hr><p>Dos</p></article>';
    const asked: { now?: { wide: number; tall: number; x: number; y: number; said?: string } } = {
      now: { wide: 794, tall: 1123, x: 76, y: 83, said: t("sheetsBreak") },
    };
    const laid = sheeting(document, () => asked.now);
    const root = document.documentElement;
    expect(root.classList.contains("sheeted")).toBe(true);
    expect(root.style.getPropertyValue("--sheet-y")).toBe("83px");
    expect(document.querySelector(".break-said")?.textContent).toBe(t("sheetsBreak"));
    asked.now = undefined;
    laid.lay();
    expect(root.classList.contains("sheeted")).toBe(false);
    laid.stop();
  });

  it("prints a sheet laid across as its paper turned", () => {
    expect(sizeOf("a4")).toBe("A4");
    expect(sizeOf("letter", true)).toBe("Letter landscape");
    expect(sizeOf("tabloid", true)).toBe("17in 11in");
  });
});

const writing = (doc: string, at: number) =>
  new EditorView({
    parent: document.body,
    state: EditorState.create({
      doc,
      selection: EditorSelection.single(at),
      extensions: [markdown({ base: markdownLanguage }), keymap.of(blockKeys)],
    }),
  });

const pressed = (view: EditorView) =>
  runScopeHandlers(view, new KeyboardEvent("keydown", { key: "Enter", ctrlKey: true }), "editor");

describe("a page break written while writing", () => {
  it("goes in among words, and never inside code, a table or the properties", () => {
    const words = writing("Uno dos", 7);
    expect(pressed(words)).toBe(true);
    expect(words.state.doc.toString()).toContain("---");
    words.destroy();

    for (const [doc, at] of [
      ["```js\nlet a = 1;\n```", 6],
      ["| a | b |\n|---|---|\n| 1 | 2 |", 22],
      ["---\ntags: a\n---", 9],
    ] as const) {
      const view = writing(doc, at);
      expect(pressed(view)).toBe(false);
      expect(view.state.doc.toString()).toBe(doc);
      view.destroy();
    }
  });
});

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Beside from "../ui/Beside";
import type { Head } from "../ui/paging";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const headed = (many: number): Head[] =>
  Array.from({ length: many }, (_, at) => ({
    key: `h${at}`,
    level: 2,
    text: `Sección ${at + 1}`,
    go: () => {},
  }));

const beside = (heads: Head[], more: { onCopy?: () => void } = {}) => (
  <Beside
    title="Notas"
    paper="dev_a-0001"
    body="# Notas"
    tags={[]}
    kept={0}
    heads={heads}
    pages={[]}
    leaf="a4"
    onLeaf={() => {}}
    making={false}
    onPdf={() => {}}
    onSee={() => {}}
    onCopy={more.onCopy ?? (() => {})}
    onTakeOut={() => {}}
    onShut={() => {}}
    onApart={() => {}}
  />
);

const panel = (heads: Head[], more: { onCopy?: () => void } = {}) => render(beside(heads, more));

describe("the document's column", () => {
  it("opens the details with a document whose outline is short", () => {
    panel(headed(2));
    expect(
      screen.getByRole("button", { name: t("besideDetails") }).getAttribute("aria-expanded"),
    ).toBe("true");
  });

  it("keeps the details folded when the outline needs the room", () => {
    panel(headed(12));
    const details = screen.getByRole("button", { name: new RegExp(t("besideDetails")) });
    expect(details.getAttribute("aria-expanded")).toBe("false");
  });

  it("folds the details once a long outline arrives after the page is drawn", () => {
    const view = panel([]);
    view.rerender(beside(headed(12)));
    const details = screen.getByRole("button", { name: new RegExp(t("besideDetails")) });
    expect(details.getAttribute("aria-expanded")).toBe("false");
  });

  it("leaves what the person opened alone when the outline grows", async () => {
    const view = panel(headed(12));
    await userEvent.click(screen.getByRole("button", { name: new RegExp(t("besideDetails")) }));
    view.rerender(beside(headed(14)));
    expect(
      screen.getByRole("button", { name: t("besideDetails") }).getAttribute("aria-expanded"),
    ).toBe("true");
  });

  it("opens one part at a time", async () => {
    panel(headed(2));
    await userEvent.click(screen.getByRole("button", { name: new RegExp(t("besideShared")) }));
    expect(screen.getByText(t("sharedWill"))).toBeTruthy();
    expect(
      screen
        .getByRole("button", { name: new RegExp(t("besideDetails")) })
        .getAttribute("aria-expanded"),
    ).toBe("false");
  });

  it("gathers every way out of the document under one button, the web page among them", async () => {
    const onCopy = vi.fn();
    panel(headed(2), { onCopy });
    await userEvent.click(screen.getByRole("button", { name: t("exportIt") }));
    for (const way of [t("previewPdf"), t("toPdf"), t("toHtml"), t("toWord"), t("saveAsMarkdown")])
      expect(screen.getByRole("menuitem", { name: new RegExp(way) })).toBeTruthy();
    await userEvent.click(screen.getByRole("menuitem", { name: new RegExp(t("copyPlain")) }));
    expect(onCopy).toHaveBeenCalledTimes(1);
  });
});

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Beside from "../ui/Beside";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

describe("a document opened in a window of its own", () => {
  it("is asked from the document's panel, among its other ways out", async () => {
    const onApart = vi.fn();
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
        making={false}
        onPdf={() => {}}
        onSee={() => {}}
        onCopy={() => {}}
        onTakeOut={() => {}}
        onShut={() => {}}
        onApart={onApart}
      />,
    );

    expect(screen.getByText(t("bandApart"))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("viewDocApart") }));

    expect(onApart).toHaveBeenCalledTimes(1);
  });
});

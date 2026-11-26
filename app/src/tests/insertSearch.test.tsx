import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { adopt } from "../locales";
import { insertChoices } from "../ui/InsertMenu";
import Menu from "../ui/Menu";

const shown = (pick = vi.fn()) =>
  render(
    <Menu
      at={{ x: 20, y: 30 }}
      choices={insertChoices({ kinds: [], steps: [], sections: false, pick }).choices}
      label="Insertar"
      find="Busca"
      onClose={vi.fn()}
    />,
  );

afterEach(() => adopt("en"));

describe("a callout variant found by its own words in the Insert menu", () => {
  it("opens on the warning, which Spanish labels «Aviso», when «advertencia» is typed", async () => {
    adopt("es");
    const pick = vi.fn();
    shown(pick);

    await userEvent.type(screen.getByRole("searchbox"), "advertencia{Enter}");

    expect((document.activeElement as HTMLElement).textContent).toContain("Aviso");
    await userEvent.keyboard("{Enter}");
    expect(pick).toHaveBeenCalledWith(expect.objectContaining({ key: "callout-warning" }));
  });

  it("opens on the caution when «precaución» is typed", async () => {
    adopt("es");
    const pick = vi.fn();
    shown(pick);

    await userEvent.type(screen.getByRole("searchbox"), "precaución{Enter}{Enter}");

    expect(pick).toHaveBeenCalledWith(expect.objectContaining({ key: "callout-caution" }));
  });

  it("opens on the caution for «danger» and the tip for «hint» in English", async () => {
    const pick = vi.fn();
    const view = shown(pick);

    await userEvent.type(screen.getByRole("searchbox"), "danger{Enter}{Enter}");
    expect(pick).toHaveBeenLastCalledWith(expect.objectContaining({ key: "callout-caution" }));
    view.unmount();

    shown(pick);
    await userEvent.type(screen.getByRole("searchbox"), "hint{Enter}{Enter}");
    expect(pick).toHaveBeenLastCalledWith(expect.objectContaining({ key: "callout-tip" }));
  });
});

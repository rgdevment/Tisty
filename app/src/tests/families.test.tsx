import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Pick from "../ui/Pick";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "icons") return Promise.resolve(["home", "bed", "coffee", "wine"]);
    if (cmd === "families")
      return Promise.resolve([
        ["home", 2],
        ["table", 2],
      ]);
    return Promise.resolve(null);
  },
}));

const show = () => render(<Pick onIcon={vi.fn()} opens="icons" />);

describe("a catalogue offered by family", () => {
  it("names each family in the reader's own words", async () => {
    show();

    expect(await screen.findByRole("button", { name: "Kitchen" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Home" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "All" })).toBeTruthy();
  });

  it("narrows to the family that was pressed", async () => {
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Kitchen" }));

    expect(screen.getByRole("button", { name: "coffee" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "bed" })).toBeNull();
  });

  it("gives the whole catalogue back", async () => {
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Kitchen" }));
    await userEvent.click(screen.getByRole("button", { name: "All" }));

    expect(screen.getByRole("button", { name: "bed" })).toBeTruthy();
  });

  it("searches across them all, so a family cannot hide what was typed", async () => {
    show();
    await userEvent.click(await screen.findByRole("button", { name: "Kitchen" }));
    await userEvent.type(screen.getByRole("textbox"), "bed");

    expect(screen.getByRole("button", { name: "bed" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Kitchen" })).toBeNull();
  });

  it("writes the family over its own rows when they are all shown at once", async () => {
    Object.defineProperty(HTMLFieldSetElement.prototype, "clientWidth", { value: 200 });
    Object.defineProperty(HTMLFieldSetElement.prototype, "clientHeight", { value: 400 });
    show();
    await waitFor(() => screen.getByRole("button", { name: "home" }));

    expect(screen.getAllByText("Kitchen").length).toBe(2);
  });
});

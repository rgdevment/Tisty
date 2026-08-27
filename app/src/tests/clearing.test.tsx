import { render, screen } from "@testing-library/react";
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

describe("the button that clears an icon already chosen", () => {
  it("stays off the page where no icon can be unset, as in the doc's own picker", () => {
    render(<Pick onIcon={vi.fn()} clears={false} />);

    expect(screen.queryByRole("button", { name: "No icon" })).toBeNull();
  });

  it("shows up beside the search box once an icon can be unset", async () => {
    render(<Pick onIcon={vi.fn()} />);

    expect(await screen.findByRole("button", { name: "No icon" })).toBeTruthy();
  });

  it("hands back no key at all when pressed", async () => {
    const onIcon = vi.fn();
    render(<Pick onIcon={onIcon} icon="home" />);

    await userEvent.click(await screen.findByRole("button", { name: "No icon" }));

    expect(onIcon).toHaveBeenCalledWith(undefined);
  });

  it("keeps its place beside the search box while the family chips are typed away", async () => {
    render(<Pick onIcon={vi.fn()} />);
    await screen.findByRole("button", { name: "Kitchen" });

    await userEvent.type(screen.getByRole("textbox"), "bed");

    expect(screen.queryByRole("button", { name: "Kitchen" })).toBeNull();
    expect(screen.getByRole("button", { name: "No icon" })).toBeTruthy();
  });
});

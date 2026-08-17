import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import TaskList from "../ui/TaskList";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
}));

const shown = (onBack?: () => void) =>
  render(<TaskList tasks={[]} lists={[]} title="Compras" onBack={onBack} onSelect={vi.fn()} />);

describe("getting out of a list or a tag without the menu", () => {
  it("offers a way back when there is somewhere to go back to", async () => {
    const back = vi.fn();
    shown(back);

    await userEvent.click(screen.getByRole("button", { name: /back/i }));

    expect(back).toHaveBeenCalledTimes(1);
  });

  it("offers none on a screen that is already the top of the road", () => {
    shown();

    expect(screen.queryByRole("button", { name: /back/i })).toBeNull();
  });

  it("says what it does, so it is not a bare arrow", () => {
    shown(vi.fn());

    expect(screen.getByRole("button", { name: /back/i }).getAttribute("title")).toBeTruthy();
  });
});

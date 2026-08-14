import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Glyphs from "../ui/Glyphs";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) =>
    Promise.resolve(
      cmd === "icons"
        ? [
            ["home", "🏠"],
            ["work", "💼"],
            ["homework", "📚"],
          ]
        : null,
    ),
}));

describe("picking an icon to write into a document", () => {
  it("hands over the drawing, not the name it is filed under", async () => {
    const onPick = vi.fn();
    render(<Glyphs onPick={onPick} />);
    await waitFor(() => screen.getByRole("button", { name: "home" }));

    await userEvent.click(screen.getByRole("button", { name: "home" }));

    expect(onPick).toHaveBeenCalledWith("🏠");
  });

  it("narrows by the name, so a hundred of them stay usable", async () => {
    render(<Glyphs onPick={vi.fn()} />);
    await waitFor(() => screen.getByRole("button", { name: "home" }));

    await userEvent.type(screen.getByRole("textbox"), "work");

    expect(screen.getByRole("button", { name: "work" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "homework" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "home" })).toBeNull();
  });

  it("never steals the caret from the words being written", async () => {
    render(<Glyphs onPick={vi.fn()} />);
    await waitFor(() => screen.getByRole("button", { name: "home" }));
    const before = document.activeElement;

    await userEvent.click(screen.getByRole("button", { name: "home" }));

    expect(document.activeElement).toBe(before);
  });
});

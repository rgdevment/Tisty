import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Hue, { painted } from "../ui/Hue";

describe("the colour a list is painted with", () => {
  it("hands back the key, never the hex, so the palette can be retuned later", async () => {
    const user = userEvent.setup();
    const picked = vi.fn();
    render(<Hue onPick={picked} />);

    await user.click(screen.getByRole("button", { name: "Teal" }));

    expect(picked).toHaveBeenCalledWith("teal");
  });

  it("offers a way back to no colour at all", async () => {
    const user = userEvent.setup();
    const picked = vi.fn();
    render(<Hue chosen="teal" onPick={picked} />);

    await user.click(screen.getByRole("button", { name: "No colour" }));

    expect(picked).toHaveBeenCalledWith(undefined);
  });

  it("falls back to the quiet ink for a colour the palette no longer has", () => {
    expect(painted("chartreuse")).toBe("text-soft");
    expect(painted(null)).toBe("text-soft");
  });

  it("names a class Tailwind can see written out, or it would arrive unstyled", () => {
    expect(painted("pink")).toBe("text-hue-pink");
  });
});

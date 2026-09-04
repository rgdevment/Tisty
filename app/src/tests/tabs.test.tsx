import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Pick from "../ui/Pick";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "icons") return Promise.resolve(["\u{1F600}", "\u{1F697}", "home", "bed"]);
    if (cmd === "families")
      return Promise.resolve([
        ["faces", 1],
        ["travel", 1],
        ["home", 2],
      ]);
    return Promise.resolve(null);
  },
}));

const show = () => render(<Pick onIcon={vi.fn()} onColour={vi.fn()} />);

describe("marks and icons, each in its own tab", () => {
  it("opens on the marks, which are what get used", async () => {
    show();

    const tabs = await screen.findAllByRole("tab");
    expect(tabs.map((one) => one.textContent)).toEqual(["Emoji", "Icons"]);
    expect(tabs[0].getAttribute("aria-selected")).toBe("true");
  });

  it("offers no colours for a mark, since a mark brings its own", async () => {
    show();
    await screen.findAllByRole("tab");

    expect(screen.queryByRole("button", { name: "Teal" })).toBeNull();

    await userEvent.click(screen.getByRole("tab", { name: "Icons" }));
    expect(screen.getByRole("button", { name: "Teal" })).toBeTruthy();
  });

  it("keeps each tab to its own families", async () => {
    show();
    await screen.findAllByRole("tab");
    const chips = () => screen.getAllByRole("button").map((one) => one.textContent);

    expect(chips()).toContain("Faces");
    expect(chips()).not.toContain("Home");

    await userEvent.click(screen.getByRole("tab", { name: "Icons" }));
    expect(chips()).toContain("Home");
    expect(chips()).not.toContain("Faces");
  });

  it("opens where the icon already chosen lives", async () => {
    render(<Pick icon="home" onIcon={vi.fn()} onColour={vi.fn()} />);

    const tabs = await screen.findAllByRole("tab");
    expect(tabs[1].getAttribute("aria-selected")).toBe("true");
  });
});

import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt } from "../locales";
import { ownOf } from "../tistyOwn";
import { TistyOwnCard } from "../ui/TistyOwn";
import { knownFor, skillsFor, wiredOne } from "./knownFor";

const mending = vi.hoisted(() => ({
  addExtras: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

vi.mock("../agentsMending", async (original) => ({
  ...(await original<typeof import("../agentsMending")>()),
  addExtras: mending.addExtras,
}));

const claude = wiredOne("claude-code", {
  name: "Claude Code",
  wired: true,
  hooks: { on: false, at: "/home/someone/.claude/settings.json" },
});
const door = { on: true, called: "espino 3" };
const skills = skillsFor({
  on: false,
  landing: [{ assistant: "claude", at: "/h/.claude/skills/tisty/SKILL.md", standing: "write" }],
});

beforeEach(() => {
  adopt("en");
  mending.addExtras.mockReset();
});

describe("when turning on is refused", () => {
  it("shows the refusal in the card and leaves the button usable again", async () => {
    mending.addExtras.mockRejectedValue({ code: "cannotWrite", about: "disk full" });
    render(
      <TistyOwnCard
        own={ownOf({ agent: door, agents: [claude], skills })}
        skills={skills}
        known={knownFor({ agent: door, agents: [claude] })}
        onSkills={vi.fn()}
        onClose={vi.fn()}
      />,
    );
    await userEvent.click(screen.getByRole("button", { name: "Turn on both" }));
    await userEvent.click(screen.getByRole("button", { name: "Turn on" }));
    await waitFor(() => expect(mending.addExtras).toHaveBeenCalledTimes(1));
    const trouble = await waitFor(() => {
      const found = document.querySelector(".text-urgent");
      expect(found).not.toBeNull();
      return found as HTMLElement;
    });
    expect(trouble.textContent?.length).toBeGreaterThan(0);
    expect(screen.queryByRole("status")).toBeNull();
    await waitFor(() =>
      expect((screen.getByRole("button", { name: "Turn on" }) as HTMLButtonElement).disabled).toBe(
        false,
      ),
    );
    mending.addExtras.mockResolvedValue([
      { id: "claude-code", name: "Claude Code", done: "hooks" },
    ]);
    await userEvent.click(screen.getByRole("button", { name: "Turn on" }));
    expect(await screen.findByRole("status")).toBeTruthy();
    expect(mending.addExtras).toHaveBeenCalledTimes(2);
  });
});

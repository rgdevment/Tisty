import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Settings } from "../core";
import UrgentLine from "../ui/UrgentLine";

const ipc = vi.hoisted(() => ({ sent: [] as string[], offered: true }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.sent.push(cmd);
    if (cmd === "urgent_offered") return Promise.resolve(ipc.offered);
    return Promise.resolve(null);
  },
}));

const kept = (more: Partial<Settings> = {}): Settings =>
  ({ quiet: [], holds: "everywhere", shares: false, onlySharedAbove: 0, ...more }) as Settings;

function shown(settings: Settings) {
  const remembered: Settings[] = [];
  render(<UrgentLine kept={settings} held={false} remember={(next) => remembered.push(next)} />);
  return remembered;
}

const knob = () => screen.getByRole("switch", { name: /important reminders/i });

beforeEach(() => {
  ipc.sent = [];
  ipc.offered = true;
});

describe("the switch for important reminders", () => {
  it("says nothing about the system until the system has answered", () => {
    ipc.offered = false;
    shown(kept());

    expect(screen.queryByText(/cannot let a reminder through/i)).toBeNull();
    expect(knob().hasAttribute("disabled")).toBe(true);
  });

  it("comes off and turns on only by the person's hand", async () => {
    const remembered = shown(kept());
    await waitFor(() => expect(knob().hasAttribute("disabled")).toBe(false));

    expect(knob().getAttribute("aria-checked")).toBe("false");
    await userEvent.click(knob());

    expect(remembered[remembered.length - 1]?.urgent).toBe(true);
  });

  it("once on, says where to allow it and opens that page of the system", async () => {
    shown(kept({ urgent: true }));
    const open = await screen.findByRole("button", { name: /open the system settings/i });

    await userEvent.click(open);

    expect(ipc.sent).toContain("open_notice_settings");
  });

  it("cannot be turned on where the system or this copy cannot let it through", async () => {
    ipc.offered = false;
    shown(kept({ urgent: true }));
    await waitFor(() => expect(ipc.sent).toContain("urgent_offered"));

    expect(knob().hasAttribute("disabled")).toBe(true);
    expect(knob().getAttribute("aria-checked")).toBe("false");
    expect(screen.getByText(/cannot let a reminder through/i)).toBeTruthy();
  });

  it("waits while system notifications are off, since nothing is booked then", async () => {
    shown(kept({ urgent: true, quiet: ["screen"] }));
    await waitFor(() => expect(ipc.sent).toContain("urgent_offered"));

    expect(knob().hasAttribute("disabled")).toBe(true);
    expect(screen.getByText(/turn on «a notification from the system» first/i)).toBeTruthy();
  });
});

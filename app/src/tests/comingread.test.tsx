import { act, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ended } from "../coming";
import { t } from "../locales";
import { lookAgain } from "../syncSeen";
import Coming from "../ui/Coming";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string) =>
    Promise.resolve(
      command === "sync_state"
        ? { mode: "cloud", cloud: "a@example.invalid", cloudProvider: "onedrive" }
        : command === "cloud_status"
          ? {
              provider: "onedrive",
              account: "a",
              email: "a@example.invalid",
              joined: false,
              running: true,
              reading: { done: 312, whole: 1040, at: 1 },
            }
          : null,
    ),
}));

describe("a read of the cloud no round tells about", () => {
  it("still shows how far it came, and leaves the last round's trouble out while it runs", async () => {
    act(() => ended({ code: "cloudSealed" }));
    render(<Coming />);
    await act(() => lookAgain());

    expect(screen.getByRole("status").textContent).toContain(t("comingLog"));
    expect(screen.getByRole("status").textContent).toMatch(/312.*1.040/);
    expect(screen.queryByRole("alert")).toBeNull();
  });
});

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { vouchedSaid } from "../ui/Tidying";
import Unvouched from "../ui/Unvouched";

const ipc = vi.hoisted(() => ({
  asked: 0,
  found: [] as { at: string; bytes: number }[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "unvouched_attachments") {
      ipc.asked += 1;
      return Promise.resolve(ipc.found);
    }
    return Promise.resolve(null);
  },
}));

beforeEach(() => {
  ipc.asked = 0;
  ipc.found = [];
});

const MB = 1024 * 1024;

describe("the attachments with no print in Maintenance", () => {
  it("counts them, weighs them all and names each one before reading any", async () => {
    ipc.found = [
      { at: "attachments/67/sydunj2ghj-7d02eb9e.mp4", bytes: 53 * MB },
      { at: "attachments/bb/final-zip-b0b1e867.001", bytes: 507 * MB },
    ];
    render(<Unvouched held={false} vouch={() => {}} className="" />);

    expect(await screen.findByText(fill("unvouchedCount", "2", weigh(560 * MB)))).toBeTruthy();
    expect(screen.getByText("sydunj2ghj-7d02eb9e.mp4")).toBeTruthy();
    expect(screen.getByText("final-zip-b0b1e867.001")).toBeTruthy();
    expect(screen.getByRole("button", { name: t("unvouchedDo") })).toBeTruthy();
  });

  it("says every attachment has its print, and offers nothing to do", async () => {
    render(<Unvouched held={false} vouch={() => {}} className="" />);

    expect(await screen.findByText(t("unvouchedNone"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("unvouchedDo") })).toBeNull();
  });

  it("writes them down on a click and looks again afterwards", async () => {
    ipc.found = [{ at: "attachments/67/video-7d02eb9e.mp4", bytes: MB }];
    const vouch = vi.fn((then: () => void) => {
      ipc.found = [];
      then();
    });
    render(<Unvouched held={false} vouch={vouch} className="" />);

    await userEvent.click(await screen.findByRole("button", { name: t("unvouchedDo") }));

    expect(vouch).toHaveBeenCalledTimes(1);
    expect(await screen.findByText(t("unvouchedNone"))).toBeTruthy();
    expect(ipc.asked).toBe(2);
  });

  it("says how many were written down, and only then what did not match or was not there", () => {
    expect(vouchedSaid({ kept: 15, unlike: [], gone: [] })).toBe(fill("unvouchedDone", "15"));
    expect(vouchedSaid({ kept: 13, unlike: ["a"], gone: ["b"] })).toBe(
      [fill("unvouchedDone", "13"), fill("unvouchedUnlike", "1"), fill("unvouchedGone", "1")].join(
        " ",
      ),
    );
  });
});

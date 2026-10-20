import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { vouchedSaid } from "../ui/Tidying";
import Unvouched from "../ui/Unvouched";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const MB = 1024 * 1024;

describe("the attachments to verify in Maintenance", () => {
  it("counts them, weighs them all and names each one before reading any", () => {
    render(
      <Unvouched
        seen={[
          { at: "attachments/67/sydunj2ghj-7d02eb9e.mp4", bytes: 53 * MB },
          { at: "attachments/bb/final-zip-b0b1e867.001", bytes: 507 * MB },
        ]}
        held={false}
        vouch={() => {}}
        className=""
      />,
    );

    expect(screen.getByText(fill("unvouchedCount", "2", weigh(560 * MB)))).toBeTruthy();
    expect(screen.getByText("sydunj2ghj-7d02eb9e.mp4")).toBeTruthy();
    expect(screen.getByText("final-zip-b0b1e867.001")).toBeTruthy();
    expect(screen.getByRole("button", { name: t("unvouchedDo") })).toBeTruthy();
  });

  it("says every attachment is verified, and offers nothing to do", () => {
    render(<Unvouched seen={[]} held={false} vouch={() => {}} className="" />);

    expect(screen.getByText(t("unvouchedNone"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("unvouchedDo") })).toBeNull();
  });

  it("verifies them on a click", async () => {
    const vouch = vi.fn();
    render(
      <Unvouched
        seen={[{ at: "attachments/67/video-7d02eb9e.mp4", bytes: MB }]}
        held={false}
        vouch={vouch}
        className=""
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: t("unvouchedDo") }));

    expect(vouch).toHaveBeenCalledTimes(1);
  });

  it("says it is working on the button while the verifying runs", () => {
    render(
      <Unvouched
        seen={[{ at: "attachments/67/video-7d02eb9e.mp4", bytes: MB }]}
        held
        vouch={() => {}}
        className=""
        label={t("unvouchedWorking")}
      />,
    );

    expect(screen.getByRole("button", { name: t("unvouchedWorking") })).toBeTruthy();
  });

  it("says how many were verified, and only then what did not match or was not there", () => {
    expect(vouchedSaid({ kept: 15, unlike: [], gone: [] })).toBe(fill("unvouchedDone", "15"));
    expect(vouchedSaid({ kept: 13, unlike: ["a"], gone: ["b"] })).toBe(
      [fill("unvouchedDone", "13"), fill("unvouchedUnlike", "1"), fill("unvouchedGone", "1")].join(
        " ",
      ),
    );
  });
});

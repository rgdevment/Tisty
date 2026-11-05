import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Ready } from "../core";
import { t } from "../locales";
import Older, { waitingOn } from "../ui/Older";

const ipc = vi.hoisted(() => ({ ready: null as unknown, installs: 0, build: null as unknown }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "update_ready") return Promise.resolve(ipc.ready);
    if (cmd === "update_install") ipc.installs += 1;
    if (cmd === "about") return Promise.resolve(ipc.build);
    return Promise.resolve(null);
  },
}));

beforeEach(() => {
  ipc.ready = null;
  ipc.installs = 0;
  ipc.build = null;
});

const offered = (more: Partial<Ready>): Ready => ({
  version: "1.31.0",
  route: "download",
  installs: false,
  ...more,
});

const clicked = async (ready: Ready) => {
  ipc.ready = ready;
  render(<Older busy={false} onStay={() => {}} onOther={() => {}} onError={() => {}} />);
  await userEvent.click(screen.getByRole("button", { name: t("updateInstall") }));
};

describe("updating from the folder a newer Tisty wrote", () => {
  it("says what About says for every copy that cannot install itself", () => {
    expect(waitingOn(offered({ route: "store", coming: "queued" }))).toBe("updateComingWhy");
    expect(waitingOn(offered({ route: "store", coming: "paused" }))).toBe("updatePausedWhy");
    expect(waitingOn(offered({ route: "store", coming: "notYet" }))).toBe("updateNotYetWhy");
    expect(waitingOn(offered({ route: "store" }))).toBe("updateLanded");
    expect(waitingOn(offered({ route: "download" }))).toBe("updateFromTheDisk");
  });

  it("tells a copy opened from the disk image to move to Applications", async () => {
    await clicked(offered({ route: "download" }));

    expect(await screen.findByText(t("updateFromTheDisk"))).toBeTruthy();
  });

  it("says the Store is still bringing it while it is on its way", async () => {
    await clicked(offered({ route: "store", coming: "queued" }));

    expect(await screen.findByText(t("updateComingWhy"))).toBeTruthy();
  });

  it("does not call a Store copy the newest when the Store has nothing for it", async () => {
    ipc.build = { keptByTheStore: true };
    render(<Older busy={false} onStay={() => {}} onOther={() => {}} onError={() => {}} />);
    await userEvent.click(screen.getByRole("button", { name: t("updateInstall") }));

    expect(await screen.findByText(t("lookNowNoneStore"))).toBeTruthy();
    expect(screen.queryByText(t("lookNowNone"))).toBeNull();
  });

  it("installs a copy that can install itself without saying anything", async () => {
    await clicked(offered({ installs: true }));

    await vi.waitFor(() => expect(ipc.installs).toBe(1));
  });
});

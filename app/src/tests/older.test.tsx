import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Ready } from "../core";
import { t } from "../locales";
import Older, { waitingOn } from "../ui/Older";

const ipc = vi.hoisted(() => ({ ready: null as unknown, installs: 0 }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "update_ready") return Promise.resolve(ipc.ready);
    if (cmd === "update_install") ipc.installs += 1;
    return Promise.resolve(null);
  },
}));

beforeEach(() => {
  ipc.ready = null;
  ipc.installs = 0;
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
    expect(waitingOn(offered({ route: "store", coming: true }))).toBe("updateComingWhy");
    expect(waitingOn(offered({ route: "store" }))).toBe("updateLanded");
    expect(waitingOn(offered({ route: "download" }))).toBe("updateFromTheDisk");
  });

  it("tells a copy opened from the disk image to move to Applications", async () => {
    await clicked(offered({ route: "download" }));

    expect(await screen.findByText(t("updateFromTheDisk"))).toBeTruthy();
  });

  it("says the Store is still bringing it while it is on its way", async () => {
    await clicked(offered({ route: "store", coming: true }));

    expect(await screen.findByText(t("updateComingWhy"))).toBeTruthy();
  });

  it("installs a copy that can install itself without saying anything", async () => {
    await clicked(offered({ installs: true }));

    await vi.waitFor(() => expect(ipc.installs).toBe(1));
  });
});

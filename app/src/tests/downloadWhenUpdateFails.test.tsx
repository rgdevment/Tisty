import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const calls: { cmd: string; args: unknown }[] = [];
const state: { refuse: unknown; autoFailed: string | null } = { refuse: null, autoFailed: null };

import type { Ready } from "../core";
import About from "../ui/About";
import { Alarm } from "../ui/Banners";

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => {
    calls.push({ cmd, args });
    if (state.refuse && cmd === "update_install") return Promise.reject(state.refuse);
    return Promise.resolve({
      version: "0.2.0",
      sandbox: null,
      repository: "https://example.invalid/tisty",
      license: "AGPL-3.0",
      store: "C:/store",
      candidates: false,
      candidatesApply: true,
      autoUpdates: false,
      autoUpdatesApply: true,
      autoFailed: state.autoFailed,
    });
  },
}));

beforeEach(() => {
  calls.length = 0;
  state.refuse = null;
  state.autoFailed = null;
});

afterEach(cleanup);

const ready = (route: Ready["route"] = "download"): Ready => ({
  version: "0.3.0",
  route,
  installs: true,
});

const website = () => screen.queryByRole("button", { name: "Download from the website" });

describe("the way to the website when updating fails", () => {
  it("is not offered while nothing has failed", async () => {
    render(<About ready={ready()} onError={vi.fn()} />);

    await screen.findByRole("button", { name: "Update" });
    expect(website()).toBeNull();
    expect(screen.queryByText(/did not finish/)).toBeNull();
  });

  it("appears once the install is refused, and opens the releases page", async () => {
    state.refuse = { code: "updateFailed", name: "no room left" };
    render(<About ready={ready()} onError={vi.fn()} />);

    await userEvent.click(await screen.findByRole("button", { name: "Update" }));
    await userEvent.click(await screen.findByRole("button", { name: "Download from the website" }));

    expect(calls).toContainEqual({
      cmd: "open_link",
      args: { url: "https://github.com/rgdevment/Tisty/releases/latest" },
    });
  });

  it("appears with its reason when the automatic update to the offered version did not finish", async () => {
    state.autoFailed = "0.3.0";
    render(<About ready={ready()} onError={vi.fn()} />);

    expect(await screen.findByRole("button", { name: "Download from the website" })).toBeTruthy();
    expect(screen.getByText("The automatic update to 0.3.0 did not finish.")).toBeTruthy();
  });

  it("stays away when the automatic failure was for another version", async () => {
    state.autoFailed = "0.2.5";
    render(<About ready={ready()} onError={vi.fn()} />);

    await screen.findByRole("button", { name: "Update" });
    await waitFor(() => expect(calls.some((c) => c.cmd === "about")).toBe(true));
    expect(website()).toBeNull();
    expect(screen.queryByText(/did not finish/)).toBeNull();
  });

  it("is never offered to a Store copy, even after a failure", async () => {
    state.refuse = { code: "updateFailed", name: "no room left" };
    state.autoFailed = "0.3.0";
    render(<About ready={ready("store")} onError={vi.fn()} />);

    await userEvent.click(await screen.findByRole("button", { name: "Update" }));

    await waitFor(() =>
      expect(document.querySelector("[data-setting=update] [data-note=err]")).toBeTruthy(),
    );
    expect(website()).toBeNull();
  });
});

describe("the alarm's way to the website", () => {
  const alarm = (onDownload?: () => void) =>
    render(
      <Alarm
        error="The update stopped"
        stuck={false}
        offer={null}
        underway={null}
        onTakeMe={vi.fn()}
        onInstall={vi.fn()}
        onDownload={onDownload}
        onClose={vi.fn()}
      />,
    );

  it("shows the button and calls back when it is pressed", async () => {
    const onDownload = vi.fn();
    alarm(onDownload);

    await userEvent.click(screen.getByRole("button", { name: "Download from the website" }));

    expect(onDownload).toHaveBeenCalledTimes(1);
  });

  it("shows no button when there is nothing to download", () => {
    alarm();

    expect(website()).toBeNull();
  });
});

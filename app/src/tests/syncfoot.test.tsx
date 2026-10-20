import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type Foot from "../ui/SyncFoot";

const ipc = vi.hoisted(() => ({
  calls: [] as string[],
  state: {} as Record<string, unknown>,
  status: null as Record<string, unknown> | null,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.calls.push(cmd);
    if (cmd === "sync_state") return Promise.resolve(ipc.state);
    if (cmd === "cloud_status") return Promise.resolve(ipc.status);
    return Promise.resolve(null);
  },
}));

const cloud = (over: Record<string, unknown> = {}) => ({
  provider: "google",
  account: "a",
  email: "a@example.invalid",
  joined: true,
  unitsSpent: 0,
  unitsADay: 0,
  bytesSpent: 0,
  bytesADay: 0,
  pausedHere: false,
  owed: 0,
  ...over,
});

// The foot reads a store kept by its module, so each test starts from a fresh one.
let SyncFoot: typeof Foot;

beforeEach(async () => {
  vi.resetModules();
  SyncFoot = (await import("../ui/SyncFoot")).default;
  ipc.calls.length = 0;
  ipc.state = { mode: "cloud", cloudProvider: "google", last: "2026-06-10T08:30:00Z" };
  ipc.status = cloud({ last: { at: 1, code: "ok", left: 0, names: [] } });
});

describe("the sync foot in the sidebar", () => {
  it("says nothing when this computer does not sync", async () => {
    ipc.state = { mode: "alone" };
    render(<SyncFoot onSync={vi.fn()} onOpen={vi.fn()} />);

    await waitFor(() => expect(ipc.calls).toContain("sync_state"));
    await act(async () => {});
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("leaves the mode in its place when this computer does not sync", async () => {
    ipc.state = { mode: "alone" };
    render(<SyncFoot onSync={vi.fn()} onOpen={vi.fn()} fallback={<span>Only here</span>} />);

    await waitFor(() => expect(ipc.calls).toContain("sync_state"));
    await act(async () => {});
    expect(screen.getByText("Only here")).toBeTruthy();
  });

  it("syncs now when everything is up to date", async () => {
    const onSync = vi.fn(() => Promise.resolve());
    render(<SyncFoot onSync={onSync} onOpen={vi.fn()} />);

    const foot = await screen.findByRole("button", { name: /Up to date/ });
    expect(foot.textContent).toContain("Google Drive");
    await userEvent.click(foot);

    expect(onSync).toHaveBeenCalledOnce();
  });

  it("names a folder by its own name", async () => {
    ipc.state = { mode: "folder", chosen: "D:\\Shared\\Tisty" };
    ipc.status = null;
    render(<SyncFoot onSync={vi.fn()} onOpen={vi.fn()} />);

    const foot = await screen.findByRole("button", { name: /Up to date/ });
    expect(foot.textContent).toContain("Tisty");
    expect(ipc.calls).not.toContain("cloud_status");
  });

  it("resumes with the same turn-on the Sync tab uses", async () => {
    ipc.status = cloud({ pausedHere: true });
    const onSync = vi.fn(() => Promise.resolve());
    render(<SyncFoot onSync={onSync} onOpen={vi.fn()} />);

    await userEvent.click(await screen.findByRole("button", { name: /Paused · Resume/ }));

    await waitFor(() => expect(ipc.calls).toContain("cloud_pause"));
    await waitFor(() => expect(onSync).toHaveBeenCalledOnce());
  });

  it("never offers a sync the cloud would turn away: it says why and opens Settings › Sync", async () => {
    ipc.status = cloud({ last: { at: 1, code: "cloudNotNow", left: 0, names: [] } });
    const onSync = vi.fn();
    const onOpen = vi.fn();
    render(<SyncFoot onSync={onSync} onOpen={onOpen} />);

    const foot = await screen.findByRole("button", { name: /asks to wait/ });
    expect(foot.getAttribute("aria-label")).toMatch(/tries again on its own/);
    await userEvent.click(foot);

    expect(onOpen).toHaveBeenCalledOnce();
    expect(onSync).not.toHaveBeenCalled();
  });

  it("opens Settings › Sync when the access is gone", async () => {
    ipc.status = cloud({ last: { at: 1, code: "cloudLost", left: 0, names: [] } });
    const onOpen = vi.fn();
    render(<SyncFoot onSync={vi.fn()} onOpen={onOpen} />);

    await userEvent.click(await screen.findByRole("button", { name: /Sign in again/ }));

    expect(onOpen).toHaveBeenCalledOnce();
  });
});

import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Leave from "../ui/Leave";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  refuse: null as string | null,
  leftOut: 0,
  stayed: null as string[] | null,
}));

const asked = vi.hoisted(() => ({ file: null as string | null }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === ipc.refuse) return Promise.reject({ code: "forgetUntold" });
    if (cmd === "leave_here") return Promise.resolve(args?.anyway ? null : ipc.stayed);
    if (cmd === "back_up")
      return Promise.resolve({ bytes: 10, leftOut: ipc.leftOut, leftOutBytes: ipc.leftOut * 1024 });
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: () => Promise.resolve(asked.file),
}));

const said = () => ipc.calls.map((one) => one.cmd);

beforeEach(() => {
  ipc.calls.length = 0;
  ipc.refuse = null;
  ipc.leftOut = 0;
  ipc.stayed = null;
  asked.file = null;
});

describe("leaving Tisty", () => {
  it("lets the cloud go, shows the folder and then leaves, in that order", async () => {
    render(<Leave cloud revokes store="C:/Tisty" onClose={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: "Leave Tisty" }));

    await waitFor(() =>
      expect(said()).toEqual(["cloud_pause", "cloud_forget", "revealed", "leave_here"]),
    );
    expect(ipc.calls[2].args).toEqual({ path: "C:/Tisty" });
  });

  it("takes the access away on every computer only when asked", async () => {
    render(<Leave cloud revokes store="C:/Tisty" onClose={vi.fn()} />);

    const everywhere = screen.getByRole("checkbox", { name: /on every computer/ });
    expect((everywhere as HTMLInputElement).checked).toBe(false);
    await userEvent.click(everywhere);
    await userEvent.click(screen.getByRole("button", { name: "Leave Tisty" }));

    await waitFor(() =>
      expect(said()).toEqual(["cloud_pause", "cloud_revoke", "revealed", "leave_here"]),
    );
  });

  it("has no cloud step when there is no cloud", async () => {
    render(<Leave cloud={false} revokes={false} store="C:/Tisty" onClose={vi.fn()} />);

    expect(screen.queryByRole("checkbox")).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Leave Tisty" }));

    await waitFor(() => expect(said()).toEqual(["revealed", "leave_here"]));
  });

  it("names what another program kept open, and closes when asked", async () => {
    ipc.stayed = ["C:/Users/me/.tisty/cache"];
    render(<Leave cloud={false} revokes={false} store="C:/Tisty" onClose={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: "Leave Tisty" }));
    expect(await screen.findByText("C:/Users/me/.tisty/cache")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Close Tisty" }));

    expect(ipc.calls[ipc.calls.length - 1]).toEqual({ cmd: "leave_here", args: { anyway: true } });
  });

  it("pauses the cloud before letting it go, and resumes it when it will not let go", async () => {
    ipc.refuse = "cloud_forget";
    render(<Leave cloud revokes store="C:/Tisty" onClose={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: "Leave Tisty" }));
    await screen.findByRole("alert");

    expect(said().filter((cmd) => cmd !== "note_trouble")).toEqual([
      "cloud_pause",
      "cloud_forget",
      "cloud_pause",
    ]);
    expect(ipc.calls[0].args).toEqual({ paused: true });
    expect(ipc.calls[2].args).toEqual({ paused: false });
  });

  it("says the cloud is already let go when leaving fails after it", async () => {
    ipc.refuse = "leave_here";
    render(<Leave cloud revokes store="C:/Tisty" onClose={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: "Leave Tisty" }));

    expect((await screen.findByRole("alert")).textContent).toContain("already disconnected");
  });

  it("stays when the cloud will not let go, and says why", async () => {
    ipc.refuse = "cloud_forget";
    render(<Leave cloud revokes store="C:/Tisty" onClose={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: "Leave Tisty" }));

    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(said()).not.toContain("leave_here");
  });

  it("makes a backup on the way and marks it done", async () => {
    asked.file = "C:/out/tisty-backup.zip";
    render(<Leave cloud={false} revokes={false} store="C:/Tisty" onClose={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: "Create backup…" }));

    expect((await screen.findByRole("status")).textContent).toContain("Backup made");
    expect(ipc.calls[0]).toEqual({ cmd: "back_up", args: { into: "C:/out/tisty-backup.zip" } });
  });

  it("says what the backup left out instead of marking it done", async () => {
    asked.file = "C:/out/tisty-backup.zip";
    ipc.leftOut = 3;
    render(<Leave cloud revokes store="C:/Tisty" onClose={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: "Create backup…" }));

    const status = await screen.findByRole("status");
    expect(status.textContent).toContain("stayed out of it");
    expect(screen.queryByText(/Backup made/)).toBeNull();
  });

  it("cancels without touching anything", async () => {
    const onClose = vi.fn();
    render(<Leave cloud revokes store="C:/Tisty" onClose={onClose} />);

    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));

    expect(onClose).toHaveBeenCalledOnce();
    expect(said()).toEqual([]);
  });
});

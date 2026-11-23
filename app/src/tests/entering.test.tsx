import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Entering from "../ui/Entering";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  heard: [] as unknown,
  refuse: false as false | string,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "sealing_heard") return Promise.resolve(ipc.heard);
    if (cmd === "sealing_phrase" && ipc.refuse)
      return Promise.reject({ code: ipc.refuse, name: "OneDrive" });
    return Promise.resolve(null);
  },
}));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

beforeEach(() => {
  ipc.calls.length = 0;
  ipc.heard = [];
  ipc.refuse = false;
});

describe("typing the phrase on a computer that cannot open the cloud", () => {
  it("names the other computers heard from and opens the cloud with the words typed", async () => {
    ipc.heard = [{ device: "dev_a", name: "Mac de la oficina", os: "macOS", at: 1_761_400_000 }];
    const onIn = vi.fn();
    const onClose = vi.fn();
    render(<Entering onReconnect={() => {}} onClose={onClose} onIn={onIn} />);

    expect(await screen.findByText(/Mac de la oficina · macOS/)).toBeTruthy();
    expect(screen.getByText(/^last heard /)).toBeTruthy();
    expect(screen.getByRole("textbox").getAttribute("aria-describedby")).toBe(
      screen.getByText(/Type it only here, in your own Tisty/).id,
    );
    const go = screen.getByRole("button", { name: "Open my cloud" });
    expect(go.hasAttribute("disabled")).toBe(true);
    await userEvent.type(screen.getByRole("textbox"), "  abandon ability able  ");
    await userEvent.click(go);

    await waitFor(() => expect(onIn).toHaveBeenCalled());
    expect(sent("sealing_phrase")[0]?.args.said).toBe("abandon ability able");
    expect(onClose).toHaveBeenCalled();
  });

  it("says why a phrase was not taken and stays open", async () => {
    ipc.refuse = "phraseNotThisCloud";
    const onClose = vi.fn();
    render(<Entering onReconnect={() => {}} onClose={onClose} onIn={() => {}} />);
    await userEvent.type(screen.getByRole("textbox"), "abandon");

    await userEvent.click(screen.getByRole("button", { name: "Open my cloud" }));

    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.getByRole("alert").textContent).toMatch(
      /not the one your cloud is encrypted with/,
    );
    expect(onClose).not.toHaveBeenCalled();
  });

  it("offers to reconnect, and promises no other computer, when the access is gone", async () => {
    ipc.refuse = "cloudLost";
    ipc.heard = [{ device: "dev_a", name: "Mac de la oficina", os: "macOS", at: 1_761_400_000 }];
    const onReconnect = vi.fn();
    const onIn = vi.fn();
    render(<Entering onReconnect={onReconnect} onClose={() => {}} onIn={onIn} />);
    await screen.findByText(/Mac de la oficina/);
    await userEvent.type(screen.getByRole("textbox"), "abandon");

    await userEvent.click(screen.getByRole("button", { name: "Open my cloud" }));

    expect((await screen.findByRole("alert")).textContent).toMatch(
      /OneDrive no longer lets Tisty in/,
    );
    expect(screen.queryByText(/Mac de la oficina/)).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Reconnect" }));
    expect(onReconnect).toHaveBeenCalled();
    expect(onIn).not.toHaveBeenCalled();
  });

  it("still says to try later when the cloud is only out of reach", async () => {
    ipc.refuse = "cloudNotNow";
    render(<Entering onReconnect={() => {}} onClose={() => {}} onIn={() => {}} />);
    await userEvent.type(screen.getByRole("textbox"), "abandon");

    await userEvent.click(screen.getByRole("button", { name: "Open my cloud" }));

    expect((await screen.findByRole("alert")).textContent).toMatch(/Try again in a while/);
    expect(screen.queryByRole("button", { name: "Reconnect" })).toBeNull();
  });

  it("points to where the phrase itself is kept", () => {
    render(<Entering onReconnect={() => {}} onClose={() => {}} onIn={() => {}} />);

    expect(screen.getByText(/use «Keep my phrase on this computer»/)).toBeTruthy();
  });

  it("names no other computer when this one has heard from none", async () => {
    render(<Entering onReconnect={() => {}} onClose={() => {}} onIn={() => {}} />);
    await waitFor(() => expect(sent("sealing_heard")).toHaveLength(1));

    expect(screen.queryByText(/This is when each was last heard/)).toBeNull();
  });

  it("takes nothing said back about the other computers as none", async () => {
    ipc.heard = null;
    render(<Entering onReconnect={() => {}} onClose={() => {}} onIn={() => {}} />);
    await waitFor(() => expect(sent("sealing_heard")).toHaveLength(1));

    expect(screen.getByRole("button", { name: "Open my cloud" })).toBeTruthy();
  });
});

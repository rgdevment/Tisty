import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Entering from "../ui/Entering";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  heard: [] as unknown,
  refuse: false,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "sealing_heard") return Promise.resolve(ipc.heard);
    if (cmd === "sealing_phrase" && ipc.refuse)
      return Promise.reject({ code: "phraseNotThisCloud" });
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
    ipc.heard = [{ name: "Mac de la oficina", os: "macOS", at: 1_761_400_000 }];
    const onIn = vi.fn();
    const onClose = vi.fn();
    render(<Entering onClose={onClose} onIn={onIn} />);

    expect(await screen.findByText(/Mac de la oficina · macOS/)).toBeTruthy();
    expect(screen.getByText(/last seen/)).toBeTruthy();
    const go = screen.getByRole("button", { name: "Open my cloud" });
    expect(go.hasAttribute("disabled")).toBe(true);
    await userEvent.type(screen.getByRole("textbox"), "  abandon ability able  ");
    await userEvent.click(go);

    await waitFor(() => expect(onIn).toHaveBeenCalled());
    expect(sent("sealing_phrase")[0]?.args.said).toBe("abandon ability able");
    expect(onClose).toHaveBeenCalled();
  });

  it("says why a phrase was not taken and stays open", async () => {
    ipc.refuse = true;
    const onClose = vi.fn();
    render(<Entering onClose={onClose} onIn={() => {}} />);
    await userEvent.type(screen.getByRole("textbox"), "abandon");

    await userEvent.click(screen.getByRole("button", { name: "Open my cloud" }));

    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.getByRole("alert").textContent).toMatch(
      /not the one your cloud is encrypted with/,
    );
    expect(onClose).not.toHaveBeenCalled();
  });

  it("names no other computer when this one has heard from none", async () => {
    render(<Entering onClose={() => {}} onIn={() => {}} />);
    await waitFor(() => expect(sent("sealing_heard")).toHaveLength(1));

    expect(screen.queryByText(/If one of these computers is at hand/)).toBeNull();
  });
});

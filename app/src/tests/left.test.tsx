import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Trace } from "../core";
import Left from "../ui/Left";

const ipc = vi.hoisted(() => ({
  left: [] as Trace[],
  opened: [] as string[],
  shown: [] as string[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (name: string, args: { path?: string }) => {
    if (name === "revealed") {
      ipc.shown.push(args.path ?? "");
      return Promise.resolve();
    }
    return Promise.resolve(ipc.left);
  },
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (at: string) => {
    ipc.opened.push(at);
    return Promise.resolve();
  },
}));

const shown = async (left: Trace[]) => {
  ipc.left = left;
  const opened = vi.fn();
  render(<Left task="01A" onDoc={opened} heading={<h3>What it left</h3>} />);
  if (left.length) await screen.findByRole("list");
  return opened;
};

beforeEach(() => {
  ipc.left = [];
  ipc.opened = [];
  ipc.shown = [];
});

describe("what a closed task left behind", () => {
  it("shows nothing at all, heading included, when it left nothing", async () => {
    ipc.left = [];
    const { container } = render(<Left task="01A" heading={<h3>What it left</h3>} />);

    await Promise.resolve();
    expect(container.textContent).toBe("");
  });

  it("says a document is put away, because the tree no longer shows it", async () => {
    await shown([
      { kind: "doc", target: "tisty:doc/01D", label: "Certificates and signing", away: true },
    ]);

    expect(screen.getByText("Certificates and signing")).toBeTruthy();
    expect(screen.getByText(/put away/i)).toBeTruthy();
  });

  it("marks a document that is no longer there and refuses to open it", async () => {
    const user = userEvent.setup();
    const opened = await shown([{ kind: "doc", target: "tisty:doc/01D", gone: true }]);

    expect(screen.getByText(/no longer there/i)).toBeTruthy();
    await user.click(screen.getByRole("button"));
    expect(opened).not.toHaveBeenCalled();
  });

  it("opens a living document by its id, not by its path", async () => {
    const user = userEvent.setup();
    const opened = await shown([
      { kind: "doc", target: "tisty:doc/01D", label: "Release notes 0.3.0" },
    ]);

    await user.click(screen.getByRole("button"));

    expect(opened).toHaveBeenCalledWith("01D");
  });

  it("weighs an attachment and names it by its file, not its path", async () => {
    await shown([{ kind: "file", target: "attachments/ab/tisty-0.3.0.msix", bytes: 84_000_000 }]);

    expect(screen.getByText("tisty-0.3.0.msix")).toBeTruthy();
    expect(screen.getByText(/84/)).toBeTruthy();
  });

  it("says an attachment is gone rather than weighing nothing", async () => {
    await shown([{ kind: "file", target: "attachments/ab/old.png", gone: true }]);

    expect(screen.getByText(/no longer there/i)).toBeTruthy();
  });

  it("shortens a link to where it goes", async () => {
    await shown([{ kind: "link", target: "https://gl.example/mr/7" }]);

    expect(screen.getByText("gl.example/mr/7")).toBeTruthy();
  });

  it("opens a link rather than showing it as a dead button", async () => {
    const user = userEvent.setup();
    await shown([{ kind: "link", target: "https://gl.example/mr/7" }]);

    await user.click(screen.getByRole("button"));

    expect(ipc.opened).toEqual(["https://gl.example/mr/7"]);
  });

  it("shows an attachment in the file manager", async () => {
    const user = userEvent.setup();
    await shown([{ kind: "file", target: "attachments/ab/note.png", bytes: 12 }]);

    await user.click(screen.getByRole("button"));

    expect(ipc.shown).toEqual(["attachments/ab/note.png"]);
  });

  it("refuses to open an attachment that is no longer there", async () => {
    const user = userEvent.setup();
    await shown([{ kind: "file", target: "attachments/ab/old.png", gone: true }]);

    await user.click(screen.getByRole("button"));

    expect(ipc.shown).toEqual([]);
  });

  it("leaves a named reference as the name it was written with", async () => {
    await shown([{ kind: "named", target: "CUSLEG-3465" }]);

    expect(screen.getByText("CUSLEG-3465")).toBeTruthy();
  });
});

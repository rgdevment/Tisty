import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Keeping from "../ui/Keeping";
import Welcome from "../ui/Welcome";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> =>
    Promise.resolve(null),
}));

const asked = vi.hoisted(() => ({
  folder: null as string | null,
  file: null as string | null,
  sure: false,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return ipc.answer(cmd, args ?? {});
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: (opts: { directory?: boolean }) =>
    Promise.resolve(opts.directory ? asked.folder : asked.file),
  save: () => Promise.resolve(asked.file),
  ask: () => Promise.resolve(asked.sure),
}));

const standing = { shipped: true, withinReach: false, at: "C:/Programs/Tisty", through: "C:/Programs/Tisty" };

const carrying = {
  chosen: undefined as string | undefined,
  asked: true,
  backsUp: true,
  last: undefined as string | undefined,
  loose: 0,
};

beforeEach(() => {
  ipc.calls = [];
  Object.assign(standing, { shipped: true, withinReach: false, at: "C:/Programs/Tisty", through: "C:/Programs/Tisty" });
  asked.folder = null;
  asked.file = null;
  asked.sure = false;
  Object.assign(carrying, {
    chosen: undefined,
    asked: true,
    backsUp: true,
    last: undefined,
    loose: 0,
  });
  ipc.answer = (cmd) => {
    switch (cmd) {
      case "sync_state":
        return Promise.resolve({ ...carrying });
      case "sync_now":
        return Promise.resolve("came");
      case "reachable":
        return Promise.resolve({ ...standing });
      case "reach_for":
        standing.withinReach = Boolean(ipc.calls[ipc.calls.length - 1]?.args.wanted);
        return Promise.resolve({ ...standing });
      case "checked":
        return Promise.resolve({ tasks: 7, lists: 2, agrees: true, loose: 3, looseBytes: 311_000 });
      case "back_up":
        return Promise.resolve(4096);
      default:
        return Promise.resolve(null);
    }
  };
});

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

describe("the maintenance panel", () => {
  it("offers to turn syncing on when there is no folder", async () => {
    render(<Keeping onChanged={() => {}} />);

    expect(await screen.findByText(/only on this machine/i)).toBeTruthy();
    expect(screen.getByText("no destination")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /sync now/i })).toBeNull();
  });

  it("remembers the folder that was picked", async () => {
    asked.folder = "G:/My Drive/tisty";
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /turn on/i }));

    await waitFor(() => expect(sent("choose_sync").length).toBe(1));
    expect(sent("choose_sync")[0].args.dest).toBe("G:/My Drive/tisty");
  });

  /** The whole point of the exclusion: two truths beside each other. */
  it("hides backing up once a shared folder holds every machine", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    carrying.backsUp = false;
    render(<Keeping onChanged={() => {}} />);

    expect(await screen.findByText(/already holds every machine/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /create backup/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /restore/i })).toBeNull();
  });

  it("never restores without asking first", async () => {
    asked.file = "C:/keep/tisty-backup.zip";
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /restore/i }));

    await waitFor(() => expect(sent("back_up").length + sent("restore").length).toBe(0));
  });

  it("restores once the warning is accepted", async () => {
    asked.file = "C:/keep/tisty-backup.zip";
    asked.sure = true;
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /restore/i }));

    await waitFor(() => expect(sent("restore").length).toBe(1));
    expect(sent("restore")[0].args.from).toBe("C:/keep/tisty-backup.zip");
  });

  it("says what the review found", async () => {
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/7 tasks · 2 lists/i)).toBeTruthy();
    expect(screen.getByText(/311.0 kB/)).toBeTruthy();
  });
});

describe("the first-run assistant", () => {
  it("takes «only here» as an answer, not as a blank", async () => {
    const done = vi.fn();
    render(<Welcome onDone={done} />);

    await userEvent.click(screen.getByRole("button", { name: /only on this machine/i }));

    await waitFor(() => expect(done).toHaveBeenCalled());
    expect(sent("choose_sync")[0].args.dest).toBeUndefined();
  });

  /** Closing it must not be filed as a choice, or the folder is never offered again. */
  it("decides nothing when it is put off", async () => {
    const done = vi.fn();
    render(<Welcome onDone={done} />);

    await userEvent.click(screen.getByRole("button", { name: /decide later/i }));

    expect(done).toHaveBeenCalled();
    expect(sent("choose_sync").length).toBe(0);
  });

  /// The installer used to do this and wiped a whole PATH; from here it is
  /// asked for, and the reply says a new terminal is needed.
  it("offers the command line, and says what to do next", async () => {
    render(<Keeping onChanged={() => {}} />);
    expect(await screen.findByText(/no terminal can find it yet/i)).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: /make it reachable/i }));

    await waitFor(() => expect(sent("reach_for").length).toBe(1));
    expect(sent("reach_for")[0].args.wanted).toBe(true);
    expect(await screen.findByText(/open a new terminal/i)).toBeTruthy();
  });

  it("takes it back off when asked", async () => {
    standing.withinReach = true;
    render(<Keeping onChanged={() => {}} />);
    expect(await screen.findByText(/a terminal can find/i)).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: /take it back out/i }));

    await waitFor(() => expect(sent("reach_for")[0].args.wanted).toBe(false));
  });

  /// A dev run has no CLI beside the window; offering it would be a lie.
  it("says nothing when there is no command line to offer", async () => {
    standing.shipped = false;
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);

    expect(screen.queryByRole("button", { name: /make it reachable/i })).toBeNull();
  });
});

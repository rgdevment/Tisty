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

const carrying = {
  chosen: undefined as string | undefined,
  asked: true,
  backsUp: true,
  last: undefined as string | undefined,
  loose: 0,
};

beforeEach(() => {
  ipc.calls = [];
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
        return Promise.resolve(true);
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
    render(<Keeping onChanged={() => {}} onError={() => {}} />);

    expect(await screen.findByText(/only on this machine/i)).toBeTruthy();
    expect(screen.getByText("no destination")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /sync now/i })).toBeNull();
  });

  it("remembers the folder that was picked", async () => {
    asked.folder = "G:/My Drive/tisty";
    render(<Keeping onChanged={() => {}} onError={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /turn on/i }));

    await waitFor(() => expect(sent("choose_sync").length).toBe(1));
    expect(sent("choose_sync")[0].args.dest).toBe("G:/My Drive/tisty");
  });

  /** The whole point of the exclusion: two truths beside each other. */
  it("hides backing up once a shared folder holds every machine", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    carrying.backsUp = false;
    render(<Keeping onChanged={() => {}} onError={() => {}} />);

    expect(await screen.findByText(/already holds every machine/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /create backup/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /restore/i })).toBeNull();
  });

  it("never restores without asking first", async () => {
    asked.file = "C:/keep/tisty-backup.zip";
    render(<Keeping onChanged={() => {}} onError={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /restore/i }));

    await waitFor(() => expect(sent("back_up").length + sent("restore").length).toBe(0));
  });

  it("restores once the warning is accepted", async () => {
    asked.file = "C:/keep/tisty-backup.zip";
    asked.sure = true;
    render(<Keeping onChanged={() => {}} onError={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /restore/i }));

    await waitFor(() => expect(sent("restore").length).toBe(1));
    expect(sent("restore")[0].args.from).toBe("C:/keep/tisty-backup.zip");
  });

  it("says what the review found", async () => {
    render(<Keeping onChanged={() => {}} onError={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/7 tasks · 2 lists/i)).toBeTruthy();
    expect(screen.getByText(/311.0 kB/)).toBeTruthy();
  });
});

describe("the first-run assistant", () => {
  it("takes «only here» as an answer, not as a blank", async () => {
    const done = vi.fn();
    render(<Welcome onDone={done} onError={() => {}} />);

    await userEvent.click(screen.getByRole("button", { name: /only on this machine/i }));

    await waitFor(() => expect(done).toHaveBeenCalled());
    expect(sent("choose_sync")[0].args.dest).toBeUndefined();
  });

  /** Closing it must not be filed as a choice, or the folder is never offered again. */
  it("decides nothing when it is put off", async () => {
    const done = vi.fn();
    render(<Welcome onDone={done} onError={() => {}} />);

    await userEvent.click(screen.getByRole("button", { name: /decide later/i }));

    expect(done).toHaveBeenCalled();
    expect(sent("choose_sync").length).toBe(0);
  });
});

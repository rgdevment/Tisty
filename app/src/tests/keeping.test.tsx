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

const kept = { lines: [] as string[] };

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
  kept.lines = [];
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
      case "facts":
        return Promise.resolve({
          version: "0.1.0",
          dev: true,
          sandbox: null,
          locale: "en",
          zone: "America/Santiago",
          os: "Windows 11",
          arch: "x86_64",
          webview: "132",
          store: "C:/tisty",
          devices: 1,
          events: 10,
          open: 7,
          archived: 2,
          lists: 2,
          tags: 0,
          listNames: [],
          tagNames: [],
          cache: "agrees",
          attachments: 0,
          attachmentBytes: 0,
          loose: 0,
          looseBytes: 0,
          weight: 1000,
          syncs: false,
          shared: false,
          backedUpAt: null,
          quiet: [],
          attachUpTo: 5 * 1024 * 1024,
          inPath: true,
          shortcut: null,
        });
      case "logs":
        return Promise.resolve({
          at: "C:/tisty/private/tisty.log",
          bytes: kept.lines.length === 0 ? 0 : 240,
          lines: kept.lines,
        });
      case "settings":
        return Promise.resolve({ quiet: [], attachUpTo: 5 * 1024 * 1024, logsAll: false });
      default:
        return Promise.resolve(null);
    }
  };
});

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

/// The screen holds four sections now; a card only exists once its own is open.
const go = (tab: RegExp) => userEvent.click(screen.getByRole("tab", { name: tab }));

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

  /// Work anywhere on the screen holds every button on it — restoring on top of
  /// a running carry is the pair that must never overlap — and only the card
  /// that started it said anything. The rest looked broken.
  it("says why the other cards went quiet", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now" ? new Promise(() => {}) : otherwise(cmd, args);
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/leaving copies in/i);

    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));

    expect((await screen.findAllByText(/waiting for «syncing»/i)).length).toBeGreaterThan(1);
    expect(screen.getByRole("button", { name: /restore from/i }).hasAttribute("disabled")).toBe(
      true,
    );
  });

  /// `disabled:opacity-50` put the ink at 2:1 and left the dropdown looking
  /// perfectly usable while it was not. Work started in one section holds the
  /// controls in every other one, so the state travels across the tabs.
  it("draws what cannot be pressed as such, in a colour the palette declares", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "settings"
        ? Promise.resolve({ quiet: [], attachUpTo: 5 * 1024 * 1024 })
        : cmd === "keep_settings"
          ? new Promise(() => {})
          : otherwise(cmd, args);
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/notices/i);

    await userEvent.click(await screen.findByRole("checkbox", { name: /a short tone/i }));
    await go(/writing/i);

    const size = await screen.findByRole("combobox");
    await waitFor(() => expect(size.hasAttribute("disabled")).toBe(true));
    expect(size.className).toContain("disabled:text-soft");
    expect(size.className).not.toContain("opacity-50");
  });

  it("says what the review found", async () => {
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

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
    await screen.findByText(/only on this machine/i);
    await go(/writing/i);
    expect(await screen.findByText(/no terminal can find it yet/i)).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: /make it reachable/i }));

    await waitFor(() => expect(sent("reach_for").length).toBe(1));
    expect(sent("reach_for")[0].args.wanted).toBe(true);
    expect(await screen.findByText(/next time you sign in/i)).toBeTruthy();
  });

  it("takes it back off when asked", async () => {
    standing.withinReach = true;
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/writing/i);
    expect(await screen.findByText(/a terminal can find/i)).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: /take it back out/i }));

    await waitFor(() => expect(sent("reach_for")[0].args.wanted).toBe(false));
  });

  /// A dev run has no CLI beside the window; offering it would be a lie.
  it("says nothing when there is no command line to offer", async () => {
    standing.shipped = false;
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/writing/i);

    expect(screen.queryByRole("button", { name: /make it reachable/i })).toBeNull();
  });
});

describe("the report a bug gets attached to", () => {
  const upkeep = async () => {
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
  };

  /// The log is not something to administer: it is material for a report, and
  /// the only question is whether it goes in.
  it("offers the log as one more answer, on by default", async () => {
    await upkeep();

    const box = screen.getByRole("checkbox", { name: /include the error log/i });
    expect((box as HTMLInputElement).checked).toBe(true);
  });

  it("shows the log inside what would be sent", async () => {
    kept.lines = ["2026-08-11 10:00:00-04  WARN   sync      folder unreachable"];
    await upkeep();

    await userEvent.click(screen.getByText(/see the report/i));

    expect(await screen.findByText(/folder unreachable/)).toBeTruthy();
  });

  it("leaves it out once it is unticked", async () => {
    kept.lines = ["2026-08-11 10:00:00-04  WARN   sync      folder unreachable"];
    await upkeep();

    await userEvent.click(screen.getByRole("checkbox", { name: /include the error log/i }));
    await userEvent.click(screen.getByText(/see the report/i));

    await waitFor(() => expect(screen.queryByText(/folder unreachable/)).toBeNull());
  });

  /// One file, not a text file beside a log file beside a screenshot.
  it("writes one zip, and says whether the log goes in it", async () => {
    asked.file = "D:/issues/tisty-report.zip";
    await upkeep();

    await userEvent.click(screen.getByRole("button", { name: /save the report/i }));

    await waitFor(() => expect(sent("keep_report").length).toBe(1));
    expect(sent("keep_report")[0].args.at).toBe("D:/issues/tisty-report.zip");
    expect(sent("keep_report")[0].args.logs).toBe(true);
  });
});

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

const standing = {
  shipped: true,
  withinReach: false,
  onPath: true,
  at: "C:/Programs/Tisty",
  through: "C:/Programs/Tisty",
};

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
  Object.assign(standing, {
    shipped: true,
    withinReach: false,
    onPath: true,
    at: "C:/Programs/Tisty",
    through: "C:/Programs/Tisty",
  });
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
        return Promise.resolve({
          tasks: 7,
          lists: 2,
          agrees: true,
          loose: 3,
          looseBytes: 311_000,
          astray: [
            { at: "attachments/ab/charla-a3f9.mp4", bytes: 300_000, when: 1_754_000_000 },
            { at: "attachments/cd/notas-b1c2.pdf", bytes: 11_000, when: 1_754_000_000 },
          ],
          events: 42,
          machines: [
            { id: "mac0-0001", when: Math.floor(Date.now() / 1000), mine: true },
            { id: "win1-0002", when: Math.floor(Date.now() / 1000) - 60 * 60 * 24 * 12, mine: false },
          ],
          logBytes: 4_096,
          docsBytes: 20_480,
          heldBytes: 900_000,
          heldFiles: 9,
        });
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
      case "docs":
        return Promise.resolve({
          folders: [],
          docs: [
            { id: "1", file: "a-0001", title: "Limpio", folder: null, archived: false },
            { id: "2", file: "a-0002", title: "Minuta del lunes", folder: null, archived: false },
          ],
        });
      case "doc_read":
        return Promise.resolve(
          String(ipc.calls[ipc.calls.length - 1]?.args.id) === "a-0002"
            ? "---\ntitle: algo\n---\n\n# Minuta"
            : "# Limpio\n\nun parrafo",
        );
      default:
        return Promise.resolve(null);
    }
  };
});

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

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

  it("names every machine and when each last wrote", async () => {
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/mac0-0001/)).toBeTruthy();
    expect(screen.getByText(/win1-0002/)).toBeTruthy();
    expect(screen.getByText(/this one/i)).toBeTruthy();
  });

  it("says out loud that a machine has been away, so nothing is judged on stale news", async () => {
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/have not synced in a while/i)).toBeTruthy();
  });

  it("keeps quiet about machines when every one of them is up to date", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "checked"
        ? otherwise(cmd, args).then((was) => ({
            ...(was as Record<string, unknown>),
            machines: [{ id: "mac0-0001", when: Math.floor(Date.now() / 1000), mine: true }],
          }))
        : otherwise(cmd, args);
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    await screen.findByText(/mac0-0001/);
    expect(screen.queryByText(/have not synced in a while/i)).toBeNull();
  });

  it("never lets go of an attachment without being told twice", async () => {
    asked.sure = false;
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));
    await screen.findByText("charla-a3f9.mp4");

    await userEvent.click(screen.getAllByRole("button", { name: /take it out/i })[0]);

    await waitFor(() => expect(sent("retire_attachment")).toHaveLength(0));
  });

  it("lets go of the one it was pointed at, and looks again afterwards", async () => {
    asked.sure = true;
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));
    await screen.findByText("charla-a3f9.mp4");
    const looks = sent("checked").length;

    await userEvent.click(screen.getAllByRole("button", { name: /take it out/i })[0]);

    await waitFor(() => expect(sent("retire_attachment")).toHaveLength(1));
    expect(sent("retire_attachment")[0].args.reference).toBe("attachments/ab/charla-a3f9.mp4");
    await waitFor(() => expect(sent("checked").length).toBeGreaterThan(looks));
  });

  it("says what letting go means before it happens", async () => {
    asked.sure = false;
    const said: string[] = [];
    const dialog = await import("@tauri-apps/plugin-dialog");
    const was = dialog.ask;
    vi.spyOn(dialog, "ask").mockImplementation((text: string) => {
      said.push(text);
      return was(text);
    });
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));
    await screen.findByText("charla-a3f9.mp4");

    await userEvent.click(screen.getAllByRole("button", { name: /take it out/i })[0]);

    await waitFor(() => expect(said).toHaveLength(1));
    expect(said[0]).toMatch(/30 days/i);
    expect(said[0]).toMatch(/does not get it back/i);
  });

  it("says so instead of letting go of something that got referenced meanwhile", async () => {
    asked.sure = true;
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "retire_attachment"
        ? Promise.reject({ code: "stillReferenced", name: "charla-a3f9.mp4" })
        : otherwise(cmd, args);
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));
    await screen.findByText("charla-a3f9.mp4");

    await userEvent.click(screen.getAllByRole("button", { name: /take it out/i })[0]);

    await waitFor(() =>
      expect(screen.getAllByText(/references that attachment now/i).length).toBeGreaterThan(0),
    );
  });

  it("offers to remove another machine, never this one", async () => {
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    await screen.findByText("win1-0002");
    expect(screen.getAllByRole("button", { name: /^remove$/i })).toHaveLength(1);
  });

  it("never removes a machine without being told twice", async () => {
    asked.sure = false;
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));
    await screen.findByText("win1-0002");

    await userEvent.click(screen.getByRole("button", { name: /^remove$/i }));

    await waitFor(() => expect(sent("remove_machine")).toHaveLength(0));
  });

  it("removes the machine it was pointed at, and looks again afterwards", async () => {
    asked.sure = true;
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));
    await screen.findByText("win1-0002");
    const looks = sent("checked").length;

    await userEvent.click(screen.getByRole("button", { name: /^remove$/i }));

    await waitFor(() => expect(sent("remove_machine")).toHaveLength(1));
    expect(sent("remove_machine")[0].args.id).toBe("win1-0002");
    await waitFor(() => expect(sent("checked").length).toBeGreaterThan(looks));
  });

  it("tells you to settle the machines before judging what is left over", async () => {
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/before deciding what is left over/i)).toBeTruthy();
  });

  it("stops nagging about the machines once none of them is behind", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "checked"
        ? otherwise(cmd, args).then((was) => ({
            ...(was as Record<string, unknown>),
            machines: [{ id: "mac0-0001", when: Math.floor(Date.now() / 1000), mine: true }],
          }))
        : otherwise(cmd, args);
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    await screen.findByText("charla-a3f9.mp4");
    expect(screen.queryByText(/before deciding what is left over/i)).toBeNull();
  });

  it("shows each loose attachment by name, weight and date", async () => {
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText("charla-a3f9.mp4")).toBeTruthy();
    expect(screen.getByText("notas-b1c2.pdf")).toBeTruthy();
  });

  it("says plainly that another machine may still be using them", async () => {
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/another one may still reference them/i)).toBeTruthy();
  });

  it("breaks the weight down, so the size has somewhere to come from", async () => {
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/the log weighs/i)).toBeTruthy();
    expect(screen.getByText(/the documents weigh/i)).toBeTruthy();
    expect(screen.getByText(/the attachments weigh/i)).toBeTruthy();
  });

  it("never empties this machine without being told twice", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.reject({ code: "wouldReset", name: "01MOTHER" })
        : otherwise(cmd, args);
    asked.sure = false;
    asked.file = "C:/keep/tisty-before-joining.zip";
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/leaving copies in/i);

    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));

    await waitFor(() => expect(screen.getByText(/does not merge/i)).toBeTruthy());
    expect(sent("join_them")).toHaveLength(0);
  });

  it("empties nothing when there is nowhere to put the backup", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.reject({ code: "wouldReset", name: "01MOTHER" })
        : otherwise(cmd, args);
    asked.sure = true;
    asked.file = null;
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/leaving copies in/i);

    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));

    await waitFor(() => expect(screen.getByText(/does not merge/i)).toBeTruthy());
    expect(sent("join_them")).toHaveLength(0);
  });

  it("backs this machine up before it joins the other history", async () => {
    let refused = true;
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) => {
      if (cmd !== "sync_now") return otherwise(cmd, args);
      if (refused) {
        refused = false;
        return Promise.reject({ code: "wouldReset", name: "01MOTHER" });
      }
      return Promise.resolve("came");
    };
    asked.sure = true;
    asked.file = "C:/keep/tisty-before-joining.zip";
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/leaving copies in/i);

    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));

    await waitFor(() => expect(sent("join_them")).toHaveLength(1));
    expect(sent("join_them")[0].args.into).toBe("C:/keep/tisty-before-joining.zip");
    expect(sent("sync_now")).toHaveLength(2);
  });

  it("names the documents that would open read only, and what each brings", async () => {
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /go through the documents/i }));

    expect(await screen.findByText("Minuta del lunes")).toBeTruthy();
    expect(screen.getByText(/the front matter at the top/i)).toBeTruthy();
    expect(screen.queryByText("Limpio")).toBeNull();
  });

  it("says so plainly when every document survives being saved", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "doc_read" ? Promise.resolve("# Limpio\n\nun parrafo") : otherwise(cmd, args);
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /go through the documents/i }));

    expect(await screen.findByText(/every document survives being saved/i)).toBeTruthy();
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

  it("decides nothing when it is put off", async () => {
    const done = vi.fn();
    render(<Welcome onDone={done} />);

    await userEvent.click(screen.getByRole("button", { name: /decide later/i }));

    expect(done).toHaveBeenCalled();
    expect(sent("choose_sync").length).toBe(0);
  });

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

  it("writes one zip, and says whether the log goes in it", async () => {
    asked.file = "D:/issues/tisty-report.zip";
    await upkeep();

    await userEvent.click(screen.getByRole("button", { name: /save the report/i }));

    await waitFor(() => expect(sent("keep_report").length).toBe(1));
    expect(sent("keep_report")[0].args.at).toBe("D:/issues/tisty-report.zip");
    expect(sent("keep_report")[0].args.logs).toBe(true);
  });
});

describe("the command line on a Mac", () => {
  it("says so when the link lands where no shell looks", async () => {
    standing.withinReach = true;
    standing.onPath = false;
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/writing/i);

    expect(await screen.findByText(/no shell looks in that folder/i)).toBeTruthy();
    expect(screen.getByText(/\$HOME\/\.local\/bin/)).toBeTruthy();
    expect(screen.getByText(/brew install/)).toBeTruthy();
  });

  it("stays quiet where the folder is already searched", async () => {
    standing.withinReach = true;
    standing.onPath = true;
    render(<Keeping onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/writing/i);

    await screen.findByText(/a terminal can find/i);
    expect(screen.queryByText(/no shell looks in that folder/i)).toBeNull();
  });
});

import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Keeping from "../ui/Keeping";
import Welcome from "../ui/Welcome";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> => Promise.resolve(null),
}));

const serving = vi.hoisted(() => ({
  on: false,
  called: undefined as string | undefined,
  id: undefined as string | undefined,
  filed: 0,
}));

const installed = vi.hoisted(() => ({
  seen: [] as {
    id: string;
    name: string;
    at: string;
    wired: boolean;
    astray: boolean;
    points?: string;
  }[],
}));

const asked = vi.hoisted(() => ({
  folder: null as string | null,
  file: null as string | null,
  sure: false,
  said: "",
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
  ask: (said: string) => {
    asked.said = said;
    return Promise.resolve(asked.sure);
  },
}));

const standing = {
  shipped: true,
  withinReach: false,
  onPath: true,
  at: "C:/Programs/Tisty",
  through: "C:/Programs/Tisty",
};

const kept = { lines: [] as string[] };

const rousing = {
  offered: true,
  wakes: false,
  theirs: false,
};

const carrying = {
  chosen: undefined as string | undefined,
  asked: true,
  backsUp: true,
  last: undefined as string | undefined,
  heard: undefined as string | undefined,
  loose: 0,
};

beforeEach(() => {
  vi.restoreAllMocks();
  ipc.calls = [];
  Object.assign(rousing, { offered: true, wakes: false, theirs: false });
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
  Object.assign(serving, { on: false, called: undefined, id: undefined, filed: 0 });
  installed.seen = [
    {
      id: "claude-code",
      name: "Claude Code",
      at: "C:/Users/someone/.claude.json",
      wired: true,
      astray: false,
      points: "C:/Programs/Tisty/tisty.exe",
    },
    {
      id: "antigravity",
      name: "Antigravity",
      at: "C:/Users/someone/.gemini/config/mcp_config.json",
      wired: false,
      astray: false,
    },
  ];
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
      case "agent":
        return Promise.resolve({ ...serving });
      case "agent_turn": {
        const on = Boolean(ipc.calls[ipc.calls.length - 1]?.args.on);
        Object.assign(serving, {
          on,
          called: on ? "espino 3" : undefined,
          id: on ? "dev_wskajy01" : undefined,
        });
        return Promise.resolve({ ...serving });
      }
      case "sync_now":
        return Promise.resolve({ carried: "came", undecided: [] });
      case "reachable":
        return Promise.resolve({ ...standing });
      case "wiring":
        return Promise.resolve(installed.seen.map((one) => ({ ...one })));
      case "wire":
      case "unwire": {
        const id = String(ipc.calls[ipc.calls.length - 1]?.args.id);
        const on = cmd === "wire";
        installed.seen = installed.seen.map((one) =>
          one.id === id ? { ...one, wired: on, astray: false } : one
        );
        return Promise.resolve(installed.seen.map((one) => ({ ...one })));
      }
      case "reach_for":
        standing.withinReach = Boolean(ipc.calls[ipc.calls.length - 1]?.args.wanted);
        return Promise.resolve({ ...standing });
      case "guide":
        return Promise.resolve({ id: "guide-0001", title: "Cómo funciona Tisty" });
      case "waking":
        return Promise.resolve({ ...rousing });
      case "wake_for":
        if (!rousing.theirs) {
          rousing.wakes = Boolean(ipc.calls[ipc.calls.length - 1]?.args.wanted);
        }
        return Promise.resolve({ ...rousing });
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
            {
              id: "mac0-0001",
              called: "cedro 14",
              when: Math.floor(Date.now() / 1000),
              mine: true,
            },
            {
              id: "win1-0002",
              called: "salvia 07",
              when: Math.floor(Date.now() / 1000) - 60 * 60 * 24 * 12,
              mine: false,
            },
          ],
          stranded: 0,
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
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);

    expect(await screen.findByText(/only on this machine/i)).toBeTruthy();
    expect(screen.getByText("no destination")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /sync now/i })).toBeNull();
  });

  it("remembers the folder that was picked", async () => {
    asked.folder = "G:/My Drive/tisty";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /turn on/i }));

    await waitFor(() => expect(sent("choose_sync").length).toBe(1));
    expect(sent("choose_sync")[0].args.dest).toBe("G:/My Drive/tisty");
  });

  it("hides backing up once a shared folder holds every machine", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    carrying.backsUp = false;
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);

    expect(await screen.findByText(/already holds every machine/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /create backup/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /restore/i })).toBeNull();
  });

  it("never restores without asking first", async () => {
    asked.file = "C:/keep/tisty-backup.zip";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /restore/i }));

    await waitFor(() => expect(sent("back_up").length + sent("restore").length).toBe(0));
  });

  it("restores once the warning is accepted", async () => {
    asked.file = "C:/keep/tisty-backup.zip";
    asked.sure = true;
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /restore/i }));

    await waitFor(() => expect(sent("restore").length).toBe(1));
    expect(sent("restore")[0].args.from).toBe("C:/keep/tisty-backup.zip");
  });

  it("says why the other cards went quiet", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) => (cmd === "sync_now" ? new Promise(() => {}) : otherwise(cmd, args));
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
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
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
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
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/mac0-0001/)).toBeTruthy();
    expect(screen.getByText(/win1-0002/)).toBeTruthy();
    expect(screen.getByText("This machine")).toBeTruthy();
  });

  it("calls every machine something a person can read out loud", async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText("cedro 14")).toBeTruthy();
    expect(screen.getByText("salvia 07")).toBeTruthy();
  });

  it("never offers to remove the machine you are on", async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    await screen.findByText("This machine");
    expect(screen.getAllByRole("button", { name: /^remove$/i })).toHaveLength(1);
    expect(screen.getByText(/not removable/i)).toBeTruthy();
  });

  it("names the machine and when it last wrote before removing it", async () => {
    asked.sure = false;
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));
    await screen.findByText("salvia 07");

    await userEvent.click(screen.getByRole("button", { name: /^remove$/i }));

    await waitFor(() => expect(asked.said).toMatch(/salvia 07/));
    expect(asked.said).toMatch(/last wrote/i);
    expect(asked.said).not.toMatch(/win1-0002/);
  });

  it("says out loud that a machine has been away, so nothing is judged on stale news", async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    const said = await screen.findByText(/may still be using what looks loose below/i);

    expect(said.textContent).toMatch(/have not written here in a while/i);
    expect(said.textContent).not.toMatch(/synced/i);
    expect(said.textContent).not.toMatch(/remove/i);
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
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    await screen.findByText(/mac0-0001/);
    expect(screen.queryByText(/have not synced in a while/i)).toBeNull();
  });

  it("never lets go of an attachment without being told twice", async () => {
    asked.sure = false;
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));
    await screen.findByText("charla-a3f9.mp4");

    await userEvent.click(screen.getAllByRole("button", { name: /take it out/i })[0]);

    await waitFor(() => expect(sent("retire_attachment")).toHaveLength(0));
  });

  it("lets go of the one it was pointed at, and looks again afterwards", async () => {
    asked.sure = true;
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
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
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
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
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
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
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    await screen.findByText("win1-0002");
    expect(screen.getAllByRole("button", { name: /^remove$/i })).toHaveLength(1);
  });

  it("never removes a machine without being told twice", async () => {
    asked.sure = false;
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));
    await screen.findByText("win1-0002");

    await userEvent.click(screen.getByRole("button", { name: /^remove$/i }));

    await waitFor(() => expect(sent("remove_machine")).toHaveLength(0));
  });

  it("removes the machine it was pointed at, and looks again afterwards", async () => {
    asked.sure = true;
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
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
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
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
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    await screen.findByText("charla-a3f9.mp4");
    expect(screen.queryByText(/before deciding what is left over/i)).toBeNull();
  });

  it("shows each loose attachment by name, weight and date", async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText("charla-a3f9.mp4")).toBeTruthy();
    expect(screen.getByText("notas-b1c2.pdf")).toBeTruthy();
  });

  it("looks for copies only when asked, so opening the tab reads nothing", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "twinned"
        ? Promise.resolve([
            {
              bytes: 300_000,
              at: ["attachments/ab/charla-a3f9.mp4", "attachments/ab/video-a3f9.mp4"],
            },
          ])
        : otherwise(cmd, args);
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    expect(sent("twinned")).toHaveLength(0);

    await userEvent.click(screen.getByRole("button", { name: /look for copies/i }));

    expect(await screen.findByText("ab/video-a3f9.mp4")).toBeTruthy();
    expect(screen.getByText(/does not choose which|no elige cuál/i)).toBeTruthy();
  });

  it("says nothing is kept twice when nothing is", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) => (cmd === "twinned" ? Promise.resolve([]) : otherwise(cmd, args));
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /look for copies/i }));

    expect(await screen.findByText(/nothing is kept twice/i)).toBeTruthy();
  });

  it("says plainly that another machine may still be using them", async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/another one may still reference them/i)).toBeTruthy();
  });

  it("breaks the weight down, so the size has somewhere to come from", async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/the log weighs/i)).toBeTruthy();
    expect(screen.getByText(/the documents weigh/i)).toBeTruthy();
    expect(screen.getByText(/the attachments weigh/i)).toBeTruthy();
  });

  it("asks after days of hearing nothing, and blames neither side", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    carrying.heard = new Date(Date.now() - 5 * 86_400_000).toISOString();
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);

    const said = await screen.findByText(/without anything arriving/i);

    expect(said.textContent).toMatch(/may be off/i);
    expect(said.textContent).toMatch(/may not be running/i);
  });

  it("stays quiet while the other machines are still turning up", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    carrying.heard = new Date(Date.now() - 3_600_000).toISOString();
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/leaving copies in/i);

    expect(screen.queryByText(/without anything arriving/i)).toBeNull();
  });

  it("says the work reached the folder, and never that a cloud took it", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.resolve({ carried: "sent", undecided: [], unreadable: [] })
        : otherwise(cmd, args);
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/leaving copies in/i);

    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));

    const said = await screen.findByText(/were written to the folder/i);

    expect(said.textContent).not.toMatch(/uploaded|went out/i);
  });

  it("offers keeping both first, because it is the only answer that loses nothing", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.resolve({ carried: "came", undecided: ["dev_a-0001"] })
        : otherwise(cmd, args);
    const said: string[] = [];
    const dialog = await import("@tauri-apps/plugin-dialog");
    vi.spyOn(dialog, "ask").mockImplementation((text: string) => {
      said.push(text);
      return Promise.resolve(true);
    });
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/leaving copies in/i);

    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));

    await waitFor(() => expect(sent("settle_paper")).toHaveLength(1));
    expect(said[0]).toMatch(/keep both/i);
    expect(sent("settle_paper")[0].args.keep).toBe("both");
  });

  it("only asks whose wins after keeping both was turned down", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.resolve({ carried: "came", undecided: ["dev_a-0001"] })
        : otherwise(cmd, args);
    const dialog = await import("@tauri-apps/plugin-dialog");
    let asked = 0;
    vi.spyOn(dialog, "ask").mockImplementation(() => Promise.resolve(++asked > 1));
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/leaving copies in/i);

    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));

    await waitFor(() => expect(sent("settle_paper")).toHaveLength(1));
    expect(sent("settle_paper")[0].args.keep).toBe("mine");
    expect(asked).toBe(2);
  });

  it("asks nothing at all when nothing is at odds", async () => {
    const dialog = await import("@tauri-apps/plugin-dialog");
    const spy = vi.spyOn(dialog, "ask");
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/leaving copies in/i);

    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));

    await waitFor(() => expect(sent("sync_now").length).toBeGreaterThan(0));
    expect(sent("settle_paper")).toHaveLength(0);
    expect(spy).not.toHaveBeenCalled();
  });

  const apart = (code = "wouldReset") => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now" ? Promise.reject({ code, name: "01MOTHER" }) : otherwise(cmd, args);
  };

  const onceApart = (code = "wouldReset") => {
    let refused = true;
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) => {
      if (cmd !== "sync_now") return otherwise(cmd, args);
      if (refused) {
        refused = false;
        return Promise.reject({ code, name: "01MOTHER" });
      }
      return Promise.resolve({ carried: "came", undecided: [] });
    };
  };

  const carried = async () => {
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/leaving copies in/i);
    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));
  };

  it("carries as soon as a folder is picked, so the doors open there and not later", async () => {
    asked.folder = "G:/My Drive/tisty";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /turn on/i }));

    await waitFor(() => expect(sent("choose_sync")).toHaveLength(1));
    await waitFor(() => expect(sent("sync_now")).toHaveLength(1));
  });

  it("opens the doors on picking a folder that already holds another history", async () => {
    apart();
    asked.folder = "G:/My Drive/tisty";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);

    await userEvent.click(screen.getByRole("button", { name: /turn on/i }));

    expect((await screen.findByRole("dialog")).textContent).toMatch(/already holds another Tisty/i);
  });

  it("does not leave you pointing at a folder you walked away from", async () => {
    apart();
    asked.folder = "G:/My Drive/tisty";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await userEvent.click(screen.getByRole("button", { name: /turn on/i }));

    const doors = await screen.findByRole("dialog");
    await userEvent.click(within(doors).getByRole("button", { name: /^close$/i }));

    await waitFor(() => expect(sent("choose_sync")).toHaveLength(2));
    expect(sent("choose_sync")[1].args.dest).toBeUndefined();
  });

  it("keeps the folder you did accept, and does not undo it", async () => {
    onceApart();
    asked.folder = "G:/My Drive/tisty";
    asked.file = "C:/keep/tisty-folder-before.zip";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await userEvent.click(screen.getByRole("button", { name: /turn on/i }));

    await userEvent.click(await screen.findByRole("button", { name: /keep this machine/i }));

    await waitFor(() => expect(sent("take_over")).toHaveLength(1));
    expect(sent("choose_sync")).toHaveLength(1);
  });

  it("asks nothing when the folder already holds this machine's own history", async () => {
    onceApart();
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_kin" ? Promise.resolve("sameLineage") : otherwise(cmd, args);
    asked.file = "C:/keep/tisty-before-joining-both.zip";
    await carried();

    const said = await screen.findByRole("dialog");
    expect(said.textContent).toMatch(/already holds this machine/i);
    expect(said.textContent).not.toMatch(/two lists by the same name/i);
    expect(screen.queryByRole("button", { name: /keep this machine/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /take what the folder has/i })).toBeNull();
  });

  it("shuts the merging door when the two clash under one machine name", async () => {
    apart();
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_kin" ? Promise.resolve("clash") : otherwise(cmd, args);
    await carried();

    const said = await screen.findByRole("dialog");
    expect(said.textContent).toMatch(/wrote different things/i);
    expect(screen.queryByRole("button", { name: /merge the two/i })).toBeNull();
    expect(screen.getByRole("button", { name: /keep this machine/i })).toBeTruthy();
  });

  it("refuses to guess when the folder could not be read well enough", async () => {
    apart();
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_kin" ? Promise.resolve("unsure") : otherwise(cmd, args);
    await carried();

    const said = await screen.findByRole("dialog");
    expect(said.textContent).toMatch(/could not be read/i);
    for (const door of [/merge the two/i, /keep this machine/i, /take what the folder has/i]) {
      expect(screen.queryByRole("button", { name: door })).toBeNull();
    }
  });

  it("says which document was put together, by its title", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.resolve({
            carried: "came",
            undecided: [],
            unreadable: [],
            astray: [],
            joined: ["a-0002"],
          })
        : otherwise(cmd, args);
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/leaving copies in/i);

    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));

    const said = await screen.findByText(/put together/i);
    expect(said.textContent).toMatch(/Minuta del lunes/);
    expect(said.textContent).not.toMatch(/a-0002/);
  });

  it("opens the three doors instead of a yes or no", async () => {
    apart();
    await carried();

    const said = await screen.findByRole("dialog");
    expect(said.textContent).toMatch(/already holds another Tisty/i);
    for (const door of [/merge the two/i, /keep this machine/i, /take what the folder has/i]) {
      expect(screen.getByRole("button", { name: door })).toBeTruthy();
    }
  });

  it("opens them for the other refusal too, which is the one people meet", async () => {
    apart("otherStore");
    await carried();

    expect((await screen.findByRole("dialog")).textContent).toMatch(/already holds another Tisty/i);
  });

  it("never shows a store identifier where a person reads", async () => {
    apart();
    await carried();

    expect((await screen.findByRole("dialog")).textContent).not.toContain("01MOTHER");
  });

  it("empties nothing when the doors are closed without picking", async () => {
    apart();
    asked.file = "C:/keep/tisty-before-joining.zip";
    await carried();

    const doors = await screen.findByRole("dialog");
    await userEvent.click(within(doors).getByRole("button", { name: /^close$/i }));

    await waitFor(() => expect(screen.getByText(/nothing was changed/i)).toBeTruthy());
    expect(sent("join_them")).toHaveLength(0);
    expect(sent("take_over")).toHaveLength(0);
  });

  it("empties nothing when there is nowhere to put the backup", async () => {
    apart();
    asked.file = null;
    await carried();

    await userEvent.click(await screen.findByRole("button", { name: /take what the folder has/i }));

    await waitFor(() => expect(screen.getByText(/nothing was changed/i)).toBeTruthy());
    expect(sent("join_them")).toHaveLength(0);
  });

  it("backs this machine up before it joins the other history", async () => {
    onceApart();
    asked.file = "C:/keep/tisty-before-joining.zip";
    await carried();

    await userEvent.click(await screen.findByRole("button", { name: /take what the folder has/i }));

    await waitFor(() => expect(sent("join_them")).toHaveLength(1));
    expect(sent("join_them")[0].args.into).toBe("C:/keep/tisty-before-joining.zip");
    expect(sent("sync_now")).toHaveLength(2);
  });

  it("backs the folder up before this machine takes it over", async () => {
    onceApart();
    asked.file = "C:/keep/tisty-folder-before.zip";
    await carried();

    await userEvent.click(await screen.findByRole("button", { name: /keep this machine/i }));

    await waitFor(() => expect(sent("take_over")).toHaveLength(1));
    expect(sent("take_over")[0].args.into).toBe("C:/keep/tisty-folder-before.zip");
    expect(sent("join_them")).toHaveLength(0);
    expect(sent("sync_now")).toHaveLength(2);
  });

  it("never empties this machine when the machine is what was kept", async () => {
    onceApart();
    asked.file = "C:/keep/tisty-folder-before.zip";
    await carried();

    await userEvent.click(await screen.findByRole("button", { name: /keep this machine/i }));

    await waitFor(() => expect(sent("take_over")).toHaveLength(1));
    expect(sent("join_them")).toHaveLength(0);
  });

  it("joins the two histories when that is the door taken", async () => {
    onceApart();
    asked.file = "C:/keep/tisty-before-joining-both.zip";
    await carried();

    await userEvent.click(await screen.findByRole("button", { name: /merge the two/i }));

    await waitFor(() => expect(sent("merge_stores")).toHaveLength(1));
    expect(sent("merge_stores")[0].args.into).toBe("C:/keep/tisty-before-joining-both.zip");
    expect(sent("join_them")).toHaveLength(0);
    expect(sent("take_over")).toHaveLength(0);
    expect(sent("sync_now")).toHaveLength(2);
  });

  it("says what merging costs before it is taken, not after", async () => {
    apart();
    await carried();

    const said = (await screen.findByRole("dialog")).textContent ?? "";
    expect(said).toMatch(/without losing anything/i);
    expect(said).toMatch(/two lists by the same name stay as two/i);
  });

  it("empties nothing when there is nowhere to put the backup for a merge", async () => {
    apart();
    asked.file = null;
    await carried();

    await userEvent.click(await screen.findByRole("button", { name: /merge the two/i }));

    await waitFor(() => expect(screen.getByText(/nothing was changed/i)).toBeTruthy());
    expect(sent("merge_stores")).toHaveLength(0);
  });

  it("lets someone walk away to another folder instead of choosing", async () => {
    onceApart();
    asked.folder = "G:/My Drive/otra";
    await carried();

    await userEvent.click(await screen.findByRole("button", { name: /pick another folder/i }));

    await waitFor(() => expect(sent("choose_sync")).toHaveLength(1));
    expect(sent("join_them")).toHaveLength(0);
    expect(sent("take_over")).toHaveLength(0);
  });

  it("names the documents that would open read only, and what each brings", async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
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
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /go through the documents/i }));

    expect(await screen.findByText(/every document survives being saved/i)).toBeTruthy();
  });

  it("says what the review found", async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);

    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/7 tasks · 2 lists/i)).toBeTruthy();
    expect(screen.getByText(/311.0 kB/)).toBeTruthy();
  });

  it("reads its settings again once the welcome has been through", async () => {
    const { rerender } = render(<Keeping greeted={0} onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    const before = sent("settings").length;

    rerender(<Keeping greeted={1} onGreet={() => {}} onChanged={() => {}} />);

    await waitFor(() => expect(sent("settings").length).toBe(before + 1));
  });

  it("holds a language of its own when one is picked", async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/notices and startup/i);

    await userEvent.selectOptions(await screen.findByLabelText(/^language$/i), "es");

    await waitFor(() => expect(sent("keep_locale").length).toBe(1));
    expect(sent("keep_locale")[0].args.locale).toBe("es");
  });

  it("offers the welcome again, without touching what is written", async () => {
    const greet = vi.fn();
    render(<Keeping onGreet={greet} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/notices and startup/i);

    await userEvent.click(await screen.findByRole("button", { name: /show it again/i }));

    expect(greet).toHaveBeenCalled();
    expect(sent("choose_sync").length).toBe(0);
  });
});

describe("the first-run assistant", () => {
  const spoken = async () => {
    await userEvent.click(await screen.findByRole("button", { name: /^english$/i }));
  };

  it("asks for the language before anything else, and keeps it", async () => {
    render(<Welcome onDone={vi.fn()} />);

    await spoken();

    expect(sent("keep_locale")[0].args.locale).toBe("en");
    expect(await screen.findByRole("button", { name: /only on this machine/i })).toBeTruthy();
  });

  it("takes «only here» as an answer, not as a blank", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await userEvent.click(screen.getByRole("button", { name: /only on this machine/i }));

    await waitFor(() => expect(sent("choose_sync").length).toBe(1));
    expect(sent("choose_sync")[0].args.dest).toBeUndefined();
  });

  it("decides nothing about copies when they are put off", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await userEvent.click(screen.getByRole("button", { name: /decide later/i }));

    expect(sent("choose_sync").length).toBe(0);
    expect(await screen.findByRole("button", { name: /open it at sign-in/i })).toBeTruthy();
  });

  it("asks the machine to open at sign-in when that is the answer", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(screen.getByRole("button", { name: /decide later/i }));

    await userEvent.click(await screen.findByRole("button", { name: /open it at sign-in/i }));

    await waitFor(() => expect(sent("wake_for").length).toBe(1));
    expect(sent("wake_for")[0].args.wanted).toBe(true);
  });

  it("skips the sign-in step where the machine cannot offer it", async () => {
    rousing.offered = false;
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await userEvent.click(screen.getByRole("button", { name: /decide later/i }));

    expect(await screen.findByRole("button", { name: /leave it in the tray/i })).toBeTruthy();
    expect(sent("wake_for").length).toBe(0);
  });

  it("remembers what closing the window should do", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(screen.getByRole("button", { name: /decide later/i }));
    await userEvent.click(await screen.findByRole("button", { name: /open it at sign-in/i }));

    await userEvent.click(await screen.findByRole("button", { name: /leave it in the tray/i }));

    await waitFor(() => expect(sent("keep_closing").length).toBe(1));
    expect(sent("keep_closing")[0].args.how).toBe("hide");
  });

  it("still marks the run as done when copies were put off", async () => {
    const done = vi.fn();
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(screen.getByRole("button", { name: /decide later/i }));
    await userEvent.click(await screen.findByRole("button", { name: /open it at sign-in/i }));
    await userEvent.click(await screen.findByRole("button", { name: /leave it in the tray/i }));

    await userEvent.click(await screen.findByRole("button", { name: /open the guide/i }));

    await waitFor(() => expect(done).toHaveBeenCalled());
    expect(sent("choose_sync")).toHaveLength(1);
    expect(sent("choose_sync")[0].args.dest).toBeUndefined();
  });

  it("ends on the guide, and hands back the document it planted", async () => {
    const done = vi.fn();
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(screen.getByRole("button", { name: /decide later/i }));
    await userEvent.click(await screen.findByRole("button", { name: /open it at sign-in/i }));
    await userEvent.click(await screen.findByRole("button", { name: /leave it in the tray/i }));

    await userEvent.click(await screen.findByRole("button", { name: /open the guide/i }));

    await waitFor(() => expect(sent("guide")).toHaveLength(1));
    expect(done).toHaveBeenCalledWith("guide-0001");
  });

  it("still lets the run end when the guide cannot be written", async () => {
    const done = vi.fn();
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "guide"
        ? Promise.reject({ code: "cannotRead", name: "os error 3" })
        : answered(cmd, args);

    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(screen.getByRole("button", { name: /decide later/i }));
    await userEvent.click(await screen.findByRole("button", { name: /open it at sign-in/i }));
    await userEvent.click(await screen.findByRole("button", { name: /leave it in the tray/i }));

    await userEvent.click(await screen.findByRole("button", { name: /open the guide/i }));
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(done).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole("button", { name: /get started/i }));

    await waitFor(() => expect(done).toHaveBeenCalled());
    expect(done.mock.calls[0][0]).toBeUndefined();
  });

  it("goes back, and shows what was already chosen", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await screen.findByRole("button", { name: /only on this machine/i });

    await userEvent.click(screen.getByRole("button", { name: /^back$/i }));

    const english = await screen.findByRole("button", { name: /^english$/i });
    expect(english.getAttribute("aria-pressed")).toBe("true");
  });

  it("takes a change of mind about the language", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /^back$/i }));

    await userEvent.click(screen.getByRole("button", { name: /^español$/i }));

    expect(sent("keep_locale").map((one) => one.args.locale)).toEqual(["en", "es"]);
  });

  it("is done only once the last step says so", async () => {
    const done = vi.fn();
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(screen.getByRole("button", { name: /decide later/i }));
    await userEvent.click(await screen.findByRole("button", { name: /open it at sign-in/i }));
    await userEvent.click(await screen.findByRole("button", { name: /leave it in the tray/i }));

    expect(done).not.toHaveBeenCalled();
    await userEvent.click(await screen.findByRole("button", { name: /open the guide/i }));

    expect(done).toHaveBeenCalled();
  });

  it("offers the command line, and says what to do next", async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/writing/i);
    expect(await screen.findByText(/cannot find it yet/i)).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: /make it reachable/i }));

    await waitFor(() => expect(sent("reach_for").length).toBe(1));
    expect(sent("reach_for")[0].args.wanted).toBe(true);
    expect(await screen.findByText(/next time you sign in/i)).toBeTruthy();
  });

  it("takes it back off when asked", async () => {
    standing.withinReach = true;
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/writing/i);
    expect(await screen.findByText(/already finds/i)).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: /take it back out/i }));

    await waitFor(() => expect(sent("reach_for")[0].args.wanted).toBe(false));
  });

  it("says nothing when there is no command line to offer", async () => {
    standing.shipped = false;
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/writing/i);

    expect(screen.queryByRole("button", { name: /make it reachable/i })).toBeNull();
  });
});

describe("the report a bug gets attached to", () => {
  const upkeep = async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
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
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/writing/i);

    expect(await screen.findByText(/no shell looks in that folder/i)).toBeTruthy();
    expect(screen.getByText(/\$HOME\/\.local\/bin/)).toBeTruthy();
    expect(screen.getByText(/brew install/)).toBeTruthy();
  });

  it("stays quiet where the folder is already searched", async () => {
    standing.withinReach = true;
    standing.onPath = true;
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/writing/i);

    await screen.findByText(/already finds/i);
    expect(screen.queryByText(/no shell looks in that folder/i)).toBeNull();
  });
});

describe("opening with the machine", () => {
  const notices = async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/notices/i);
  };

  it("asks the machine to open Tisty at sign-in", async () => {
    await notices();
    expect(await screen.findByText(/opens only when you open it/i)).toBeTruthy();

    const box = screen.getByRole("checkbox", { name: /open it at sign-in/i }) as HTMLInputElement;
    expect(box.checked).toBe(false);
    await userEvent.click(box);

    await waitFor(() => expect(sent("wake_for").length).toBe(1));
    expect(sent("wake_for")[0].args.wanted).toBe(true);
    expect(await screen.findByText(/next time you sign in/i)).toBeTruthy();
  });

  it("leaves it closed again when asked", async () => {
    rousing.wakes = true;
    await notices();
    expect(await screen.findByText(/opens by itself when you sign in/i)).toBeTruthy();

    const box = screen.getByRole("checkbox", { name: /open it at sign-in/i }) as HTMLInputElement;
    expect(box.checked).toBe(true);
    await userEvent.click(box);

    await waitFor(() => expect(sent("wake_for")[0].args.wanted).toBe(false));
    expect(await screen.findByText(/will not open on its own/i)).toBeTruthy();
  });

  it("says who holds the switch when the system took it", async () => {
    rousing.theirs = true;
    await notices();

    expect(await screen.findByText(/Settings → Apps → Startup/i)).toBeTruthy();
  });

  it("announces nothing when the system refused to hand the switch back", async () => {
    rousing.theirs = true;
    await notices();

    await userEvent.click(screen.getByRole("checkbox", { name: /open it at sign-in/i }));

    await waitFor(() => expect(sent("wake_for").length).toBe(1));
    expect(screen.queryByText(/next time you sign in/i)).toBeNull();
    expect(screen.getByText(/Settings → Apps → Startup/i)).toBeTruthy();
  });

  it("offers nothing where nothing can be offered", async () => {
    rousing.offered = false;
    await notices();

    expect(screen.queryByRole("checkbox", { name: /open it at sign-in/i })).toBeNull();
  });
});

describe("stranded document files", () => {
  it("says when the log does not know about a file on disk", async () => {
    const was = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "checked"
        ? was(cmd, args).then((one) => ({ ...(one as object), stranded: 3 }))
        : was(cmd, args);
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    expect(await screen.findByText(/does not know about/i)).toBeTruthy();
  });

  it("says nothing when every file on disk is in the log", async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await go(/maintenance/i);
    await userEvent.click(screen.getByRole("button", { name: /^review$/i }));

    await screen.findByText(/mac0-0001/);
    expect(screen.queryByText(/does not know about/i)).toBeNull();
  });
});

describe("looking for an update without waiting for tomorrow", () => {
  const openTab = async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await userEvent.click(screen.getByRole("tab", { name: /notices/i }));
  };

  it("asks the moment the person asks, not on the daily schedule", async () => {
    ipc.answer = (
      (was) => (cmd, args) =>
        cmd === "update_ready" ? Promise.resolve(null) : was(cmd, args)
    )(ipc.answer);

    await openTab();
    await userEvent.click(screen.getByRole("button", { name: /check for updates/i }));

    await waitFor(() => expect(sent("update_ready").length).toBeGreaterThan(0));
    const asked = sent("update_ready");
    expect(asked[asked.length - 1].args.nowPlease).toBe(true);
    expect(await screen.findByText(/on the newest version/i)).toBeTruthy();
  });

  it("offers to install it right there, not somewhere else", async () => {
    ipc.answer = (
      (was) => (cmd, args) =>
        cmd === "update_ready"
          ? Promise.resolve({ version: "0.14.0", installs: true, route: "download" })
          : was(cmd, args)
    )(ipc.answer);

    await openTab();
    await userEvent.click(screen.getByRole("button", { name: /check for updates/i }));

    expect(await screen.findByText(/0\.14\.0 is out/i)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /^update$/i }));
    await waitFor(() => expect(sent("update_install").length).toBe(1));
  });

  it("says how to get it when this copy cannot update itself", async () => {
    ipc.answer = (
      (was) => (cmd, args) =>
        cmd === "update_ready"
          ? Promise.resolve({ version: "0.14.0", installs: false, route: "store" })
          : was(cmd, args)
    )(ipc.answer);

    await openTab();
    await userEvent.click(screen.getByRole("button", { name: /check for updates/i }));

    expect(await screen.findByText(/Microsoft Store|the Store/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /^update$/i })).toBeNull();
  });
});

describe("letting an assistant file work here", () => {
  const openTab = async () => {
    render(<Keeping onGreet={() => {}} onChanged={() => {}} />);
    await screen.findByText(/only on this machine/i);
    await userEvent.click(screen.getByRole("tab", { name: /assistants/i }));
  };

  it("says nothing can file here until the person turns one on", async () => {
    await openTab();

    expect(await screen.findByText(/No assistant can file work here/i)).toBeTruthy();
    expect(screen.getByRole("button", { name: /Let an assistant file here/i })).toBeTruthy();
  });

  it("registering is a click of the person's, never something that arrives over the wire", async () => {
    await openTab();

    await userEvent.click(screen.getByRole("button", { name: /Let an assistant file here/i }));

    await waitFor(() => expect(sent("agent_turn").length).toBe(1));
    expect(sent("agent_turn")[0].args.on).toBe(true);
    expect(await screen.findByText(/espino 3 can file work here/i)).toBeTruthy();
  });

  it("lists the assistants on this computer, with the file each one reads", async () => {
    await openTab();

    expect(await screen.findByText("Claude Code")).toBeTruthy();
    expect(screen.getByText("C:/Users/someone/.claude.json")).toBeTruthy();
    expect(screen.getByText(/^Connected$/)).toBeTruthy();
    expect(screen.getByRole("button", { name: /^Connect$/ })).toBeTruthy();
  });

  it("connecting one writes into that assistant's own settings and asks for a restart", async () => {
    await openTab();

    await userEvent.click(await screen.findByRole("button", { name: /^Connect$/ }));

    await waitFor(() => expect(sent("wire").length).toBe(1));
    expect(sent("wire")[0].args.id).toBe("antigravity");
    expect(await screen.findByText(/Close it and open it again/i)).toBeTruthy();
  });

  it("one already connected is taken back out rather than written twice", async () => {
    await openTab();

    await userEvent.click(await screen.findByRole("button", { name: /^Remove$/ }));

    await waitFor(() => expect(sent("unwire").length).toBe(1));
    expect(sent("unwire")[0].args.id).toBe("claude-code");
    expect(sent("wire").length).toBe(0);
  });

  it("a copy that is no longer there is said plainly, and pointing it here writes again", async () => {
    installed.seen = [
      {
        id: "codex",
        name: "Codex",
        at: "C:/Users/someone/.codex/config.toml",
        wired: true,
        astray: true,
        points: "C:/Programs/Gone/tisty.exe",
      },
    ];

    await openTab();

    expect(await screen.findByText(/no longer there/i)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /Point it here/i }));

    await waitFor(() => expect(sent("wire").length).toBe(1));
    expect(sent("wire")[0].args.id).toBe("codex");
  });

  it("with none of them installed it says so instead of showing an empty list", async () => {
    installed.seen = [];

    await openTab();

    expect(await screen.findByText(/knows its way around/i)).toBeTruthy();
  });

  it("spells out what it can never do, where the person decides", async () => {
    Object.assign(serving, { on: true, called: "espino 3", id: "dev_wskajy01", filed: 4 });

    await openTab();

    expect(await screen.findByText(/4 filed so far/i)).toBeTruthy();
    expect(screen.getByText(/Complete, reopen, drop or delete anything/i)).toBeTruthy();
    expect(screen.getByText(/your undo never reaches what it filed/i)).toBeTruthy();
    expect(screen.getByText(/"mcpServers"/)).toBeTruthy();
    expect(screen.getByText(/mcp add tisty/)).toBeTruthy();
  });
});

import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { heard, toldAside } from "../coming";
import { credit } from "../crediting";
import { adopt, fill, t } from "../locales";
import { ROW_IDS, ROWS, type Row, type RowId, type Shelf } from "../settingsRows";
import About from "../ui/About";
import Coming from "../ui/Coming";
import HelpBand from "../ui/HelpBand";
import HiddenFolders from "../ui/HiddenFolders";
import Settings from "../ui/Keeping";
import SettingsNav from "../ui/SettingsNav";
import Welcome from "../ui/Welcome";
import type { Tab } from "../views";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> => Promise.resolve(null),
}));

const serving = vi.hoisted(() => ({
  on: false,
  called: undefined as string | undefined,
  asks: true,
  hooks: true,
}));

const installed = vi.hoisted(() => ({
  seen: [] as {
    id: string;
    name: string;
    at: string;
    wired: boolean;
    astray: boolean;
    gone?: boolean;
    points?: string;
    maybe?: boolean;
    byHand?: boolean;
  }[],
  inDistros: [] as {
    id: string;
    name: string;
    place: string;
    at: string;
    wired: boolean;
    astray: boolean;
    gone: boolean;
    points: string;
  }[],
  hands: [] as {
    via: string | null;
    named: string;
    wired: boolean | null;
    filed: number;
    wrote: number;
    last: string | null;
  }[],
}));

const signing = vi.hoisted(() => ({
  alias: null as string | null,
  name: null as string | null,
  before: [] as string[],
  mine: 0,
}));

const careful = async (go: string) => {
  const dialog = await screen.findByRole("dialog");
  const box = within(dialog).queryByLabelText(t("carefulUndone"));
  if (box && go !== t("cancel")) await userEvent.click(box);
  await userEvent.click(within(dialog).getByRole("button", { name: go }));
};

const DRAWN_BY_THEIR_OWN_TESTS = [
  "cloudForget",
  "cloudRevoke",
  "cloudAbandon",
  "sealWipe",
  "sealOff",
];

const asked = vi.hoisted(() => ({
  folder: null as string | null,
  file: null as string | null,
  sure: false,
  said: "",
}));

const bus = vi.hoisted(() => ({
  heard: new Map<string, ((said: { payload: unknown }) => void)[]>(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return ipc.answer(cmd, args ?? {});
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (named: string, fn: (said: { payload: unknown }) => void) => {
    bus.heard.set(named, [...(bus.heard.get(named) ?? []), fn]);
    return Promise.resolve(() => {
      bus.heard.set(
        named,
        (bus.heard.get(named) ?? []).filter((one) => one !== fn),
      );
    });
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
  binary: "C:/Programs/Tisty/tisty.exe",
};

const kept = { lines: [] as string[] };

const rousing = {
  offered: true,
  wakes: false,
  theirs: false,
};

const holders = {
  offers: [] as { key: string; named: string; at?: string; into?: string }[],
  told: { keeper: "plain" } as { keeper: string; named?: string; into?: string },
};

const carrying = {
  chosen: undefined as string | undefined,
  asked: true,
  last: undefined as string | undefined,
  heard: undefined as string | undefined,
  weight: 1_000,
  carries: 8 * 1024 * 1024 * 1024,
  sharedWas: undefined as string | undefined,
  sharedWasLater: undefined as boolean | undefined,
  later: undefined as boolean | undefined,
  freesSpace: false,
  mode: undefined as string | undefined,
};

const arriving = {
  kin: "strangers" as string,
  fresh: true,
  holds: false,
  alias: null as string | null,
};

beforeEach(() => {
  vi.restoreAllMocks();
  adopt("en");
  ipc.calls = [];
  bus.heard.clear();
  Object.assign(arriving, { kin: "strangers", fresh: true, holds: false, alias: null });
  Object.assign(rousing, { offered: true, wakes: false, theirs: false });
  Object.assign(standing, {
    binary: "C:/Programs/Tisty/tisty.exe",
  });
  kept.lines = [];
  asked.folder = null;
  asked.file = null;
  asked.sure = false;
  Object.assign(serving, { on: false, called: undefined, asks: true, hooks: true });
  installed.inDistros = [];
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
    mode: undefined,
    asked: true,
    last: undefined,
    sharedWas: undefined,
    sharedWasLater: undefined,
    later: undefined,
  });
  holders.offers = [
    { key: "drive", named: "Google Drive", at: "G:/My Drive", into: "G:/My Drive/Tisty" },
    { key: "onedrive", named: "OneDrive" },
    { key: "dropbox", named: "Dropbox" },
    { key: "icloud", named: "iCloud Drive" },
  ];
  holders.told = { keeper: "plain" };
  signing.alias = null;
  signing.name = null;
  signing.before = [];
  signing.mine = 0;
  ipc.answer = (cmd) => {
    switch (cmd) {
      case "keepers":
        return Promise.resolve(holders.offers.map((one) => ({ ...one })));
      case "keeper_of": {
        const at = String(ipc.calls[ipc.calls.length - 1]?.args.at);
        return Promise.resolve({ into: at, ...holders.told });
      }
      case "make_room":
        return Promise.resolve(null);
      case "sign_the_rest": {
        const many = signing.mine;
        signing.mine = 0;
        return Promise.resolve(many);
      }
      case "sync_state":
        return Promise.resolve({
          ...carrying,
          mode:
            carrying.mode ?? (carrying.freesSpace ? "cloud" : carrying.chosen ? "folder" : "alone"),
        });
      case "signed":
        return Promise.resolve({ ...signing });
      case "sign_name": {
        const said = ipc.calls[ipc.calls.length - 1]?.args.name;
        signing.name = typeof said === "string" ? said : null;
        return Promise.resolve({ ...signing });
      }
      case "sign": {
        const said = ipc.calls[ipc.calls.length - 1]?.args.alias;
        if ((typeof said === "string" ? said : null) === signing.alias)
          return Promise.resolve({ ...signing });
        signing.alias = typeof said === "string" ? said : null;
        if (signing.alias)
          signing.before = [
            signing.alias,
            ...signing.before.filter((one) => one !== signing.alias),
          ];
        return Promise.resolve({ ...signing });
      }
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
      case "agent_asks":
        serving.asks = Boolean(ipc.calls[ipc.calls.length - 1]?.args.on);
        return Promise.resolve({ ...serving });
      case "agent_hooks":
        serving.hooks = Boolean(ipc.calls[ipc.calls.length - 1]?.args.on);
        return Promise.resolve({ ...serving });
      case "sync_now":
        return Promise.resolve({ carried: "came", undecided: [] });
      case "joining":
        return Promise.resolve({ ...arriving });
      case "place_peek":
        return Promise.resolve({ fresh: arriving.fresh, holds: arriving.holds });
      case "reachable":
        return Promise.resolve({ ...standing });
      case "wiring_watched":
        return Promise.resolve({
          agent: { ...serving },
          seen: serving.on ? installed.seen.map((one) => ({ ...one })) : null,
        });
      case "wsl_watched":
        return Promise.resolve(serving.on ? installed.inDistros.map((one) => ({ ...one })) : []);
      case "wiring":
        return Promise.resolve(installed.seen.map((one) => ({ ...one })));
      case "assistants":
        return Promise.resolve(installed.hands.map((one) => ({ ...one })));
      case "mend":
        return Promise.resolve({ seen: installed.seen.map((one) => ({ ...one })) });
      case "wire":
      case "unwire": {
        const id = String(ipc.calls[ipc.calls.length - 1]?.args.id);
        const on = cmd === "wire";
        installed.seen = installed.seen.map((one) =>
          one.id === id ? { ...one, wired: on, astray: false } : one,
        );
        return Promise.resolve(installed.seen.map((one) => ({ ...one })));
      }
      case "guide":
        return Promise.resolve({ id: "guide-0001", title: "Cómo funciona Tisty" });
      case "waking":
        return Promise.resolve({ ...rousing });
      case "wake_for":
        if (!rousing.theirs) {
          rousing.wakes = Boolean(ipc.calls[ipc.calls.length - 1]?.args.wanted);
        }
        return Promise.resolve({ ...rousing });
      case "keep_settings": {
        const asked = ipc.calls[ipc.calls.length - 1]?.args.settings as Record<string, unknown>;
        return Promise.resolve({
          ...asked,
          shares: true,
          onlySharedAbove: 50 * 1024 * 1024,
        });
      }
      case "free_up":
      case "free_space":
        return new Promise(() => {});
      case "unused_attachments":
        return Promise.resolve({ files: 3, bytes: 3 * 1024 * 1024 });
      case "stop_freeing":
        return Promise.resolve(null);
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
            {
              at: "attachments/ef/lejos-c3d4.mp4",
              bytes: 90_000_000,
              when: 1_754_000_000,
              shared: true,
            },
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
          stranded: [],
          missing: [],
          logBytes: 4_096,
          docsBytes: 20_480,
          heldBytes: 900_000,
          heldFiles: 9,
        });
      case "retire_attachments":
        return Promise.resolve(3);
      case "doc_adopt":
        return Promise.resolve({ id: "mac0-0042", title: "Minutes" });
      case "doc_let_go":
      case "doc_drop":
        return Promise.resolve(null);
      case "back_up":
        return Promise.resolve({ bytes: 4096, leftOut: 0, leftOutBytes: 0 });
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
          installed: "download",
          assistant: false,
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
          holds: "everywhere",
          shares: true,
          onlySharedAbove: 50 * 1024 * 1024,
          shortcut: null,
        });
      case "logs":
        return Promise.resolve({
          at: "C:/tisty/private/tisty.log",
          bytes: kept.lines.length === 0 ? 0 : 240,
          lines: kept.lines,
        });
      case "settings":
        return Promise.resolve({
          quiet: [],
          logsAll: false,
          holds: "everywhere",
          shares: true,
          onlySharedAbove: 50 * 1024 * 1024,
          onClose: "quit",
        });
      case "docs":
        return Promise.resolve({
          folders: [],
          docs: [
            {
              id: "1",
              file: "a-0001",
              title: "Limpio",
              folder: null,
              archived: false,
              away: false,
            },
            {
              id: "2",
              file: "a-0002",
              title: "Minuta del lunes",
              folder: null,
              archived: false,
              away: false,
            },
          ],
        });
      default:
        return Promise.resolve(null);
    }
  };
});

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

type Shown = Omit<Parameters<typeof Settings>[0], "tab" | "onTab"> & { start?: Tab };

// The sections are listed in the window's own sidebar, so the settings are tried with that list beside them.
function Keeping({ start, ...props }: Shown) {
  const [tab, setTab] = useState<Tab>(start ?? "summary");
  return (
    <>
      <SettingsNav tab={tab} onTab={setTab} onBack={() => {}} />
      <Settings {...props} tab={tab} onTab={setTab} />
    </>
  );
}

const ready = () => screen.findByRole("tab", { name: /^overview$/i });

const data = async () => {
  await userEvent.click(await screen.findByRole("tab", { name: /^copies$/i }));
};

const notices = async () => {
  await userEvent.click(await screen.findByRole("tab", { name: /^tasks and reminders$/i }));
};

const writing = async () => {
  await userEvent.click(await screen.findByRole("tab", { name: /^documents and signature$/i }));
};

const storage = async () => {
  await userEvent.click(await screen.findByRole("tab", { name: /^storage$/i }));
};

const security = async () => {
  await userEvent.click(await screen.findByRole("tab", { name: /^security$/i }));
};

const sync = async () => {
  await userEvent.click(await screen.findByRole("tab", { name: /^sync$/i }));
};

const upkeep = async () => {
  await userEvent.click(await screen.findByRole("tab", { name: /^health$/i }));
  await screen.findByRole("button", { name: /^check again$/i });
};

const folders = async () => {
  await userEvent.click(await screen.findByRole("button", { name: /cloud through a folder/i }));
};

const turnedOn = async () => {
  await userEvent.click(screen.getByRole("button", { name: /turn on/i }));
  await folders();
  await userEvent.click(await screen.findByRole("button", { name: /another folder/i }));
  await userEvent.click(await screen.findByRole("button", { name: /save here/i }));
};

describe("the maintenance panel", () => {
  it("offers to turn syncing on when there is no folder", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);

    await sync();

    expect(screen.getByText(/only on this computer/i)).toBeTruthy();
    expect(screen.getByText("no destination")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /sync now/i })).toBeNull();
  });

  it("offers the same list here as on the first run, never a bare file dialog", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

    await userEvent.click(screen.getByRole("button", { name: /turn on/i }));
    await folders();

    expect(await screen.findByRole("button", { name: /google drive/i })).toBeTruthy();
    expect(screen.getByRole("button", { name: /another folder/i })).toBeTruthy();
  });

  it("remembers the folder that was picked", async () => {
    asked.folder = "G:/My Drive/tisty";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

    await turnedOn();

    await waitFor(() => expect(sent("choose_sync").length).toBe(1));
    expect(sent("choose_sync")[0].args.dest).toBe("G:/My Drive/tisty");
  });

  it("keeps backing up on offer once a shared folder holds every machine", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);

    await sync();

    expect(screen.getByText(/leaving copies in/i)).toBeTruthy();
    await data();
    expect(screen.getByRole("button", { name: /create backup/i })).toBeTruthy();
    expect(screen.getByRole("button", { name: /restore/i })).toBeTruthy();
  });

  it("says how much a backup left out when this machine does not hold it all", async () => {
    asked.file = "C:/keep/tisty-backup.zip";
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "back_up"
        ? Promise.resolve({ bytes: 4096, leftOut: 3, leftOutBytes: 3 * 1024 * 1024 })
        : otherwise(cmd, args);
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await data();

    await userEvent.click(screen.getByRole("button", { name: /create backup/i }));

    expect(await screen.findByText(/stayed out of it: 3 ·/i)).toBeTruthy();
    ipc.answer = otherwise;
  });

  it("never restores without asking first", async () => {
    asked.file = "C:/keep/tisty-backup.zip";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await data();

    const careful = screen.getByRole("region", { name: t("delicateZone") });
    expect(within(careful).getByRole("button", { name: t("leaveOpen") })).toBeTruthy();
    await userEvent.click(within(careful).getByRole("button", { name: /restore/i }));

    await waitFor(() => expect(sent("back_up").length + sent("restore").length).toBe(0));
  });

  it("restores once the consequences are read and accepted", async () => {
    asked.file = "C:/keep/tisty-backup.zip";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await data();

    await userEvent.click(screen.getByRole("button", { name: /restore from a file/i }));
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getByText(t("carefulRestoreLead"))).toBeTruthy();
    expect(within(dialog).getByText(t("carefulRestoreTwo"))).toBeTruthy();
    await careful(t("carefulRestoreGo"));

    await waitFor(() => expect(sent("restore").length).toBe(1));
    expect(sent("restore")[0].args.from).toBe("C:/keep/tisty-backup.zip");
  });

  it("says once why everything else went quiet", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) => (cmd === "sync_now" ? new Promise(() => {}) : otherwise(cmd, args));
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));

    expect((await screen.findAllByText(/waiting for «syncing»/i)).length).toBe(1);
    await data();
    expect(screen.getByRole("button", { name: /restore from/i }).hasAttribute("disabled")).toBe(
      true,
    );
  });

  it("draws what cannot be pressed as such, in a colour the palette declares", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "settings"
        ? Promise.resolve({
            quiet: [],
            holds: "everywhere",
            shares: true,
            onlySharedAbove: 50 * 1024 * 1024,
          })
        : cmd === "keep_settings"
          ? new Promise(() => {})
          : otherwise(cmd, args);
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await notices();

    await userEvent.click(await screen.findByRole("switch", { name: /a short tone/i }));
    await storage();

    const where = await screen.findByRole("combobox", { name: /where large attachments/i });
    expect(where.hasAttribute("disabled")).toBe(false);
    await userEvent.selectOptions(where, "mine");
    await waitFor(() => expect(where.hasAttribute("disabled")).toBe(true));
    expect(where.className).toContain("disabled:text-soft");
    expect(where.className).not.toContain("opacity-50");
  });

  it("offers where the big attachments live, and says when the choice is idle", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

    const where = await screen.findByRole("combobox", { name: /where large attachments/i });
    expect((where as HTMLSelectElement).value).toBe("everywhere");

    await userEvent.selectOptions(where, "shared");
    await waitFor(() => expect(sent("keep_settings")).toHaveLength(1));
    const asked = sent("keep_settings")[0].args.settings as { holds: string };
    expect(asked.holds).toBe("shared");
  });

  it("frees the choice the moment a shared folder exists, without leaving Settings", async () => {
    asked.folder = "G:/My Drive/tisty";
    let sharing = false;
    ipc.answer = (
      (was) => (cmd: string, args: Record<string, unknown>) =>
        cmd === "settings"
          ? Promise.resolve({
              quiet: [],
              logsAll: false,
              holds: "everywhere",
              shares: sharing,
              onlySharedAbove: 50 * 1024 * 1024,
            })
          : was(cmd, args)
    )(ipc.answer);
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

    const where = await screen.findByRole("combobox", { name: /where large attachments/i });
    expect(where.hasAttribute("disabled")).toBe(true);

    sharing = true;
    await sync();
    await turnedOn();
    await storage();

    const now = await screen.findByRole("combobox", { name: /where large attachments/i });
    await waitFor(() => expect(now.hasAttribute("disabled")).toBe(false));
  });

  it("frees the disk when told the big ones live in the shared folder, and can be stopped", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

    const where = await screen.findByRole("combobox", { name: /where large attachments/i });
    await userEvent.selectOptions(where, "shared");

    await waitFor(() => expect(sent("free_up")).toHaveLength(1));
    await screen.findByText(/Freeing what the shared folder/i);
    await userEvent.click(await screen.findByRole("button", { name: /^stop$/i }));
    await waitFor(() => expect(sent("stop_freeing")).toHaveLength(1));
  });

  it("with a cloud of its own, offers to free unused attachments instead of where big ones stay", async () => {
    carrying.freesSpace = true;
    try {
      render(
        <Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />,
      );
      await storage();
      expect(screen.queryByRole("combobox", { name: /where large attachments/i })).toBeNull();

      const unused = await screen.findByRole("combobox", { name: /unused for/i });
      await screen.findByText(/set by your cloud provider, not by Tisty/i);
      await screen.findByText(/3 unused/i);

      await userEvent.selectOptions(unused, "60");
      await waitFor(() =>
        expect(sent("unused_attachments").map((one) => one.args.days)).toContain(60),
      );
      await userEvent.click(await screen.findByRole("button", { name: /^free$/i }));
      await waitFor(() => expect(sent("free_space")).toHaveLength(1));
      expect(sent("free_space")[0].args.days).toBe(60);
      await screen.findByText(/Freeing space/i);
    } finally {
      carrying.freesSpace = false;
    }
  });

  it("says when nothing is unused that long, and says to try later when the cloud cannot be asked", async () => {
    carrying.freesSpace = true;
    let unused = 0;
    let refusing = "cloudNotNow";
    ipc.answer = ((was) => (cmd: string, args: Record<string, unknown>) => {
      if (cmd === "unused_attachments") return Promise.resolve({ files: unused, bytes: 0 });
      if (cmd === "free_space") return Promise.reject({ code: refusing });
      return was(cmd, args);
    })(ipc.answer);
    try {
      render(
        <Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />,
      );
      await storage();

      await screen.findByText(/nothing unused that long/i);
      expect(screen.getByRole("button", { name: /^free$/i }).hasAttribute("disabled")).toBe(true);

      unused = 2;
      await userEvent.selectOptions(screen.getByRole("combobox", { name: /unused for/i }), "60");
      await screen.findByText(/2 unused/i);
      await userEvent.click(screen.getByRole("button", { name: /^free$/i }));

      await screen.findByText(/could not be asked first/i);
      expect(screen.queryByText(/Freeing space/i)).toBeNull();

      refusing = "cloudLost";
      await userEvent.click(screen.getByRole("button", { name: /^free$/i }));
      await screen.findByText(/no longer lets Tisty in/i);
      expect(screen.queryByText(/could not be asked first/i)).toBeNull();
    } finally {
      carrying.freesSpace = false;
    }
  });

  it("never offers to free space when a folder is the place", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

    await screen.findByRole("combobox", { name: /where large attachments/i });
    expect(screen.queryByRole("combobox", { name: /unused for/i })).toBeNull();
    expect(sent("unused_attachments")).toHaveLength(0);
  });

  it("with a cloud of its own, still offers a backup but no restore, and says what stays in the cloud", async () => {
    carrying.freesSpace = true;
    asked.file = "C:/keep/tisty-backup.zip";
    try {
      render(
        <Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />,
      );
      await data();

      expect(screen.getByText(t("backupCloudWhat"))).toBeTruthy();
      expect(screen.queryByText(t("backupWhat"))).toBeNull();
      expect(screen.queryByRole("button", { name: /restore from/i })).toBeNull();
      await userEvent.click(screen.getByRole("button", { name: /create backup/i }));
      await waitFor(() => expect(sent("back_up")).toHaveLength(1));
    } finally {
      carrying.freesSpace = false;
    }
  });

  it("keeps where it syncs, the computers and the backup on tabs of their own", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);

    await sync();
    expect(screen.getByText(t("whereItSyncs"))).toBeTruthy();
    expect(screen.queryByText("salvia 07")).toBeNull();
    expect(screen.queryByRole("button", { name: /create backup/i })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: t("seeInSecurity") }));
    expect(await screen.findByText("salvia 07")).toBeTruthy();
    expect(screen.getByRole("tab", { name: /^security$/i }).getAttribute("aria-selected")).toBe(
      "true",
    );

    await data();
    expect(screen.getByRole("button", { name: /create backup/i })).toBeTruthy();
    expect(screen.getByText(t("parcelsTitle"))).toBeTruthy();
    expect(screen.queryByText(t("whereItLives"))).toBeNull();
    expect(screen.queryByText(t("whereItSyncs"))).toBeNull();

    await storage();
    expect(screen.getByText(t("whereItLives"))).toBeTruthy();
  });

  it("lists the computers as soon as the tab opens, without a review first", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);

    await security();

    expect(await screen.findByText("cedro 14")).toBeTruthy();
    expect(sent("checked")).toHaveLength(1);
  });

  it("keeps one list of your computers in Security, saying which ones your identity calls yours", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "identity_shown"
        ? Promise.resolve({
            held: true,
            elsewhere: false,
            fingerprint: "48210 93755 12880 63349",
            suffix: "k7qp",
            confirmed: true,
            alias: "Mario",
            machines: [
              { id: "mac0-0001", name: "cedro 14", here: true, ours: true },
              { id: "win1-0002", name: "salvia 07", here: false, ours: false },
            ],
          })
        : otherwise(cmd, args);
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await security();

    expect(await screen.findByText("48210 93755 12880 63349")).toBeTruthy();
    const page = within(screen.getByRole("main"));
    await waitFor(() => expect(page.getAllByText(t("identityOurs"))).toHaveLength(1));
    expect(page.getAllByText("cedro 14")).toHaveLength(1);
    expect(page.getAllByText("salvia 07")).toHaveLength(1);
    expect(page.queryByRole("button", { name: "Remove a computer" })).toBeNull();
  });

  it("folds the exits nobody takes every day, and keeps moving and changing folder in sight", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);

    await sync();

    const fold = screen.getByText(t("moreOptions")).closest("details");
    expect(fold).toBeTruthy();
    expect(fold?.open).toBe(false);
    const inside = within(fold as HTMLElement);
    expect(inside.getByRole("button", { name: t("syncAgain") })).toBeTruthy();
    expect(inside.queryByRole("button", { name: t("syncOffNow") })).toBeNull();
    const careful = screen.getByRole("region", { name: t("delicateZone") });
    expect(within(careful).getByRole("button", { name: t("syncOffNow") })).toBeTruthy();
    expect(within(careful).getByText(t("syncOffTitle"))).toBeTruthy();
    expect(inside.queryByRole("button", { name: t("changeFolder") })).toBeNull();
    expect(screen.getByRole("button", { name: t("changeFolder") })).toBeTruthy();
  });

  it("opens on the Sync tab when asked to move, and on Copies when asked for it", async () => {
    const { unmount } = render(
      <Keeping
        onPack={() => {}}
        onUnpack={() => {}}
        onChanged={() => {}}
        onDoc={() => {}}
        start="copies"
      />,
    );
    expect(
      (await screen.findByRole("tab", { name: /^copies$/i })).getAttribute("aria-selected"),
    ).toBe("true");
    unmount();

    render(
      <Keeping
        onPack={() => {}}
        onUnpack={() => {}}
        onChanged={() => {}}
        onDoc={() => {}}
        start="sync"
      />,
    );
    expect(
      (await screen.findByRole("tab", { name: /^sync$/i })).getAttribute("aria-selected"),
    ).toBe("true");
  });

  it("fetches the ones that are not here when told to hold them all", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();
    const where = await screen.findByRole("combobox", { name: /where large attachments/i });
    await userEvent.selectOptions(where, "shared");
    await waitFor(() => expect(sent("free_up")).toHaveLength(1));
    const before = sent("sync_now").length;

    await userEvent.selectOptions(where, "everywhere");

    await waitFor(() => expect(sent("sync_now").length).toBeGreaterThan(before));
  });

  it("says which loose files are up in the shared folder", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

    const row = (await screen.findByText("lejos-c3d4.mp4")).closest("li") as HTMLElement;
    expect(within(row).getByText(/in the shared folder/)).toBeTruthy();
  });

  it("names every machine and when each last wrote", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await security();

    expect(await screen.findByText(/mac0-0001/)).toBeTruthy();
    expect(screen.getByText(/win1-0002/)).toBeTruthy();
    expect(within(screen.getByRole("main")).getByText("This computer")).toBeTruthy();
  });

  it("calls every machine something a person can read out loud", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await security();

    expect(await screen.findByText("cedro 14")).toBeTruthy();
    expect(screen.getByText("salvia 07")).toBeTruthy();
  });

  it("never offers to remove the machine you are on", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await security();

    await within(screen.getByRole("main")).findByText("This computer");
    expect(screen.getAllByRole("button", { name: /^remove$/i })).toHaveLength(1);
    expect(screen.getByText(/not removable/i)).toBeTruthy();
  });

  it("names the machine and when it last wrote before removing it", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await security();
    await screen.findByText("salvia 07");

    await userEvent.click(screen.getByRole("button", { name: /^remove$/i }));

    const said = (await screen.findByRole("dialog")).textContent ?? "";
    expect(said).toMatch(/salvia 07/);
    expect(said).toMatch(/last wrote/i);
    expect(said).not.toMatch(/win1-0002/);
    await careful(t("cancel"));
    expect(sent("remove_machine")).toHaveLength(0);
  });

  it("says out loud that a machine has been away, so nothing is judged on stale news", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await security();

    const said = await screen.findByText(/may still be using attachments that look unused/i);

    expect(said.textContent).toMatch(/have not written here in a while/i);
    expect(said.textContent).not.toMatch(/synced/i);
    expect(said.textContent).not.toMatch(/remove/i);
  });

  const ONE = "5c2a9d37e104b6ca88f3d2504e8b71f0a3f19c2d7b40e81a6239ce4f0dab5712";
  const OTHER = "b7102fae93c4d0167a51e8c2403f9bd68e2174ac55d930bf1c6e480729ab35df";

  const theseMachines = (all: Record<string, unknown>[]) => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "checked"
        ? otherwise(cmd, args).then((was) => ({
            ...(was as Record<string, unknown>),
            machines: all,
          }))
        : otherwise(cmd, args);
  };

  const reviewed = async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await security();
  };

  const aMachine = (more: Record<string, unknown>) => ({
    id: "win1-0002",
    called: "salvia 07",
    when: Math.floor(Date.now() / 1000),
    mine: false,
    signs: null,
    confirmed: null,
    confirmedWhen: 0,
    turnedAway: null,
    ...more,
  });

  it("offers to confirm a key nobody has answered for yet", async () => {
    theseMachines([aMachine({ signs: ONE })]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.getByText(/unconfirmed/i)).toBeTruthy();
    expect(screen.getByText(/5c2a9d37/)).toBeTruthy();
    expect(screen.getByRole("button", { name: /^confirm$/i })).toBeTruthy();
  });

  it("sends the key it showed, so one that moved meanwhile is never confirmed blind", async () => {
    theseMachines([aMachine({ signs: ONE })]);
    await reviewed();
    await screen.findByText(/win1-0002/);

    await userEvent.click(screen.getByRole("button", { name: /^confirm$/i }));
    const whole = await screen.findByText(/5c2a9d37\s+e104b6ca/);
    expect(whole).toBeTruthy();
    const dialog = within(screen.getByRole("dialog"));
    await userEvent.click(dialog.getByRole("button", { name: /^confirm$/i }));

    await waitFor(() => {
      const said = ipc.calls.find((one) => one.cmd === "confirm_machine_key");
      expect(said?.args).toEqual({ id: "win1-0002", key: ONE });
    });
  });

  it("never offers to confirm again what somebody already answered for", async () => {
    theseMachines([aMachine({ signs: ONE, confirmed: ONE, confirmedWhen: 1_754_000_000 })]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.queryByRole("button", { name: /^confirm$/i })).toBeNull();
    expect(screen.getByRole("button", { name: /see the code/i })).toBeTruthy();
  });

  it("says a machine signs with another key than the one confirmed, and offers no way to accept it", async () => {
    theseMachines([aMachine({ signs: OTHER, confirmed: ONE, confirmedWhen: 1_754_000_000 })]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.getByText(/not the confirmed one/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /^confirm$/i })).toBeNull();

    await userEvent.click(screen.getByRole("button", { name: /what happened/i }));
    expect(await screen.findByText(/has come in here, and it will not/i)).toBeTruthy();
    expect(screen.getByText(/b7102fae/)).toBeTruthy();
  });

  it("reads this machine's own key out without ever asking it to be confirmed", async () => {
    theseMachines([aMachine({ mine: true, signs: ONE })]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.getByText("its own")).toBeTruthy();
    expect(screen.queryByText(/unconfirmed/i)).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: /see the code/i }));

    expect(await screen.findByText(/read it out when one of them asks/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /^confirm$/i })).toBeNull();
  });

  it("never claims somebody confirmed this machine's own key", async () => {
    theseMachines([aMachine({ mine: true, signs: ONE, confirmed: ONE, confirmedWhen: 1_754_000 })]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.queryByText(/^confirmed /i)).toBeNull();
    expect(screen.getByText("its own")).toBeTruthy();
  });

  it("says a key somebody answered for stands, even where it was never published here", async () => {
    theseMachines([aMachine({ signs: null, confirmed: ONE, confirmedWhen: 1_754_000_000 })]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.queryByText(/has not said what it signs with/i)).toBeNull();
    expect(screen.getByText(/5c2a9d37/)).toBeTruthy();
    expect(screen.getByRole("button", { name: /see the code/i })).toBeTruthy();
  });

  it("never dates a confirmation it has no day for", async () => {
    theseMachines([aMachine({ signs: ONE, confirmed: ONE, confirmedWhen: 0 })]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.queryByText(/1970|Jan 1/)).toBeNull();
    expect(screen.getByText(/^confirmed$/i)).toBeTruthy();
  });

  it("tells its own machine the history being turned away is its own", async () => {
    theseMachines([
      aMachine({ mine: true, signs: OTHER, confirmed: ONE, confirmedWhen: 1_754_000_000 }),
    ]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    await userEvent.click(screen.getByRole("button", { name: /what happened/i }));

    expect(
      await screen.findByText(/its own history is the one the others are turning away/i),
    ).toBeTruthy();
    expect(
      screen.queryByText(/has to join the folder afresh, which gives it a new name/i),
    ).toBeNull();
  });

  it("shows both keys whole when they differ, never two briefs that read alike", async () => {
    const twinA = `5c2a9d37${"a".repeat(48)}0dab5712`;
    const twinB = `5c2a9d37${"b".repeat(48)}0dab5712`;
    theseMachines([aMachine({ signs: twinB, confirmed: twinA, confirmedWhen: 1_754_000_000 })]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    await userEvent.click(screen.getByRole("button", { name: /what happened/i }));

    const dialog = within(await screen.findByRole("dialog"));
    expect(dialog.getByText(/5c2a9d37 aaaaaaaa aaaaaaaa aaaaaaaa/)).toBeTruthy();
    expect(dialog.getByText(/5c2a9d37 bbbbbbbb bbbbbbbb bbbbbbbb/)).toBeTruthy();
  });

  it("says a machine was turned away when the round says so, whatever the two keys agree on", async () => {
    theseMachines([
      aMachine({
        signs: ONE,
        confirmed: ONE,
        confirmedWhen: 1_754_000_000,
        turnedAway: "disowned",
      }),
    ]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.getByText(/not the confirmed one/i)).toBeTruthy();
    expect(screen.queryByText(/^confirmed /i)).toBeNull();
    expect(screen.getByRole("button", { name: /what happened/i })).toBeTruthy();
  });

  it("stops warning once the round takes that history in again", async () => {
    theseMachines([
      aMachine({ signs: ONE, confirmed: ONE, confirmedWhen: 1_754_000_000, turnedAway: null }),
    ]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.queryByText(/not the confirmed one/i)).toBeNull();
    expect(screen.getByText(/^confirmed /i)).toBeTruthy();
  });

  it("shows a machine nobody answered for as waiting, and offers to confirm it", async () => {
    theseMachines([aMachine({ signs: ONE, turnedAway: "unconfirmed" })]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.getByText(/^waiting$/i)).toBeTruthy();
    expect(screen.getByText(/nothing it writes comes in until you confirm it/i)).toBeTruthy();
    expect(screen.getByText(/5c2a9d37/)).toBeTruthy();
    expect(screen.getByRole("button", { name: /^confirm$/i })).toBeTruthy();
    expect(screen.queryByText(/not the confirmed one/i)).toBeNull();
  });

  it("offers no confirm it could not carry out when the folder cannot say what it signs with", async () => {
    theseMachines([aMachine({ signs: null, turnedAway: "unconfirmed" })]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.getByText(/^waiting$/i)).toBeTruthy();
    expect(screen.getByText(/has not said what it signs with/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /^confirm$/i })).toBeNull();
  });

  it("stops saying a machine waits once the round lets it in", async () => {
    theseMachines([
      aMachine({ signs: ONE, confirmed: ONE, confirmedWhen: 1_754_000_000, turnedAway: null }),
    ]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.queryByText(/^waiting$/i)).toBeNull();
    expect(screen.getByText(/^confirmed /i)).toBeTruthy();
  });

  it("looks again when a confirm is refused, so the stale key never stays on the row", async () => {
    theseMachines([aMachine({ signs: ONE })]);
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "confirm_machine_key" ? Promise.reject({ code: "keyMoved" }) : otherwise(cmd, args);
    await reviewed();
    await screen.findByText(/win1-0002/);

    await userEvent.click(screen.getByRole("button", { name: /^confirm$/i }));
    const dialog = within(screen.getByRole("dialog"));
    await userEvent.click(dialog.getByRole("button", { name: /^confirm$/i }));

    expect(await screen.findByText(/publishes another key now/i)).toBeTruthy();
    await waitFor(() => {
      expect(ipc.calls.filter((one) => one.cmd === "checked").length).toBeGreaterThan(1);
    });
  });

  it("says nothing about a key for a machine that never published one", async () => {
    theseMachines([aMachine({})]);
    await reviewed();

    await screen.findByText(/win1-0002/);
    expect(screen.getByText(/has not said what it signs with/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /^confirm$/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /see the code/i })).toBeNull();
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
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await security();

    await screen.findByText(/mac0-0001/);
    expect(screen.queryByText(/have not synced in a while/i)).toBeNull();
  });

  it("never lets go of an attachment without being told twice", async () => {
    asked.sure = false;
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();
    await screen.findByText("charla-a3f9.mp4");

    await userEvent.click(screen.getAllByRole("button", { name: /^move to the bin$/i })[0]);

    await waitFor(() => expect(sent("retire_attachment")).toHaveLength(0));
  });

  it("lets go of the one it was pointed at, and looks again afterwards", async () => {
    asked.sure = true;
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();
    await screen.findByText("charla-a3f9.mp4");
    const looks = sent("checked").length;

    await userEvent.click(screen.getAllByRole("button", { name: /^move to the bin$/i })[0]);

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
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();
    await screen.findByText("charla-a3f9.mp4");

    await userEvent.click(screen.getAllByRole("button", { name: /^move to the bin$/i })[0]);

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
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();
    await screen.findByText("charla-a3f9.mp4");

    await userEvent.click(screen.getAllByRole("button", { name: /^move to the bin$/i })[0]);

    await waitFor(() =>
      expect(screen.getAllByText(/references that attachment now/i).length).toBeGreaterThan(0),
    );
  });

  it("offers to remove another machine, never this one", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await security();

    await screen.findByText("win1-0002");
    expect(screen.getAllByRole("button", { name: /^remove$/i })).toHaveLength(1);
  });

  it("never removes a machine without being told twice", async () => {
    asked.sure = false;
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await security();
    await screen.findByText("win1-0002");

    await userEvent.click(screen.getByRole("button", { name: /^remove$/i }));
    await careful(t("cancel"));

    await waitFor(() => expect(sent("remove_machine")).toHaveLength(0));
  });

  it("removes the machine it was pointed at, and looks again afterwards", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await security();
    await screen.findByText("win1-0002");
    const looks = sent("checked").length;

    await userEvent.click(screen.getByRole("button", { name: /^remove$/i }));
    await careful(t("carefulDropGo"));

    await waitFor(() => expect(sent("remove_machine")).toHaveLength(1));
    expect(sent("remove_machine")[0].args.id).toBe("win1-0002");
    await waitFor(() => expect(sent("checked").length).toBeGreaterThan(looks));
  });

  it("tells you to settle the machines before judging what is left over", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

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
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

    await screen.findByText("charla-a3f9.mp4");
    expect(screen.queryByText(/before deciding what is left over/i)).toBeNull();
  });

  it("shows each loose attachment by name, weight and date", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

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
              stays: "attachments/charla-a3f9.mp4",
              copies: [
                { at: "attachments/charla-a3f9.mp4", links: [] },
                {
                  at: "attachments/video-a3f9.mp4",
                  links: [{ kind: "doc", id: "a3f1-0001", title: "Demo" }],
                },
              ],
            },
          ])
        : otherwise(cmd, args);
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

    expect(sent("twinned")).toHaveLength(0);

    await userEvent.click(screen.getByRole("button", { name: /^see which$/i }));

    expect(await screen.findByText("video-a3f9.mp4")).toBeTruthy();
    expect(screen.getByText(/proposes which copy stays/i)).toBeTruthy();
    expect(screen.getByText("Stays")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Demo" })).toBeTruthy();
  });

  it("joins the copies into the one chosen to stay and says what was freed", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) => {
      if (cmd === "twinned")
        return Promise.resolve([
          {
            bytes: 300_000,
            stays: "attachments/charla-a3f9.mp4",
            copies: [
              { at: "attachments/charla-a3f9.mp4", links: [] },
              { at: "attachments/video-a3f9.mp4", links: [] },
            ],
          },
        ]);
      if (cmd === "twins_join") return Promise.resolve({ freed: 300_000, relinked: 2, left: [] });
      return otherwise(cmd, args);
    };
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();
    await userEvent.click(screen.getByRole("button", { name: /^see which$/i }));

    await userEvent.click(await screen.findByRole("button", { name: "Keep this one" }));
    await userEvent.click(screen.getByRole("button", { name: "Join" }));

    expect(sent("twins_join")[0].args).toEqual({
      keep: "attachments/video-a3f9.mp4",
      go: ["attachments/charla-a3f9.mp4"],
    });
    expect(await screen.findByText(/2 links changed/)).toBeTruthy();
  });

  it("says nothing is kept twice when nothing is", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) => (cmd === "twinned" ? Promise.resolve([]) : otherwise(cmd, args));
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

    await userEvent.click(screen.getByRole("button", { name: /^see which$/i }));

    expect(await screen.findByText(/nothing is kept twice/i)).toBeTruthy();
  });

  it("says plainly where taken-out files go, the shared folder's too", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

    expect(await screen.findByText(/your cloud keeps it in its own bin/i)).toBeTruthy();
  });

  it("breaks the weight down, so the size has somewhere to come from", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();

    expect(await screen.findByText(/the history weighs/i)).toBeTruthy();
    expect(screen.getByText(/the documents weigh/i)).toBeTruthy();
    expect(screen.getByText(/the attachments weigh/i)).toBeTruthy();
  });

  it("asks after days of hearing nothing, and blames neither side", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    carrying.heard = new Date(Date.now() - 5 * 86_400_000).toISOString();
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

    const title = await screen.findByText(t("attSyncQuiet"));
    const said = title.closest("[data-note]") as HTMLElement;

    expect(said.closest("[data-setting]")?.getAttribute("data-setting")).toBe("syncing");
    expect(said.textContent).toMatch(/may be off/i);
    expect(said.textContent).toMatch(/may not be running/i);
  });

  it("stays quiet while the other machines are still turning up", async () => {
    carrying.chosen = "G:/My Drive/tisty";
    carrying.heard = new Date(Date.now() - 3_600_000).toISOString();
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

    expect(screen.queryByText(/without anything arriving/i)).toBeNull();
  });

  it("says the work reached the folder, and never that a cloud took it", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.resolve({ carried: "sent", undecided: [], unreadable: [] })
        : otherwise(cmd, args);
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

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
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

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
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));

    await waitFor(() => expect(sent("settle_paper")).toHaveLength(1));
    expect(sent("settle_paper")[0].args.keep).toBe("mine");
    expect(asked).toBe(2);
  });

  it("asks nothing at all when nothing is at odds", async () => {
    const dialog = await import("@tauri-apps/plugin-dialog");
    const spy = vi.spyOn(dialog, "ask");
    carrying.chosen = "G:/My Drive/tisty";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

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
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();
    await userEvent.click(screen.getByRole("button", { name: /sync now/i }));
  };

  it("carries as soon as a folder is picked, so the doors open there and not later", async () => {
    asked.folder = "G:/My Drive/tisty";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

    await turnedOn();

    await waitFor(() => expect(sent("choose_sync")).toHaveLength(1));
    await waitFor(() => expect(sent("sync_now")).toHaveLength(1));
  });

  it("says a computer that only had what Tisty starts with took what the folder holds", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.resolve({ carried: "came", undecided: [], adopted: true })
        : otherwise(cmd, args);
    await carried();

    expect(await screen.findByText(fill("syncAdopted", t("syncTheFolder")))).toBeTruthy();
    expect(screen.queryByText(/already holds another tisty/i)).toBeNull();
  });

  it("says the card waits on the person while the doors are open, not that a round runs", async () => {
    apart();
    await carried();

    await screen.findByText(/already holds another tisty/i);
    expect(screen.getByText(t("syncChoosing"))).toBeTruthy();
    expect(screen.queryByText(/my last round/i)).toBeNull();
    expect(screen.queryByText(t("syncBusy"))).toBeNull();
  });

  it("opens the doors on picking a folder that already holds another history", async () => {
    apart();
    asked.folder = "G:/My Drive/tisty";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

    await turnedOn();

    expect((await screen.findByRole("dialog")).textContent).toMatch(/already holds another Tisty/i);
  });

  it("does not leave you pointing at a folder you walked away from", async () => {
    apart();
    asked.folder = "G:/My Drive/tisty";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();
    await turnedOn();

    const doors = await screen.findByRole("dialog");
    await userEvent.click(within(doors).getByRole("button", { name: /^close$/i }));

    await waitFor(() => expect(sent("choose_sync")).toHaveLength(2));
    expect(sent("choose_sync")[1].args.dest).toBeUndefined();
  });

  it("keeps the folder you did accept, and does not undo it", async () => {
    onceApart();
    asked.folder = "G:/My Drive/tisty";
    asked.file = "C:/keep/tisty-folder-before.zip";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();
    await turnedOn();

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
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await sync();

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
    expect(sent("sync_now")).toHaveLength(3);
    expect(sent("tidy_merged")).toHaveLength(1);
  });

  it("says what merging costs before it is taken, not after", async () => {
    apart();
    await carried();

    const said = (await screen.findByRole("dialog")).textContent ?? "";
    expect(said).toMatch(/without losing anything/i);
    expect(said).toMatch(/two lists by the same name become one/i);
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

  it("says what the review found", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await upkeep();
    await storage();

    expect(await screen.findByText(/7 tasks · 2 lists/i)).toBeTruthy();
    expect(screen.getByText(/you can free up to 311.0 kB/i)).toBeTruthy();
  });

  it("reads its settings again once the welcome has been through", async () => {
    const { rerender } = render(
      <Keeping
        onPack={() => {}}
        onUnpack={() => {}}
        greeted={0}
        onChanged={() => {}}
        onDoc={() => {}}
      />,
    );
    await sync();
    const before = sent("settings").length;

    rerender(
      <Keeping
        onPack={() => {}}
        onUnpack={() => {}}
        greeted={1}
        onChanged={() => {}}
        onDoc={() => {}}
      />,
    );

    await waitFor(() => expect(sent("settings").length).toBe(before + 1));
  });

  it("holds a language of its own when one is picked", async () => {
    render(
      <Keeping
        start="window"
        onPack={() => {}}
        onUnpack={() => {}}
        onChanged={() => {}}
        onDoc={() => {}}
      />,
    );
    await ready();

    await userEvent.selectOptions(await screen.findByLabelText(/^language$/i), "es");

    await waitFor(() => expect(sent("keep_locale").length).toBe(1));
    expect(sent("keep_locale")[0].args.locale).toBe("es");
  });

  it("shows the close choice the person once remembered, and lets it be asked again", async () => {
    render(
      <Keeping
        start="window"
        onPack={() => {}}
        onUnpack={() => {}}
        onChanged={() => {}}
        onDoc={() => {}}
      />,
    );
    await ready();
    const choice = (await screen.findByLabelText(
      /^when you close the window$/i,
    )) as HTMLSelectElement;
    expect(choice.value).toBe("quit");

    await userEvent.selectOptions(choice, "ask");

    await waitFor(() => expect(sent("keep_closing").length).toBe(1));
    expect(sent("keep_closing")[0].args.how).toBe("ask");
  });

  it("holds a look of its own, and lets it go back to the computer's", async () => {
    render(
      <Keeping
        start="window"
        onPack={() => {}}
        onUnpack={() => {}}
        onChanged={() => {}}
        onDoc={() => {}}
      />,
    );
    await ready();
    const look = await screen.findByLabelText<HTMLSelectElement>(/^look$/i);
    expect(look.value).toBe("");

    await userEvent.selectOptions(look, "dark");
    await waitFor(() => expect(sent("keep_theme").length).toBe(1));
    expect(sent("keep_theme")[0].args.theme).toBe("dark");

    await userEvent.selectOptions(look, "");
    await waitFor(() => expect(sent("keep_theme").length).toBe(2));
    expect(sent("keep_theme")[1].args.theme).toBeUndefined();
  });
});

describe("the first-run assistant", () => {
  const spoken = async () => {
    await userEvent.click(await screen.findByRole("button", { name: /^english$/i }));
    await folders();
  };

  const alone = async () => {
    await userEvent.click(await screen.findByRole("button", { name: /^back$/i }));
    await userEvent.click(await screen.findByRole("button", { name: /only on this machine/i }));
    await userEvent.click(await screen.findByRole("button", { name: t("welcomeNotNow") }));
  };

  it("asks for the language before anything else, and keeps it", async () => {
    render(<Welcome onDone={vi.fn()} />);

    await spoken();

    expect(sent("keep_locale")[0].args.locale).toBe("en");
    expect(await screen.findByRole("button", { name: /google drive/i })).toBeTruthy();
  });

  it("offers what it found and turns away what it did not", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    const drive = await screen.findByRole("button", { name: /google drive/i });

    expect(drive.hasAttribute("disabled")).toBe(false);
    expect(screen.getByRole("button", { name: /dropbox/i }).hasAttribute("disabled")).toBe(true);
  });

  it("takes «only here» as an answer, not as a blank", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await alone();

    await waitFor(() => expect(sent("choose_sync").length).toBe(1));
    expect(sent("choose_sync")[0].args.dest).toBeUndefined();
  });

  it("keeps a folder of ours inside the one the provider holds", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    expect(await screen.findByText("G:/My Drive/Tisty")).toBeTruthy();
    expect(screen.getByText(/has to be open and up to date/i)).toBeTruthy();
  });

  it("keeps one way back, never two at once", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    expect(screen.getAllByRole("button", { name: /^back$/i })).toHaveLength(1);
  });

  it("makes that folder before writing it down", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    await waitFor(() => expect(sent("choose_sync").length).toBe(1));
    expect(sent("make_room")[0].args.at).toBe("G:/My Drive/Tisty");
    expect(sent("choose_sync")[0].args.dest).toBe("G:/My Drive/Tisty");
  });

  it("brings the folder home before it plants anything of its own", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));
    await userEvent.click(await screen.findByRole("button", { name: t("welcomeNotNow") }));

    await waitFor(() => expect(sent("guide")).toHaveLength(1));
    expect(
      ipc.calls
        .map((one) => one.cmd)
        .filter((cmd) => ["sync_now", "sow_lists", "guide"].includes(cmd)),
    ).toEqual(["sync_now", "sow_lists", "guide"]);
  });

  it("asks for the folders it knows once, even when the asking fails", async () => {
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "keepers" ? Promise.reject({ code: "cannotRead" }) : answered(cmd, args);
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await waitFor(() => expect(sent("keepers")).toHaveLength(1));
    await new Promise((settle) => setTimeout(settle, 50));

    expect(sent("keepers")).toHaveLength(1);
  });

  it("asks about the meeting place itself, not a folder below it", async () => {
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "strays_at" ? Promise.resolve({ adrift: 0, unreadable: false }) : answered(cmd, args);
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    await waitFor(() => expect(sent("strays_at").length).toBe(1));
    expect(sent("strays_at")[0].args.at).toBe("G:/My Drive/Tisty");
    expect(sent("make_room")).toHaveLength(0);
  });

  it("says a folder whose history it could not read at all, not that it is empty", async () => {
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "strays_at" ? Promise.resolve({ adrift: 0, unreadable: true }) : answered(cmd, args);
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    const said = await screen.findByRole("alert");
    expect(said.textContent).toMatch(/history could not be read/i);
  });

  it("says when the folder is holding documents no history accounts for", async () => {
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "strays_at"
        ? Promise.resolve({ adrift: 239, unreadable: false })
        : answered(cmd, args);
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    expect(await screen.findByText(fill("keepersStrays", "239"))).toBeTruthy();
    expect(screen.getByText(t("keepersStraysWhy"))).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("stays quiet about strays when the folder accounts for everything", async () => {
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "strays_at" ? Promise.resolve({ adrift: 0, unreadable: false }) : answered(cmd, args);
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    await screen.findByRole("button", { name: /save here/i });
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("sows nothing on top of the history a new computer took without asking", async () => {
    const done = vi.fn();
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.resolve({ carried: "came", undecided: [], adopted: true })
        : answered(cmd, args);
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));
    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    await userEvent.click(await screen.findByRole("button", { name: t("welcomeNotNow") }));

    await waitFor(() => expect(done).toHaveBeenCalled());
    expect(screen.queryByText(/already holds another tisty/i)).toBeNull();
    expect(sent("sow_lists")).toHaveLength(0);
    expect(sent("guide")).toHaveLength(0);
  });

  it("says a carry that would not finish instead of going quiet", async () => {
    const done = vi.fn();
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.reject({ code: "noMeetingPlace", name: "G:/My Drive/Tisty" })
        : answered(cmd, args);
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    await screen.findByRole("alert");
    expect(done).not.toHaveBeenCalled();
    expect(sent("guide")).toHaveLength(0);
  });

  it("lets you in anyway once it has said what went wrong", async () => {
    const done = vi.fn();
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.reject({ code: "noMeetingPlace", name: "G:/My Drive/Tisty" })
        : answered(cmd, args);
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));
    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));
    await screen.findByRole("alert");

    await userEvent.click(screen.getByRole("button", { name: /go in anyway/i }));
    await userEvent.click(await screen.findByRole("button", { name: t("welcomeNotNow") }));

    await waitFor(() => expect(done).toHaveBeenCalled());
  });

  it("asks about two histories where the trouble turned up, not in Settings", async () => {
    const done = vi.fn();
    const answered = ipc.answer;
    ipc.answer = (cmd, args) => {
      if (cmd === "sync_now") return Promise.reject({ code: "otherStore", name: "01ABC" });
      if (cmd === "sync_kin") return Promise.resolve("strangers");
      return answered(cmd, args);
    };
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    await screen.findByText(/already holds another tisty/i);
    expect(done).not.toHaveBeenCalled();
  });

  it("says what it is doing while the first round runs", async () => {
    let letGo: () => void = () => {};
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? new Promise((keep) => {
            letGo = () => keep({ carried: "came", undecided: [] });
          })
        : answered(cmd, args);
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    await screen.findByRole("status");
    letGo();
  });

  const underway = () => act(() => heard({ stage: "log", done: 0, whole: 2 }));

  const halfway = async () => {
    const answered = ipc.answer;
    ipc.answer = (cmd, args) => (cmd === "sync_now" ? new Promise(() => {}) : answered(cmd, args));
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));
    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));
    await screen.findByRole("status");
  };

  it("counts what comes while the round brings, and lets the person go in meanwhile", async () => {
    const done = vi.fn();
    render(<Welcome onDone={done} />);
    await halfway();
    expect(screen.getByText(fill("welcomeBringing", t("syncTheFolder")))).toBeTruthy();

    await underway();

    expect(await screen.findByText(`${t("comingLog")} ${fill("comingOf", "0", "2")}`)).toBeTruthy();
    expect(screen.queryByRole("textbox", { name: /^alias$/i })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: t("welcomeKeepWorking") }));
    await waitFor(() => expect(done).toHaveBeenCalledTimes(1));
    expect(screen.queryByRole("textbox", { name: /^alias$/i })).toBeNull();
    act(() => heard(null));
  });

  const hiddenThen = async (ends: unknown) => {
    let letGo: () => void = () => {};
    let over = false;
    const answered = ipc.answer;
    ipc.answer = (cmd, args) => {
      if (cmd !== "sync_now") return answered(cmd, args);
      if (over) return Promise.resolve(ends);
      return new Promise((keep) => {
        letGo = () => {
          over = true;
          keep(ends);
        };
      });
    };
    const done = vi.fn();
    render(
      <>
        <Welcome onDone={done} />
        <Coming />
      </>,
    );
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));
    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));
    await userEvent.click(await screen.findByRole("button", { name: t("welcomeKeepWorking") }));
    await waitFor(() => expect(done).toHaveBeenCalledTimes(1));
    letGo();
    return done;
  };

  it("tells on the bottom line how a round the person went in on ended, never pulling them back", async () => {
    const done = await hiddenThen({
      carried: "same",
      undecided: [],
      deferred: { reason: "offline", left: 0, names: [], cause: "filtered", rounds: 1 },
    });

    expect(await screen.findByText(t("deferredOffline"))).toBeTruthy();
    expect(screen.queryByText(t("roundFiltered")), "a single blip named a filter").toBeNull();
    expect(done).toHaveBeenCalledTimes(1);
    expect(screen.queryByText(t("welcomeCarryStuck"))).toBeNull();
    act(() => toldAside(null));
  });

  it("tells a round turned away after the person went in, too", async () => {
    await hiddenThen({ carried: "busy", undecided: [] });

    expect(await screen.findByText(t("syncBusy"), {}, { timeout: 4000 })).toBeTruthy();
    act(() => toldAside(null));
  });

  it("lets the person in on a first blip and keeps trying, without a cause it cannot be sure of", async () => {
    const done = vi.fn();
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.resolve({
            carried: "same",
            undecided: [],
            deferred: { reason: "offline", left: 0, names: [], cause: "filtered", rounds: 1 },
          })
        : answered(cmd, args);
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    expect(await screen.findByRole("textbox", { name: /^alias$/i })).toBeTruthy();
    expect(screen.queryByText(t("roundFiltered"))).toBeNull();
    expect(screen.queryByRole("button", { name: t("welcomeAnyway") })).toBeNull();
  });

  it("says what stands in the way once the first round keeps failing the same way", async () => {
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.resolve({
            carried: "same",
            undecided: [],
            deferred: { reason: "offline", left: 0, names: [], cause: "filtered", rounds: 2 },
          })
        : answered(cmd, args);
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    expect(await screen.findByText(t("roundFiltered"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("welcomeAnyway") })).toBeTruthy();
  });

  it("decides from what the round brought when the look before it failed", async () => {
    Object.assign(arriving, { holds: true, alias: "rgdevment" });
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "place_peek" ? Promise.reject({ code: "cloudNotNow" }) : answered(cmd, args);
    const done = vi.fn();
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    await waitFor(() => expect(done).toHaveBeenCalled());
    expect(sent("sow_lists"), "lists were sown over the history it brought").toHaveLength(0);
    expect(sent("guide"), "a second guide over the history it brought").toHaveLength(0);
    expect(sent("sign")).toHaveLength(0);
  });

  it("sows a place found empty only once the round says so, after the person went in", async () => {
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "place_peek" ? Promise.reject({ code: "cloudNotNow" }) : answered(cmd, args);
    await hiddenThen({ carried: "sent", undecided: [] });

    await waitFor(() => expect(sent("sow_lists")).toHaveLength(1));
    expect(sent("guide"), "a guide opened under what the person is doing").toHaveLength(0);
  });

  const olderThanTheFolder = async () => {
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.reject({ code: "syncNewer", name: "dev_b" })
        : answered(cmd, args);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));
    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));
    await screen.findByText(t("welcomeOlder"));
  };

  it("offers to stay on this machine when the folder runs a newer Tisty, not to go in anyway", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await olderThanTheFolder();

    expect(screen.queryByRole("button", { name: t("welcomeAnyway") })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: t("welcomeStayHere") }));

    await waitFor(() => expect(sent("choose_sync")).toHaveLength(2));
    expect(sent("choose_sync")[1].args.dest).toBeUndefined();
    expect(await screen.findByRole("textbox", { name: /^alias$/i })).toBeTruthy();
  });

  it("names the machine that is ahead", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await olderThanTheFolder();

    expect(screen.getByText(fill("welcomeOlderBy", "dev_b"))).toBeTruthy();
  });

  it("forgets the history it meant to join once it stays on this machine instead", async () => {
    Object.assign(arriving, { holds: true, alias: "rgdevment" });
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "sync_now"
        ? Promise.reject({ code: "syncNewer", name: "dev_b" })
        : answered(cmd, args);
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));
    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));
    await screen.findByText(t("welcomeOlder"));

    await userEvent.click(screen.getByRole("button", { name: t("welcomeStayHere") }));

    expect(await screen.findByRole("textbox", { name: /^alias$/i })).toBeTruthy();
  });

  it("says why staying on this machine did not go through", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await olderThanTheFolder();
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "choose_sync" && args.dest === undefined
        ? Promise.reject({ code: "sharedAwayToLeave", name: "G:/x" })
        : answered(cmd, args);

    await userEvent.click(screen.getByRole("button", { name: t("welcomeStayHere") }));

    expect(await screen.findByText(fill("sharedAwayToLeave", "G:/x"))).toBeTruthy();
  });

  const otherHistory = () => {
    const answered = ipc.answer;
    ipc.answer = (cmd, args) => {
      if (cmd === "sync_now" && sent("sync_now").length === 1)
        return Promise.reject({ code: "otherStore", name: "01ABC" });
      if (cmd === "sync_kin") return Promise.resolve("strangers");
      return answered(cmd, args);
    };
  };

  it("says nothing moved when the other folder is not picked after all", async () => {
    asked.folder = null;
    otherHistory();
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));
    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    await userEvent.click(await screen.findByRole("button", { name: t("apartElse") }));

    expect(await screen.findByText(t("wouldReset"))).toBeTruthy();
  });

  it("meets a newer folder picked from the doors the same way", async () => {
    asked.folder = "D:/Otra";
    otherHistory();
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "choose_sync" && args.dest === "D:/Otra"
        ? Promise.reject({ code: "syncNewer", name: "dev_b" })
        : answered(cmd, args);
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));
    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    await userEvent.click(await screen.findByRole("button", { name: t("apartElse") }));

    expect(await screen.findByText(t("welcomeOlder"))).toBeTruthy();
  });

  it("says so when there is no newer Tisty to update to", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await olderThanTheFolder();
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "update_ready" ? Promise.resolve(null) : answered(cmd, args);

    await userEvent.click(screen.getByRole("button", { name: t("updateInstall") }));

    expect(await screen.findByText(t("lookNowNone"))).toBeTruthy();
    expect(sent("update_install")).toHaveLength(0);
  });

  it("updates from where it stands when this copy can update itself", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await olderThanTheFolder();
    const answered = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "update_ready"
        ? Promise.resolve({ version: "1.24.0", route: "download", installs: true })
        : answered(cmd, args);

    await userEvent.click(screen.getByRole("button", { name: t("updateInstall") }));

    await waitFor(() => expect(sent("update_install")).toHaveLength(1));
  });

  it("goes back to the folders when another one is wanted instead", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await olderThanTheFolder();

    await userEvent.click(screen.getByRole("button", { name: t("welcomeOtherFolder") }));

    expect(await screen.findByRole("button", { name: /google drive/i })).toBeTruthy();
  });

  it("takes the name the folder already signs with, and does not ask for one", async () => {
    Object.assign(arriving, { holds: true, alias: "rgdevment" });
    const done = vi.fn();
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));
    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    await waitFor(() => expect(done).toHaveBeenCalled());
    expect(sent("sync_now"), "a machine that never wrote anything was asked first").toHaveLength(1);
    expect(sent("sign")).toHaveLength(0);
    expect(screen.queryByRole("textbox", { name: /^alias$/i })).toBeNull();
    expect(sent("sow_lists"), "the history it joins already has its lists").toHaveLength(0);
    expect(sent("guide"), "nor a second guide").toHaveLength(0);
  });

  it("still asks for a name when the folder it joins never had one", async () => {
    Object.assign(arriving, { holds: true, alias: null });
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));
    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    expect(await screen.findByRole("textbox", { name: /^alias$/i })).toBeTruthy();
  });

  it("looks again at what it holds when another folder is picked from the doors", async () => {
    asked.folder = "D:/Otra";
    otherHistory();
    const done = vi.fn();
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));
    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));
    Object.assign(arriving, { holds: true, alias: "rgdevment" });

    await userEvent.click(await screen.findByRole("button", { name: t("apartElse") }));

    await waitFor(() => expect(done).toHaveBeenCalled());
    expect(sent("place_peek")).toHaveLength(2);
    expect(screen.queryByRole("textbox", { name: /^alias$/i })).toBeNull();
  });

  it("waits its turn instead of walking in with nothing when a round is already running", async () => {
    let turns = 0;
    const answered = ipc.answer;
    ipc.answer = (cmd, args) => {
      if (cmd !== "sync_now") return answered(cmd, args);
      turns += 1;
      return Promise.resolve({
        carried: turns === 1 ? "busy" : "came",
        undecided: [],
      });
    };
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    await screen.findByRole("textbox", { name: /^alias$/i }, { timeout: 3000 });
    expect(turns).toBeGreaterThan(1);
  });

  it("has nowhere to bring anything home from when you stay on this machine", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await alone();

    await waitFor(() => expect(sent("guide")).toHaveLength(1));
    expect(sent("sync_now")).toHaveLength(0);
    expect(sent("sow_lists")).toHaveLength(1);
  });

  it("says as much when nobody is known to keep a folder in step", async () => {
    asked.folder = "D:/Tasks";
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await userEvent.click(await screen.findByRole("button", { name: /another folder/i }));

    expect(await screen.findByText(/does not recognise/i)).toBeTruthy();
  });

  it("hangs a folder of ours inside the one you pointed at", async () => {
    asked.folder = "D:/Tasks";
    holders.told = { keeper: "plain", into: "D:/Tasks/Tisty" };
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /another folder/i }));

    await userEvent.click(await screen.findByRole("button", { name: /save here/i }));

    await waitFor(() => expect(sent("choose_sync")).toHaveLength(1));
    expect(sent("make_room")[0].args.at).toBe("D:/Tasks/Tisty");
    expect(sent("choose_sync")[0].args.dest).toBe("D:/Tasks/Tisty");
  });

  it("names the provider of a folder it does recognise", async () => {
    asked.folder = "G:/My Drive/Tasks";
    holders.told = { keeper: "cloud", named: "Google Drive" };
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await userEvent.click(await screen.findByRole("button", { name: /another folder/i }));

    expect(await screen.findByText(/google drive has to be open/i)).toBeTruthy();
  });

  it("turns on start-up and the tray without asking", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();

    await alone();

    await waitFor(() => expect(sent("wake_for").length).toBe(1));
    expect(sent("wake_for")[0].args.wanted).toBe(true);
    expect(sent("keep_closing")[0].args.how).toBe("hide");
  });

  it("ends on the guide, and hands back the document it planted", async () => {
    const done = vi.fn();
    render(<Welcome onDone={done} />);
    await spoken();

    await alone();

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

    await alone();

    await waitFor(() => expect(done).toHaveBeenCalled());
    expect(done.mock.calls[0][0]).toBeUndefined();
  });

  it("asks where things go before it asks who you are, and writes the name down", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await userEvent.click(await screen.findByRole("button", { name: /^english$/i }));

    expect(await screen.findByRole("button", { name: /only on this machine/i })).toBeTruthy();
    expect(screen.queryByRole("textbox", { name: /^alias$/i })).toBeNull();

    await userEvent.click(screen.getByRole("button", { name: /only on this machine/i }));
    const field = await screen.findByRole("textbox", { name: /^alias$/i });
    await userEvent.type(field, "rgdevment");
    await userEvent.click(screen.getByRole("button", { name: t("welcomeSigned") }));

    await waitFor(() => expect(sent("sign")).toHaveLength(1));
    expect(sent("sign")[0].args.alias).toBe("rgdevment");
  });

  it("lets the alias wait, and writes nothing down when it is skipped", async () => {
    const done = vi.fn();
    render(<Welcome onDone={done} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /^back$/i }));
    await userEvent.click(await screen.findByRole("button", { name: /only on this machine/i }));
    await screen.findByRole("textbox", { name: /^alias$/i });

    await userEvent.click(screen.getByRole("button", { name: t("welcomeNotNow") }));

    await waitFor(() => expect(done).toHaveBeenCalled());
    expect(sent("sign")).toHaveLength(0);
  });

  it("goes back, and shows what was already chosen", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await screen.findByRole("button", { name: /google drive/i });

    await userEvent.click(screen.getByRole("button", { name: /^back$/i }));
    await userEvent.click(await screen.findByRole("button", { name: /^back$/i }));

    const english = await screen.findByRole("button", { name: /^english$/i });
    expect(english.getAttribute("aria-pressed")).toBe("true");
  });

  it("takes a change of mind about the language", async () => {
    render(<Welcome onDone={vi.fn()} />);
    await spoken();
    await userEvent.click(await screen.findByRole("button", { name: /^back$/i }));
    await userEvent.click(await screen.findByRole("button", { name: /^back$/i }));

    await userEvent.click(await screen.findByRole("button", { name: /^español$/i }));

    expect(sent("keep_locale").map((one) => one.args.locale)).toEqual(["en", "es"]);
  });

  it("keeps the alias the person signs with, and shows it again on the next look", async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await writing();

    const field = await screen.findByRole("textbox", { name: /^alias$/i });
    expect((field as HTMLInputElement).value).toBe("");
    await userEvent.type(field, "  rgdevment  ");
    fireEvent.blur(field);

    await waitFor(() => expect(sent("sign")).toHaveLength(1));
    expect(sent("sign")[0].args.alias).toBe("rgdevment");
    await waitFor(() => expect((field as HTMLInputElement).value).toBe("rgdevment"));
  });

  it("offers the aliases this store signed with before, and not the one in use", async () => {
    signing.alias = "rgdevment";
    signing.before = ["rgdevment", "mario"];
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await writing();

    const field = await screen.findByRole("combobox", { name: /^alias$/i });
    expect((field as HTMLInputElement).value).toBe("rgdevment");
    const offered = Array.from(document.querySelectorAll("#signed-before option")).map(
      (one) => (one as HTMLOptionElement).value,
    );
    expect(offered).toEqual(["mario"]);
  });

  it("asks in plain words when the alias changes, and only then signs the older ones", async () => {
    signing.alias = null;
    signing.mine = 243;
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await writing();

    const field = await screen.findByRole("textbox", { name: /^alias$/i });
    await userEvent.type(field, "rgdevment");
    fireEvent.blur(field);

    await screen.findByText("You now sign as rgdevment");
    await screen.findByText(
      "Nothing already written changes hands on its own. Do you want the 243 documents behind you to stand in your name?",
    );
    await screen.findByText("What arrived signed by somebody else keeps their name, always.");

    await userEvent.click(screen.getByRole("button", { name: /only the new ones/i }));
    expect(sent("sign_the_rest")).toHaveLength(0);

    await userEvent.click(await screen.findByRole("button", { name: /sign the older ones/i }));
    await userEvent.click(await screen.findByRole("button", { name: /put them in my name/i }));

    await waitFor(() => expect(sent("sign_the_rest")).toHaveLength(1));
    await screen.findByText("243 documents signed");
  });

  it("says nothing about signing the rest while there is nothing to sign", async () => {
    signing.alias = "rgdevment";
    signing.mine = 0;
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await writing();
    await screen.findByRole("textbox", { name: /^alias$/i });

    expect(screen.queryByRole("button", { name: /sign the older ones/i })).toBeNull();
  });

  it("takes an emptied alias as leaving it unsigned", async () => {
    signing.alias = "rgdevment";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await writing();

    const field = await screen.findByRole("textbox", { name: /^alias$/i });
    await waitFor(() => expect((field as HTMLInputElement).value).toBe("rgdevment"));
    await userEvent.clear(field);
    fireEvent.blur(field);

    await waitFor(() => expect(sent("sign")).toHaveLength(1));
    expect(sent("sign")[0].args.alias).toBeUndefined();
  });

  const keeping = async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await writing();
    const field = await screen.findByRole("textbox", { name: t("signName") });
    return field as HTMLInputElement;
  };

  it("keeps the name the person signs PDFs with, apart from the alias, and says so", async () => {
    signing.alias = "rgdevment";
    const field = await keeping();
    expect(field.placeholder).toBe("rgdevment");

    await userEvent.type(field, "  Mario Hidalgo  ");
    fireEvent.blur(field);

    await waitFor(() => expect(sent("sign_name")).toHaveLength(1));
    expect(sent("sign_name")[0].args.name).toBe("Mario Hidalgo");
    expect(sent("sign")).toHaveLength(0);
    expect(await screen.findByText(fill("signNameKept", "Mario Hidalgo"))).toBeTruthy();
    expect(field.value).toBe("Mario Hidalgo");
  });

  it("takes an emptied name as going back to the alias", async () => {
    signing.name = "Mario Hidalgo";
    const field = await keeping();
    await waitFor(() => expect(field.value).toBe("Mario Hidalgo"));

    await userEvent.clear(field);
    fireEvent.blur(field);

    await waitFor(() => expect(sent("sign_name")).toHaveLength(1));
    expect(sent("sign_name")[0].args.name).toBeUndefined();
    expect(await screen.findByText(t("signNameGone"))).toBeTruthy();
  });

  it("writes nothing when the name is left as it was", async () => {
    signing.name = "Mario Hidalgo";
    const field = await keeping();
    await waitFor(() => expect(field.value).toBe("Mario Hidalgo"));

    fireEvent.blur(field);

    expect(sent("sign_name")).toHaveLength(0);
  });
});

describe("the quick capture shortcut", () => {
  const shown = (keep: (keys: string) => Promise<unknown>) => {
    let bound: string | null = "Ctrl+Shift+Space";
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) => {
      const idea = args?.door === "idea";
      if (cmd === "shortcuts_offered")
        return Promise.resolve(idea ? ["Ctrl+Alt+I"] : ["Ctrl+Shift+Space", "Ctrl+Shift+T"]);
      if (cmd === "shortcut") return Promise.resolve(idea ? "Ctrl+Alt+I" : bound);
      if (cmd === "keep_shortcut")
        return keep(String(args?.keys)).then((now) => {
          bound = now as string | null;
          return now;
        });
      return otherwise(cmd, args);
    };
    render(
      <Keeping
        onPack={() => {}}
        onUnpack={() => {}}
        onChanged={() => {}}
        onDoc={() => {}}
        start="notices"
      />,
    );
  };

  it("keeps the shortcut the person picks", async () => {
    shown((keys) => Promise.resolve(keys));
    const picker = await screen.findByRole("combobox", { name: t("quickKeys") });
    await waitFor(() => expect(screen.getByRole("option", { name: "Ctrl+Shift+T" })).toBeTruthy());

    await userEvent.selectOptions(picker, "Ctrl+Shift+T");

    await waitFor(() => expect(sent("keep_shortcut")[0]?.args.keys).toBe("Ctrl+Shift+T"));
    await waitFor(() => expect((picker as HTMLSelectElement).value).toBe("Ctrl+Shift+T"));
  });

  it("says which application holds a shortcut that could not be taken", async () => {
    shown((keys) => Promise.reject({ code: "keysTaken", name: keys }));
    const picker = await screen.findByRole("combobox", { name: t("quickKeys") });
    await waitFor(() => expect(screen.getByRole("option", { name: "Ctrl+Shift+T" })).toBeTruthy());

    await userEvent.selectOptions(picker, "Ctrl+Shift+T");

    expect(await screen.findByText(fill("keysTaken", "Ctrl+Shift+T"))).toBeTruthy();
    expect((picker as HTMLSelectElement).value).toBe("Ctrl+Shift+Space");
  });

  it("keeps a shortcut of its own for an idea, beside the one for a task", async () => {
    shown((keys) => Promise.resolve(keys));
    const picker = await screen.findByRole("combobox", { name: t("quickIdeaKeys") });
    await waitFor(() => expect((picker as HTMLSelectElement).value).toBe("Ctrl+Alt+I"));

    await userEvent.selectOptions(picker, "off");

    await waitFor(() =>
      expect(sent("keep_shortcut")[0]?.args).toEqual({ keys: "off", door: "idea" }),
    );
  });
});

describe("the report a bug gets attached to", () => {
  const report = async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await upkeep();
    await userEvent.click(screen.getByRole("button", { name: /prepare/i }));
  };

  it("closes and leaves the maintenance page as it was", async () => {
    await report();

    await userEvent.click(screen.getByRole("button", { name: t("close") }));

    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("offers the log as one more answer, on by default", async () => {
    await report();

    const box = screen.getByRole("checkbox", { name: /include the error log/i });
    expect((box as HTMLInputElement).checked).toBe(true);
  });

  it("shows the log inside what would be sent", async () => {
    kept.lines = ["2026-08-11 10:00:00-04  WARN   sync      folder unreachable"];
    await report();

    await userEvent.click(screen.getByText(/see the report/i));

    expect(await screen.findByText(/folder unreachable/)).toBeTruthy();
  });

  it("leaves it out once it is unticked", async () => {
    kept.lines = ["2026-08-11 10:00:00-04  WARN   sync      folder unreachable"];
    await report();

    await userEvent.click(screen.getByRole("checkbox", { name: /include the error log/i }));
    await userEvent.click(screen.getByText(/see the report/i));

    await waitFor(() => expect(screen.queryByText(/folder unreachable/)).toBeNull());
  });

  it("writes one zip, and says whether the log goes in it", async () => {
    asked.file = "D:/issues/tisty-report.zip";
    await report();

    await userEvent.click(screen.getByRole("button", { name: /save the report/i }));

    await waitFor(() => expect(sent("keep_report").length).toBe(1));
    expect(sent("keep_report")[0].args.at).toBe("D:/issues/tisty-report.zip");
    expect(sent("keep_report")[0].args.logs).toBe(true);
  });
});

describe("Maintenance looking by itself", () => {
  const shown = () =>
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);

  it("reviews as soon as the tab opens, and says when everything is in order", async () => {
    shown();
    await userEvent.click(await screen.findByRole("tab", { name: /^health$/i }));

    expect(await screen.findByText(/everything in order · checked/i)).toBeTruthy();
    expect(sent("checked")).toHaveLength(1);
    expect(sent("repeated_lists")).toHaveLength(1);
    expect(sent("unvouched_attachments")).toHaveLength(1);
    expect(screen.queryByText(t("toLookAt"))).toBeNull();
  });

  it("does not check again when the tab is opened a second time", async () => {
    shown();
    await userEvent.click(await screen.findByRole("tab", { name: /^health$/i }));
    await screen.findByText(/everything in order · checked/i);
    await userEvent.click(screen.getByRole("tab", { name: /^overview$/i }));
    await userEvent.click(screen.getByRole("tab", { name: /^health$/i }));

    expect(await screen.findByText(/everything in order · checked/i)).toBeTruthy();
    expect(sent("repeated_lists")).toHaveLength(1);
  });

  it("says when a check could not run instead of calling everything in order", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "unvouched_attachments" ? Promise.reject({ code: "internal" }) : otherwise(cmd, args);
    shown();
    await userEvent.click(await screen.findByRole("tab", { name: /^health$/i }));

    expect(await screen.findByText(t("reviewPartly"))).toBeTruthy();
    expect(screen.queryByText(/everything in order · checked/i)).toBeNull();
  });

  it("says it is checking on the button while the review runs", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) => (cmd === "checked" ? new Promise(() => {}) : otherwise(cmd, args));
    shown();
    await userEvent.click(await screen.findByRole("tab", { name: /^health$/i }));

    const button = await screen.findByRole("button", { name: t("reviewing") });
    expect((button as HTMLButtonElement).disabled).toBe(true);
  });

  it("counts what needs a look, and shows only those cards", async () => {
    const otherwise = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "checked"
        ? otherwise(cmd, args).then((was) => ({
            ...(was as Record<string, unknown>),
            agrees: false,
            stranded: [{ file: "mac0-0042", title: "Minutes", bytes: 21_000, when: 1_754_000_000 }],
          }))
        : otherwise(cmd, args);
    shown();
    await upkeep();

    expect(screen.getByText(fill("healthMany", "2"))).toBeTruthy();
    expect(screen.getByText(t("toLookAt"))).toBeTruthy();
    expect(screen.getByText(t("cacheOffTitle"))).toBeTruthy();
    expect(screen.getByText(t("upkeepLook"))).toBeTruthy();
    expect(screen.queryByText(t("missingTitle"))).toBeNull();
    expect(screen.queryByText(t("repeatedLists"))).toBeNull();
  });

  it("checks again when asked", async () => {
    shown();
    await upkeep();

    await userEvent.click(screen.getByRole("button", { name: /^check again$/i }));

    await waitFor(() => expect(sent("checked")).toHaveLength(2));
  });

  it("rebuilds the quick index from one place, and says beside it whether it agrees", async () => {
    shown();
    await upkeep();

    expect(screen.getAllByRole("button", { name: t("reviewRedo") })).toHaveLength(1);
    expect(screen.getByText(t("tools"))).toBeTruthy();
    expect(await screen.findByText(t("cacheAgrees"))).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: t("reviewRedo") }));
    await waitFor(() => expect(sent("rebuild")).toHaveLength(1));
    expect(await screen.findByText(t("reviewRebuilt"))).toBeTruthy();
  });

  it("says the report in one line, and opens it only when asked", async () => {
    shown();
    await upkeep();

    expect(screen.queryByRole("checkbox", { name: /include the error log/i })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: /prepare/i }));
    expect(screen.getByRole("dialog")).toBeTruthy();
    expect(screen.getByRole("checkbox", { name: /include the error log/i })).toBeTruthy();
  });
});

describe("opening with the machine", () => {
  const notices = async () => {
    render(
      <Keeping
        start="window"
        onPack={() => {}}
        onUnpack={() => {}}
        onChanged={() => {}}
        onDoc={() => {}}
      />,
    );
    await ready();
  };

  it("asks the machine to open Tisty at sign-in", async () => {
    await notices();
    expect(await screen.findByText(/opens only when you open it/i)).toBeTruthy();

    const box = screen.getByRole("switch", { name: /open it at sign-in/i });
    expect(box.getAttribute("aria-checked")).toBe("false");
    await userEvent.click(box);

    await waitFor(() => expect(sent("wake_for").length).toBe(1));
    expect(sent("wake_for")[0].args.wanted).toBe(true);
    expect(await screen.findByText(/next time you sign in/i)).toBeTruthy();
  });

  it("leaves it closed again when asked", async () => {
    rousing.wakes = true;
    await notices();
    expect(await screen.findByText(/opens by itself when you sign in/i)).toBeTruthy();

    const box = screen.getByRole("switch", { name: /open it at sign-in/i });
    expect(box.getAttribute("aria-checked")).toBe("true");
    await userEvent.click(box);

    await waitFor(() => expect(sent("wake_for")[0].args.wanted).toBe(false));
    expect(await screen.findByText(/will not open on its own/i)).toBeTruthy();
  });

  it("says who holds the switch when the system took it", async () => {
    rousing.theirs = true;
    await notices();

    expect(await screen.findByText(/Settings → Apps → Startup/i)).toBeTruthy();
  });

  it("holds the switch while the system has it, and offers the system's own page", async () => {
    rousing.theirs = true;
    await notices();

    const knob = screen.getByRole("switch", { name: /open it at sign-in/i }) as HTMLButtonElement;
    expect(knob.disabled).toBe(true);
    const note = screen
      .getByText(/Settings → Apps → Startup/i)
      .closest("[data-note]") as HTMLElement;
    expect(note.getAttribute("data-note")).toBe("sys");
    expect(within(note).getByRole("button", { name: t("wakeOpenWindows") })).toBeTruthy();
    expect(sent("wake_for")).toHaveLength(0);
  });

  it("offers nothing where nothing can be offered", async () => {
    rousing.offered = false;
    await notices();

    expect(screen.queryByRole("switch", { name: /open it at sign-in/i })).toBeNull();
  });
});

describe("documents on disk the log does not name", () => {
  const strayed = () => {
    const was = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "checked"
        ? was(cmd, args).then((one) => ({
            ...(one as object),
            stranded: [{ file: "mac0-0042", title: "Minutes", bytes: 21_000, when: 1_754_000_000 }],
          }))
        : was(cmd, args);
  };

  const opened = async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await upkeep();
  };

  it("offers to take one in before it offers to let it go", async () => {
    strayed();
    await opened();

    expect(await screen.findByText("Minutes")).toBeTruthy();
    const takeIn = screen.getByRole("button", { name: /^recover$/i });
    const letGo = screen.getByRole("button", { name: /delete for good/i });
    expect(takeIn.compareDocumentPosition(letGo) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("takes one in without asking, because that can never be wrong", async () => {
    strayed();
    await opened();

    await userEvent.click(await screen.findByRole("button", { name: /^recover$/i }));
    await waitFor(() => expect(sent("doc_adopt")).toHaveLength(1));
    expect(sent("doc_adopt")[0]?.args.file).toBe("mac0-0042");
  });

  it("never lets one go without being told twice", async () => {
    strayed();
    await opened();

    await userEvent.click(await screen.findByRole("button", { name: /delete for good/i }));
    const dialog = await screen.findByRole("dialog");
    expect(dialog.textContent).toMatch(/for good/i);
    expect(within(dialog).getByText(t("carefulDocLead"))).toBeTruthy();
    await careful(t("cancel"));
    expect(sent("doc_let_go")).toHaveLength(0);
  });

  it("says nothing when every file on disk is in the log", async () => {
    await opened();

    expect(screen.queryByRole("button", { name: /^recover$/i })).toBeNull();
  });
});

describe("letting an assistant file work here", () => {
  const openTab = async (shelf?: RegExp, onHelp?: () => void) => {
    render(
      <Keeping
        onPack={() => {}}
        onUnpack={() => {}}
        onChanged={() => {}}
        onDoc={() => {}}
        onHelp={onHelp}
      />,
    );
    await sync();
    await userEvent.click(screen.getByRole("tab", { name: /^agents$/i }));
    if (shelf) await userEvent.click(await screen.findByRole("tab", { name: shelf }));
  };

  it("says nothing can file here until the person turns one on", async () => {
    await openTab();

    expect(await screen.findByText(/No agent can file work here/i)).toBeTruthy();
    expect(screen.getByRole("button", { name: /Let an agent file here/i })).toBeTruthy();
  });

  it("leaves how assistants work to the page's own link to the help", async () => {
    const onHelp = vi.fn();
    await openTab(undefined, onHelp);

    expect(screen.queryByRole("button", { name: /How agents work with Tisty/i })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: /^How it works/ }));

    expect(onHelp).toHaveBeenCalledWith({ slug: "agents-and-mcp" });
  });

  it("registering is a click of the person's, never something that arrives over the wire", async () => {
    await openTab();

    await userEvent.click(screen.getByRole("button", { name: /Let an agent file here/i }));

    await waitFor(() => expect(sent("agent_turn").length).toBe(1));
    expect(sent("agent_turn")[0].args.on).toBe(true);
    expect(await screen.findByText(/An agent can file work here/i)).toBeTruthy();
    expect(screen.getByText(/sign the log as «espino 3»/i)).toBeTruthy();
  });

  it("asks the assistant to keep its task up to date unless the person turns it off", async () => {
    Object.assign(serving, { on: true, called: "espino 3" });
    await openTab(/^door$/i);

    const asking = await screen.findByRole("switch", { name: /keep its task up to date/i });
    expect(asking.getAttribute("aria-checked")).toBe("true");

    await userEvent.click(asking);

    await waitFor(() => expect(sent("agent_asks").length).toBe(1));
    expect(sent("agent_asks")[0].args.on).toBe(false);
    await waitFor(() => expect(asking.getAttribute("aria-checked")).toBe("false"));
  });

  it("reminds the assistant when a turn starts and ends unless the person turns it off", async () => {
    Object.assign(serving, { on: true, called: "espino 3" });
    await openTab(/^door$/i);

    const reminding = await screen.findByRole("switch", { name: /when a turn starts and ends/i });
    expect(reminding.getAttribute("aria-checked")).toBe("true");

    await userEvent.click(reminding);

    await waitFor(() => expect(sent("agent_hooks").length).toBe(1));
    expect(sent("agent_hooks")[0].args.on).toBe(false);
    await waitFor(() => expect(reminding.getAttribute("aria-checked")).toBe("false"));
  });

  it("lists the assistants on this computer, with the file each one reads", async () => {
    await openTab(/^connected$/i);

    expect(await screen.findByText("Claude Code")).toBeTruthy();
    expect(screen.getByText("C:/Users/someone/.claude.json")).toBeTruthy();
    expect(
      within(document.getElementById("agents-shelf") as HTMLElement).getByText(/^Connected$/),
    ).toBeTruthy();
    expect(screen.getByRole("button", { name: /^Connect$/ })).toBeTruthy();
  });

  it("connecting one writes into that assistant's own settings and asks for a restart", async () => {
    await openTab(/^connected$/i);

    await userEvent.click(await screen.findByRole("button", { name: /^Connect$/ }));

    await waitFor(() => expect(sent("wire").length).toBe(1));
    expect(sent("wire")[0].args.id).toBe("antigravity");
    expect(await screen.findByText(/Quit it completely and open it again/i)).toBeTruthy();
  });

  it("one already connected is taken back out rather than written twice", async () => {
    await openTab(/^connected$/i);

    await userEvent.click(await screen.findByRole("button", { name: /^Remove$/ }));
    expect(sent("unwire").length).toBe(0);
    await userEvent.click(screen.getByRole("button", { name: "Yes, remove" }));

    await waitFor(() => expect(sent("unwire").length).toBe(1));
    expect(sent("unwire")[0].args.id).toBe("claude-code");
    expect(sent("wire").length).toBe(0);
  });

  it("one pointing at another Tisty is said plainly, and fixing it asks first", async () => {
    installed.seen = [
      {
        id: "codex",
        name: "Codex",
        at: "C:/Users/someone/.codex/config.toml",
        wired: false,
        astray: true,
        gone: true,
        points: "C:/Programs/Gone/tisty.exe",
      },
    ];

    await openTab(/^connected$/i);

    expect(await screen.findByText(/Points at another Tisty/i)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /^Fix$/ }));
    expect(sent("mend")).toHaveLength(0);
    await userEvent.click(screen.getByRole("button", { name: "Yes, fix" }));

    await waitFor(() => expect(sent("mend").length).toBe(1));
    expect(sent("mend")[0].args).toEqual({
      id: "codex",
      ask: { connection: "C:/Programs/Gone/tisty.exe", hooks: [], hooksOut: false },
    });
  });

  it("looks at the agents with the door open before anyone opens their page, never with it shut", async () => {
    installed.seen = [
      {
        id: "codex",
        name: "Codex",
        at: "C:/Users/someone/.codex/config.toml",
        wired: false,
        astray: true,
        gone: true,
        points: "C:/Programs/Gone/tisty.exe",
      },
    ];
    const opened = () =>
      render(
        <Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />,
      );

    const shut = opened();
    await sync();
    expect(sent("wiring")).toHaveLength(0);
    shut.unmount();

    Object.assign(serving, { on: true, called: "espino 3" });
    opened();
    await sync();

    await waitFor(() => expect(sent("wiring_watched").length).toBeGreaterThan(0));
    expect(sent("agent")).toHaveLength(0);
    expect(sent("wsl_seen")).toHaveLength(0);
    await userEvent.click(screen.getByRole("tab", { name: /^overview/i }));
    expect(await screen.findByText("Codex points to a Tisty that no longer exists")).toBeTruthy();
  });

  it("also says an agent left behind in a distro already looked into, naming the distro, and asks only for the names looked into", async () => {
    installed.seen = [];
    installed.inDistros = [
      {
        id: "wsl:Ubuntu:claude-code",
        name: "Claude Code",
        place: "Ubuntu",
        at: "//wsl.localhost/Ubuntu/home/someone/.claude.json",
        wired: false,
        astray: true,
        gone: true,
        points: "/old/tisty",
      },
    ];
    window.localStorage.setItem("tisty.wsl.looked", '["Ubuntu"]');
    Object.assign(serving, { on: true, called: "espino 3" });
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);

    await sync();

    await waitFor(() => expect(sent("wsl_watched").length).toBeGreaterThan(0));
    expect(sent("wsl_watched")[0].args).toEqual({ names: ["Ubuntu"] });
    expect(sent("wsl_seen")).toHaveLength(0);
    await userEvent.click(screen.getByRole("tab", { name: /^overview/i }));
    expect(
      await screen.findByText("Claude Code (Ubuntu · WSL) points to a Tisty that no longer exists"),
    ).toBeTruthy();
  });

  // The list is one row per assistant, by the name it gives itself, and says what each wrote;
  // what was written before clients were named gets a row of its own, with no button.
  it("says what each assistant wrote, and gathers the unnamed under one row", async () => {
    installed.hands = [
      {
        via: "claude-code",
        named: "Claude Code",
        wired: true,
        filed: 27,
        wrote: 140,
        last: "2026-09-16T10:00:00Z",
      },
      { via: null, named: "", wired: null, filed: 132, wrote: 570, last: "2026-09-10T10:00:00Z" },
    ];
    try {
      await openTab(/^connected$/i);

      const claude = await screen.findByText("Claude Code");
      expect(claude.parentElement?.textContent).toContain("27 filed");
      expect(claude.parentElement?.textContent).toContain("the last");
      const unnamed = screen.getByText("An agent with no name");
      expect(unnamed.parentElement?.textContent).toContain("132 filed");
      expect(screen.getByText("Antigravity").parentElement?.textContent).toContain(
        "nothing written yet",
      );
      expect(screen.queryByText(/espino|canelo|mirto/)).toBeNull();
    } finally {
      installed.hands = [];
    }
  });

  it("with none of them installed it says so instead of showing an empty list", async () => {
    installed.seen = [];

    await openTab(/^connected$/i);

    expect(await screen.findByText(/knows its way around/i)).toBeTruthy();
  });

  // A machine that only receives the log by Sync still reads what the other machine's hands
  // wrote: the table does not hang from a client being installed here.
  it("still says what the hands wrote when none of them is installed here", async () => {
    installed.seen = [];
    installed.hands = [
      {
        via: "codex",
        named: "Codex",
        wired: null,
        filed: 40,
        wrote: 90,
        last: "2026-09-16T10:00:00Z",
      },
      { via: null, named: "", wired: null, filed: 132, wrote: 570, last: "2026-09-10T10:00:00Z" },
    ];
    try {
      await openTab(/^connected$/i);

      const codex = await screen.findByText("Codex");
      expect(codex.parentElement?.textContent).toContain("40 filed");
      expect(screen.getByText("An agent with no name").parentElement?.textContent).toContain(
        "132 filed",
      );
      expect(screen.getByText(/knows its way around/i)).toBeTruthy();
    } finally {
      installed.hands = [];
    }
  });

  it("spells out what it can never do, where the person decides", async () => {
    Object.assign(serving, { on: true, called: "espino 3" });

    await openTab(/^door$/i);

    expect(await screen.findByText(/An agent can file work here/i)).toBeTruthy();
    expect(screen.getByText(/your undo never reaches what it filed/i)).toBeTruthy();

    await userEvent.click(screen.getByRole("tab", { name: /^what it can and cannot do$/i }));
    const never = screen.getByRole("region", { name: t("agentsCannotTitle") });
    expect(within(never).getByText("Complete, reopen, drop or delete a task")).toBeTruthy();
    expect(within(never).getAllByRole("listitem")).toHaveLength(
      t("agentsCannot").split(" · ").length,
    );

    await userEvent.click(screen.getByRole("tab", { name: /^connected$/i }));
    expect(screen.getByText(/"mcpServers"/)).toBeTruthy();
    expect(screen.getByText(/mcp add tisty/)).toBeTruthy();
  });

  it("keeps the assistants on five shelves, and opens on the door while it is shut", async () => {
    await openTab();

    const shelves = screen.getByRole("tablist", { name: /^agents$/i });
    expect(
      within(shelves)
        .getAllByRole("tab")
        .map((one) => one.textContent),
    ).toEqual([
      "Door",
      "Connected",
      "Agents folder",
      "How its signature looks",
      "What it can and cannot do",
    ]);
    expect(within(shelves).getByRole("tab", { name: "Door" }).getAttribute("aria-selected")).toBe(
      "true",
    );
    expect(screen.queryByText(t("wiringTitle"))).toBeNull();
  });

  const group = (name: string) => within(screen.getByRole("radiogroup", { name }));
  const pick = (name: string, how: string) =>
    userEvent.click(group(name).getByRole("radio", { name: how }));
  const checkedIn = (name: string) =>
    group(name)
      .getAllByRole("radio")
      .filter((one) => (one as HTMLInputElement).checked)
      .map((one) => (one as HTMLInputElement).value);

  it("checks the radio each place has stored", async () => {
    credit("rows", "mark");
    credit("inside", "agent");
    await openTab(/^how its signature looks$/i);

    expect(checkedIn(t("creditRows"))).toEqual(["mark"]);
    expect(checkedIn(t("creditInside"))).toEqual(["agent"]);
    expect(group(t("creditRows")).getAllByRole("radio")).toHaveLength(3);
    credit("rows", "agent");
    credit("inside", "named");
  });

  it("previews each place on its own as its radios change", async () => {
    credit("rows", "agent");
    credit("inside", "named");
    await openTab(/^how its signature looks$/i);

    const row = screen.getByTestId("credit-preview-rows");
    const inside = screen.getByTestId("credit-preview-inside");
    expect(screen.getByText(t("creditPreviewTask"))).toBeTruthy();
    expect(row.textContent).toBe("by an agent");
    expect(row.getAttribute("title")).toBe("Claude Code wrote on this machine");
    expect(inside.textContent).toBe("by Claude Code");
    expect(inside.getAttribute("title")).toBe(t("fromThisMachine"));

    await pick(t("creditRows"), t("creditAsNamed"));
    expect(row.textContent).toBe("by Claude Code");
    expect(inside.textContent).toBe("by Claude Code");

    await pick(t("creditInside"), t("creditAsMark"));
    expect(inside.textContent).toBe("◆");
    expect(
      within(inside).getByRole("img", { name: "Claude Code wrote on this machine" }),
    ).toBeTruthy();
    expect(row.textContent).toBe("by Claude Code");

    await pick(t("creditRows"), t("creditAsMark"));
    expect(row.textContent).toBe("◆");
    expect(
      within(row).getByRole("img", { name: "Claude Code wrote on this machine" }),
    ).toBeTruthy();

    await pick(t("creditInside"), t("creditAsAgent"));
    expect(inside.textContent).toBe("by an agent");
    expect(inside.getAttribute("title")).toBe("Claude Code wrote on this machine");
    expect(row.textContent).toBe("◆");
    expect(checkedIn(t("creditRows"))).toEqual(["mark"]);
    expect(checkedIn(t("creditInside"))).toEqual(["agent"]);
    credit("rows", "agent");
    credit("inside", "named");
  });
});

describe("documents the log names with no file", () => {
  const gone = (quiet: boolean) => {
    const was = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "checked"
        ? was(cmd, args).then((one) => ({
            ...(one as object),
            missing: [{ id: "01M1ER4WK1E6BZJF2S9H9V0KYB", file: "win1-0007", title: "Recipes" }],
            machines: [
              {
                id: "mac0-0001",
                called: "cedar 14",
                when: Math.floor(Date.now() / 1000),
                mine: true,
              },
              {
                id: "win1-0002",
                called: "sage 07",
                when: Math.floor(Date.now() / 1000) - (quiet ? 60 * 60 * 24 * 12 : 60),
                mine: false,
              },
            ],
          }))
        : was(cmd, args);
  };

  const opened = async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await upkeep();
  };

  it("will not let one be forgotten while a machine has been quiet", async () => {
    gone(true);
    await opened();

    expect(await screen.findByText("Recipes")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /delete on every computer/i })).toBeNull();
    expect(screen.getByText(/until sage 07 syncs/i)).toBeTruthy();
  });

  it("offers to forget one once every machine has spoken, and asks first", async () => {
    gone(false);
    await opened();

    await userEvent.click(await screen.findByRole("button", { name: /delete on every computer/i }));
    expect((await screen.findByRole("dialog")).textContent).toMatch(/for good/i);
    await careful(t("cancel"));
    expect(sent("doc_drop")).toHaveLength(0);

    await userEvent.click(screen.getByRole("button", { name: /delete on every computer/i }));
    await careful(t("carefulDocGo"));
    await waitFor(() => expect(sent("doc_drop")).toHaveLength(1));
    expect(sent("doc_drop")[0]?.args.id).toBe("01M1ER4WK1E6BZJF2S9H9V0KYB");
  });
});

describe("taking every loose attachment out at once", () => {
  const opened = async () => {
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);
    await storage();
  };

  it("says how many and where they go before it takes any, and takes none when told no", async () => {
    await opened();

    await userEvent.click(await screen.findByRole("button", { name: /move 3 to the bin/i }));
    const asking = await screen.findByRole("dialog");
    expect(asking.textContent).toContain(fill("carefulLooseTitle", "3"));
    expect(asking.textContent).toContain(t("carefulLooseLead"));
    expect(asking.textContent).toContain(t("carefulLooseOne"));
    await careful(t("cancel"));
    expect(sent("retire_attachments")).toHaveLength(0);
  });

  it("takes each one out and looks again afterwards", async () => {
    await opened();

    await userEvent.click(await screen.findByRole("button", { name: /move 3 to the bin/i }));
    await careful(t("carefulLooseGo"));
    await waitFor(() => expect(sent("retire_attachments")).toHaveLength(1));
    const references = sent("retire_attachments")[0]?.args.references as string[];
    expect(references).toHaveLength(3);
    expect(references).toContain("attachments/ab/charla-a3f9.mp4");
  });

  it("will not offer a copy that is past what one file holds", async () => {
    carrying.weight = 9 * 1024 * 1024 * 1024;
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);

    await data();

    expect(
      (screen.getByRole("button", { name: /create backup/i }) as HTMLButtonElement).disabled,
    ).toBe(true);
    expect(screen.getByText(/past the/i)).toBeTruthy();
  });

  it("says which folder it stopped sharing with when a restore let one go", async () => {
    carrying.sharedWas = "G:/My Drive/tisty";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);

    await sync();

    expect(screen.getByText(/not sharing since you restored/i)).toBeTruthy();
    expect(screen.getByText(/G:\/My Drive\/tisty/)).toBeTruthy();
  });

  it("says what a restore let go was a way it cannot name, without naming a folder", async () => {
    carrying.sharedWasLater = true;
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);

    await sync();

    expect(screen.getByText(t("syncOffRestoredLater"))).toBeTruthy();
    expect(screen.queryByText(t("syncOff"))).toBeNull();
  });

  it("says a way a newer version chose is not known here, and offers no way to replace it", async () => {
    carrying.later = true;
    carrying.sharedWas = "G:/My Drive/tisty";
    render(<Keeping onPack={() => {}} onUnpack={() => {}} onChanged={() => {}} onDoc={() => {}} />);

    await sync();

    expect(screen.getByText(t("syncLaterCard"))).toBeTruthy();
    expect(screen.queryByText(/not sharing since you restored/i)).toBeNull();
    expect(screen.queryByRole("button", { name: t("turnSyncOn") })).toBeNull();
    expect(screen.queryByText(t("noDestination"))).toBeNull();
  });
});

describe("the settings search index", () => {
  const SHELVES = [
    ["door", /^door$/i],
    ["wired", /^connected$/i],
    ["folder", /^agents folder$/i],
    ["credit", /^how its signature looks$/i],
    ["can", /^what it can and cannot do$/i],
  ] as const;
  const PAGES: Tab[] = [
    "window",
    "notices",
    "writing",
    "sync",
    "security",
    "copies",
    "storage",
    "agents",
    "health",
  ];

  const drawn = (tab: Tab, shelf?: Shelf): string[] => {
    const rows = [...document.querySelectorAll<HTMLElement>("[data-line], [data-card]")].filter(
      (one) => !one.hasAttribute("data-item"),
    );
    for (const one of rows)
      expect(one.dataset.setting, `${tab}: «${one.textContent?.slice(0, 50)}»`).toBeTruthy();
    return [...document.querySelectorAll<HTMLElement>("[data-setting]")].map((one) => {
      const id = one.dataset.setting as RowId;
      const row: Row | undefined = ROWS[id];
      expect(row, id).toBeTruthy();
      expect(row.page, id).toBe(tab);
      if (shelf) expect(row.shelf, id).toBe(shelf);
      const said = `${one.textContent ?? ""} ${one.getAttribute("aria-label") ?? ""}`;
      const named = [row.title, ...(row.words ?? [])].some((key) => said.includes(t(key)));
      expect(named, `${id} shows its title`).toBe(true);
      return id;
    });
  };

  const settle = () => new Promise((done) => setTimeout(done, 40));

  const visit = async (pages = PAGES): Promise<string[]> => {
    const seen: string[] = [];
    for (const tab of pages) {
      const { unmount } = render(
        <Keeping
          start={tab}
          onPack={() => {}}
          onUnpack={() => {}}
          onChanged={() => {}}
          onDoc={() => {}}
          hidden={<HiddenFolders papers={{ folders: [], docs: [] }} />}
        />,
      );
      await ready();
      await waitFor(() => expect(document.querySelector("[data-setting]")).toBeTruthy());
      await settle();
      if (tab !== "agents") seen.push(...drawn(tab));
      else
        for (const [shelf, name] of SHELVES) {
          await userEvent.click(screen.getByRole("tab", { name }));
          await settle();
          seen.push(...drawn(tab, shelf));
        }
      unmount();
    }
    return seen;
  };

  it("holds every row a page draws, alone and synced, the door shut and open", async () => {
    const alone = await visit();
    carrying.chosen = "C:/Users/someone/OneDrive/Tisty";
    Object.assign(serving, { on: true, called: "espino 3" });
    const synced = await visit();
    carrying.freesSpace = true;
    const clouded = await visit(["storage"]);

    const seen = new Set([...alone, ...synced, ...clouded, ...DRAWN_BY_THEIR_OWN_TESTS]);
    const unseen = ROW_IDS.filter((id) => ROWS[id].page !== "about" && !seen.has(id));
    expect(unseen).toEqual([]);
  });

  it("holds every row the about page draws", async () => {
    const before = ipc.answer;
    ipc.answer = (cmd, args) =>
      cmd === "about"
        ? Promise.resolve({
            version: "1.49.0",
            sandbox: null,
            repository: "",
            license: "AGPL-3.0-only",
            store: "",
            candidates: false,
            candidatesApply: true,
            autoUpdates: false,
            autoUpdatesApply: true,
            keptByTheStore: false,
          })
        : before(cmd, args);
    render(
      <About
        ready={{ version: "1.50.0", route: "download", installs: true }}
        onError={() => {}}
        help={<HelpBand onGreet={() => {}} onDoc={() => {}} onChanged={() => {}} />}
      />,
    );
    await waitFor(() => expect(document.querySelector('[data-setting="candidates"]')).toBeTruthy());
    const seen = new Set(drawn("about"));
    expect(ROW_IDS.filter((id) => ROWS[id].page === "about" && !seen.has(id))).toEqual([]);
  });
});

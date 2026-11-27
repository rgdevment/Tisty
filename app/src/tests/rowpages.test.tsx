import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Carrying, Settings } from "../core";
import { credit } from "../crediting";
import { fill, t } from "../locales";
import Agents from "../ui/Agents";
import type { Which } from "../ui/Card";
import Notices from "../ui/Notices";
import Sealing from "../ui/Sealing";
import SettingsNav from "../ui/SettingsNav";
import Storage from "../ui/Storage";
import Syncing from "../ui/Syncing";
import { PANEL } from "../ui/settingsPanel";
import Window from "../ui/Window";
import { IDLE } from "../ui/working";
import Yours from "../ui/Yours";
import { knownFor } from "./knownFor";
import { LiveRows, Rows } from "./rowsShell";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  urgent: true,
  sealed: { state: "none", cloud: "google" } as unknown,
  sealedFails: 0,
  urls: [] as string[],
  distros: [] as string[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown> = {}) => {
    ipc.calls.push({ cmd, args });
    if (cmd === "urgent_offered") return Promise.resolve(ipc.urgent);
    if (cmd === "shortcuts_offered") return Promise.resolve(["Ctrl+Alt+T"]);
    if (cmd === "sealing_shown")
      return ipc.sealedFails-- > 0 ? Promise.reject({ code: "x" }) : Promise.resolve(ipc.sealed);
    if (cmd === "wsl_distros") return Promise.resolve(ipc.distros);
    if (cmd === "wsl_seen") return Promise.resolve({ distro: { shut: false }, seen: [] });
    if (cmd === "move_pending") return Promise.resolve(null);
    if (cmd === "checked") return new Promise(() => {});
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (url: string) => {
    ipc.urls.push(url);
    return Promise.resolve();
  },
}));

beforeEach(() => {
  ipc.calls.length = 0;
  ipc.urls.length = 0;
  ipc.urgent = true;
  ipc.distros = [];
  localStorage.clear();
});

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const working = (...cards: Which[]) => new Map(cards.map((one) => [one, Date.now()]));

const kept = (over: Partial<Settings> = {}): Settings => ({
  quiet: [],
  holds: "everywhere",
  shares: true,
  onlySharedAbove: 50,
  historyDays: 30,
  ...over,
});

const carrying = (over: Partial<Carrying> = {}): Carrying => ({
  asked: true,
  open: 3,
  archived: 1,
  lists: 2,
  attachments: 0,
  weight: 10,
  carries: 100,
  mode: "folder",
  cloudOffered: false,
  clouds: [],
  freesSpace: false,
  ...over,
});

const rowOf = (id: string) => document.querySelector(`[data-setting=${id}]`) as HTMLElement;

describe("a switch the system holds", () => {
  it("says so in its row and opens Windows' own page", async () => {
    render(
      <Window
        kept={kept()}
        amend={() => {}}
        wake={{ offered: true, wakes: false, theirs: true }}
        setWake={() => {}}
        busy={IDLE}
        run={() => {}}
        tell={() => {}}
        fail={() => {}}
        remember={() => {}}
        onChanged={() => {}}
      />,
    );
    const row = within(rowOf("wake"));
    expect((row.getByRole("switch") as HTMLButtonElement).disabled).toBe(true);
    expect(row.getByText(t("noteSys"))).toBeTruthy();
    await userEvent.click(row.getByRole("button", { name: t("wakeOpenWindows") }));
    expect(ipc.urls).toEqual(["ms-settings:startupapps"]);
  });

  it("stays usable while a backup runs", () => {
    render(
      <Window
        kept={kept()}
        amend={() => {}}
        wake={{ offered: true, wakes: true, theirs: false }}
        setWake={() => {}}
        busy={working("backup")}
        run={() => {}}
        tell={() => {}}
        fail={() => {}}
        remember={() => {}}
        onChanged={() => {}}
      />,
    );
    for (const box of screen.getAllByRole("combobox"))
      expect((box as HTMLSelectElement).disabled).toBe(false);
    expect((within(rowOf("wake")).getByRole("switch") as HTMLButtonElement).disabled).toBe(false);
    expect(screen.queryByText(fill("waitFor", t("backup")))).toBeNull();
  });
});

describe("important reminders", () => {
  const notices = (settings: Settings, remember = vi.fn()) => {
    render(
      <Notices
        kept={settings}
        amend={() => {}}
        keys="Ctrl+Alt+T"
        setKeys={() => {}}
        busy={IDLE}
        run={() => {}}
        tell={() => {}}
        remember={remember}
      />,
    );
    return remember;
  };

  it("say what is missing and turn it on from the row", async () => {
    const remember = notices(kept({ quiet: ["screen"] }));
    const row = within(rowOf("urgent"));
    expect(await row.findByText(t("noticeUrgentScreen"))).toBeTruthy();
    expect((row.getByRole("switch") as HTMLButtonElement).disabled).toBe(true);
    await userEvent.click(row.getByRole("button", { name: t("urgentScreenOn") }));
    expect(remember).toHaveBeenCalledWith(expect.objectContaining({ quiet: [] }), "screen");
  });

  it("send you to the system once they are on", async () => {
    notices(kept({ urgent: true }));
    const row = within(rowOf("urgent"));
    await userEvent.click(await row.findByRole("button", { name: t("noticeUrgentOpen") }));
    expect(sent("open_notice_settings")).toHaveLength(1);
  });

  it("say nothing more where the system cannot let one through", async () => {
    ipc.urgent = false;
    notices(kept({ quiet: ["screen"] }));
    expect(await screen.findByText(t("noticeUrgentAway"))).toBeTruthy();
    expect(within(rowOf("urgent")).queryByText(t("noticeUrgentScreen"))).toBeNull();
  });

  it("keep the alarm switches free while another one saves", () => {
    render(
      <Notices
        kept={kept()}
        amend={() => {}}
        keys="Ctrl+Alt+T"
        setKeys={() => {}}
        busy={working("chime")}
        run={() => {}}
        tell={() => {}}
        remember={() => {}}
      />,
    );
    expect((within(rowOf("chime")).getByRole("switch") as HTMLButtonElement).disabled).toBe(true);
    expect((within(rowOf("screen")).getByRole("switch") as HTMLButtonElement).disabled).toBe(false);
  });
});

describe("the assistants' page", () => {
  const known = (on: boolean) =>
    knownFor({
      agent: { on, called: on ? "espino 3" : undefined, asks: true, hooks: true },
      agents: [{ id: "claude", name: "Claude Code", wired: true }] as never,
    });
  const shown = (on: boolean, sent?: "credit" | "wired") =>
    render(
      <Rows>
        <Agents
          sent={sent}
          busy={IDLE}
          quietly={(_card, work, done) => {
            void (typeof work === "function" ? work() : work).then(done);
          }}
          tell={() => {}}
          fail={() => {}}
          known={known(on)}
        />
      </Rows>,
    );

  it("holds both habits while the door is shut, and says why", () => {
    shown(false);
    expect((within(rowOf("asks")).getByRole("switch") as HTMLButtonElement).disabled).toBe(true);
    expect((within(rowOf("hooks")).getByRole("switch") as HTMLButtonElement).disabled).toBe(true);
    expect(within(rowOf("hooks")).getByText(t("doorNeeded"))).toBeTruthy();
  });

  it("opens on the door while it is shut and on Connected once it is open", () => {
    shown(false);
    expect(screen.getByRole("tab", { name: "Door" }).getAttribute("aria-selected")).toBe("true");
    cleanup();
    shown(true);
    const connected = screen.getByRole("tab", { name: "Connected" });
    expect(connected.getAttribute("aria-selected")).toBe("true");
  });

  it("looks into remembered WSL distros only once the tab itself is pressed", async () => {
    window.localStorage.setItem("tisty.wsl.looked", '["Ubuntu"]');
    ipc.distros = ["Ubuntu"];
    shown(true, "wired");
    expect(await screen.findByRole("button", { name: fill("wslLook", "Ubuntu") })).toBeTruthy();
    expect(sent("wsl_seen")).toHaveLength(0);
    await userEvent.click(screen.getByRole("tab", { name: "Door" }));
    expect(sent("wsl_seen")).toHaveLength(0);
    await userEvent.click(screen.getByRole("tab", { name: "Connected" }));
    await waitFor(() => expect(sent("wsl_seen")).toHaveLength(1));
  });

  it("frees them once the door is open", async () => {
    shown(true);
    await userEvent.click(screen.getByRole("tab", { name: "Door" }));
    expect((within(rowOf("asks")).getByRole("switch") as HTMLButtonElement).disabled).toBe(false);
    expect(screen.queryByText(t("doorNeeded"))).toBeNull();
  });

  it("asks before taking the voice away, and keeps it when told no", async () => {
    shown(true);
    await userEvent.click(screen.getByRole("tab", { name: "Door" }));
    await userEvent.click(screen.getByRole("button", { name: t("agentsTurnOff") }));
    const dialog = within(await screen.findByRole("dialog"));
    expect(dialog.getByText(t("carefulDoorTwo"))).toBeTruthy();
    await userEvent.click(dialog.getByRole("button", { name: t("cancel") }));
    expect(sent("agent_turn")).toHaveLength(0);

    await userEvent.click(screen.getByRole("button", { name: t("agentsTurnOff") }));
    const again = within(await screen.findByRole("dialog"));
    await userEvent.click(again.getByRole("button", { name: t("agentsTurnOff") }));
    await waitFor(() => expect(sent("agent_turn")[0]?.args.on).toBe(false));
  });

  it("opens the door without asking", async () => {
    shown(false);
    await userEvent.click(screen.getByRole("button", { name: t("agentsTurnOn") }));
    await waitFor(() => expect(sent("agent_turn")[0]?.args.on).toBe(true));
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("sends a connected assistant to the door when it is shut", async () => {
    shown(false, "wired");
    const note = screen.getByText(t("wiringMute")).closest("[data-note]") as HTMLElement;
    await userEvent.click(within(note).getByRole("button", { name: t("goToDoor") }));
    expect(
      screen.getByRole("tab", { name: t("agentsTabDoor") }).getAttribute("aria-selected"),
    ).toBe("true");
  });

  it("gives each place its own three choices, with no switch holding the other", () => {
    credit("rows", "mark");
    credit("inside", "named");
    shown(true, "credit");
    const rows = within(rowOf("creditRows"));
    const inside = within(rowOf("creditInside"));
    const radio = (row: typeof rows, name: string) =>
      row.getByRole("radio", { name }) as HTMLInputElement;
    expect(rows.queryByRole("switch")).toBeNull();
    expect(inside.queryByRole("switch")).toBeNull();
    expect(rows.getAllByRole("radio")).toHaveLength(3);
    expect(radio(rows, t("creditAsMark")).checked).toBe(true);
    expect(radio(inside, t("creditAsNamed")).checked).toBe(true);
    expect(radio(inside, t("creditAsMark")).disabled).toBe(false);
    credit("rows", "agent");
  });

  it("moves through its tabs with the arrows, each one tied to the panel", () => {
    shown(false);
    const door = screen.getByRole("tab", { name: t("agentsTabDoor") });
    expect(door.getAttribute("aria-controls")).toBe("agents-shelf");
    expect(door.tabIndex).toBe(0);
    expect(screen.getByRole("tab", { name: t("agentsTabWired") }).tabIndex).toBe(-1);
    door.focus();
    fireEvent.keyDown(door, { key: "ArrowRight" });
    const wired = screen.getByRole("tab", { name: t("agentsTabWired") });
    expect(wired.getAttribute("aria-selected")).toBe("true");
    expect(document.activeElement).toBe(wired);
    expect(screen.getByRole("tabpanel").getAttribute("aria-labelledby")).toBe(wired.id);
    fireEvent.keyDown(wired, { key: "End" });
    expect(screen.getByRole("tab", { name: t("agentsTabCan") }).getAttribute("aria-selected")).toBe(
      "true",
    );
  });
});

describe("the bar's pages", () => {
  it("each points at the page and moves the focus with the arrows without opening it", () => {
    const onTab = vi.fn();
    render(
      <>
        <SettingsNav tab="window" onTab={onTab} onBack={() => {}} />
        <div id={PANEL} />
      </>,
    );
    const tabs = screen.getAllByRole("tab");
    for (const one of tabs) expect(one.getAttribute("aria-controls")).toBe(PANEL);
    const window = tabs.find((one) => one.getAttribute("aria-selected") === "true") as HTMLElement;
    expect(window.tabIndex).toBe(0);
    window.focus();
    fireEvent.keyDown(window, { key: "ArrowDown" });
    expect(document.activeElement).toBe(tabs[tabs.indexOf(window) + 1]);
    fireEvent.keyDown(document.activeElement as Element, { key: "Home" });
    expect(document.activeElement).toBe(tabs[0]);
    fireEvent.keyDown(document.activeElement as Element, { key: "ArrowUp" });
    expect(document.activeElement).toBe(tabs[tabs.length - 1]);
    expect(onTab).not.toHaveBeenCalled();
  });

  it("keeps the overview reachable while a search shows no page", () => {
    render(
      <SettingsNav
        tab="window"
        onTab={() => {}}
        onBack={() => {}}
        keeping={{ attention: [], finding: "tono", setFinding: () => {} }}
      />,
    );
    const tabs = screen.getAllByRole("tab");
    expect(tabs.filter((one) => one.tabIndex === 0)).toEqual([tabs[0]]);
    expect(tabs.every((one) => one.getAttribute("aria-selected") === "false")).toBe(true);
  });
});

describe("copies while a backup runs", () => {
  it("keeps restore, leaving and bringing documents waiting, and says for what", async () => {
    render(
      <Rows busy={working("backup")}>
        <Yours
          state={carrying()}
          holds="3 tasks"
          build={null}
          busy={working("backup")}
          run={() => {}}
          fail={() => {}}
          onPack={() => {}}
          onUnpack={() => {}}
          tell={() => {}}
        />
      </Rows>,
    );
    for (const id of ["restore", "leave", "parcels"]) {
      const row = within(rowOf(id));
      for (const button of row.getAllByRole("button"))
        expect((button as HTMLButtonElement).disabled).toBe(true);
      expect(await row.findByText(fill("waitFor", t("backup")))).toBeTruthy();
    }
  });
});

describe("turning a shared folder off", () => {
  it("asks first, naming what stays and what stops", async () => {
    const run = vi.fn();
    render(
      <Rows>
        <Syncing
          state={carrying({ chosen: "D:/Shared/Tisty" })}
          kept={kept()}
          busy={IDLE}
          run={run}
          fail={() => {}}
          carry={() => {}}
          pickFolder={() => {}}
          onMove={() => {}}
        />
      </Rows>,
    );
    const zone = within(screen.getByRole("region", { name: t("delicateZone") }));
    await userEvent.click(zone.getByRole("button", { name: t("syncOffNow") }));
    const dialog = within(await screen.findByRole("dialog"));
    expect(dialog.getByText(t("carefulSyncOffTwo"))).toBeTruthy();
    await userEvent.click(dialog.getByRole("button", { name: t("cancel") }));
    expect(run).not.toHaveBeenCalled();

    await userEvent.click(zone.getByRole("button", { name: t("syncOffNow") }));
    await userEvent.click(
      within(await screen.findByRole("dialog")).getByRole("button", { name: t("syncOffNow") }),
    );
    await waitFor(() =>
      expect(run).toHaveBeenCalledWith("syncOff", expect.any(Function), expect.any(Function)),
    );
  });
});

describe("where large attachments stay without a shared folder", () => {
  it("says what is missing and sends you to Sync", async () => {
    const onSync = vi.fn();
    render(
      <Rows>
        <Storage
          state={carrying()}
          kept={kept({ shares: false })}
          build={null}
          audit={null}
          setAudit={() => {}}
          freeing={null}
          setFreeing={() => {}}
          freeDays={90}
          setFreeDays={() => {}}
          busy={IDLE}
          run={() => {}}
          tell={() => {}}
          fail={() => {}}
          clear={() => {}}
          carry={() => {}}
          remember={() => {}}
          onSync={onSync}
        />
      </Rows>,
    );
    const row = within(rowOf("holds"));
    expect((row.getByRole("combobox") as HTMLSelectElement).disabled).toBe(true);
    await userEvent.click(row.getByRole("button", { name: t("goToSync") }));
    expect(onSync).toHaveBeenCalled();
  });
});

describe("encrypting without a cloud", () => {
  it("says a cloud is missing and sends you to Sync", async () => {
    ipc.sealed = { state: "none", cloud: "google" };
    const onSync = vi.fn();
    render(
      <Rows>
        <Sealing
          busy={IDLE}
          run={() => {}}
          tell={() => {}}
          fail={() => {}}
          onChanged={() => {}}
          onSync={onSync}
        />
      </Rows>,
    );
    const note = (await screen.findByText(t("sealNoCloud"))).closest("[data-note]") as HTMLElement;
    expect(note.getAttribute("data-note")).toBe("dep");
    await userEvent.click(within(note).getByRole("button", { name: t("goToSync") }));
    expect(onSync).toHaveBeenCalled();
  });
});

describe("a row that works again after its error", () => {
  it("drops the error once the retry answers", async () => {
    ipc.sealed = { state: "none", cloud: "google" };
    ipc.sealedFails = 1;
    render(
      <LiveRows>
        {(cards) => (
          <Sealing
            busy={cards.busy}
            run={cards.run}
            tell={cards.tell}
            fail={cards.fail}
            clear={cards.clear}
            onChanged={() => {}}
          />
        )}
      </LiveRows>,
    );
    await userEvent.click(await screen.findByRole("button", { name: t("tryAgain") }));
    expect(await screen.findByText(t("sealNoCloud"))).toBeTruthy();
    expect(document.querySelector("[data-note=err]")).toBeNull();
  });
});

describe("a look the window keeps", () => {
  it("says Saved under its row", async () => {
    const tell = vi.fn();
    render(
      <Window
        kept={kept()}
        amend={() => {}}
        wake={null}
        setWake={() => {}}
        busy={IDLE}
        run={(_card, work, done) => {
          void (typeof work === "function" ? work() : work).then(done);
        }}
        tell={tell}
        fail={() => {}}
        remember={() => {}}
        onChanged={() => {}}
      />,
    );
    await userEvent.selectOptions(screen.getByRole("combobox", { name: t("look") }), "dark");
    await waitFor(() =>
      expect(tell).toHaveBeenCalledWith({ card: "look", text: t("settingsKept") }),
    );
  });
});

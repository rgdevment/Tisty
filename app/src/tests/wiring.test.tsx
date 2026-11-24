import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { type ComponentProps, useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { offeredHere } from "../agentsOffered";
import { AGENTS_FIRST } from "../agentsSync";
import type { Wired } from "../core";
import { adopt, fill, t } from "../locales";
import Wiring from "../ui/Wiring";
import { skillsFor, strayHooks } from "./knownFor";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return ipc.answer(cmd, args ?? {});
  },
}));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const one = (id: string, more: Partial<Wired> = {}): Wired => ({
  id,
  name: id,
  at: `/home/someone/${id}.json`,
  wired: false,
  astray: false,
  maybe: false,
  byHand: false,
  manual: {
    file: `/home/someone/${id}.json`,
    snippet: `"mcpServers": { "tisty": { "command": "/opt/tisty" } }`,
    cli: `${id} mcp add tisty -- /opt/tisty mcp`,
  },
  ...more,
});

const lostHooks = strayHooks();

const shown = (agents: Wired[], more: Partial<ComponentProps<typeof Wiring>> = {}) => {
  const quietly = <T,>(
    _card: string,
    work: Promise<T> | (() => Promise<T>),
    then: (answer: T) => void,
  ) => {
    void (typeof work === "function" ? work() : work).then(then);
  };
  return render(
    <Wiring
      agents={agents}
      setAgents={() => {}}
      hands={[]}
      held={false}
      quietly={quietly as never}
      tell={() => {}}
      fail={() => {}}
      wroteSaid={() => "nothing written yet"}
      {...more}
    />,
  );
};

beforeEach(() => {
  adopt("en");
  ipc.calls = [];
  ipc.answer = (cmd) => {
    if (cmd === "wsl_distros") return Promise.resolve([]);
    if (["wire", "unwire", "hook", "unhook", "wiring"].includes(cmd)) return Promise.resolve([]);
    return Promise.resolve(null);
  };
  window.localStorage.clear();
  offeredHere.put([]);
});

describe("connecting assistants", () => {
  it("connects every one found at once, leaving the doubtful and the by-hand ones out", async () => {
    shown([
      one("codex"),
      one("copilot", { also: "VS Code" }),
      one("cursor", { maybe: true }),
      one("goose", { byHand: true }),
      one("claude-code", { wired: true }),
    ]);

    expect(screen.getByText(/also VS Code/)).toBeTruthy();
    expect(screen.getByText("May be here")).toBeTruthy();
    expect(screen.getAllByText("By hand").length).toBeGreaterThan(0);
    await userEvent.click(screen.getByRole("button", { name: "Connect the 2 found" }));

    await waitFor(() => expect(sent("wire").length).toBe(2));
    expect(sent("wire").map((call) => call.args.id)).toEqual(["codex", "copilot"]);
  });

  it("asks before fixing one, naming every file it writes, and says what it did", async () => {
    const tell = vi.fn();
    ipc.answer = (cmd, args) => {
      if (cmd === "wsl_distros") return Promise.resolve([]);
      if (cmd === "mend") return Promise.resolve({ seen: [one(String(args.id), { wired: true })] });
      return Promise.resolve([]);
    };
    shown(
      [
        one("codex"),
        one("gemini", { astray: true, gone: true, points: "/old/debug/tisty", hooks: lostHooks }),
      ],
      { tell },
    );

    expect(screen.getByText("Points at /old/debug/tisty, which is no longer there")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Connect the 1 found" })).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Fix" }));
    const ask = within(screen.getByRole("region", { name: "Fix gemini: connection and hooks" }));
    expect(ask.getByText("/home/someone/gemini.json")).toBeTruthy();
    expect(ask.getByText("/home/someone/hooks.json")).toBeTruthy();
    expect(document.activeElement?.getAttribute("aria-label")).toBe(
      "Fix gemini: connection and hooks",
    );
    await userEvent.click(ask.getByRole("button", { name: "Not now" }));
    expect(sent("mend")).toHaveLength(0);

    await userEvent.click(screen.getByRole("button", { name: "Fix" }));
    await userEvent.click(screen.getByRole("button", { name: "Yes, fix" }));
    await waitFor(() => expect(tell).toHaveBeenCalled());
    expect(sent("mend")[0].args).toEqual({
      id: "gemini",
      ask: { connection: "/old/debug/tisty", hooks: ["/old/tisty"], hooksOut: false },
    });
    expect(tell.mock.calls[0][0].text).toBe(
      "Fixed in gemini: connection and hooks. Quit it completely and open it again.",
    );
  });

  it("fixes hooks left behind by a Tisty that is gone, with the connection already right", async () => {
    const fail = vi.fn();
    ipc.answer = (cmd) => {
      if (cmd === "wsl_distros") return Promise.resolve([]);
      if (cmd === "mend") return Promise.reject({ code: "cannotWrite", name: "disk full" });
      return Promise.resolve([]);
    };
    const hooks = { ...lostHooks, plugged: true };
    shown([one("cursor", { wired: true, hooks })], { fail });

    expect(screen.getByText("They run a Tisty that is gone")).toBeTruthy();
    expect(
      screen.getByText("Its hooks point at /old/tisty, which is no longer there"),
    ).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /^Fix · Hooks/ }));
    expect(screen.getByText(t("mendHooksGo"))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Yes, fix" }));

    await waitFor(() => expect(fail).toHaveBeenCalled());
    expect(sent("mend")[0].args.ask).toMatchObject({ hooks: ["/old/tisty"], hooksOut: true });
    expect(fail.mock.calls[0][0].text).toMatch(/^cursor · /);
    expect(sent("wiring")).toHaveLength(1);
  });

  it("leaves one written the person's own way, and one that cannot be read, as they are", () => {
    shown([
      one("cursor", { apart: true, points: "~/.local/bin/tisty" }),
      one("zed", { apart: true }),
      one("codex", { unread: true }),
      one("gemini", { hooks: { on: false, at: "/hooks.json", unread: true } }),
      one("kiro", { wired: true, hooks: strayHooks("/other/tisty", false) }),
    ]);

    expect(screen.getAllByText("Set up your own way")).toHaveLength(2);
    expect(screen.getByText(/says ~\/.local\/bin\/tisty/)).toBeTruthy();
    expect(screen.getByText(t("wiringApartPlain"))).toBeTruthy();
    expect(screen.getAllByText("Could not be read")).toHaveLength(2);
    expect(screen.getByText(/^\/hooks.json could not be read/)).toBeTruthy();
    expect(screen.getByText("They run another Tisty")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Connect the 1 found" })).toBeTruthy();
  });

  it("shows how to connect one Tisty cannot write to, and each agent's skills beside its hooks", () => {
    shown([one("goose", { byHand: true }), one("claude-code", { wired: true, hooks: lostHooks })], {
      skills: skillsFor({ plugged: ["claude"] }),
      setSkills: () => {},
    });

    const ways = screen.getAllByRole("button", { name: "How to connect it by hand" });
    expect(ways.map((way) => way.textContent)).toEqual(["How to connect it", "By hand"]);
    expect(screen.getByText("The Tisty plugin brings them")).toBeTruthy();
    expect(screen.getByRole("button", { name: "See · Tisty's skills" })).toBeTruthy();
  });

  it.each([
    [
      skillsFor({ on: true, landing: [{ assistant: "cursor", at: "/s", standing: "upToDate" }] }),
      "cursor",
      "On this computer",
    ],
    [skillsFor({ on: true }), "cursor", "Tisty found no folder of this agent to put them in."],
    [
      skillsFor({
        on: true,
        skills: [{ name: "tisty", label: "tisty", about: null, files: [], yours: true }],
      }),
      "cursor",
      "You already have skills with those names in Agents, so Tisty's stay out.",
    ],
    [
      skillsFor({
        on: true,
        landing: [{ assistant: "cursor", at: "/s", standing: "upToDate" }],
        skills: [{ name: "tisty", label: "tisty", about: null, files: [], yours: true }],
      }),
      "cursor",
      "Not all up to date",
    ],
    [skillsFor(), "cursor", "Off"],
    [
      skillsFor({ on: true, landing: [{ assistant: "codex", at: "/s", standing: "write" }] }),
      "gemini-cli",
      "To bring up to date",
    ],
    [
      skillsFor({ on: true, landing: [{ assistant: "cursor", at: "/s", standing: "foreign" }] }),
      "cursor",
      "A skill that is not Tisty's: left as it is",
    ],
  ])("says how Tisty's skills stand on that agent", (skills, id, said) => {
    shown([one(id, { wired: true })], { skills, setSkills: () => {} });
    expect(screen.getByText(said)).toBeTruthy();
  });

  it("never starts a WSL distro on its own, even one looked into before", async () => {
    window.localStorage.setItem("tisty.wsl.looked", '["Ubuntu"]');
    ipc.answer = (cmd) => Promise.resolve(cmd === "wsl_distros" ? ["Ubuntu"] : []);
    shown([], { wakes: false });

    expect(await screen.findByRole("button", { name: "Look in Ubuntu" })).toBeTruthy();
    expect(sent("wsl_seen")).toHaveLength(0);
  });

  it("looks again for assistants without closing the window", async () => {
    shown([one("codex")]);
    const before = sent("wiring").length;

    await userEvent.click(screen.getByRole("button", { name: "Look again" }));

    await waitFor(() => expect(sent("wiring").length).toBe(before + 1));
  });

  it("shows how to connect one by hand, its own command and its lines", async () => {
    shown([one("codex")]);

    await userEvent.click(screen.getByRole("button", { name: "How to connect it by hand" }));

    expect(screen.getByText("codex mcp add tisty -- /opt/tisty mcp")).toBeTruthy();
    expect(screen.getByText(/Or in \/home\/someone\/codex.json, add:/)).toBeTruthy();
  });

  it("looks into a WSL distro only when asked and remembers it was asked", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "wsl_distros") return Promise.resolve(["Ubuntu"]);
      if (cmd === "wsl_seen")
        return Promise.resolve({
          distro: { name: "Ubuntu", shut: false, command: "/mnt/c/tisty.exe" },
          seen: [one("wsl:Ubuntu:claude-code", { name: "Claude Code", place: "Ubuntu" })],
        });
      return Promise.resolve([]);
    };
    shown([]);

    await userEvent.click(await screen.findByRole("button", { name: "Look in Ubuntu" }));

    expect(await screen.findByText("Claude Code")).toBeTruthy();
    expect(sent("wsl_seen")[0].args.name).toBe("Ubuntu");
    expect(window.localStorage.getItem("tisty.wsl.looked")).toBe('["Ubuntu"]');
  });

  it("says plainly when a distro cannot run Windows programs", async () => {
    window.localStorage.setItem("tisty.wsl.looked", '["Ubuntu"]');
    ipc.answer = (cmd) => {
      if (cmd === "wsl_distros") return Promise.resolve(["Ubuntu"]);
      if (cmd === "wsl_seen")
        return Promise.resolve({ distro: { name: "Ubuntu", shut: true }, seen: [] });
      return Promise.resolve([]);
    };
    shown([]);

    expect(await screen.findByText(/Windows programs are turned off/)).toBeTruthy();
  });

  it("brings the skills up to date while fixing one agent, and says so", async () => {
    const tell = vi.fn();
    ipc.answer = (cmd, args) => {
      if (cmd === "wsl_distros") return Promise.resolve([]);
      if (cmd === "agents_tisty_round") return Promise.resolve(skillsFor({ on: true }));
      return Promise.resolve({ seen: [one(String(args.id), { wired: true })] });
    };
    const landing = [{ assistant: "cursor", at: "/s/skills/tisty/SKILL.md", standing: "write" }];
    shown([one("cursor", { astray: true, gone: true, points: "/old/tisty" })], {
      skills: skillsFor({ on: true, landing }),
      setSkills: () => {},
      tell,
    });
    await userEvent.click(screen.getByRole("button", { name: "Fix" }));
    expect(screen.getByText("/s/skills")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Yes, fix" }));
    await waitFor(() => expect(tell).toHaveBeenCalled());
    expect(sent("agents_tisty_round")).toHaveLength(1);
    expect(tell.mock.calls[0][0].text).toContain("Tisty's skills: up to date");
  });

  it("fixes one inside a distro there", async () => {
    const broken = one("wsl:Ubuntu:codex", {
      name: "Codex",
      place: "Ubuntu",
      astray: true,
      points: "/mnt/c/old/tisty.exe",
    });
    ipc.answer = (cmd) => {
      if (cmd === "wsl_distros") return Promise.resolve(["Ubuntu"]);
      if (cmd === "wsl_seen")
        return Promise.resolve({ distro: { name: "Ubuntu", shut: false }, seen: [broken] });
      if (cmd === "mend") return Promise.reject({ code: "wslUnread", name: "Ubuntu" });
      return Promise.resolve([]);
    };
    shown([]);
    await userEvent.click(await screen.findByRole("button", { name: "Look in Ubuntu" }));
    await userEvent.click(await screen.findByRole("button", { name: "Fix" }));
    await userEvent.click(screen.getByRole("button", { name: "Yes, fix" }));
    await waitFor(() => expect(sent("wsl_seen")).toHaveLength(2));
    expect(sent("mend")[0].args.id).toBe("wsl:Ubuntu:codex");
    expect(sent("agents_tisty_round")).toHaveLength(0);
  });

  it("finds an assistant Tisty knows but did not see, to connect by hand", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "wiring_known") return Promise.resolve([one("zed", { name: "Zed" })]);
      return Promise.resolve([]);
    };
    shown([]);

    const search = screen.getByRole("searchbox");
    await userEvent.click(search);
    await userEvent.type(search, "ze");

    expect(await screen.findByText("Zed")).toBeTruthy();
  });

  it("adds an assistant's hooks only after the person reads where and agrees", async () => {
    const hooks = { on: false, at: "/home/someone/.claude/settings.json" };
    ipc.answer = (cmd) => {
      if (cmd === "wsl_distros") return Promise.resolve([]);
      if (cmd === "hook")
        return Promise.resolve([
          one("claude-code", { wired: true, hooks: { ...hooks, on: true } }),
        ]);
      return Promise.resolve([]);
    };
    shown([one("claude-code", { wired: true, hooks }), one("codex", { wired: true })]);

    expect(screen.getAllByText("Hooks: when a turn starts and ends")).toHaveLength(1);
    await userEvent.click(screen.getByRole("button", { name: /^Add · Hooks/ }));
    expect(sent("hook")).toHaveLength(0);
    expect(
      screen.getByText(/Tisty will add its hooks to \/home\/someone\/.claude\/settings.json/),
    ).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: "Add them" }));

    await waitFor(() => expect(sent("hook")).toHaveLength(1));
    expect(sent("hook")[0].args.id).toBe("claude-code");
  });

  it("never offers hooks to an assistant that is not connected, and removes them in one click", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "wsl_distros") return Promise.resolve([]);
      if (cmd === "unhook") return Promise.resolve([]);
      return Promise.resolve([]);
    };
    shown([
      one("cursor", { hooks: { on: false, at: "/home/someone/.cursor/hooks.json" } }),
      one("gemini-cli", {
        wired: true,
        hooks: { on: true, at: "/home/someone/.gemini/settings.json" },
      }),
    ]);

    expect(screen.getAllByText("Hooks: when a turn starts and ends")).toHaveLength(1);
    expect(screen.getByText("Added")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /^Remove · Hooks/ }));

    await waitFor(() => expect(sent("unhook")).toHaveLength(1));
    expect(sent("unhook")[0].args.id).toBe("gemini-cli");
  });

  it("says the Tisty plugin brings Claude Code's hooks and offers none of its own", async () => {
    shown([
      one("claude-code", {
        wired: true,
        hooks: { on: false, at: "/home/someone/.claude/settings.json", plugged: true },
      }),
    ]);

    expect(screen.getByText("The Tisty plugin brings them")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /^Add · Hooks/ })).toBeNull();
    expect(screen.queryByText(/run twice/)).toBeNull();
  });

  it("warns that hooks run twice when Tisty's and the plugin's are both there", async () => {
    shown([
      one("claude-code", {
        wired: true,
        hooks: { on: true, at: "/home/someone/.claude/settings.json", plugged: true },
      }),
    ]);

    expect(screen.getByText(/The Tisty plugin brings them too, so they run twice/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /^Remove · Hooks/ }));
    await waitFor(() => expect(sent("unhook")).toHaveLength(1));
  });

  it("still offers to remove hooks left on an assistant that is no longer connected", async () => {
    ipc.answer = (cmd) => Promise.resolve(cmd === "wsl_distros" ? [] : []);
    shown([one("claude-code", { hooks: { on: true, at: "/home/someone/.claude/settings.json" } })]);

    await userEvent.click(screen.getByRole("button", { name: /^Remove · Hooks/ }));

    await waitFor(() => expect(sent("unhook")).toHaveLength(1));
  });
});

const tistySkills = (more: Record<string, unknown> = {}) => ({
  version: "1.50.0",
  on: false,
  leftover: [],
  skills: [{ name: "tisty", label: "tisty", about: null, files: ["tisty/SKILL.md"], yours: false }],
  landing: [],
  plugged: [],
  ...more,
});

const joining = (after: Wired, skills: unknown) => {
  ipc.answer = (cmd) => {
    if (cmd === "wsl_distros") return Promise.resolve([]);
    if (cmd === "wire") return Promise.resolve([after]);
    if (cmd === "unwire") return Promise.resolve([{ ...after, wired: false }]);
    if (cmd === "agents_tisty_skills") return Promise.resolve(skills);
    return Promise.resolve([]);
  };
};

describe("taking Tisty out of an agent", () => {
  const hooks = { on: true, at: "/home/someone/settings.json" };
  const remove = () => userEvent.click(screen.getByRole("button", { name: "Remove" }));

  it("asks first, naming both files it writes, also when it was set up the person's own way", async () => {
    for (const more of [{ wired: true }, { apart: true, points: "npx" }]) {
      ipc.calls = [];
      const { unmount } = shown([one("claude-code", { ...more, hooks })]);
      await remove();
      expect(sent("unwire")).toHaveLength(0);
      const ask = within(screen.getByRole("region", { name: "Remove Tisty from claude-code" }));
      expect(ask.getByText("/home/someone/claude-code.json")).toBeTruthy();
      expect(ask.getByText("/home/someone/settings.json")).toBeTruthy();
      await userEvent.click(ask.getByRole("button", { name: "Not now" }));
      expect(screen.queryByRole("region")).toBeNull();

      await remove();
      await userEvent.click(screen.getByRole("button", { name: "Yes, remove" }));
      await waitFor(() => expect(sent("unwire")).toHaveLength(1));
      unmount();
    }
  });

  it("names only the file that holds something of Tisty's", async () => {
    shown([one("cursor", { wired: true, hooks: { ...hooks, on: false } })]);
    await remove();
    expect(screen.queryByText("/home/someone/settings.json")).toBeNull();
  });

  it("looks again when it could not be done whole", async () => {
    const trouble = vi.fn();
    ipc.answer = (cmd) => {
      if (cmd === "wsl_distros") return Promise.resolve([]);
      if (cmd === "unwire") return Promise.reject({ code: "settingsUnread", name: "/s.json" });
      return Promise.resolve([one("codex")]);
    };
    const setAgents = vi.fn();
    shown([one("codex", { wired: true })], {
      setAgents,
      quietly: ((_card: string, work: Promise<unknown>) => void work.catch(trouble)) as never,
    });
    await remove();
    await userEvent.click(screen.getByRole("button", { name: "Yes, remove" }));
    await waitFor(() => expect(trouble).toHaveBeenCalled());
    await waitFor(() => expect(setAgents).toHaveBeenCalledWith([one("codex")]));
  });
});

describe("hooks the person runs their own way", () => {
  const hooks = { on: false, apart: true, at: "/home/someone/settings.json" };

  it("are said, and neither taken out nor joined by Tisty's", () => {
    shown([one("claude-code", { wired: true, hooks })]);
    expect(screen.getByText("Set up your own way")).toBeTruthy();
    expect(screen.getByText(t("hooksOwnWayWhy"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: /· Hooks/ })).toBeNull();
  });

  it("stay when Tisty's are removed, and it says so", async () => {
    const tell = vi.fn();
    ipc.answer = (cmd) =>
      Promise.resolve(cmd === "unhook" ? [one("claude-code", { wired: true, hooks })] : []);
    shown([one("claude-code", { wired: true, hooks: { ...hooks, on: true } })], { tell });
    await userEvent.click(screen.getByRole("button", { name: /^Remove · Hooks/ }));
    await waitFor(() =>
      expect(tell).toHaveBeenCalledWith({ card: "wiring", text: t("hooksGoneApart") }),
    );
  });

  it("are not said to be gone when they read as still there", async () => {
    const fail = vi.fn();
    const still = one("claude-code", { wired: true, hooks: { ...hooks, on: true } });
    ipc.answer = (cmd) => Promise.resolve(cmd === "unhook" ? [still] : []);
    shown([still], { fail });
    await userEvent.click(screen.getByRole("button", { name: /^Remove · Hooks/ }));
    await waitFor(() =>
      expect(fail).toHaveBeenCalledWith({
        card: "wiring",
        text: fill("hooksPuzzling", hooks.at),
      }),
    );
  });
});

function Live({ first }: Readonly<{ first: Wired[] }>) {
  const [agents, setAgents] = useState(first);
  const quietly = <T,>(
    _card: string,
    work: Promise<T> | (() => Promise<T>),
    then: (answer: T) => void,
  ) => {
    void (typeof work === "function" ? work() : work).then(then);
  };
  return (
    <Wiring
      agents={agents}
      setAgents={setAgents}
      hands={[]}
      held={false}
      quietly={quietly as never}
      tell={() => {}}
      fail={() => {}}
      wroteSaid={() => ""}
    />
  );
}

const live = (first: Wired[]) => render(<Live first={first} />);

describe("right after connecting an agent", () => {
  const hooks = { on: false, at: "/home/someone/.claude/settings.json" };

  it("offers its hooks, still asking first, and Tisty's skills through their own card", async () => {
    joining(one("claude-code", { wired: true, hooks }), tistySkills());
    const first = vi.fn();
    window.addEventListener(AGENTS_FIRST, first);
    live([one("claude-code", { hooks })]);
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));

    expect(
      await screen.findByText("claude-code is connected. This helps it work with Tisty:"),
    ).toBeTruthy();
    expect(screen.getByText(/With its hooks, claude-code tells Tisty/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /^Add · Hooks/ }));
    expect(sent("hook")).toHaveLength(0);
    expect(screen.getByText(/Tisty will add its hooks to/)).toBeTruthy();

    await userEvent.click(await screen.findByRole("button", { name: "See Tisty's skills" }));
    expect(screen.getByRole("button", { name: "Turn on here" })).toBeTruthy();
    expect(first).not.toHaveBeenCalled();
    expect(sent("agents_tisty_skills_take")).toHaveLength(0);
    window.removeEventListener(AGENTS_FIRST, first);
  });

  it("offers only Tisty's skills to an agent without hooks, through their card", async () => {
    joining(one("codex", { wired: true }), tistySkills());
    live([one("codex")]);
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));

    expect(await screen.findByText(/Tisty's skills teach codex to pick up/)).toBeTruthy();
    expect(screen.queryByText(/With its hooks/)).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "See Tisty's skills" }));
    expect(screen.getByRole("button", { name: "Turn on here" })).toBeTruthy();
    expect(sent("agents_tisty_skills_take")).toHaveLength(0);
  });

  it("says the plugin brings what it already brings", async () => {
    joining(
      one("claude-code", { wired: true, hooks: { ...hooks, plugged: true } }),
      tistySkills({ plugged: ["claude"] }),
    );
    live([one("claude-code", { hooks })]);
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));

    expect(await screen.findByText("The Tisty plugin already brings its hooks.")).toBeTruthy();
    expect(
      await screen.findByText("claude-code already has Tisty's skills through the Tisty plugin."),
    ).toBeTruthy();
    expect(screen.queryByRole("button", { name: "See Tisty's skills" })).toBeNull();
  });

  it("takes «Not now» and never offers again, and an agent already connected is never offered", async () => {
    joining(one("codex", { wired: true }), tistySkills({ on: true }));
    live([one("codex"), one("cursor", { wired: true })]);
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));
    expect(await screen.findByText("Tisty's skills are already on this computer.")).toBeTruthy();
    expect(screen.getAllByRole("region")).toHaveLength(1);
    await userEvent.click(screen.getByRole("button", { name: "Not now" }));
    expect(screen.queryByRole("region")).toBeNull();

    await userEvent.click(screen.getAllByRole("button", { name: "Remove" })[0]);
    await userEvent.click(screen.getByRole("button", { name: "Yes, remove" }));
    await userEvent.click(await screen.findByRole("button", { name: "Connect" }));
    await waitFor(() => expect(sent("wire")).toHaveLength(2));
    expect(screen.queryByRole("region")).toBeNull();
  });
});

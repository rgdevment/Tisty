import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { type ComponentProps, useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { offeredHere } from "../agentsOffered";
import type { TistySkills } from "../agentsSync";
import type { Wired } from "../core";
import { adopt, fill, t } from "../locales";
import { trustedByHand } from "../ui/Hooking";
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

describe("what a row says of a connection that leads to another Tisty", () => {
  it("says it uses the other install while that one is still there, and points at another only once it is gone", () => {
    shown([
      one("codex", { astray: true, gone: false, points: "/other/tisty" }),
      one("gemini", { astray: true, gone: true, points: "/old/tisty" }),
    ]);

    expect(screen.getAllByText("Uses your other Tisty install")).toHaveLength(1);
    expect(screen.getAllByText("Points at another Tisty")).toHaveLength(1);
    expect(screen.getByText("Points at /old/tisty, which is no longer there")).toBeTruthy();
  });
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
    await userEvent.click(screen.getByRole("button", { name: /^Fix · Tisty.s hooks/ }));
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
    expect(screen.queryByRole("switch", { name: /^Tisty's skills/ })).toBeNull();
  });

  it.each([
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

  it("turns Tisty's skills off in one press, and on only from the card that says where", async () => {
    const setSkills = vi.fn();
    const off = skillsFor();
    ipc.answer = (cmd) => Promise.resolve(cmd === "agents_tisty_skills_take" ? off : []);
    const landing = [{ assistant: "cursor", at: "/s/skills/tisty/SKILL.md", standing: "upToDate" }];
    const on = skillsFor({ on: true, landing });
    const { unmount } = shown([one("cursor", { wired: true })], { skills: on, setSkills });
    await userEvent.click(
      screen.getByRole("switch", { name: "Tisty's skills · cursor", checked: true }),
    );
    await waitFor(() => expect(setSkills).toHaveBeenCalledWith(off));
    expect(sent("agents_tisty_skills_take")[0].args).toEqual({ on: false });
    unmount();

    shown([one("cursor", { wired: true })], { skills: off, setSkills });
    await userEvent.click(
      screen.getByRole("switch", { name: "Tisty's skills · cursor", checked: false }),
    );
    expect(sent("agents_tisty_skills_take")).toHaveLength(1);
    await userEvent.click(screen.getByRole("button", { name: "Turn on here" }));
    await waitFor(() => expect(sent("agents_tisty_skills_take")).toHaveLength(2));
    expect(sent("agents_tisty_skills_take")[1].args).toEqual({ on: true });
  });

  it("says why Tisty's skills could not be turned off", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_tisty_skills_take" ? Promise.reject("no luck") : Promise.resolve([]);
    shown([one("cursor", { wired: true })], {
      skills: skillsFor({ on: true }),
      setSkills: () => {},
    });
    await userEvent.click(screen.getByRole("switch", { name: "Tisty's skills · cursor" }));
    expect(await screen.findByText(/no luck/)).toBeTruthy();
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

    expect(screen.getAllByText("Tisty's hooks")).toHaveLength(1);
    await userEvent.click(
      screen.getByRole("switch", { name: "Tisty's hooks · claude-code", checked: false }),
    );
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

    expect(screen.getAllByText("Tisty's hooks")).toHaveLength(1);
    await userEvent.click(
      screen.getByRole("switch", { name: "Tisty's hooks · gemini-cli", checked: true }),
    );

    await waitFor(() => expect(sent("unhook")).toHaveLength(1));
    expect(sent("unhook")[0].args.id).toBe("gemini-cli");
  });

  const added = { on: true, at: "/home/someone/settings.json" };

  it("holds all of an agent's hooks in one switch, and says when the newest is missing", () => {
    shown([
      one("gemini-cli", { wired: true, hooks: added }),
      one("copilot", { wired: true, hooks: { ...added, lacksBefore: true } }),
      one("claude-code", { wired: true, hooks: { ...added, plugged: true, lacksBefore: true } }),
      one("cursor", { wired: true, hooks: { ...added, apart: true, lacksBefore: true } }),
    ]);
    const hooks = screen.getAllByRole("switch", { name: /^Tisty's hooks/ });
    expect(hooks.map((one) => one.getAttribute("aria-checked"))).toEqual(Array(4).fill("true"));
    expect(screen.getAllByText(t("hooksMissOne"))).toHaveLength(1);
    expect(screen.queryByRole("button", { name: "Remove this one" })).toBeNull();
  });

  const bare = { on: false, at: "/home/someone/hooks.json" };
  const joinable = (hook: () => Promise<unknown>) => {
    ipc.answer = (cmd, args) => {
      if (cmd === "wire") return Promise.resolve([one(String(args.id), { wired: true })]);
      if (cmd === "hook") return hook();
      return Promise.resolve([]);
    };
  };

  it("connects an agent with its hooks in one yes that shows both files first", async () => {
    joinable(() => Promise.resolve([one("cursor", { wired: true, hooks: { ...bare, on: true } })]));
    shown([one("cursor", { hooks: bare })]);

    await userEvent.click(screen.getByRole("button", { name: "Connect" }));
    expect(sent("wire")).toHaveLength(0);
    const asked = screen.getByRole("region", { name: "Connect cursor to Tisty" });
    expect(within(asked).getByText("/home/someone/cursor.json")).toBeTruthy();
    expect(within(asked).getByText("/home/someone/hooks.json")).toBeTruthy();

    await userEvent.click(within(asked).getByRole("button", { name: "Yes, connect" }));
    await waitFor(() => expect(sent("hook")).toHaveLength(1));
    expect(sent("wire")).toHaveLength(1);
  });

  it("connects without hooks when the person unticks them", async () => {
    joinable(() => Promise.resolve([]));
    shown([one("cursor", { hooks: bare })]);

    await userEvent.click(screen.getByRole("button", { name: "Connect" }));
    await userEvent.click(screen.getByRole("checkbox"));
    expect(screen.queryByText("/home/someone/hooks.json")).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Yes, connect" }));

    await waitFor(() => expect(sent("wire")).toHaveLength(1));
    expect(sent("hook")).toHaveLength(0);
  });

  it("stays connected and says why when its hooks could not be added", async () => {
    joinable(() => Promise.reject({ code: "hooksByPlugin" }));
    const told: string[] = [];
    shown([one("claude-code", { hooks: bare })], {
      tell: (word) => void told.push(JSON.stringify(word)),
    });

    await userEvent.click(screen.getByRole("button", { name: "Connect" }));
    await userEvent.click(screen.getByRole("button", { name: "Yes, connect" }));

    await waitFor(() => expect(told).toHaveLength(1));
    expect(told[0]).toContain("claude-code");
    expect(told[0]).toContain("plugin");
  });

  it("connects one without hooks in a single click", async () => {
    joinable(() => Promise.resolve([]));
    shown([one("codex")]);

    await userEvent.click(screen.getByRole("button", { name: "Connect" }));

    await waitFor(() => expect(sent("wire")).toHaveLength(1));
  });

  it("connects all the found ones with the hooks of those that run them", async () => {
    joinable(() => Promise.resolve([]));
    shown([one("cursor", { installed: true, hooks: bare }), one("codex", { installed: true })]);

    await userEvent.click(screen.getByRole("button", { name: /^Connect the 2/ }));
    expect(sent("wire")).toHaveLength(0);
    await userEvent.click(screen.getByRole("button", { name: "Yes, connect" }));

    await waitFor(() => expect(sent("wire")).toHaveLength(2));
    await waitFor(() => expect(sent("hook")).toHaveLength(1));
    expect(sent("hook")[0].args.id).toBe("cursor");
  });

  it("says the Tisty plugin brings Claude Code's hooks and offers none of its own", async () => {
    shown([
      one("claude-code", {
        wired: true,
        hooks: { on: false, at: "/home/someone/.claude/settings.json", plugged: true },
      }),
    ]);

    expect(screen.getByText("The Tisty plugin brings them")).toBeTruthy();
    expect(screen.queryByRole("switch", { name: /^Tisty.s hooks/ })).toBeNull();
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
    await userEvent.click(screen.getByRole("switch", { name: /^Tisty.s hooks/, checked: true }));
    await waitFor(() => expect(sent("unhook")).toHaveLength(1));
  });

  it("still offers to remove hooks left on an assistant that is no longer connected", async () => {
    ipc.answer = (cmd) => Promise.resolve(cmd === "wsl_distros" ? [] : []);
    shown([one("claude-code", { hooks: { on: true, at: "/home/someone/.claude/settings.json" } })]);

    await userEvent.click(screen.getByRole("switch", { name: /^Tisty.s hooks/, checked: true }));

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
    expect(screen.queryByRole("switch", { name: /^Tisty.s hooks/ })).toBeNull();
  });

  it("stay when Tisty's are removed, and it says so", async () => {
    const tell = vi.fn();
    ipc.answer = (cmd) =>
      Promise.resolve(cmd === "unhook" ? [one("claude-code", { wired: true, hooks })] : []);
    shown([one("claude-code", { wired: true, hooks: { ...hooks, on: true } })], { tell });
    await userEvent.click(screen.getByRole("switch", { name: /^Tisty.s hooks/, checked: true }));
    await waitFor(() =>
      expect(tell).toHaveBeenCalledWith({ card: "wiring", text: t("hooksGoneApart") }),
    );
  });

  it("are not said to be gone when they read as still there", async () => {
    const fail = vi.fn();
    const still = one("claude-code", { wired: true, hooks: { ...hooks, on: true } });
    ipc.answer = (cmd) => Promise.resolve(cmd === "unhook" ? [still] : []);
    shown([still], { fail });
    await userEvent.click(screen.getByRole("switch", { name: /^Tisty.s hooks/, checked: true }));
    await waitFor(() =>
      expect(fail).toHaveBeenCalledWith({
        card: "wiring",
        text: fill("hooksPuzzling", hooks.at),
      }),
    );
  });
});

function Live({ first, taught }: Readonly<{ first: Wired[]; taught: TistySkills }>) {
  const [agents, setAgents] = useState(first);
  const [skills, setSkills] = useState(taught);
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
      skills={skills}
      setSkills={setSkills}
      held={false}
      quietly={quietly as never}
      tell={(word) => said.push(JSON.stringify(word))}
      fail={() => {}}
      wroteSaid={() => ""}
    />
  );
}

const said: string[] = [];

const live = (first: Wired[], taught = tistySkills() as TistySkills) => {
  said.length = 0;
  return render(<Live first={first} taught={taught} />);
};

describe("right after connecting an agent", () => {
  const hooks = { on: false, at: "/home/someone/.claude/settings.json" };
  const landing = [
    { assistant: "claude", at: "/home/someone/.claude/skills/tisty/SKILL.md", standing: "write" },
  ];
  const off = tistySkills({ landing }) as TistySkills;
  const on = tistySkills({
    on: true,
    landing: landing.map((one) => ({ ...one, standing: "upToDate" })),
  }) as TistySkills;
  const claude = one("claude-code", { wired: true, hooks: { ...hooks, on: true } });
  const answering = (take: () => Promise<unknown>) => {
    ipc.answer = (cmd) => {
      if (cmd === "wire" || cmd === "hook") return Promise.resolve([claude]);
      if (cmd === "agents_tisty_skills_take") return take();
      return Promise.resolve([]);
    };
  };
  const knob = (named: string) => screen.findByRole("switch", { name: `${named} · claude-code` });

  it("leaves its hooks and Tisty's skills on, with one yes that shows where both go", async () => {
    answering(() => Promise.resolve(on));
    live([one("claude-code", { hooks })], off);
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));
    const asked = screen.getByRole("region", { name: "Connect claude-code to Tisty" });
    expect(within(asked).getByText("/home/someone/.claude/skills")).toBeTruthy();
    expect(within(asked).getByText(t("tistySkillsAsk"))).toBeTruthy();
    expect(sent("agents_tisty_skills_take")).toHaveLength(0);
    await userEvent.click(within(asked).getByRole("button", { name: "Yes, connect" }));

    expect((await knob("Tisty's hooks")).getAttribute("aria-checked")).toBe("true");
    await waitFor(async () =>
      expect((await knob("Tisty's skills")).getAttribute("aria-checked")).toBe("true"),
    );
    expect(sent("agents_tisty_skills_take")[0].args).toEqual({ on: true });
    expect(sent("wire")).toHaveLength(1);
  });

  it("leaves Tisty's skills off when they are unticked", async () => {
    answering(() => Promise.resolve(on));
    live([one("claude-code", { hooks })], off);
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));
    await userEvent.click(screen.getByRole("checkbox", { name: /^Tisty's skills/ }));
    await userEvent.click(screen.getByRole("button", { name: "Yes, connect" }));

    expect((await knob("Tisty's skills")).getAttribute("aria-checked")).toBe("false");
    expect(sent("agents_tisty_skills_take")).toHaveLength(0);
  });

  it("stays connected and says why when Tisty's skills could not be turned on", async () => {
    answering(() => Promise.reject("no luck"));
    live([one("claude-code", { hooks })], off);
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));
    await userEvent.click(screen.getByRole("button", { name: "Yes, connect" }));

    expect((await knob("Tisty's skills")).getAttribute("aria-checked")).toBe("false");
    expect(said.join()).toContain("no luck");
  });

  it("turns them on once for all the found ones", async () => {
    answering(() => Promise.resolve(on));
    live([one("claude-code", { installed: true, hooks }), one("codex", { installed: true })], off);
    await userEvent.click(screen.getByRole("button", { name: /^Connect the 2/ }));
    await userEvent.click(screen.getByRole("button", { name: "Yes, connect" }));
    await waitFor(() => expect(sent("agents_tisty_skills_take")).toHaveLength(1));
    expect(sent("wire")).toHaveLength(2);
  });

  it.each([
    ["they are already on", on, "claude-code"],
    ["the plugin brings them", { ...off, plugged: ["claude"] }, "claude-code"],
    ["it reads no skills", off, "claude-desktop"],
    ["they have nowhere to go", { ...off, landing: [] }, "claude-code"],
  ])("asks nothing of Tisty's skills when %s", async (_, taught, id) => {
    answering(() => Promise.resolve(on));
    live([one(id)], taught);
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));
    await waitFor(() => expect(sent("wire")).toHaveLength(1));
    expect(sent("agents_tisty_skills_take")).toHaveLength(0);
  });
});

describe("the hooks of Codex, which the person approves by hand", () => {
  const hooks = { on: true, at: "/home/someone/.codex/hooks.json" };
  const trust = () => screen.queryByText(t("hooksCodexTrust"));

  it("says how to approve them under Codex's row, in English and in Spanish", () => {
    const { unmount } = shown([one("codex", { wired: true, hooks })]);
    expect(trust()?.textContent).toBe("Approve it in Codex with /hooks.");
    unmount();

    adopt("es");
    shown([one("codex", { wired: true, hooks })]);
    expect(trust()?.textContent).toBe("Apruébalo en Codex con /hooks.");
  });

  it("says it only while they are on", () => {
    shown([one("codex", { wired: true, hooks: { ...hooks, on: false } })]);
    expect(trust()).toBeNull();
  });

  it("says it for Codex in a WSL distro too, and for no other assistant", () => {
    shown([one("wsl:Ubuntu:codex", { wired: true, hooks })]);
    expect(trust()).not.toBeNull();
  });

  it.each(["claude-code", "cursor", "gemini-cli", "copilot", "antigravity"])(
    "does not say it under %s",
    (id) => {
      shown([one(id, { wired: true, hooks })]);
      expect(screen.getAllByRole("switch", { name: /^Tisty.s hooks/ })).toHaveLength(1);
      expect(trust()).toBeNull();
    },
  );

  it("knows Codex by the end of its id", () => {
    expect(trustedByHand(one("codex"))).toBe(true);
    expect(trustedByHand(one("wsl:Ubuntu:codex"))).toBe(true);
    expect(trustedByHand(one("antigravity"))).toBe(false);
    expect(trustedByHand(one("codex-extra"))).toBe(false);
  });

  const toggled = async (id: string, on: boolean) => {
    const tell = vi.fn();
    const now = one(id, { wired: true, hooks: { ...hooks, on } });
    ipc.answer = (cmd) => Promise.resolve(["hook", "unhook"].includes(cmd) ? [now] : []);
    shown([one(id, { wired: true, hooks: { ...hooks, on: !on } })], { tell });
    await userEvent.click(screen.getByRole("switch", { name: /^Tisty.s hooks/, checked: !on }));
    if (on) await userEvent.click(screen.getByRole("button", { name: "Add them" }));
    await waitFor(() => expect(tell).toHaveBeenCalled());
    return tell;
  };

  it("adds the sentence to the toast when Codex's hooks are added", async () => {
    const tell = await toggled("codex", true);

    expect(tell).toHaveBeenCalledWith({
      card: "wiring",
      text: `${t("hooksFresh")} ${t("hooksCodexTrust")}`,
    });
  });

  it("keeps the toast as it was when another assistant's hooks are added", async () => {
    const tell = await toggled("antigravity", true);

    expect(tell).toHaveBeenCalledWith({ card: "wiring", text: t("hooksFresh") });
  });

  it("says nothing of approving when Codex's hooks are removed", async () => {
    const tell = await toggled("codex", false);

    expect(tell).toHaveBeenCalledWith({ card: "wiring", text: t("hooksGone") });
  });
});

describe("the agents already running in a WSL distro", () => {
  const inUbuntu = (more: Partial<Wired> = {}) =>
    one("wsl:Ubuntu:codex", { name: "Codex", place: "Ubuntu", ...more });

  const props = (more: Partial<ComponentProps<typeof Wiring>> = {}) => ({
    agents: [] as Wired[],
    setAgents: () => {},
    hands: [],
    held: false,
    quietly: ((
      _card: string,
      work: Promise<unknown> | (() => Promise<unknown>),
      then: (answer: unknown) => void,
    ) => {
      void (typeof work === "function" ? work() : work).then(then);
    }) as never,
    tell: () => {},
    fail: () => {},
    wroteSaid: () => "nothing written yet",
    ...more,
  });

  const onlyUbuntu = (more: Record<string, unknown> = {}) => {
    ipc.answer = (cmd) => {
      if (cmd === "wsl_distros") return Promise.resolve(["Ubuntu"]);
      if (cmd in more) return Promise.resolve(more[cmd]);
      return Promise.resolve([]);
    };
  };

  const section = async () =>
    within((await screen.findByText("Ubuntu · WSL")).closest("section") as HTMLElement);

  it("shows what the startup look found in its distro, and follows it when it changes", async () => {
    onlyUbuntu();
    const { rerender } = render(<Wiring {...props({ inDistros: [inUbuntu({ wired: true })] })} />);

    const first = await section();
    expect(await first.findByText("Codex")).toBeTruthy();
    expect(first.getByText("Connected")).toBeTruthy();
    expect(first.queryByRole("button", { name: /^Look in/ })).toBeNull();
    expect(sent("wsl_seen")).toHaveLength(0);

    rerender(
      <Wiring
        {...props({ inDistros: [inUbuntu({ astray: true, gone: true, points: "/old/tisty" })] })}
      />,
    );

    await waitFor(async () => expect((await section()).queryByText("Connected")).toBeNull());
    expect((await section()).getByText("Points at another Tisty")).toBeTruthy();
  });

  it("asks the startup look to read again once a fix in one of its rows is answered", async () => {
    const lookDistros = vi.fn();
    onlyUbuntu({ mend: { seen: [inUbuntu({ wired: true })] } });
    render(
      <Wiring
        {...props({
          inDistros: [inUbuntu({ astray: true, gone: true, points: "/old/tisty" })],
          lookDistros,
        })}
      />,
    );

    const rows = await section();
    await userEvent.click(await rows.findByRole("button", { name: "Fix" }));
    expect(lookDistros).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Yes, fix" }));

    await waitFor(() => expect(lookDistros).toHaveBeenCalledTimes(1));
    expect(sent("mend")[0].args.id).toBe("wsl:Ubuntu:codex");
    expect((await section()).getByText("Connected")).toBeTruthy();
  });

  it("asks the startup look to read again after an explicit look into the distro", async () => {
    const lookDistros = vi.fn();
    onlyUbuntu({ wsl_seen: { distro: { name: "Ubuntu", shut: false }, seen: [inUbuntu()] } });
    render(<Wiring {...props({ lookDistros })} />);

    await userEvent.click(await screen.findByRole("button", { name: "Look in Ubuntu" }));

    await waitFor(() => expect(lookDistros).toHaveBeenCalledTimes(1));
    expect(sent("wsl_seen")[0].args.name).toBe("Ubuntu");
  });
});

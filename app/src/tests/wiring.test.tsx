import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AGENTS_FIRST } from "../agentsSync";
import type { Wired } from "../core";
import { adopt } from "../locales";
import Wiring from "../ui/Wiring";

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

const shown = (agents: Wired[]) => {
  const quietly = <T,>(_card: string, work: Promise<T>, then: (answer: T) => void) => {
    void work.then(then);
  };
  render(
    <Wiring
      agents={agents}
      setAgents={() => {}}
      hands={[]}
      held={false}
      quietly={quietly as never}
      tell={() => {}}
      fail={() => {}}
      wroteSaid={() => "nothing written yet"}
    />,
  );
};

beforeEach(() => {
  adopt("en");
  ipc.calls = [];
  ipc.answer = (cmd) => {
    if (cmd === "wsl_distros") return Promise.resolve([]);
    if (cmd === "wire" || cmd === "wiring") return Promise.resolve([]);
    return Promise.resolve(null);
  };
  window.localStorage.clear();
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
    expect(screen.getByText("By hand")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Connect the 2 found" }));

    await waitFor(() => expect(sent("wire").length).toBe(2));
    expect(sent("wire").map((call) => call.args.id)).toEqual(["codex", "copilot"]);
  });

  it("repairs the ones pointing at another Tisty or at one that is gone, apart from the new ones", async () => {
    shown([
      one("codex"),
      one("copilot", { astray: true, points: "/home/someone/.local/bin/tisty-mcp" }),
      one("gemini", { astray: true, gone: true, points: "/old/debug/tisty" }),
    ]);

    expect(screen.getByText("Points at /old/debug/tisty, which is no longer there")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Connect the 1 found" })).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Repair 2" }));

    await waitFor(() => expect(sent("wire").length).toBe(2));
    expect(sent("wire").map((call) => call.args.id)).toEqual(["copilot", "gemini"]);
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

function Live({ first }: Readonly<{ first: Wired[] }>) {
  const [agents, setAgents] = useState(first);
  const quietly = <T,>(_card: string, work: Promise<T>, then: (answer: T) => void) => {
    void work.then(then);
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
    await userEvent.click(await screen.findByRole("button", { name: "Connect" }));
    await waitFor(() => expect(sent("wire")).toHaveLength(2));
    expect(screen.queryByRole("region")).toBeNull();
  });
});

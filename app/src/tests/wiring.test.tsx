import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
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
});

import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AGENTS_CHANGED } from "../agentsSync";
import type { InDistro } from "../agentsWsl";
import { adopt } from "../locales";
import DistroAgents from "../ui/DistroAgents";

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

const distro = (more: Partial<InDistro> = {}): InDistro => ({
  name: "Ubuntu",
  agents: false,
  skills: false,
  running: true,
  looked: false,
  wrote: 0,
  left: 0,
  ...more,
});

const asking = (first: InDistro, rest: Record<string, InDistro> = {}) => {
  ipc.answer = (cmd) => {
    if (cmd === "agents_wsl") return Promise.resolve([first]);
    return Promise.resolve(rest[cmd] ?? first);
  };
};

const shown = (held = false) => {
  const fail = vi.fn();
  render(<DistroAgents name="Ubuntu" held={held} fail={fail} />);
  return fail;
};

const rules = () => screen.findByRole("switch", { name: "Agents' rules and skills" });

beforeEach(() => {
  adopt("en");
  ipc.calls = [];
});

describe("Agents inside a WSL distro", () => {
  it("turns the rules on and off with their own flag and leaves the skills alone", async () => {
    asking(distro(), { agents_wsl_set: distro({ agents: true }) });
    shown();

    await userEvent.click(await rules());
    await waitFor(() => expect(sent("agents_wsl_set")).toHaveLength(1));
    expect(sent("agents_wsl_set")[0].args).toEqual({
      name: "Ubuntu",
      agents: true,
      skills: null,
      wake: false,
    });

    await waitFor(() =>
      expect(
        screen
          .getByRole("switch", { name: "Agents' rules and skills" })
          .getAttribute("aria-checked"),
      ).toBe("true"),
    );
    await userEvent.click(screen.getByRole("switch", { name: "Agents' rules and skills" }));
    await waitFor(() => expect(sent("agents_wsl_set")).toHaveLength(2));
    expect(sent("agents_wsl_set")[1].args.agents).toBe(false);
  });

  it("turns Tisty's skills on with their own flag and leaves the rules alone", async () => {
    asking(distro(), { agents_wsl_set: distro({ skills: true }) });
    shown();

    await userEvent.click(await screen.findByRole("switch", { name: "Tisty's skills" }));

    await waitFor(() => expect(sent("agents_wsl_set")).toHaveLength(1));
    expect(sent("agents_wsl_set")[0].args).toEqual({
      name: "Ubuntu",
      agents: null,
      skills: true,
      wake: false,
    });
  });

  it("waits for a stopped distro with a yes, and starts it only when the button is pressed", async () => {
    asking(distro({ agents: true, running: false }), {
      agents_wsl_round: distro({ agents: true, running: true, looked: true, wrote: 3 }),
    });
    shown();

    expect(await screen.findByText(/Waiting: the distro is not running/)).toBeTruthy();
    expect(sent("agents_wsl_round")).toHaveLength(0);
    await userEvent.click(screen.getByRole("button", { name: "Start Ubuntu and write" }));

    await waitFor(() => expect(sent("agents_wsl_round")).toHaveLength(1));
    expect(sent("agents_wsl_round")[0].args).toEqual({ name: "Ubuntu", wake: true });
    expect(await screen.findByText("3 files written")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Start Ubuntu and write" })).toBeNull();
  });

  it("offers no wake button while nothing is on", async () => {
    asking(distro({ running: false }));
    shown();

    await rules();
    expect(screen.queryByRole("button", { name: "Start Ubuntu and write" })).toBeNull();
    expect(screen.queryByText(/Waiting/)).toBeNull();
    expect(sent("agents_wsl_round")).toHaveLength(0);
  });

  it("catches up a running distro with a yes without starting anything", async () => {
    asking(distro({ skills: true }), {
      agents_wsl_round: distro({ skills: true, looked: true, wrote: 0 }),
    });
    shown();

    expect(await screen.findByText("Up to date")).toBeTruthy();
    expect(sent("agents_wsl_round")).toHaveLength(1);
    expect(sent("agents_wsl_round")[0].args).toEqual({ name: "Ubuntu", wake: false });
  });

  it("says how many files were left as they are", async () => {
    asking(distro({ agents: true }), {
      agents_wsl_round: distro({ agents: true, looked: true, wrote: 1, left: 2 }),
    });
    shown();

    expect(await screen.findByText(/^2 files are left as they are/)).toBeTruthy();
  });

  it("never turns anything on or off by itself", async () => {
    asking(distro({ agents: true, skills: true }), {
      agents_wsl_round: distro({ agents: true, skills: true, looked: true, wrote: 2 }),
    });
    shown();

    await screen.findByText("2 files written");
    expect(sent("agents_wsl_set")).toHaveLength(0);
  });

  it("asks a running distro with no yes nothing but its own state", async () => {
    asking(distro());
    shown();

    await rules();
    expect(sent("agents_wsl")).toHaveLength(1);
    expect(sent("agents_wsl")[0].args).toEqual({ names: ["Ubuntu"] });
    expect(sent("agents_wsl_round")).toHaveLength(0);
    expect(sent("agents_wsl_set")).toHaveLength(0);
  });

  it("holds the switches while Agents is held", async () => {
    asking(distro());
    shown(true);

    expect((await rules()).hasAttribute("disabled")).toBe(true);
    expect(screen.getByRole("switch", { name: "Tisty's skills" }).hasAttribute("disabled")).toBe(
      true,
    );
  });

  it("says why a switch could not be turned", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_wsl" ? Promise.resolve([distro()]) : Promise.reject("not written");
    const fail = shown();

    await userEvent.click(await rules());

    await waitFor(() => expect(fail).toHaveBeenCalled());
    expect(fail.mock.calls[0][0].card).toBe("wiring");
  });

  describe("while the log stirs", () => {
    const stir = () => act(() => void window.dispatchEvent(new Event(AGENTS_CHANGED)));
    const later = (ms: number) => act(() => void vi.advanceTimersByTime(ms));

    const stopped = async () => {
      asking(distro({ agents: true, running: false }));
      const view = render(<DistroAgents name="Ubuntu" held={false} fail={vi.fn()} />);
      await screen.findByText(/Waiting/);
      expect(sent("agents_wsl_round")).toHaveLength(0);
      vi.useFakeTimers({ toFake: ["Date", "setTimeout", "clearTimeout"] });
      return view;
    };

    afterEach(() => vi.useRealTimers());

    it("rounds at once, then holds the rest to one more when the 15 seconds end", async () => {
      await stopped();

      stir();
      stir();
      stir();
      expect(sent("agents_wsl_round")).toHaveLength(1);
      later(14_000);
      stir();
      expect(sent("agents_wsl_round")).toHaveLength(1);
      later(1_500);
      expect(sent("agents_wsl_round")).toHaveLength(2);
      expect(sent("agents_wsl_round")[1].args).toEqual({ name: "Ubuntu", wake: false });
      later(60_000);
      expect(sent("agents_wsl_round")).toHaveLength(2);
    });

    it("gives exactly one extra round for many events inside the window", async () => {
      await stirredOnce();
      for (let i = 0; i < 20; i++) {
        stir();
        later(500);
      }
      expect(sent("agents_wsl_round")).toHaveLength(1);
      later(15_000);
      expect(sent("agents_wsl_round")).toHaveLength(2);
    });

    it("gives none when the distro is left before the window ends", async () => {
      const view = await stirredOnce();
      stir();
      stir();
      view.unmount();
      later(30_000);
      expect(sent("agents_wsl_round")).toHaveLength(1);
    });

    const stirredOnce = async () => {
      const view = await stopped();
      stir();
      expect(sent("agents_wsl_round")).toHaveLength(1);
      return view;
    };
  });
});

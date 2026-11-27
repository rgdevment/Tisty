import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AGENTS_CHANGED, type TistySkills } from "../agentsSync";
import type { Agent, Wired } from "../core";
import { adopt } from "../locales";
import { ownOf } from "../tistyOwn";
import {
  OwnDot,
  ownSaid,
  TistyOwnCard,
  TistyOwnEntry,
  TistyOwnRow,
  TistyOwnSettingsRow,
} from "../ui/TistyOwn";
import { knownFor, skillsFor, wiredOne } from "./knownFor";

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

const claudeAt = "/home/someone/.claude/settings.json";
const codexAt = "/home/someone/.codex/hooks.json";
const door: Agent = { on: true, called: "espino 3" };
const landing = [
  { assistant: "claude", at: "/h/.claude/skills/tisty/SKILL.md", standing: "write" },
];

const claude = (on = false, more: Partial<Wired> = {}) =>
  wiredOne("claude-code", {
    name: "Claude Code",
    wired: true,
    hooks: { on, at: claudeAt },
    ...more,
  });

const codex = (on = false, more: Partial<Wired> = {}) =>
  wiredOne("codex", { name: "Codex", wired: true, hooks: { on, at: codexAt }, ...more });

const off = (more: Partial<TistySkills> = {}) => skillsFor({ on: false, landing, ...more });
const on = (more: Partial<TistySkills> = {}) => skillsFor({ on: true, ...more });

const answering = (skills: TistySkills, more: Record<string, unknown> = {}) => {
  ipc.answer = (cmd) => {
    if (cmd in more) return Promise.resolve(more[cmd]);
    if (cmd === "agents_tisty_skills") return Promise.resolve(skills);
    if (cmd === "agents_tisty_skills_take") return Promise.resolve({ ...skills, on: true });
    if (cmd === "wiring") return Promise.resolve([]);
    return Promise.resolve(null);
  };
};

const card = (
  agents: Wired[],
  skills: TistySkills | undefined,
  more: Partial<Parameters<typeof TistyOwnCard>[0]> = {},
  seen: Agent = door,
) => {
  const own = ownOf({ agent: seen, agents, skills });
  const props = {
    own,
    skills,
    known: knownFor({ agent: seen, agents }),
    onSkills: vi.fn(),
    onHooks: vi.fn(),
    onClose: vi.fn(),
    ...more,
  };
  render(<TistyOwnCard {...props} />);
  return props;
};

const button = (name: string | RegExp) => screen.getByRole("button", { name });

const footer = (name: string) => {
  const all = screen.getAllByRole("button", { name });
  return all[all.length - 1];
};

beforeEach(() => {
  adopt("en");
  ipc.calls = [];
  answering(off());
});

describe("the dot", () => {
  it.each([
    ["on", "bg-hue-green"],
    ["partly", "bg-hue-amber"],
    ["off", "bg-transparent"],
  ] as const)("draws %s as %s", (dot, colour) => {
    const { container } = render(<OwnDot dot={dot} />);
    const mark = container.querySelector(`[data-dot=${dot}]`);
    expect(mark?.className).toContain(colour);
    expect(mark?.getAttribute("aria-hidden")).toBe("true");
  });

  it("says the state in words, in both languages", () => {
    const all = (one: Wired[], skills: TistySkills) =>
      ownSaid(ownOf({ agent: door, agents: one, skills }));
    expect(all([claude(true)], on())).toBe("On");
    expect(all([claude(false)], off({ landing: [] }))).toBe("Off");
    expect(all([claude(false)], on())).toBe("Hooks missing");
    expect(all([claude(true)], off())).toBe("Skills missing");
    expect(all([claude(true), codex(false)], on())).toBe("Hooks only in Claude Code");
    expect(all([claude(false), codex(false)], on())).toBe("Hooks missing");
    expect(
      ownSaid(ownOf({ agent: { ...door, hooks: false }, agents: [claude(true)], skills: on() })),
    ).toBe("Silenced in Door");
    adopt("es");
    expect(all([claude(true)], on())).toBe("Encendidos");
    expect(all([claude(false)], on())).toBe("Faltan los hooks");
  });
});

describe("the row of the tree", () => {
  const row = (agents: Wired[], more: Partial<Parameters<typeof TistyOwnRow>[0]> = {}) =>
    render(
      <ul>
        <TistyOwnRow depth={1} step={12} known={knownFor({ agent: door, agents })} {...more} />
      </ul>,
    );

  it("is one row for skills and hooks, and no separate rows", async () => {
    answering(on());
    row([claude(true), codex(true)]);
    const one = await screen.findByRole("button", { name: /^Tisty's skills and hooks/ });
    expect(screen.getAllByRole("button")).toHaveLength(1);
    expect(one.textContent).toBe("Skills and hooks");
    expect(screen.queryByText("Tisty's skills")).toBeNull();
    expect(screen.queryByText("Tisty's hooks")).toBeNull();
  });

  it.each([
    ["on", [true, true], on(), "Tisty's skills and hooks: On", "bg-hue-green"],
    [
      "partly",
      [true, false],
      on(),
      "Tisty's skills and hooks: Hooks only in Claude Code",
      "bg-hue-amber",
    ],
    ["off", [false, false], off(), "Tisty's skills and hooks: Off", "bg-transparent"],
  ] as const)(
    "shows %s with a dot and says it in the label and the title",
    async (dot, hooks, skills, said, colour) => {
      answering(skills);
      row([claude(hooks[0]), codex(hooks[1])]);
      const one = await screen.findByRole("button", { name: said });
      expect(one.getAttribute("data-dot")).toBe(dot);
      expect(one.getAttribute("title")).toBe(said);
      expect(one.querySelector(`[data-dot=${dot}]`)?.className).toContain(colour);
    },
  );

  it("says it in Spanish", async () => {
    adopt("es");
    answering(on());
    row([claude(true)]);
    const one = await screen.findByRole("button", { name: "Skills y hooks de Tisty: Encendidos" });
    expect(one.textContent).toBe("Skills y hooks");
  });

  it("draws the lead heading with the row, and nothing when there is nothing to show", async () => {
    answering(on());
    const first = row([claude(true)], { lead: <li>De Tisty lead</li> });
    await screen.findByText("De Tisty lead");
    first.unmount();

    answering(on({ skills: [] }));
    const { container } = row([], { lead: <li>De Tisty lead</li> });
    await waitFor(() => expect(sent("agents_tisty_skills")).toHaveLength(2));
    await act(async () => undefined);
    expect(container.querySelectorAll("li")).toHaveLength(0);
    expect(screen.queryByText("De Tisty lead")).toBeNull();
  });

  it("draws nothing when the skills cannot be read and no agent has hooks", async () => {
    ipc.answer = () => Promise.reject(new Error("unreadable"));
    const { container } = row([claude(true, { hooks: null })]);
    await act(async () => undefined);
    expect(container.querySelectorAll("li")).toHaveLength(0);
  });

  it("shows the row for hooks alone and leaves the skills part out when it is told not to read them", async () => {
    answering(on());
    row([claude(true)], { skillsShown: false, onHooks: vi.fn() });
    const one = await screen.findByRole("button", { name: /^Tisty's skills and hooks/ });
    await userEvent.click(one);
    expect(screen.queryByRole("button", { name: "Adjust · Skills" })).toBeNull();
    expect(screen.getByRole("button", { name: "Adjust · Hooks" })).toBeTruthy();
  });

  it("passes what rowOf gives, keyed tisty-own", async () => {
    answering(on());
    const rowOf = vi.fn((key: string) => ({ "data-row": key, tabIndex: -1 }));
    row([claude(true)], { rowOf });
    const one = await screen.findByRole("button", { name: /^Tisty's skills and hooks/ });
    expect(one.getAttribute("data-row")).toBe("tisty-own");
    expect(one.tabIndex).toBe(-1);
    expect(rowOf).toHaveBeenCalledWith("tisty-own");
  });

  it("uses the agents it is given when there is no known list", async () => {
    answering(on());
    render(
      <ul>
        <TistyOwnRow depth={0} step={12} agents={[claude(false)]} />
      </ul>,
    );
    expect(
      await screen.findByRole("button", { name: "Tisty's skills and hooks: Hooks missing" }),
    ).toBeTruthy();
  });

  it("opens the card when pressed and closes it with Not now", async () => {
    answering(on());
    row([claude(true)]);
    await userEvent.click(await screen.findByRole("button", { name: /^Tisty's skills and hooks/ }));
    expect(screen.getByRole("dialog")).toBeTruthy();
    expect(screen.getByText("Tisty in your agents")).toBeTruthy();
    await userEvent.click(footer("Close"));
    expect(screen.queryByRole("dialog")).toBeNull();
  });
});

describe("the entry of the folder page", () => {
  it("is a list item with one button, the name and the dot, and opens the card", async () => {
    answering(on());
    render(
      <ul>
        <TistyOwnEntry
          className="entry"
          known={knownFor({ agent: door, agents: [claude(false)] })}
        />
      </ul>,
    );
    const one = await screen.findByRole("button", {
      name: "Tisty's skills and hooks: Hooks missing",
    });
    expect(one.className).toBe("entry");
    expect(one.getAttribute("data-dot")).toBe("partly");
    expect(one.textContent?.trim()).toBe("Skills and hooks");
    await userEvent.click(one);
    expect(screen.getByText("Tisty in your agents")).toBeTruthy();
  });
});

describe("the row of Settings", () => {
  it("shows the dot and its words, and opens the same card", async () => {
    answering(on());
    render(
      <TistyOwnSettingsRow
        known={knownFor({ agent: door, agents: [claude(true), codex(false)] })}
      />,
    );
    const region = await screen.findByRole("region", { name: "Tisty in your agents" });
    expect(within(region).getByText(/Skills and hooks that help your agents/)).toBeTruthy();
    const see = within(region).getByRole("button", {
      name: "See · Tisty's skills and hooks: Hooks only in Claude Code",
    });
    expect(see.getAttribute("data-dot")).toBe("partly");
    expect(see.querySelector("[data-dot=partly]")).not.toBeNull();
    await userEvent.click(see);
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByText(/Skills teach your agent/)).toBeTruthy();
    expect(within(dialog).getByRole("button", { name: "Turn on what's missing" })).toBeTruthy();
  });

  it("is not drawn when there is nothing to show", async () => {
    answering(on({ skills: [] }));
    const { container } = render(<TistyOwnSettingsRow known={knownFor({ agent: door })} />);
    await waitFor(() => expect(sent("agents_tisty_skills")).toHaveLength(1));
    await act(async () => undefined);
    expect(container.querySelector("section")).toBeNull();
  });
});

describe("the card", () => {
  it("says one What line and one Why line when both parts are there", () => {
    card([claude(false)], off());
    expect(
      screen.getByText(/Skills teach your agent to use Tisty, and hooks remind it on their own/),
    ).toBeTruthy();
    expect(screen.getByText(/We recommend both: your agent picks up/)).toBeTruthy();
    expect(screen.queryByText(/We recommend it:/)).toBeNull();
    expect(screen.getByText("Tisty in your agents")).toBeTruthy();
  });

  it("says the skills line and the one-part Why when only skills are there", () => {
    card([], off());
    expect(screen.getByText("Skills teach your agent to use Tisty.")).toBeTruthy();
    expect(screen.getByText(/We recommend it: your agent picks up/)).toBeTruthy();
    expect(screen.queryByText(/hooks remind it on their own/)).toBeNull();
    expect(screen.queryByText(/We recommend both/)).toBeNull();
    expect(screen.queryByRole("button", { name: "Adjust · Hooks" })).toBeNull();
  });

  it("says the hooks line and the one-part Why when only hooks are there", () => {
    card([claude(false)], undefined);
    expect(
      screen.getByText("Hooks remind your agent on their own where it left off and what to note."),
    ).toBeTruthy();
    expect(screen.getByText(/We recommend it: your agent picks up/)).toBeTruthy();
    expect(screen.queryByText(/Skills teach your agent/)).toBeNull();
  });

  it("says the lines in Spanish", () => {
    adopt("es");
    card([claude(false)], off());
    expect(
      screen.getByText(/Las skills le enseñan a tu agente a usar Tisty, y los hooks/),
    ).toBeTruthy();
    expect(screen.getByText(/Te recomendamos los dos/)).toBeTruthy();
  });

  it("offers Turn on both when both parts can be added and the dot is off", () => {
    card([claude(false)], off());
    expect(button("Turn on both")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Turn on what's missing" })).toBeNull();
  });

  it("offers Turn on what's missing when a part is already on", () => {
    card([claude(true)], off());
    expect(button("Turn on what's missing")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Turn on both" })).toBeNull();
  });

  it("offers Turn on what's missing when only one part can be added", () => {
    card([claude(false)], on());
    expect(button("Turn on what's missing")).toBeTruthy();
  });

  it("has no turn-on button when everything is on", () => {
    card([claude(true)], on());
    expect(screen.queryByRole("button", { name: /Turn on/ })).toBeNull();
    expect(screen.queryByText(/can't turn on what's missing/)).toBeNull();
    expect(footer("Close").textContent).toBe("Close");
  });

  it("closes with Close when all is on, with its own button besides the corner one", async () => {
    const props = card([claude(true)], on());
    expect(screen.getAllByRole("button", { name: "Close" })).toHaveLength(2);
    expect(screen.queryByRole("button", { name: "Not now" })).toBeNull();
    await userEvent.click(footer("Close"));
    expect(props.onClose).toHaveBeenCalledTimes(1);
  });

  it("says Not now, not a second Close, when something can be turned on", () => {
    card([claude(false)], off());
    expect(button("Not now")).toBeTruthy();
    expect(screen.getAllByRole("button", { name: "Close" })).toHaveLength(1);
  });

  it("says Cerrar when all is on and Ahora no when something is missing", () => {
    adopt("es");
    card([claude(true)], on());
    expect(screen.getAllByRole("button", { name: "Cerrar" })).toHaveLength(2);
    expect(screen.queryByRole("button", { name: "Ahora no" })).toBeNull();
  });

  it("says the dot of each part and what is lacking", () => {
    card([claude(true), codex(false)], on());
    const hooks = screen.getByText("Hooks", { selector: "span" }).closest("li") as HTMLElement;
    expect(hooks.querySelector("[data-dot=partly]")).not.toBeNull();
    expect(within(hooks).getByText("Only in Claude Code")).toBeTruthy();
    const skills = screen.getByText("Skills", { selector: "span" }).closest("li") as HTMLElement;
    expect(skills.querySelector("[data-dot=on]")).not.toBeNull();
  });

  it("says Off in the hooks that are off, naming the agents", () => {
    card([claude(false), codex(false)], on());
    expect(screen.getByText("Off in Claude Code, Codex")).toBeTruthy();
  });

  it("says that the skills are on through the plugin when the agents bring them", () => {
    card([claude(true)], off({ plugged: ["claude"] }));
    expect(screen.getByText("Skills", { selector: "span" }).closest("li")?.textContent).toContain(
      "Skills",
    );
  });

  it("opens the skills card from Adjust on the skills, and comes back from it", async () => {
    card([claude(false)], off());
    await userEvent.click(button("Adjust · Skills"));
    expect(screen.getByText("Where they go on this computer")).toBeTruthy();
    expect(screen.queryByText("Tisty in your agents")).toBeNull();
    await userEvent.click(button("Not now"));
    expect(screen.getByText("Tisty in your agents")).toBeTruthy();
  });

  it("calls onHooks and closes when Adjust on the hooks is pressed", async () => {
    const props = card([claude(false)], off());
    await userEvent.click(button("Adjust · Hooks"));
    expect(props.onClose).toHaveBeenCalledTimes(1);
    expect(props.onHooks).toHaveBeenCalledTimes(1);
  });

  it("has no Adjust for the hooks without a way to open them", () => {
    card([claude(false)], off(), { onHooks: undefined });
    expect(screen.queryByRole("button", { name: "Adjust · Hooks" })).toBeNull();
    expect(button("Adjust · Skills")).toBeTruthy();
  });

  it("has no Adjust for the skills while they are not read", () => {
    card([claude(false)], undefined);
    expect(screen.queryByRole("button", { name: "Adjust · Skills" })).toBeNull();
  });

  it("closes with Not now", async () => {
    const props = card([claude(false)], off());
    await userEvent.click(button("Not now"));
    expect(props.onClose).toHaveBeenCalledTimes(1);
  });

  it("tells it is by hand when it is off and nothing can be added from here", () => {
    card([claude(true)], on(), {}, { ...door, hooks: false });
    expect(screen.getByText(/can't turn on what's missing from here/)).toBeTruthy();
    expect(screen.getByText("Silenced in Door")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Turn on/ })).toBeNull();
  });

  it("tells it is by hand when the missing hooks belong to a plugin Tisty cannot read", () => {
    card([claude(false, { hooks: { on: false, at: claudeAt, pluginUnknown: true } })], on());
    expect(screen.getByText(/can't turn on what's missing from here/)).toBeTruthy();
  });

  it("speaks Spanish", () => {
    adopt("es");
    card([claude(false)], off());
    expect(screen.getByText("Tisty en tus agentes")).toBeTruthy();
    expect(button("Encender los dos")).toBeTruthy();
    expect(button("Ajustar · Hooks")).toBeTruthy();
    expect(button("Ahora no")).toBeTruthy();
  });
});

describe("the card looks at the agents when it opens", () => {
  it("calls lookAgents once on open and not again on re-renders", async () => {
    const lookAgents = vi.fn(() => Promise.resolve());
    const known = knownFor({ agent: door, agents: [claude(false)], lookAgents });
    const own = ownOf({ agent: door, agents: [claude(false)], skills: off() });
    const props = { own, skills: off(), known, onSkills: vi.fn(), onClose: vi.fn() };
    const view = render(<TistyOwnCard {...props} />);
    expect(lookAgents).toHaveBeenCalledTimes(1);
    view.rerender(<TistyOwnCard {...props} onHooks={vi.fn()} />);
    await userEvent.click(button("Turn on both"));
    await userEvent.click(button("Back"));
    expect(lookAgents).toHaveBeenCalledTimes(1);
  });

  it("does not throw without a known list", () => {
    expect(() => card([claude(false)], off(), { known: undefined })).not.toThrow();
    expect(screen.getByText("Tisty in your agents")).toBeTruthy();
  });

  it("swallows a lookAgents that is rejected", async () => {
    const lookAgents = vi.fn(() => Promise.reject(new Error("unreadable")));
    const unhandled = vi.fn();
    process.on("unhandledRejection", unhandled);
    card([claude(false)], off(), {
      known: knownFor({ agent: door, agents: [claude(false)], lookAgents }),
    });
    await act(async () => undefined);
    process.off("unhandledRejection", unhandled);
    expect(lookAgents).toHaveBeenCalledTimes(1);
    expect(unhandled).not.toHaveBeenCalled();
    expect(screen.getByText("Tisty in your agents")).toBeTruthy();
  });
});

describe("turning on from the card", () => {
  it("asks first with one line and two short ticks listing the files, and writes nothing before the yes", async () => {
    card([claude(false), codex(false)], off());
    await userEvent.click(button("Turn on both"));
    expect(screen.getByText(/This writes only here and leaves the rest as it is/)).toBeTruthy();
    expect(screen.getByRole("checkbox", { name: "Hooks in Claude Code, Codex" })).toBeTruthy();
    expect(screen.getByRole("checkbox", { name: "Skills on this computer" })).toBeTruthy();
    expect(screen.getByText(claudeAt)).toBeTruthy();
    expect(screen.getByText(codexAt)).toBeTruthy();
    expect(screen.getByText("/h/.claude/skills")).toBeTruthy();
    expect(screen.getAllByRole("checkbox")).toHaveLength(2);
    expect(sent("hook")).toHaveLength(0);
    expect(sent("agents_tisty_skills_take")).toHaveLength(0);
    expect(screen.getByText("Tisty in your agents")).toBeTruthy();
  });

  it("keeps what they do and what Tisty reads folded in details", async () => {
    card([claude(false), codex(false)], off());
    await userEvent.click(button("Turn on both"));
    const summary = screen.getByText("What they do and what Tisty reads");
    const details = summary.closest("details") as HTMLDetailsElement;
    expect(details.open).toBe(false);
    expect(within(details).getByText(/Tisty will add its hooks to/).textContent).toContain(
      `${claudeAt}, ${codexAt}`,
    );
    expect(
      within(details).getByText(/For Claude Code, Tisty first reads its plugin's records/),
    ).toBeTruthy();
    expect(within(details).getByText(/Tisty comes with three skills/)).toBeTruthy();
    expect(within(details).getByText(/To turn them on, Tisty writes only one folder/)).toBeTruthy();
  });

  it("says the plugin line only with Claude Code, and no skills text without skills", async () => {
    card([codex(false)], on());
    await userEvent.click(button("Turn on what's missing"));
    const details = screen
      .getByText("What they do and what Tisty reads")
      .closest("details") as HTMLElement;
    expect(within(details).getByText(/Tisty will add its hooks to/)).toBeTruthy();
    expect(within(details).queryByText(/plugin's records/)).toBeNull();
    expect(within(details).queryByText(/Tisty comes with three skills/)).toBeNull();
    expect(screen.queryByRole("checkbox", { name: "Skills on this computer" })).toBeNull();
  });

  it("says no hooks text in the details when only the skills are asked", async () => {
    card([claude(true)], off());
    await userEvent.click(button("Turn on what's missing"));
    const details = screen
      .getByText("What they do and what Tisty reads")
      .closest("details") as HTMLElement;
    expect(within(details).queryByText(/Tisty will add its hooks/)).toBeNull();
    expect(within(details).getByText(/Tisty comes with three skills/)).toBeTruthy();
  });

  it("says the consent in Spanish", async () => {
    adopt("es");
    card([claude(false)], off());
    await userEvent.click(button("Encender los dos"));
    expect(screen.getByRole("checkbox", { name: "Hooks en Claude Code" })).toBeTruthy();
    expect(screen.getByRole("checkbox", { name: "Skills en este equipo" })).toBeTruthy();
    expect(screen.getByText("Qué hacen y qué lee Tisty")).toBeTruthy();
  });

  it("goes back to the card with Back, still writing nothing", async () => {
    card([claude(false)], off());
    await userEvent.click(button("Turn on both"));
    await userEvent.click(button("Back"));
    expect(screen.getByText(/We recommend both/)).toBeTruthy();
    expect(button("Turn on both")).toBeTruthy();
    expect(ipc.calls.filter((one) => one.cmd !== "agents_tisty_skills")).toHaveLength(0);
  });

  it("writes the hooks and turns on the skills only after Turn on, then tells what it did", async () => {
    const props = card([claude(false), codex(false)], off());
    const known = props.known as NonNullable<typeof props.known>;
    const learnPlugged = vi.spyOn(known, "learnPlugged");
    const lookAgents = vi.spyOn(known, "lookAgents");
    const lookSkills = vi.spyOn(known, "lookSkills");
    const changed = vi.fn();
    window.addEventListener(AGENTS_CHANGED, changed);
    await userEvent.click(button("Turn on both"));
    await userEvent.click(button("Turn on"));

    await waitFor(() => expect(sent("hook")).toHaveLength(2));
    expect(sent("hook").map((one) => one.args)).toEqual([{ id: "claude-code" }, { id: "codex" }]);
    expect(sent("agents_tisty_skills_take")).toHaveLength(1);
    expect(sent("agents_tisty_skills_take")[0].args).toEqual({ on: true });
    const told = await screen.findByRole("status");
    expect(within(told).getByText("Claude Code")).toBeTruthy();
    expect(within(told).getAllByText(/hooks$/).length).toBeGreaterThan(0);
    expect(screen.queryByText("It writes in:")).toBeNull();
    expect(learnPlugged).toHaveBeenCalled();
    expect(lookAgents).toHaveBeenCalled();
    expect(lookSkills).toHaveBeenCalled();
    expect(changed).toHaveBeenCalled();
    window.removeEventListener(AGENTS_CHANGED, changed);
  });

  it("shows the result in place of the consent and lets it be dismissed", async () => {
    card([claude(false)], off());
    await userEvent.click(button("Turn on both"));
    await userEvent.click(button("Turn on"));
    const told = await screen.findByRole("status");
    expect(screen.queryByText("It writes in:")).toBeNull();
    await userEvent.click(within(told).getByRole("button", { name: "Got it" }));
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("shows the Codex approval line when its hooks were already on before opening the card", () => {
    card([claude(true), codex(true)], on());
    expect(screen.getByText(/Approve it in Codex with \/hooks/)).toBeTruthy();
  });

  it("shows the Codex approval line when a plugged Codex hook counts as on", () => {
    card([codex(false, { hooks: { on: false, plugged: true, at: codexAt } })], on());
    expect(screen.getByText(/\/hooks/)).toBeTruthy();
  });

  it("shows no Codex approval line when Codex's hooks are off or Codex is not there", async () => {
    card([claude(true), codex(false)], on());
    expect(screen.queryByText(/\/hooks/)).toBeNull();
    await userEvent.click(button("Turn on what's missing"));
    await userEvent.click(button("Turn on"));
    await screen.findByRole("status");
    expect(screen.queryByText(/Approve it in Codex/)).toBeNull();
  });

  it("shows no Codex approval line when the door muted the hooks", () => {
    card([codex(true)], on(), {}, { ...door, hooks: false });
    expect(screen.queryByText(/\/hooks/)).toBeNull();
  });

  it("shows the Codex approval line in Spanish too", () => {
    adopt("es");
    card(
      [wiredOne("openai-codex", { name: "Codex", wired: true, hooks: { on: true, at: codexAt } })],
      on(),
    );
    expect(screen.getByText(/Apruébalo en Codex con \/hooks/)).toBeTruthy();
  });

  it("sends only the skills when the hooks are unticked", async () => {
    card([claude(false)], off());
    await userEvent.click(button("Turn on both"));
    await userEvent.click(screen.getByRole("checkbox", { name: "Hooks in Claude Code" }));
    await userEvent.click(button("Turn on"));
    await screen.findByRole("status");
    expect(sent("hook")).toHaveLength(0);
    expect(sent("agents_tisty_skills_take")).toHaveLength(1);
  });

  it("sends only the hooks when the skills are unticked", async () => {
    card([claude(false)], off());
    await userEvent.click(button("Turn on both"));
    await userEvent.click(screen.getByRole("checkbox", { name: "Skills on this computer" }));
    await userEvent.click(button("Turn on"));
    await screen.findByRole("status");
    expect(sent("agents_tisty_skills_take")).toHaveLength(0);
    expect(sent("hook").map((one) => one.args)).toEqual([{ id: "claude-code" }]);
  });

  it("cannot turn on with both parts unticked", async () => {
    card([claude(false)], off());
    await userEvent.click(button("Turn on both"));
    await userEvent.click(screen.getByRole("checkbox", { name: "Hooks in Claude Code" }));
    await userEvent.click(screen.getByRole("checkbox", { name: "Skills on this computer" }));
    expect((button("Turn on") as HTMLButtonElement).disabled).toBe(true);
  });

  it("asks only for the part that is missing", async () => {
    card([claude(true)], off());
    await userEvent.click(button("Turn on what's missing"));
    expect(screen.getAllByRole("checkbox")).toHaveLength(1);
    expect(screen.queryByText(/^Hooks for/)).toBeNull();
    await userEvent.click(button("Turn on"));
    await screen.findByRole("status");
    expect(sent("hook")).toHaveLength(0);
    expect(sent("agents_tisty_skills_take")).toHaveLength(1);
  });

  it("says why when the skills cannot be turned on, and keeps the card", async () => {
    answering(off(), {});
    ipc.answer = (cmd) => {
      if (cmd === "agents_tisty_skills_take")
        return Promise.reject({ code: "cannotWrite", about: "disk full" });
      return Promise.resolve(cmd === "wiring" ? [] : null);
    };
    card([claude(true)], off());
    await userEvent.click(button("Turn on what's missing"));
    await userEvent.click(button("Turn on"));
    const told = await screen.findByRole("status");
    expect(within(told).getByText(/✕/, { exact: false })).toBeTruthy();
  });

  it("says the plugin already brings the hooks when the record says so", async () => {
    ipc.answer = (cmd) =>
      Promise.resolve(
        cmd === "wiring"
          ? [claude(false, { hooks: { on: false, at: claudeAt, plugged: true } })]
          : null,
      );
    card([claude(false)], off());
    await userEvent.click(button("Turn on both"));
    await userEvent.click(button("Turn on"));
    const told = await screen.findByRole("status");
    expect(within(told).getByText(/the Tisty plugin already brings its hooks/)).toBeTruthy();
    expect(sent("hook")).toHaveLength(0);
  });

  it("works without a known list to refresh", async () => {
    const props = card([claude(false)], off(), { known: undefined });
    await userEvent.click(button("Turn on both"));
    await userEvent.click(button("Turn on"));
    await screen.findByRole("status");
    expect(props.onClose).not.toHaveBeenCalled();
    fireEvent.keyDown(document.body, { key: "Escape" });
  });
});

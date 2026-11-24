import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  AGENTS_CHANGED,
  AGENTS_ICON,
  AGENTS_NAME,
  holdsTistySkills,
  type TistySkills,
  tistySkills,
} from "../agentsSync";
import { agentsEn, agentsEs } from "../agentsWords";
import type { Filed, Folded } from "../core";
import { showAll } from "../hiding";
import { adopt } from "../locales";
import Folder from "../ui/Folder";
import { sayings, TistySkillsRow, TistySkillsSettingsRow } from "../ui/TistySkills";

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

const SKILLS = [
  { name: "tisty", label: "tisty", about: null, files: ["tisty/SKILL.md"], yours: false },
  {
    name: "consolidate",
    label: "consolidate",
    about: null,
    files: ["consolidate/SKILL.md"],
    yours: true,
  },
  {
    name: "writing-for-the-next-agent",
    label: "writing-for-the-next-agent",
    about: null,
    files: ["writing-for-the-next-agent/LICENSE", "writing-for-the-next-agent/SKILL.md"],
    yours: false,
  },
];

const skills = (more: Partial<TistySkills> = {}): TistySkills => ({
  version: "1.50.0",
  on: false,
  leftover: [],
  skills: SKILLS,
  landing: [
    { assistant: "cursor", at: "/h/.cursor/skills/tisty/SKILL.md", standing: "write" },
    {
      assistant: "codex",
      at: "/h/.agents/skills/writing-for-the-next-agent/SKILL.md",
      standing: "write",
    },
    {
      assistant: "codex",
      at: "/h/.agents/skills/writing-for-the-next-agent/LICENSE",
      standing: "write",
    },
  ],
  plugged: ["claude"],
  ...more,
});

// Like the app, every read after the switch tells what the switch left.
const answering = (now: TistySkills, after?: TistySkills) => {
  let held = now;
  ipc.answer = (cmd) => {
    if (cmd === "agents_tisty_skills") return Promise.resolve(held);
    if (cmd === "agents_tisty_skills_take") {
      held = after ?? now;
      return Promise.resolve(held);
    }
    return Promise.resolve(null);
  };
};

const folder = (id: string, name: string, parent: string | null, holds = 0): Folded => ({
  id,
  name,
  parent,
  icon: parent ? null : AGENTS_ICON,
  color: null,
  holds,
  archived: false,
  away: false,
});

const agents = folder("a", AGENTS_NAME, null);
const global = folder("g", "Global", "a", 1);
const claude = folder("c", "Claude", "a");
const elsewhere = folder("x", "Global", null);
const folders = [agents, global, claude, elsewhere];
const docs: Filed[] = [
  { id: "d", file: "0001", title: "Tono", folder: "g", archived: false, away: false },
];

const inFolder = (at: Folded) =>
  render(<Folder folder={at} folders={folders} docs={docs} onOpen={vi.fn()} onHere={vi.fn()} />);

beforeEach(() => {
  adopt("en");
  ipc.calls = [];
  answering(skills());
  showAll();
});

describe("Tisty's own skills", () => {
  it("only Agents' Global holds them, not a folder that happens to share the name", () => {
    expect(holdsTistySkills(global, folders)).toBe(true);
    expect(holdsTistySkills(elsewhere, folders)).toBe(false);
    expect(holdsTistySkills(claude, folders)).toBe(false);
    expect(holdsTistySkills(null, folders)).toBe(false);
  });

  it("asks once when the sidebar and the folder view ask at the same time", async () => {
    await Promise.all([tistySkills(), tistySkills()]);
    expect(sent("agents_tisty_skills")).toHaveLength(1);
    await tistySkills();
    expect(sent("agents_tisty_skills")).toHaveLength(2);
  });

  it("never hands an answer asked before a change to one asked after it", async () => {
    const before = tistySkills();
    window.dispatchEvent(new Event(AGENTS_CHANGED));
    const after = tistySkills();
    await Promise.all([before, after]);
    expect(sent("agents_tisty_skills")).toHaveLength(2);
  });

  it("before their yes says what adding them writes and reads, and needs no Agents", async () => {
    render(<TistySkillsSettingsRow />);
    await userEvent.click(await screen.findByRole("button", { name: "See" }));
    expect(
      screen.getByText(/reads its plugin's records \(installed_plugins.json and settings.json\)/),
    ).toBeTruthy();
    expect(screen.getByText(/needs no Agents folder/)).toBeTruthy();
    expect(
      (screen.getByRole("button", { name: "Turn on here" }) as HTMLButtonElement).disabled,
    ).toBe(false);
  });

  it("sits in Settings › Agents and shows where they stand", async () => {
    answering(skills({ on: true }));
    render(<TistySkillsSettingsRow />);
    expect(await screen.findByRole("region", { name: "Tisty's skills" })).toBeTruthy();
    expect(screen.getByText("On")).toBeTruthy();
    expect(screen.getByText(/They have their own yes, on this computer/)).toBeTruthy();
  });

  it("offers each copy left over for removal, through the window's own command", async () => {
    const over = { assistant: "claude", at: "/h/.claude/skills/tisty/SKILL.md", changed: true };
    answering(skills({ on: true, leftover: [over] }), skills({ on: true }));
    render(<TistySkillsSettingsRow />);
    await userEvent.click(await screen.findByRole("button", { name: "See" }));
    expect(screen.getByText(/Copies left over/)).toBeTruthy();
    expect(screen.getByText("Changed outside Tisty")).toBeTruthy();
    ipc.answer = (cmd) =>
      Promise.resolve(
        cmd === "agents_tisty_skills_leftover" ? skills({ on: true }) : skills({ on: true }),
      );
    await userEvent.click(screen.getByRole("button", { name: `Remove ${over.at}` }));
    await waitFor(() => expect(sent("agents_tisty_skills_leftover")).toHaveLength(1));
    expect(sent("agents_tisty_skills_leftover")[0].args).toEqual({ at: over.at });
    await waitFor(() => expect(screen.queryByText(/Copies left over/)).toBeNull());
  });

  it("opens the card from the sidebar with each skill, where they go and what is left out", async () => {
    render(<TistySkillsRow depth={1} step={12} />);
    await userEvent.click(await screen.findByRole("button", { name: /^Tisty's skills.*Off/ }));

    expect(
      screen.getByText("Picking up where it left off, tasks, documents and ideas."),
    ).toBeTruthy();
    expect(
      screen.getByText(
        "You already have a skill named consolidate in Agents, so Tisty's is left out.",
      ),
    ).toBeTruthy();
    expect(screen.getByText("Where they go on this computer")).toBeTruthy();
    expect(screen.getByText("/h/.cursor/skills")).toBeTruthy();
    expect(screen.getByText("/h/.agents/skills")).toBeTruthy();
    expect(screen.getByText("Codex · Copilot · Gemini")).toBeTruthy();
    expect(screen.getByText(/Codex, Copilot and Gemini share this folder/)).toBeTruthy();
    expect(screen.getByText(/Claude Code already has them through the Tisty plugin/)).toBeTruthy();
    expect(screen.getByText(/installed the Tisty plugin in Cursor/)).toBeTruthy();
    expect(screen.getByText("From Tisty 1.50.0")).toBeTruthy();
    expect(screen.queryByText(/Turning them off takes/)).toBeNull();
  });

  it("tells what to ask the agent for each skill, in the person's language", async () => {
    render(<TistySkillsRow depth={1} step={12} />);
    await userEvent.click(await screen.findByRole("button", { name: /^Tisty's skills/ }));
    expect(screen.getByText("“Pick up where we left off”")).toBeTruthy();
    expect(screen.getByText("“Leave this ready for the next agent”")).toBeTruthy();
    expect(screen.getByText("“Make this page clear for the next agent”")).toBeTruthy();
    expect(screen.queryByText("“Tidy up my Tisty”")).toBeNull();
    expect(screen.getByRole("region", { name: "How to use them" })).toBeTruthy();
  });

  it("says the lines in Spanish too", () => {
    adopt("es");
    expect(sayings("tisty")).toEqual([
      "Retoma donde quedamos",
      "Deja esto listo para el próximo agente",
    ]);
    expect(sayings("consolidate")).toEqual(["Ordena mi Tisty"]);
    expect(sayings("someone-else")).toEqual([]);
  });

  it("turns them on and closes, and then offers to turn them off", async () => {
    const on = skills({
      on: true,
      landing: [
        { assistant: "cursor", at: "/h/.cursor/skills/tisty/SKILL.md", standing: "upToDate" },
        {
          assistant: "codex",
          at: "/h/.agents/skills/tisty/SKILL.md",
          standing: "foreign",
        },
      ],
    });
    answering(skills(), on);
    const changed = vi.fn();
    window.addEventListener(AGENTS_CHANGED, changed);
    render(<TistySkillsRow depth={1} step={12} />);
    await userEvent.click(await screen.findByRole("button", { name: /^Tisty's skills/ }));
    await userEvent.click(screen.getByRole("button", { name: "Turn on here" }));

    await waitFor(() => expect(sent("agents_tisty_skills_take")).toHaveLength(1));
    expect(sent("agents_tisty_skills_take")[0].args).toEqual({ on: true });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    await userEvent.click(await screen.findByRole("button", { name: /^Tisty's skills.*On/ }));
    expect(await screen.findByRole("button", { name: "Turn off on this computer" })).toBeTruthy();
    expect(screen.getByText("In sync")).toBeTruthy();
    expect(screen.getByText("A skill that is not Tisty's: left as it is")).toBeTruthy();
    expect(screen.getByText(/Turning them off takes Tisty's untouched copies/)).toBeTruthy();
    expect(changed).toHaveBeenCalled();
    window.removeEventListener(AGENTS_CHANGED, changed);
  });

  it("says why it cannot add them when the window's command refuses", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_tisty_skills") return Promise.resolve(skills());
      return Promise.reject({ code: "cannotWrite", about: "disk full" });
    };
    render(<TistySkillsRow depth={1} step={12} />);
    await userEvent.click(await screen.findByRole("button", { name: /^Tisty's skills/ }));
    await userEvent.click(screen.getByRole("button", { name: "Turn on here" }));
    await waitFor(() => expect(sent("agents_tisty_skills_take")).toHaveLength(1));
    expect(await screen.findByRole("button", { name: "Turn on here" })).toBeTruthy();
  });

  it("a build without skills says so and has nothing to add", async () => {
    answering(skills({ skills: [], landing: [] }));
    render(<TistySkillsRow depth={1} step={12} />);
    await userEvent.click(await screen.findByRole("button", { name: /^Tisty's skills/ }));
    expect(screen.getByText(/This build of Tisty carries no skills/)).toBeTruthy();
    expect(
      (screen.getByRole("button", { name: "Turn on here" }) as HTMLButtonElement).disabled,
    ).toBe(true);
    await userEvent.click(screen.getByRole("button", { name: "Not now" }));
    expect(screen.queryByText(/This build of Tisty carries no skills/)).toBeNull();
  });

  it("asks again after each round of Agents", async () => {
    render(<TistySkillsRow depth={1} step={12} />);
    await screen.findByRole("button", { name: /^Tisty's skills/ });
    await act(async () => {
      window.dispatchEvent(new Event(AGENTS_CHANGED));
    });
    await waitFor(() => expect(sent("agents_tisty_skills")).toHaveLength(2));
  });
});

describe("Tisty's own skills in the folder view", () => {
  it("sit in the Global section when Agents is open, and inside Global when it is", async () => {
    const view = inFolder(agents);
    expect(await screen.findByRole("button", { name: /^Tisty's skills.*Off/ })).toBeTruthy();
    view.unmount();

    inFolder(global);
    await userEvent.click(await screen.findByRole("button", { name: /^Tisty's skills/ }));
    expect(screen.getByText("Where they go on this computer")).toBeTruthy();
  });

  it("never sit in a folder named Global outside Agents", async () => {
    const { container } = inFolder(elsewhere);
    expect(container.querySelector("[data-row=tisty-skills]")).toBeNull();
    expect(sent("agents_tisty_skills")).toHaveLength(0);
  });

  it("an empty Global does not say it holds nothing above Tisty's skills", async () => {
    render(
      <Folder
        folder={{ ...global, holds: 0 }}
        folders={folders}
        docs={[]}
        onOpen={vi.fn()}
        onHere={vi.fn()}
      />,
    );
    expect(await screen.findByRole("button", { name: /^Tisty's skills/ })).toBeTruthy();
    expect(screen.queryByText("nothing in here yet")).toBeNull();
  });

  it("an empty Global in the Agents view is not called empty beside Tisty's skills", async () => {
    render(
      <Folder
        folder={agents}
        folders={[agents, { ...global, holds: 0 }]}
        docs={[]}
        onOpen={vi.fn()}
        onHere={vi.fn()}
      />,
    );
    expect(await screen.findByRole("button", { name: /^Tisty's skills/ })).toBeTruthy();
    expect(screen.queryByText("empty")).toBeNull();
  });
});

describe("Tisty's own skills are turned off, never deleted", () => {
  it("the card turns them on and off, and nowhere does it offer to delete them", async () => {
    const { container } = render(<TistySkillsRow depth={1} step={12} />);
    const row = await screen.findByRole("button", { name: /^Tisty's skills.*Off/ });
    fireEvent.contextMenu(row);
    fireEvent.keyDown(row, { key: "F10", shiftKey: true });
    fireEvent.keyDown(row, { key: "Delete" });
    expect(screen.queryByRole("menu")).toBeNull();
    await userEvent.click(row);
    expect(screen.getByRole("button", { name: "Turn on here" })).toBeTruthy();
    const said = (document.body.textContent ?? "") + (container.textContent ?? "");
    expect(said).not.toMatch(/Delete|Eliminar/);
    expect(sent("agents_tisty_skills_take")).toHaveLength(0);
  });

  it("says in its row that they are off, and a click opens the card without turning anything", async () => {
    render(<TistySkillsRow depth={1} step={12} />);
    expect((await screen.findByText("Off")).className).toContain("text-faint");
    await userEvent.click(screen.getByRole("button", { name: /Tisty's skills/ }));
    expect(screen.getByRole("button", { name: "Turn on here" })).toBeTruthy();
    expect(sent("agents_tisty_skills_take")).toHaveLength(0);
  });

  it("says in Spanish that they are off, and turning them off is not deleting", async () => {
    adopt("es");
    answering(skills({ on: true }));
    render(<TistySkillsRow depth={1} step={12} />);
    await userEvent.click(
      await screen.findByRole("button", { name: /^Skills de Tisty.*Encendidas/ }),
    );
    expect(screen.getByRole("button", { name: "Apagar en este equipo" })).toBeTruthy();
    expect(
      screen.getByText(/Apagarlas saca de las carpetas de tus agentes las copias intactas/),
    ).toBeTruthy();
    expect(screen.getByText(/listas para encenderlas otra vez/)).toBeTruthy();
  });
});

describe("every state a copy of Tisty's skill can be in", () => {
  it("has its words in English and Spanish, never the code's own name", () => {
    const standings = [
      "noConsent",
      "held",
      "unproven",
      "empty",
      "linked",
      "broken",
      "upToDate",
      "agreed",
      "write",
      "wrote",
      "outside",
      "both",
      "foreign",
      "unreadable",
    ];
    for (const side of [agentsEn, agentsEs]) {
      for (const one of standings) {
        expect(side).toHaveProperty(`tistyLanding_${one}`);
      }
    }
  });
});

describe("a copy left over that Tisty may not open", () => {
  it("says nothing of whether it changed", async () => {
    const over = { assistant: "claude", at: "/h/.claude/skills/tisty/SKILL.md", changed: null };
    answering(skills({ leftover: [over] }));
    render(<TistySkillsSettingsRow />);
    await userEvent.click(await screen.findByRole("button", { name: "See" }));
    expect(screen.getByText(over.at)).toBeTruthy();
    expect(screen.queryByText("Changed outside Tisty")).toBeNull();
  });
});

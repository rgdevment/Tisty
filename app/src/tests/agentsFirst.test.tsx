import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentsLook, Both, Brought } from "../agentsSync";
import { agentsPrint, agentsRoot, inAgents } from "../agentsSync";
import type { Filed, Folded } from "../core";
import { adopt } from "../locales";
import AgentsFirst from "../ui/AgentsFirst";
import { AGENTS_CHANGED } from "../ui/AgentsKeeper";
import AgentsSettings from "../ui/AgentsSettings";

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

const look: AgentsLook = {
  folder: null,
  docs: 0,
  changes: 0,
  consented: false,
  allOn: true,
  asks: true,
  both: [],
  assistants: [
    {
      id: "claude",
      name: "Claude",
      on: true,
      files: [
        { at: "/h/.claude/CLAUDE.md", name: "CLAUDE.md", kind: "rule", skip: null, held: false },
        {
          at: "/h/.claude/rules/tone.md",
          name: "rules/tone.md",
          kind: "rule",
          skip: null,
          held: false,
        },
        {
          at: "/h/.claude/skills/k/.env",
          name: "k/.env",
          kind: "skill",
          skip: "credentials",
          held: false,
        },
      ],
    },
    {
      id: "codex",
      name: "Codex",
      on: true,
      files: [
        { at: "/h/.codex/AGENTS.md", name: "AGENTS.md", kind: "rule", skip: null, held: false },
      ],
    },
  ],
};

const both: Both = {
  at: "/h/.claude/CLAUDE.md",
  assistant: "claude",
  composed: false,
  lines: [
    { side: "same", text: "# Claude" },
    { side: "tisty", text: "Tests first." },
    { side: "file", text: "Tests last." },
  ],
};

const brought = (more: Partial<Brought> = {}): Brought => ({
  made: 2,
  refused: [],
  round: { told: [], both: [] },
  ...more,
});

beforeEach(() => {
  adopt("es");
  ipc.calls = [];
  ipc.answer = (cmd) => {
    if (cmd === "agents_look") return Promise.resolve(look);
    if (cmd === "agents_bring") return Promise.resolve(brought());
    return Promise.resolve({ told: [], both: [] });
  };
});

describe("the first time AGENTS opens", () => {
  it("says what will happen, that it stays in the person's Tisty, and counts what it found", async () => {
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    expect(
      screen.getByText(
        /Queda en tu Tisty: en tus equipos y donde tú sincronices\. Nadie más lo ve\./,
      ),
    ).toBeTruthy();
    await screen.findByText("Claude · 2 archivos");
    expect(screen.getByText("Codex · 1 archivo")).toBeTruthy();
    expect(screen.getByText(/Se sincroniza solo con Claude, Codex\./)).toBeTruthy();
    expect(screen.getByText(/respaldo/)).toBeTruthy();
  });

  it("«Ahora no» closes without bringing anything", async () => {
    const closed = vi.fn();
    render(<AgentsFirst onClose={closed} onChanged={() => {}} />);
    await screen.findByText("Claude · 2 archivos");
    await userEvent.click(screen.getByRole("button", { name: "Ahora no" }));
    expect(closed).toHaveBeenCalled();
    expect(sent("agents_bring")).toHaveLength(0);
  });

  it("brings only the files left chosen, file by file", async () => {
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await userEvent.click(await screen.findByText("Claude · 2 archivos"));
    await userEvent.click(screen.getByRole("checkbox", { name: /rules\/tone\.md/ }));
    expect((screen.getByRole("checkbox", { name: /k\/\.env/ }) as HTMLInputElement).disabled).toBe(
      true,
    );
    await userEvent.click(screen.getByRole("button", { name: "Traer y sincronizar" }));
    await waitFor(() => expect(sent("agents_bring")).toHaveLength(1));
    await screen.findByText(/Entraron 2 documentos a AGENTS/);
    expect(sent("agents_bring")[0].args.chosen).toEqual([
      "/h/.claude/CLAUDE.md",
      "/h/.codex/AGENTS.md",
    ]);
  });

  it("shows the difference when a file exists on both sides and keeps the side chosen", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(look);
      if (cmd === "agents_bring")
        return Promise.resolve(brought({ round: { told: [], both: [both] } }));
      return Promise.resolve({ told: [], both: [] });
    };
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Claude · 2 archivos");
    await userEvent.click(screen.getByRole("button", { name: "Traer y sincronizar" }));
    await screen.findByText("Existe en los dos lados");
    expect(screen.getByText(/Tests first\./)).toBeTruthy();
    expect(screen.getByText(/Tests last\./)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Conservar el archivo" }));
    await waitFor(() => expect(sent("agents_keep")).toHaveLength(1));
    expect(sent("agents_keep")[0].args).toEqual({ at: both.at, tisty: false });
  });
});

const folder = (id: string, parent: string | null, more: Partial<Folded> = {}): Folded => ({
  id,
  name: id,
  parent,
  icon: null,
  holds: 0,
  archived: false,
  away: false,
  ...more,
});

describe("what the person chooses and why something stays out", () => {
  const withMemory: AgentsLook = {
    ...look,
    assistants: [
      {
        id: "claude",
        name: "Claude",
        on: true,
        files: [
          { at: "/h/.claude/CLAUDE.md", name: "CLAUDE.md", kind: "rule", skip: null, held: false },
          {
            at: "/h/.claude/projects/p/memory/MEMORY.md",
            name: "projects/p/memory/MEMORY.md",
            kind: "memory",
            skip: null,
            held: false,
          },
        ],
      },
    ],
  };

  it("leaves an agent's memory unticked, since it would travel with sync", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(withMemory);
      if (cmd === "agents_bring") return Promise.resolve(brought({ made: 1 }));
      return Promise.resolve({ told: [], both: [] });
    };
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await userEvent.click(await screen.findByText("Claude · 2 archivos"));
    const memory = screen.getByRole("checkbox", { name: /MEMORY\.md/ }) as HTMLInputElement;
    expect(memory.checked).toBe(false);
    expect(memory.disabled).toBe(false);
    await userEvent.click(screen.getByRole("button", { name: "Traer y sincronizar" }));
    await waitFor(() => expect(sent("agents_bring")).toHaveLength(1));
    expect(sent("agents_bring")[0].args.chosen).toEqual(["/h/.claude/CLAUDE.md"]);
  });

  it("offers to allow, not to bring, when only memory was found", async () => {
    const onlyMemory: AgentsLook = {
      ...withMemory,
      assistants: [{ ...withMemory.assistants[0], files: [withMemory.assistants[0].files[1]] }],
    };
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(onlyMemory);
      return Promise.resolve(brought({ made: 0 }));
    };
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Claude · 1 archivo");
    expect(screen.getByRole("button", { name: "Autorizar y sincronizar" })).toBeTruthy();
  });

  it("says in the person's language why a file stayed out", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(look);
      if (cmd === "agents_bring")
        return Promise.resolve(
          brought({
            made: 1,
            refused: [
              {
                at: "/h/.claude/rules/keys.md",
                why: "the file reads like it holds a credential (GitHub token)",
                reason: "secret",
              },
              { at: "/h/.claude/rules/big.md", why: "too big", reason: "tooBig" },
              { at: "/h/.claude/rules/odd.md", why: "Access is denied.", reason: "other" },
            ],
          }),
        );
      return Promise.resolve({ told: [], both: [] });
    };
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Claude · 2 archivos");
    await userEvent.click(screen.getByRole("button", { name: "Traer y sincronizar" }));
    await screen.findByText(/Parece llevar una credencial/);
    expect(screen.getByText("Es más grande de lo que cabe en un documento.")).toBeTruthy();
    expect(screen.getByText("No se pudo leer: Access is denied.")).toBeTruthy();
    expect(screen.queryByText(/reads like it holds a credential/)).toBeNull();
  });
});

describe("a choice left for later", () => {
  it("opens straight at the comparison, and closes once nothing is left to choose", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_look")
        return Promise.resolve({ ...look, consented: true, docs: 3, asks: true, both: [both] });
      return Promise.resolve({ told: [], both: [] });
    };
    const closed = vi.fn();
    render(<AgentsFirst onClose={closed} onChanged={() => {}} />);
    await screen.findByText("Existe en los dos lados");
    await userEvent.click(screen.getByRole("button", { name: "Conservar el de Tisty" }));
    await waitFor(() => expect(sent("agents_keep")).toHaveLength(1));
    expect(sent("agents_keep")[0].args).toEqual({ at: both.at, tisty: true });
    await waitFor(() => expect(closed).toHaveBeenCalled());
    expect(sent("agents_bring")).toHaveLength(0);
  });
});

describe("finding AGENTS among the folders", () => {
  const folders = [
    folder("work", null),
    folder("a", null, { name: "AGENTS", icon: "robot" }),
    folder("claude", "a"),
    folder("deep", "claude"),
  ];

  it("knows the AGENTS tree by its mark, not by its name alone", () => {
    expect(agentsRoot(folders)?.id).toBe("a");
    expect(inAgents("deep", folders)).toBe(true);
    expect(inAgents("work", folders)).toBe(false);
    expect(agentsRoot([folder("x", null, { name: "AGENTS" })])).toBeUndefined();
  });

  it("changes its print only when a document inside AGENTS changes", () => {
    const doc = (file: string, at: string, wrote: string): Filed =>
      ({ id: file, file, title: file, folder: at, archived: false, away: false, wrote }) as Filed;
    const before = agentsPrint([doc("x", "claude", "1"), doc("y", "work", "1")], folders);
    expect(agentsPrint([doc("x", "claude", "1"), doc("y", "work", "2")], folders)).toBe(before);
    expect(agentsPrint([doc("x", "claude", "2"), doc("y", "work", "1")], folders)).not.toBe(before);
  });
});

describe("the switches in Settings", () => {
  it("tells the sidebar and an open card when sync is paused", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_look")
        return Promise.resolve({ ...look, folder: "f", docs: 2, consented: true, asks: false });
      return Promise.resolve({ told: [], both: [] });
    };
    const heard = vi.fn();
    window.addEventListener(AGENTS_CHANGED, heard);
    render(<AgentsSettings busy={null} />);
    await userEvent.click(await screen.findByRole("switch", { name: "Sincronizar Claude" }));
    await waitFor(() => expect(sent("agents_turn")).toHaveLength(1));
    expect(sent("agents_turn")[0].args).toEqual({ assistant: "claude", on: false });
    await waitFor(() => expect(heard).toHaveBeenCalled());
    window.removeEventListener(AGENTS_CHANGED, heard);
  });
});

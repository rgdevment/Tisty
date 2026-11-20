import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentsLook, Both, Brought, FileSeen } from "../agentsSync";
import { AGENTS_MORE, agentsPrint, agentsRoot, inAgents } from "../agentsSync";
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
    // Settings asks first whether this computer said yes; these cards always find it did.
    if (cmd === "agents_state")
      return Promise.resolve({ folder: "f", consented: true, allOn: true, assistants: [] });
    return ipc.answer(cmd, args ?? {});
  },
}));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const look: AgentsLook = {
  folder: null,
  docs: 0,
  changes: 0,
  waiting: 0,
  unagreed: 0,
  proposed: 0,
  consented: false,
  allOn: true,
  asks: true,
  both: [],
  loose: [],
  unled: [],
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
  round: { told: [], both: [], loose: [], unled: [] },
  syncs: true,
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
    await screen.findByText("Claude · 2 reglas");
    expect(screen.getByText("Codex · 1 regla")).toBeTruthy();
    expect(screen.getByText(/Si sincronizas, llega solo a Claude, Codex\./)).toBeTruthy();
    expect(screen.getByText(/respaldo/)).toBeTruthy();
  });

  it("offers only the X, «Traer» and «Traer y sincronizar», with no «Ahora no» nor «Ocultar»", async () => {
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Claude · 2 reglas");
    const names = screen
      .getAllByRole("button")
      .map((one) => one.getAttribute("aria-label") ?? one.textContent);
    expect(names).toEqual(expect.arrayContaining(["Cerrar", "Traer", "Traer y sincronizar"]));
    expect(screen.queryByRole("button", { name: "Ahora no" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Ocultar" })).toBeNull();
  });

  it("closes with the X without bringing anything", async () => {
    const closed = vi.fn();
    render(<AgentsFirst onClose={closed} onChanged={() => {}} />);
    await screen.findByText("Claude · 2 reglas");
    await userEvent.click(screen.getByRole("button", { name: "Cerrar" }));
    expect(closed).toHaveBeenCalledTimes(1);
    expect(sent("agents_bring")).toHaveLength(0);
  });

  it("closes with Esc without bringing anything", async () => {
    const closed = vi.fn();
    render(<AgentsFirst onClose={closed} onChanged={() => {}} />);
    await screen.findByText("Claude · 2 reglas");
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(closed).toHaveBeenCalledTimes(1);
    expect(sent("agents_bring")).toHaveLength(0);
  });

  it("brings only the files left chosen, file by file, and syncs them", async () => {
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await userEvent.click(await screen.findByText("Claude · 2 reglas"));
    await userEvent.click(screen.getByRole("checkbox", { name: /rules\/tone\.md/ }));
    expect((screen.getByRole("checkbox", { name: /^k · / }) as HTMLInputElement).disabled).toBe(
      true,
    );
    await userEvent.click(screen.getByRole("button", { name: "Traer y sincronizar" }));
    await waitFor(() => expect(sent("agents_bring")).toHaveLength(1));
    await screen.findByText(
      /Entraron 2 documentos a Agentes, y desde ahora Tisty los mantiene al día\./,
    );
    expect(sent("agents_bring")[0].args).toEqual({
      chosen: ["/h/.claude/CLAUDE.md", "/h/.codex/AGENTS.md"],
      sync: true,
    });
  });

  it("«Traer» copies the chosen files without syncing, and says the agents' files stay as they are", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(look);
      if (cmd === "agents_bring") return Promise.resolve(brought({ syncs: false }));
      return Promise.resolve({ told: [], both: [] });
    };
    const changed = vi.fn();
    const closed = vi.fn();
    render(<AgentsFirst onClose={closed} onChanged={changed} />);
    await screen.findByText("Claude · 2 reglas");
    await userEvent.click(screen.getByRole("button", { name: "Traer" }));
    await waitFor(() => expect(sent("agents_bring")).toHaveLength(1));
    expect(sent("agents_bring")[0].args).toEqual({
      chosen: ["/h/.claude/CLAUDE.md", "/h/.claude/rules/tone.md", "/h/.codex/AGENTS.md"],
      sync: false,
    });
    expect(
      await screen.findByText(
        "Entraron 2 documentos a Agentes. Los archivos de tus agentes quedan como están hasta que elijas Sincronizar.",
      ),
    ).toBeTruthy();
    expect(changed).toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Listo" }));
    expect(closed).toHaveBeenCalled();
  });

  it("says «Trayendo…» only on the button that was clicked", async () => {
    let give: (one: Brought) => void = () => {};
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(look);
      if (cmd === "agents_bring") return new Promise((done) => (give = done));
      return Promise.resolve({ told: [], both: [] });
    };
    const { unmount } = render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Claude · 2 reglas");
    await userEvent.click(screen.getByRole("button", { name: "Traer" }));
    expect(screen.getByRole("button", { name: "Trayendo…" })).toBeTruthy();
    expect(
      (screen.getByRole("button", { name: "Traer y sincronizar" }) as HTMLButtonElement).disabled,
    ).toBe(true);
    await act(async () => give(brought()));
    unmount();
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Claude · 2 reglas");
    await userEvent.click(screen.getByRole("button", { name: "Traer y sincronizar" }));
    expect(screen.getByRole("button", { name: "Trayendo…" })).toBeTruthy();
    expect((screen.getByRole("button", { name: "Traer" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    await act(async () => give(brought()));
  });

  it("says one or none came in, with or without sync", async () => {
    const said = async (made: number, syncs: boolean, words: string) => {
      ipc.answer = (cmd) => {
        if (cmd === "agents_look") return Promise.resolve(look);
        if (cmd === "agents_bring") return Promise.resolve(brought({ made, syncs }));
        return Promise.resolve({ told: [], both: [] });
      };
      const { unmount } = render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
      await screen.findByText("Claude · 2 reglas");
      await userEvent.click(
        screen.getByRole("button", { name: syncs ? "Traer y sincronizar" : "Traer" }),
      );
      expect(await screen.findByText(words)).toBeTruthy();
      unmount();
    };
    await said(1, true, "Entró 1 documento a Agentes, y desde ahora Tisty lo mantiene al día.");
    await said(0, true, "No entró nada nuevo. Desde ahora este equipo mantiene Agentes al día.");
    await said(
      1,
      false,
      "Entró 1 documento a Agentes. Los archivos de tus agentes quedan como están hasta que elijas Sincronizar.",
    );
    await said(0, false, "No entró nada nuevo.");
  });

  it("says why bringing failed and stays open", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(look);
      if (cmd === "agents_bring") return Promise.reject({ code: "internal" });
      return Promise.resolve({ told: [], both: [] });
    };
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Claude · 2 reglas");
    await userEvent.click(screen.getByRole("button", { name: "Traer y sincronizar" }));
    await waitFor(() => expect(sent("agents_bring")).toHaveLength(1));
    expect(screen.getByRole("button", { name: "Traer y sincronizar" })).toBeTruthy();
    expect((screen.getByRole("button", { name: "Traer" }) as HTMLButtonElement).disabled).toBe(
      false,
    );
  });

  it("shows the difference when a file exists on both sides and keeps the side chosen", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(look);
      if (cmd === "agents_bring")
        return Promise.resolve(
          brought({ round: { told: [], both: [both], loose: [], unled: [] } }),
        );
      return Promise.resolve({ told: [], both: [] });
    };
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Claude · 2 reglas");
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
    await userEvent.click(await screen.findByText("Claude · 1 regla · 1 memoria"));
    const memory = screen.getByRole("checkbox", { name: /MEMORY\.md/ }) as HTMLInputElement;
    expect(memory.checked).toBe(false);
    expect(memory.disabled).toBe(false);
    await userEvent.click(screen.getByRole("button", { name: "Traer y sincronizar" }));
    await waitFor(() => expect(sent("agents_bring")).toHaveLength(1));
    expect(sent("agents_bring")[0].args.chosen).toEqual(["/h/.claude/CLAUDE.md"]);
  });

  it("offers only to sync, not to bring, when only memory was found", async () => {
    const onlyMemory: AgentsLook = {
      ...withMemory,
      assistants: [{ ...withMemory.assistants[0], files: [withMemory.assistants[0].files[1]] }],
    };
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(onlyMemory);
      return Promise.resolve(brought({ made: 0 }));
    };
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Claude · 1 memoria");
    expect((screen.getByRole("button", { name: "Traer" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    const sync = screen.getByRole("button", { name: "Traer y sincronizar" }) as HTMLButtonElement;
    expect(sync.disabled).toBe(false);
    await userEvent.click(sync);
    await waitFor(() => expect(sent("agents_bring")).toHaveLength(1));
    expect(sent("agents_bring")[0].args).toEqual({ chosen: [], sync: true });
  });

  it("offers a skill as one row that brings every file it holds", async () => {
    const skill = (name: string, more: Partial<FileSeen> = {}): FileSeen => ({
      at: `/h/.claude/skills/${name}`,
      name,
      kind: "skill",
      skip: null,
      held: false,
      ...more,
    });
    const withSkills: AgentsLook = {
      ...look,
      assistants: [
        {
          id: "claude",
          name: "Claude",
          on: true,
          files: [
            skill("pr/SKILL.md"),
            skill("pr/plantilla.md"),
            skill("pr/scripts/abrir.sh"),
            skill("pr/key.pem", { skip: "credentials" }),
            skill("tono/SKILL.md", { held: true }),
            skill("solo/SKILL.md"),
          ],
        },
      ],
    };
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(withSkills);
      if (cmd === "agents_bring") return Promise.resolve(brought({ made: 3 }));
      return Promise.resolve({ told: [], both: [] });
    };
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await userEvent.click(await screen.findByText("Claude · 2 skills"));
    const pr = screen.getByRole("checkbox", {
      name: "pr · Skill · 3 archivos",
    }) as HTMLInputElement;
    expect(pr.checked).toBe(true);
    expect(screen.getByRole("checkbox", { name: "solo · Skill · 1 archivo" })).toBeTruthy();
    const held = screen.getByRole("checkbox", { name: /^tono · / }) as HTMLInputElement;
    expect(held.disabled).toBe(true);
    await userEvent.click(screen.getByRole("checkbox", { name: "solo · Skill · 1 archivo" }));
    await userEvent.click(pr);
    expect(pr.checked).toBe(false);
    await userEvent.click(pr);
    await userEvent.click(screen.getByRole("button", { name: "Traer y sincronizar" }));
    await waitFor(() => expect(sent("agents_bring")).toHaveLength(1));
    expect(sent("agents_bring")[0].args.chosen).toEqual([
      "/h/.claude/skills/pr/SKILL.md",
      "/h/.claude/skills/pr/plantilla.md",
      "/h/.claude/skills/pr/scripts/abrir.sh",
    ]);
  });

  it("says when an assistant has nothing to bring", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_look"
        ? Promise.resolve({
            ...look,
            assistants: [
              {
                id: "gemini",
                name: "Gemini",
                on: true,
                files: [
                  {
                    at: "/h/.gemini/skills/k/key.pem",
                    name: "k/key.pem",
                    kind: "skill",
                    skip: "credentials",
                    held: false,
                  },
                ],
              },
            ],
          })
        : Promise.resolve(null);
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    expect(await screen.findByText("Gemini · nada para traer")).toBeTruthy();
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
    await screen.findByText("Claude · 2 reglas");
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
        return Promise.resolve({
          ...look,
          folder: "f",
          consented: true,
          docs: 3,
          asks: true,
          both: [both],
        });
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

describe("bringing another file after the first time", () => {
  const consented: AgentsLook = { ...look, folder: "f", docs: 2, consented: true, asks: false };

  it("ticks nothing, waits for a choice and brings only what was ticked", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(consented);
      if (cmd === "agents_bring") return Promise.resolve(brought({ made: 1 }));
      return Promise.resolve({ told: [], both: [], loose: [], unled: [] });
    };
    render(<AgentsFirst more onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Traer otro archivo");
    expect(screen.getByText(/todavía no están en Agentes/)).toBeTruthy();
    expect(screen.queryByText(/Queda en tu Tisty/)).toBeNull();
    const bring = screen.getByRole("button", { name: "Traer" }) as HTMLButtonElement;
    expect(bring.disabled).toBe(true);
    await userEvent.click(screen.getByText("Codex · 1 regla"));
    await userEvent.click(screen.getByRole("checkbox", { name: /AGENTS\.md/ }));
    await userEvent.click(bring);
    await waitFor(() => expect(sent("agents_bring")).toHaveLength(1));
    expect(sent("agents_bring")[0].args).toEqual({ chosen: ["/h/.codex/AGENTS.md"], sync: null });
    await screen.findByText(/Entró 1 documento a Agentes/);
  });

  it("has nothing to pick on a folder that already syncs, and closes when nothing waits", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_look" ? Promise.resolve(consented) : Promise.resolve(brought());
    const closed = vi.fn();
    render(<AgentsFirst onClose={closed} onChanged={() => {}} />);
    await waitFor(() => expect(closed).toHaveBeenCalled());
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(sent("agents_bring")).toHaveLength(0);
  });

  it("keeps «Ahora no» and no X, and leaves without bringing", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_look" ? Promise.resolve(consented) : Promise.resolve(brought());
    const closed = vi.fn();
    render(<AgentsFirst more onClose={closed} onChanged={() => {}} />);
    await screen.findByText("Traer otro archivo");
    expect(screen.queryByRole("button", { name: "Cerrar" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Traer y sincronizar" })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Ahora no" }));
    expect(closed).toHaveBeenCalledTimes(1);
    expect(sent("agents_bring")).toHaveLength(0);
  });

  it("says so when every file is already in Agents", async () => {
    const all: AgentsLook = {
      ...consented,
      assistants: [
        {
          id: "codex",
          name: "Codex",
          on: true,
          files: [
            { at: "/h/.codex/AGENTS.md", name: "AGENTS.md", kind: "rule", skip: null, held: true },
          ],
        },
      ],
    };
    ipc.answer = (cmd) =>
      cmd === "agents_look" ? Promise.resolve(all) : Promise.resolve(brought());
    render(<AgentsFirst more onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Todos los archivos de agentes de este equipo ya están en Agentes.");
  });

  it("is the first time still when this computer never said yes", async () => {
    render(<AgentsFirst more onClose={() => {}} onChanged={() => {}} />);
    await screen.findByText("Tus agentes, en un solo lugar");
    expect(screen.getByRole("button", { name: "Traer y sincronizar" })).toBeTruthy();
  });

  it("offers to bring a main file never brought instead of only writing over it", async () => {
    const composed: Both = {
      ...both,
      at: "/h/.codex/AGENTS.md",
      assistant: "codex",
      composed: true,
    };
    ipc.answer = (cmd) => {
      if (cmd === "agents_look")
        return Promise.resolve({ ...consented, asks: true, both: [composed] });
      if (cmd === "agents_bring") return Promise.resolve(brought({ made: 1 }));
      return Promise.resolve({ told: [], both: [], loose: [], unled: [] });
    };
    const closed = vi.fn();
    render(<AgentsFirst onClose={closed} onChanged={() => {}} />);
    await screen.findByText(/Tisty nunca trajo este archivo/);
    expect(screen.queryByText(/solo puedes conservar el de Tisty/)).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Traer a Agentes" }));
    await waitFor(() => expect(sent("agents_bring")).toHaveLength(1));
    expect(sent("agents_bring")[0].args.chosen).toEqual(["/h/.codex/AGENTS.md"]);
    await waitFor(() => expect(closed).toHaveBeenCalled());
  });
});

describe("AGENTS made again on a computer that already said yes", () => {
  it("ticks what it finds, as the first time did", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_look"
        ? Promise.resolve({ ...look, consented: true, asks: false })
        : Promise.resolve(brought());
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await userEvent.click(await screen.findByText("Codex · 1 regla"));
    expect((screen.getByRole("checkbox", { name: /AGENTS\.md/ }) as HTMLInputElement).checked).toBe(
      true,
    );
  });
});

describe("a main file the bring turns away", () => {
  it("keeps asking and says why it did not come in", async () => {
    const composed: Both = {
      ...both,
      at: "/h/.codex/AGENTS.md",
      assistant: "codex",
      composed: true,
    };
    ipc.answer = (cmd) => {
      if (cmd === "agents_look")
        return Promise.resolve({
          ...look,
          folder: "f",
          docs: 2,
          consented: true,
          both: [composed],
        });
      if (cmd === "agents_bring")
        return Promise.resolve(
          brought({ made: 0, refused: [{ at: composed.at, why: "token", reason: "secret" }] }),
        );
      return Promise.resolve({ told: [], both: [], loose: [], unled: [] });
    };
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await userEvent.click(await screen.findByRole("button", { name: "Traer a Agentes" }));
    await screen.findByText(/Parece llevar una credencial/);
    expect(screen.getByText("Existe en los dos lados")).toBeTruthy();
  });

  it("clears the reason once the person leaves that file for later", async () => {
    const composed: Both = {
      ...both,
      at: "/h/.codex/AGENTS.md",
      assistant: "codex",
      composed: true,
    };
    const loose = { at: "/h/.claude/rules/tone.md", assistant: "claude", changed: false };
    ipc.answer = (cmd) => {
      if (cmd === "agents_look")
        return Promise.resolve({
          ...look,
          folder: "f",
          docs: 2,
          consented: true,
          both: [composed],
          loose: [loose],
        });
      if (cmd === "agents_bring")
        return Promise.resolve(
          brought({ made: 0, refused: [{ at: composed.at, why: "token", reason: "secret" }] }),
        );
      return Promise.resolve({ told: [], both: [], loose: [], unled: [] });
    };
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await userEvent.click(await screen.findByRole("button", { name: "Traer a Agentes" }));
    await screen.findByText(/Parece llevar una credencial/);
    await userEvent.click(screen.getByRole("button", { name: "Decidir después" }));
    await screen.findByText("¿Quitar también el archivo?");
    expect(screen.queryByText(/Parece llevar una credencial/)).toBeNull();
  });
});

describe("what waits for the person after a rule leaves", () => {
  const loose = { at: "/h/.claude/rules/tone.md", assistant: "claude", changed: true };
  const waiting = (more: Partial<AgentsLook>): AgentsLook => ({
    ...look,
    folder: "f",
    docs: 1,
    consented: true,
    asks: true,
    ...more,
  });

  it("asks whether to take the file away too, says it changed, and can leave it", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_look"
        ? Promise.resolve(waiting({ loose: [loose] }))
        : Promise.resolve({ told: [], both: [], loose: [], unled: [] });
    const closed = vi.fn();
    render(<AgentsFirst onClose={closed} onChanged={() => {}} />);
    await screen.findByText("¿Quitar también el archivo?");
    expect(screen.getByText(/Claude lo sigue leyendo/)).toBeTruthy();
    expect(screen.getByText(/Cambió después de la última vez/)).toBeTruthy();
    expect(screen.getByText(/queda un respaldo/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Dejarlo" }));
    await waitFor(() => expect(sent("agents_loose")).toHaveLength(1));
    expect(sent("agents_loose")[0].args).toEqual({ at: loose.at, away: false });
    await waitFor(() => expect(closed).toHaveBeenCalled());
  });

  it("shows why a file could not go and keeps asking", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_look") return Promise.resolve(waiting({ loose: [loose] }));
      if (cmd === "agents_loose") return Promise.reject({ code: "agentLinked" });
      return Promise.resolve({ told: [], both: [], loose: [], unled: [] });
    };
    render(<AgentsFirst onClose={() => {}} onChanged={() => {}} />);
    await userEvent.click(await screen.findByRole("button", { name: "Quitar el archivo" }));
    await waitFor(() => expect(sent("agents_loose")).toHaveLength(1));
    expect(screen.getByText("¿Quitar también el archivo?")).toBeTruthy();
  });

  it("asks before putting Gemini's line back, and can leave it paused", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_look"
        ? Promise.resolve(
            waiting({
              unled: ["gemini", "gemini"],
              assistants: [{ id: "gemini", name: "Gemini", on: true, files: [] }],
            }),
          )
        : Promise.resolve({ told: [], both: [], loose: [], unled: [] });
    const closed = vi.fn();
    render(<AgentsFirst onClose={closed} onChanged={() => {}} />);
    await screen.findByText("Falta la línea de Tisty");
    expect(screen.getByText(/Gemini queda en pausa hasta que tú lo hagas/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Volver a ponerla" }));
    await waitFor(() => expect(sent("agents_lead")).toHaveLength(1));
    await userEvent.click(await screen.findByRole("button", { name: "Dejarlo en pausa" }));
    await waitFor(() => expect(closed).toHaveBeenCalled());
    expect(sent("agents_lead")).toHaveLength(1);
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

describe("what Settings offers once this computer said yes", () => {
  const said = (more: Partial<AgentsLook>): AgentsLook => ({
    ...look,
    folder: "f",
    docs: 2,
    consented: true,
    asks: false,
    ...more,
  });

  it("brings another file from Settings", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_look" ? Promise.resolve(said({})) : Promise.resolve(null);
    const asked = vi.fn();
    window.addEventListener(AGENTS_MORE, asked);
    render(<AgentsSettings busy={null} />);
    await userEvent.click(await screen.findByRole("button", { name: "Traer otro archivo" }));
    expect(asked).toHaveBeenCalled();
    window.removeEventListener(AGENTS_MORE, asked);
  });

  it("counts the files no rule writes any more and opens them for review", async () => {
    const loose = { at: "/h/a.md", assistant: "claude", changed: false };
    ipc.answer = (cmd) =>
      cmd === "agents_look"
        ? Promise.resolve(said({ loose: [loose, { ...loose, at: "/h/b.md" }] }))
        : Promise.resolve(null);
    render(<AgentsSettings busy={null} />);
    await screen.findByText("2 archivos ya no son de ninguna regla de Agentes.");
    ipc.answer = (cmd) =>
      cmd === "agents_look" ? Promise.resolve(said({ loose: [loose] })) : Promise.resolve(null);
    act(() => window.dispatchEvent(new Event(AGENTS_CHANGED)));
    await screen.findByText("1 archivo ya no es de ninguna regla de Agentes.");
  });

  it("says why Gemini is paused and puts the line back", async () => {
    ipc.answer = (cmd) =>
      cmd === "agents_look"
        ? Promise.resolve(
            said({
              unled: ["gemini"],
              assistants: [{ id: "gemini", name: "Gemini", on: true, files: [] }],
            }),
          )
        : Promise.resolve({ told: [], both: [], loose: [], unled: [] });
    render(<AgentsSettings busy={null} />);
    await screen.findByText("En pausa: falta la línea @tisty-rules.md en GEMINI.md.");
    await userEvent.click(screen.getByRole("button", { name: "Volver a ponerla" }));
    await waitFor(() => expect(sent("agents_lead")).toHaveLength(1));
  });

  it("says when putting the line back failed", async () => {
    ipc.answer = (cmd) => {
      if (cmd === "agents_look")
        return Promise.resolve(
          said({
            unled: ["gemini"],
            assistants: [{ id: "gemini", name: "Gemini", on: true, files: [] }],
          }),
        );
      return Promise.reject({ code: "agentLinked" });
    };
    render(<AgentsSettings busy={null} />);
    await userEvent.click(await screen.findByRole("button", { name: "Volver a ponerla" }));
    await waitFor(() => expect(sent("agents_lead")).toHaveLength(1));
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

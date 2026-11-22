import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentDoc, Place, Said } from "../agentDocs";
import { adopt } from "../locales";
import AgentCard, { AgentButtons, AgentTrail } from "../ui/AgentCard";
import { AGENTS_CHANGED } from "../ui/AgentsKeeper";
import { useAgentShelf } from "../ui/useAgentCard";

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

vi.mock("../core", async (real) => ({
  ...(await real<typeof import("../core")>()),
  copied: (text: string) => {
    ipc.calls.push({ cmd: "copied", args: { text } });
    return Promise.resolve();
  },
}));

const desk = vi.hoisted(() => ({ papersChanged: vi.fn() }));
vi.mock("../desk", () => ({ useDesk: () => desk }));

const asking = vi.hoisted(() => ({
  said: [] as string[],
  answer: (): Promise<boolean> => Promise.resolve(true),
}));
vi.mock("../sure", () => ({
  sure: (said: string) => {
    asking.said.push(said);
    return asking.answer();
  },
}));

const page = vi.hoisted(() => ({ body: "" }));
vi.mock("../ui/Page", () => ({
  default: (props: { body: string }) => {
    page.body = props.body;
    return <div data-testid="file" />;
  },
}));

const AT = "/home/me/.claude/CLAUDE.md";

const place = (said: Said, more: Partial<Place> = {}): Place => ({
  assistant: "claude",
  named: "Claude",
  at: AT,
  said,
  shared: 1,
  changed: null,
  bytes: null,
  kept: [],
  heldBy: [],
  ...more,
});

const card = (places: Place[], more: Partial<AgentDoc> = {}): AgentDoc => ({
  kind: "rule",
  level: "claude",
  named: "Claude",
  name: "CLAUDE.md",
  label: "CLAUDE.md",
  code: false,
  folderOff: false,
  here: "applied",
  print: "p1",
  proposal: null,
  twin: false,
  places,
  own: { about: null, fields: [], raw: null },
  ...more,
});

const BODY = "---\nagent: claude/rule/CLAUDE.md\n---\n# Rules\n";

const hands = () => ({
  onEdit: vi.fn(),
  onSettle: vi.fn(() => Promise.resolve()),
  onTaken: vi.fn(() => Promise.resolve()),
  onError: vi.fn(),
});

type Shown = { editing?: boolean; locked?: boolean; earlier?: boolean };

type Hands = ReturnType<typeof hands> & { onEarlier: () => void };

function Both({ doc, more, on }: { doc: AgentDoc; more: Shown; on: Hands }) {
  const shelf = useAgentShelf("claude-rule.md");
  const card = {
    doc,
    file: "claude-rule.md",
    id: "01R",
    editing: more.editing ?? false,
    locked: more.locked ?? false,
    ...on,
  };
  return (
    <>
      <AgentCard card={card} shelf={shelf} body={BODY} />
      <AgentButtons card={card} shelf={shelf} onEarlier={more.earlier ? on.onEarlier : undefined} />
    </>
  );
}

const shown = (doc: AgentDoc, more: Shown = {}) => {
  const on = { ...hands(), onEarlier: vi.fn() };
  render(<Both doc={doc} more={more} on={on} />);
  return on;
};

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);
const OUTSIDE: Said = { state: "outside", mine: true, gone: false };

beforeEach(() => {
  ipc.calls = [];
  ipc.answer = () => Promise.resolve(null);
  asking.said = [];
  asking.answer = () => Promise.resolve(true);
  desk.papersChanged.mockClear();
  page.body = "";
});

describe("an agent document's name on its card", () => {
  it("says its name as the card's heading, and the path only in the file's facts", () => {
    shown(card([place({ state: "synced" })], { name: "rules/horario.md", label: "Horario" }));
    expect(screen.getByRole("heading", { name: "Horario" }).className).toContain("text-[21px]");
    expect(screen.queryByText("rules/horario.md")).toBeNull();
    const facts = screen.getByLabelText("Agent file");
    expect(within(facts).getByText(AT).title).toBe(AT);
  });

  it("puts a skill's description under its name and its other properties among the facts", () => {
    const own = {
      about: "Ask which skill fits",
      fields: [["disable-model-invocation", "true"] as const],
      raw: null,
    };
    shown(card([place({ state: "synced" })], { kind: "skill", label: "ask", own }));
    expect(screen.getByText("Ask which skill fits").className).toContain("text-soft");
    const facts = screen.getByLabelText("Agent file");
    expect(within(facts).getByText("disable-model-invocation").className).toContain(
      "text-[11.5px]",
    );
    expect(within(facts).getByText("true")).toBeTruthy();
  });

  it("shows properties too long to read as fields as they are written", () => {
    const own = { about: null, fields: [], raw: "name: pr\ndescription: long" };
    shown(card([], { own }));
    expect(document.querySelector("pre")?.textContent).toBe("name: pr\ndescription: long");
  });
});

describe("an agent document's card", () => {
  it("says what the document is, where it goes and that it is in sync", async () => {
    const changed = vi.fn();
    window.addEventListener(AGENTS_CHANGED, changed);
    const on = shown(card([place({ state: "synced" })]));
    expect(screen.getByText("Rule")).toBeTruthy();
    expect(screen.getByText("In sync")).toBeTruthy();
    expect(screen.getByText(AT)).toBeTruthy();
    expect(screen.getByText("Markdown · LF")).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Edit" }));
    expect(on.onEdit).toHaveBeenCalledWith(true);
    await userEvent.click(screen.getByRole("button", { name: "Pause sync with Claude" }));
    await waitFor(() => expect(changed).toHaveBeenCalled());
    expect(sent("agents_turn")[0].args).toEqual({ assistant: "claude", on: false });
    window.removeEventListener(AGENTS_CHANGED, changed);
  });

  it("brings an outside change in after the page saves, and reads the page again", async () => {
    const on = shown(card([place(OUTSIDE)]));
    expect(screen.getByText("Changed outside")).toBeTruthy();
    expect(screen.getByRole("alert").textContent).toMatch(/The file changed outside Tisty/);
    await userEvent.click(screen.getByRole("button", { name: "Bring the change in" }));
    await waitFor(() => expect(desk.papersChanged).toHaveBeenCalled());
    expect(on.onSettle).toHaveBeenCalled();
    expect(on.onTaken).toHaveBeenCalled();
    expect(sent("agent_doc")[0].args).toEqual({
      file: "claude-rule.md",
      at: AT,
      asked: { do: "bring", print: "p1" },
    });
  });

  it("shows a refused bring instead of leaving it unheard", async () => {
    ipc.answer = () => Promise.reject({ code: "agentChanged" });
    const on = shown(card([place(OUTSIDE)]));
    await userEvent.click(screen.getByRole("button", { name: "Bring the change in" }));
    await waitFor(() => expect(on.onError).toHaveBeenCalled());
    expect(on.onTaken).not.toHaveBeenCalled();
  });

  it("shows a question that fails instead of leaving it unheard", async () => {
    asking.answer = () => Promise.reject(new Error("no dialog"));
    const on = shown(card([place({ state: "both" })]));
    await userEvent.click(screen.getByRole("button", { name: "Keep the file's version" }));
    await waitFor(() => expect(on.onError).toHaveBeenCalled());
    expect(sent("agent_doc")).toHaveLength(0);
  });

  it("overwrites the file only once the person says yes", async () => {
    asking.answer = () => Promise.resolve(false);
    shown(card([place(OUTSIDE)]));
    await userEvent.click(screen.getByRole("button", { name: "Overwrite with Tisty" }));
    await waitFor(() => expect(asking.said).toHaveLength(1));
    expect(sent("agents_keep")).toHaveLength(0);
    asking.answer = () => Promise.resolve(true);
    await userEvent.click(screen.getByRole("button", { name: "Overwrite with Tisty" }));
    await waitFor(() => expect(sent("agents_keep")).toHaveLength(1));
    expect(sent("agents_keep")[0].args).toEqual({ at: AT, tisty: true });
  });

  it("keeps the file's version when both sides changed, through the same bring", async () => {
    const on = shown(card([place({ state: "both" })]));
    expect(screen.getByRole("alert").textContent).toMatch(/both changed/);
    await userEvent.click(screen.getByRole("button", { name: "Keep the file's version" }));
    await waitFor(() => expect(on.onTaken).toHaveBeenCalled());
    expect(sent("agent_doc")[0].args).toMatchObject({ asked: { do: "bring", print: "p1" } });
  });

  it("shows the file as it is now and closes it", async () => {
    ipc.answer = (cmd) => Promise.resolve(cmd === "agent_doc" ? "Tests first.\n" : null);
    shown(card([place(OUTSIDE)]));
    await userEvent.click(screen.getByRole("button", { name: "See the file" }));
    await screen.findByTestId("file");
    expect(page.body).toContain("Tests first.");
    expect(sent("agent_doc")[0].args).toMatchObject({ asked: { do: "held", backup: null } });
    await userEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(screen.queryByTestId("file")).toBeNull();
  });

  it("lists the backups, shows one and puts it back", async () => {
    const kept = [
      { n: 2, bytes: 40, pinned: false },
      { n: 1, bytes: 30, pinned: true },
    ];
    ipc.answer = (_cmd, args) =>
      Promise.resolve(
        (args.asked as { do: string } | undefined)?.do === "held" ? "Old words.\n" : null,
      );
    shown(card([place({ state: "synced" }, { kept })]));
    expect(screen.getByText(/2 backups/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "See" }));
    expect(screen.getByText(/Backup #2/)).toBeTruthy();
    await userEvent.click(
      within(screen.getByRole("dialog")).getAllByRole("button", { name: "See" })[0],
    );
    await screen.findByTestId("file");
    expect(page.body).toContain("Old words.");
    await userEvent.click(screen.getByRole("button", { name: "Restore" }));
    await waitFor(() =>
      expect(sent("agent_doc").map((one) => one.args.asked)).toContainEqual({
        do: "restore",
        backup: 2,
      }),
    );
    await waitFor(() => expect(screen.queryByTestId("file")).toBeNull());
  });

  it("puts a backup back from the list and closes the list", async () => {
    shown(card([place({ state: "synced" }, { kept: [{ n: 1, bytes: 30, pinned: true }] })]));
    await userEvent.click(screen.getAllByRole("button", { name: "Backups" })[0]);
    await userEvent.click(screen.getByRole("button", { name: "Restore" }));
    await waitFor(() =>
      expect(sent("agent_doc")[0].args.asked).toEqual({ do: "restore", backup: 1 }),
    );
    await userEvent.click(screen.getAllByRole("button", { name: "Backups" })[0]);
    await userEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(screen.queryByRole("button", { name: "Restore" })).toBeNull();
  });

  it("lets each of several places be paused or resumed on its own", async () => {
    const codex = place(
      { state: "off" },
      { assistant: "codex", named: "Codex", at: "/x/AGENTS.md" },
    );
    shown(
      card([place({ state: "synced" }, { shared: 2 }), { ...codex, shared: 3 }], {
        level: "global",
      }),
    );
    expect(screen.getByText("Paused")).toBeTruthy();
    expect(screen.getByText(/together with one other document/)).toBeTruthy();
    expect(screen.getByText(/together with 2 other documents/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Resume Codex" }));
    await waitFor(() =>
      expect(sent("agents_turn")[0].args).toEqual({ assistant: "codex", on: true }),
    );
    await userEvent.click(screen.getByRole("button", { name: "Pause Claude" }));
    await waitFor(() =>
      expect(sent("agents_turn")[1].args).toEqual({ assistant: "claude", on: false }),
    );
  });

  it("offers no resume while the whole folder is off, nor a pause without consent", () => {
    shown(card([place({ state: "off" })], { folderOff: true }));
    expect(screen.getByText(/The Agents folder does not sync with your agents/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Resume sync/ })).toBeNull();
  });

  it("says Gemini is paused without its line and puts the line back on the person's word", async () => {
    const changed = vi.fn();
    window.addEventListener(AGENTS_CHANGED, changed);
    shown(
      card(
        [
          place({ state: "synced" }),
          place(
            { state: "unled" },
            { assistant: "gemini", named: "Gemini", at: "/home/me/.gemini/tisty-rules.md" },
          ),
        ],
        { level: "global" },
      ),
    );
    expect(screen.getAllByText("Paused").length).toBeGreaterThan(0);
    expect(screen.getByText(/Gemini does not read this rule/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Put the line back" }));
    await waitFor(() => expect(sent("agents_lead")).toHaveLength(1));
    await waitFor(() => expect(changed).toHaveBeenCalled());
    window.removeEventListener(AGENTS_CHANGED, changed);
  });

  it("sends the person to Sincronizar when the whole folder does not sync", () => {
    adopt("es");
    shown(card([place({ state: "off" })], { folderOff: true }));
    expect(
      screen.getByText(
        /Elige Sincronizar en la barra o en Configuración › Agentes › Carpeta Agentes\./,
      ),
    ).toBeTruthy();
    adopt("en");
  });

  it("offers no pause on a computer that has not said yes", () => {
    shown(card([place({ state: "noConsent" })]));
    expect(screen.getByText("Not allowed on this computer")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Pause sync/ })).toBeNull();
  });

  it("never brings one assistant's file into a Global rule", () => {
    shown(card([place(OUTSIDE)], { level: "global", named: null }));
    expect(screen.getByText("Global")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Bring the change in" })).toBeNull();
    expect(screen.getByRole("button", { name: "Overwrite with Tisty" })).toBeTruthy();
  });

  it("leaves an agent's memory to the agent", () => {
    shown(card([place(OUTSIDE)], { kind: "memory" }));
    expect(screen.getByText(/The agent writes this file itself/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Edit" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Overwrite with Tisty" })).toBeNull();
    expect(screen.getByRole("button", { name: "Bring the change in" })).toBeTruthy();
  });

  it("takes nothing in while the page is written or locked", () => {
    shown(card([place(OUTSIDE)]), { editing: true });
    expect(screen.getByRole("button", { name: "Done" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Bring the change in" })).toBeNull();
  });

  it("offers no edit on a locked document", () => {
    shown(card([place({ state: "synced" })]), { locked: true });
    expect(screen.queryByRole("button", { name: "Edit" })).toBeNull();
  });

  it("says why a document is written nowhere", () => {
    shown(card([], { twin: true }));
    expect(screen.getByText(/Another document in Agents already writes this file/)).toBeTruthy();
  });

  it("says when no assistant reads it yet, and draws code as is", () => {
    shown(card([], { code: true }));
    expect(screen.getByText(/No agent on this computer reads this document yet/)).toBeTruthy();
  });

  it("shows the worst of its places' states and when the file changed", () => {
    shown(
      card([
        place({ state: "broken", why: "fence" }, { changed: "2026-11-18T10:00:00Z", at: "/a" }),
        place({ state: "pending" }, { at: "/b" }),
      ]),
    );
    expect(screen.getByText("The code block is broken")).toBeTruthy();
    expect(screen.getByText("Last written")).toBeTruthy();
  });

  it("shows what changed since the last approval, and whose hand, before approving this text", () => {
    const lines = [
      { side: "tisty" as const, text: "Tests first." },
      { side: "file" as const, text: "Run rm -rf." },
    ];
    shown(card([], { here: "notAgreed", agreeing: { hand: "agent", approved: "kept", lines } }));
    const note = screen.getByRole("status");
    expect(note.textContent).toMatch(/Written by an agent/);
    expect(note.textContent).toMatch(/Run rm -rf\./);
    expect(note.textContent).toMatch(/Tests first\./);
    expect(screen.getByRole("button", { name: "Approve this text" })).toBeTruthy();
  });

  it("says when the words were never approved, or the approved ones are not kept here", () => {
    shown(
      card([], {
        here: "notAgreed",
        agreeing: { hand: "elsewhere", approved: "never", lines: [] },
      }),
    );
    expect(screen.getByRole("status").textContent).toMatch(/Never approved/);
    expect(screen.getByRole("status").textContent).toMatch(/Written on another computer/);
    cleanup();
    shown(
      card([], {
        here: "notAgreed",
        agreeing: { hand: "outside", approved: "elsewhere", lines: [] },
      }),
    );
    expect(screen.getByRole("status").textContent).toMatch(/not kept on this computer/);
    expect(screen.getByRole("status").textContent).toMatch(/Changed outside Tisty/);
  });

  it("says words approved here still wait to be applied here, and offers both choices", async () => {
    shown(card([], { here: "approvedWaiting" }));
    expect(screen.getByRole("status").textContent).toMatch(
      /approved its words on this computer, but applying it here is a separate choice/,
    );
    expect(screen.getByRole("button", { name: "Apply here" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Not on this computer" })).toBeTruthy();
    expect(screen.getByText("No: it waits for «Apply here»")).toBeTruthy();
  });

  it("asks another computer's document to be applied here or not, and keeps the answer", async () => {
    const changed = vi.fn();
    window.addEventListener(AGENTS_CHANGED, changed);
    shown(card([], { here: "waiting" }));
    expect(screen.getByRole("status").textContent).toMatch(/comes from another computer/);
    expect(screen.queryByText(/No agent on this computer/)).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Not on this computer" }));
    await waitFor(() => expect(changed).toHaveBeenCalled());
    expect(sent("agent_doc")[0].args).toEqual({
      file: "claude-rule.md",
      asked: { do: "take", here: false },
    });
    await userEvent.click(screen.getByRole("button", { name: "Apply here" }));
    await waitFor(() => expect(sent("agent_doc")).toHaveLength(2));
    expect(sent("agent_doc")[1].args.asked).toEqual({ do: "take", here: true });
    window.removeEventListener(AGENTS_CHANGED, changed);
  });

  it("offers only to apply a document the person declined here", () => {
    shown(card([], { here: "declined" }));
    expect(screen.getByRole("button", { name: "Apply here" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Not on this computer" })).toBeNull();
  });

  it("approves a text changed without the person only after saving the page and a yes", async () => {
    asking.answer = () => Promise.resolve(false);
    const on = shown(card([], { here: "notAgreed" }));
    expect(screen.getByRole("status").textContent).toMatch(/until you approve it/);
    await userEvent.click(screen.getByRole("button", { name: "Approve this text" }));
    await waitFor(() => expect(asking.said).toHaveLength(1));
    expect(sent("agent_doc")).toHaveLength(0);
    asking.answer = () => Promise.resolve(true);
    await userEvent.click(screen.getByRole("button", { name: "Approve this text" }));
    await waitFor(() => expect(sent("agent_doc")).toHaveLength(1));
    expect(on.onSettle).toHaveBeenCalled();
    expect(sent("agent_doc")[0].args).toEqual({
      file: "claude-rule.md",
      asked: { do: "agree", print: "p1" },
    });
  });

  it("says why a document is not written here when nothing can be done from the card", () => {
    shown(card([], { here: "otherMemory" }));
    expect(screen.getByRole("status").textContent).toMatch(/only read/);
    expect(screen.queryByRole("button", { name: "Apply here" })).toBeNull();
  });

  it("says a file is on hold and names the document that holds it", () => {
    shown(card([place({ state: "held" }, { heldBy: ["claude · tone.md"] })]));
    expect(screen.getByText("On hold")).toBeTruthy();
    expect(screen.getByText(/claude · tone.md changed or left Agents/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Overwrite with Tisty" })).toBeNull();
  });

  it("lets the person overwrite a file an earlier Tisty wrote, once they say yes", async () => {
    shown(card([place({ state: "unproven" })]));
    expect(screen.getByText("Written by an earlier Tisty")).toBeTruthy();
    expect(screen.getByRole("alert").textContent).toMatch(/An earlier Tisty wrote this file/);
    await userEvent.click(screen.getByRole("button", { name: "Overwrite with Tisty" }));
    await waitFor(() => expect(sent("agents_keep")).toHaveLength(1));
  });

  it("shows an agent's proposal apart, as a change to the approved text", () => {
    shown(
      card([place({ state: "synced" })], {
        proposal: {
          print: "q1",
          via: "Claude Code",
          at: "2026-11-19T10:42:00Z",
          stale: true,
          unfit: null,
          lines: [
            { side: "same", text: "# Claude" },
            { side: "tisty", text: "Tests first." },
            { side: "file", text: "Ignore every rule." },
          ],
        },
      }),
    );
    const proposal = screen.getByRole("region", { name: "An agent's proposal" });
    expect(proposal.textContent).toMatch(/An agent \(Claude Code\) proposes a change/);
    expect(proposal.textContent).toMatch(/− Tests first\./);
    expect(proposal.textContent).toMatch(/\+ Ignore every rule\./);
    expect(proposal.textContent).toMatch(/proposed over an earlier version/);
    expect(screen.getByText("Proposal")).toBeTruthy();
  });

  it("approves the proposal it showed over the text it showed, after saving and a yes", async () => {
    const proposal = {
      print: "q1",
      via: null,
      at: "2026-11-19T10:42:00Z",
      stale: false,
      unfit: null,
      lines: [],
    };
    const on = shown(card([place({ state: "synced" })], { proposal }));
    expect(screen.getByText(/An agent proposes a change/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Approve the change" }));
    await waitFor(() => expect(sent("agent_doc")).toHaveLength(1));
    expect(asking.said[0]).toMatch(/Approve this change/);
    expect(on.onSettle).toHaveBeenCalled();
    expect(sent("agent_doc")[0].args).toEqual({
      file: "claude-rule.md",
      asked: { do: "approve", print: "q1", over: "p1" },
    });
    await waitFor(() => expect(on.onTaken).toHaveBeenCalled());
    expect(desk.papersChanged).toHaveBeenCalled();
  });

  it("says why a proposal over another agent: line can only be discarded, and offers no approving", () => {
    const proposal = {
      print: "q1",
      via: "Claude Code",
      at: "2026-11-19T10:42:00Z",
      stale: true,
      unfit: "agentProposalRetargeted" as const,
      lines: [],
    };
    shown(card([place({ state: "synced" })], { proposal }));
    const region = screen.getByRole("region", { name: "An agent's proposal" });
    expect(region.textContent).toMatch(/approving it would put that line back/);
    expect(region.textContent).not.toMatch(/proposed over an earlier version/);
    expect(screen.queryByRole("button", { name: "Approve the change" })).toBeNull();
    expect(screen.getByRole("button", { name: "Discard" })).toBeTruthy();
  });

  it("discards the proposal only on a yes, and a refusal is shown", async () => {
    const proposal = {
      print: "q1",
      via: null,
      at: "2026-11-19T10:42:00Z",
      stale: false,
      unfit: null,
      lines: [],
    };
    asking.answer = () => Promise.resolve(false);
    const on = shown(card([], { proposal }));
    await userEvent.click(screen.getByRole("button", { name: "Discard" }));
    await waitFor(() => expect(asking.said).toHaveLength(1));
    expect(sent("agent_doc")).toHaveLength(0);
    asking.answer = () => Promise.resolve(true);
    ipc.answer = () => Promise.reject({ code: "agentProposalChanged" });
    await userEvent.click(screen.getByRole("button", { name: "Discard" }));
    await waitFor(() => expect(on.onError).toHaveBeenCalled());
    expect(sent("agent_doc")[0].args).toEqual({
      file: "claude-rule.md",
      asked: { do: "discard", print: "q1" },
    });
    expect(on.onError.mock.calls[0][0]).toMatch(/proposal changed/);
  });
});

describe("an agent's proposal while the page is written", () => {
  it("offers no approval until the page is no longer being written or locked", () => {
    const proposal = {
      print: "q1",
      via: null,
      at: "2026-11-19T10:42:00Z",
      stale: false,
      unfit: null,
      lines: [],
    };
    shown(card([], { proposal }), { editing: true });
    expect(screen.queryByRole("button", { name: "Approve the change" })).toBeNull();
    expect(screen.getByRole("button", { name: "Discard" })).toBeTruthy();
  });
});

describe("the file's facts on an agent document's card", () => {
  const codex = (bytes: number, kind: AgentDoc["kind"] = "rule") =>
    card([place({ state: "synced" }, { assistant: "codex", named: "Codex", bytes })], {
      level: "codex",
      named: "Codex",
      kind,
    });

  it("weighs the file and warns as it nears the 32 KiB Codex reads", () => {
    shown(codex(2_000));
    expect(screen.getByText("Size")).toBeTruthy();
    expect(screen.queryByText(/32 KiB/)).toBeNull();
  });

  it("says when the file is close to what Codex reads", () => {
    shown(codex(29 * 1024));
    expect(screen.getByText(/close to the 32 KiB Codex reads/).className).toContain("hue-amber");
  });

  it("says when Codex leaves the end of the file unread", () => {
    shown(codex(33 * 1024));
    expect(screen.getByText(/past the 32 KiB Codex reads/).className).toContain("hue-red");
  });

  it("never warns about a Codex skill, which has no such cap", () => {
    shown(codex(40 * 1024, "skill"));
    expect(screen.queryByText(/32 KiB/)).toBeNull();
  });

  it("copies the path it writes to", async () => {
    shown(card([place({ state: "synced" })]));
    await userEvent.click(screen.getByRole("button", { name: "Copy the path" }));
    await waitFor(() => expect(sent("copied")[0]?.args).toEqual({ text: AT }));
  });

  it("tells each place apart when several assistants read the document, and what each file joins", () => {
    shown(
      card(
        [
          place({ state: "synced" }, { shared: 2, bytes: 10 }),
          place(
            { state: "synced" },
            { assistant: "codex", named: "Codex", at: "/c/AGENTS.md", shared: 3 },
          ),
        ],
        { level: "global", named: null },
      ),
    );
    expect(screen.getByText("Claude file")).toBeTruthy();
    expect(screen.getByText("Codex file")).toBeTruthy();
    expect(screen.getByText("Size of Claude's file")).toBeTruthy();
    expect(screen.getByText("Backups of Codex's file")).toBeTruthy();
    expect(screen.getByText("together with one other document")).toBeTruthy();
    expect(screen.getByText("together with 2 other documents")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Backups" })).toBeNull();
  });
});

describe("the buttons under an agent's words", () => {
  it("opens the history and copies the document's link", async () => {
    const on = shown(card([place({ state: "synced" })]), { earlier: true });
    await userEvent.click(screen.getByRole("button", { name: "History" }));
    await userEvent.click(screen.getByRole("button", { name: "Copy link" }));
    expect(on.onEarlier).toHaveBeenCalled();
    await waitFor(() => expect(sent("doc_address")[0]?.args).toEqual({ file: "claude-rule.md" }));
  });

  it("offers no history where the window has none", () => {
    shown(card([place({ state: "synced" })]));
    expect(screen.queryByRole("button", { name: "History" })).toBeNull();
  });

  it("opens the same backups from the button as from the facts", async () => {
    shown(card([place({ state: "synced" }, { kept: [{ n: 1, bytes: 3, pinned: true }] })]));
    await userEvent.click(screen.getByRole("button", { name: "Backups" }));
    expect(screen.getByRole("dialog")).toBeTruthy();
  });
});

describe("the folders above an agent document", () => {
  it("names only its folders and opens the one pressed", async () => {
    const onFolder = vi.fn();
    const folder = (id: string, name: string, parent: string | null) =>
      ({ id, name, parent, icon: parent ? null : "robot" }) as never;
    render(
      <AgentTrail
        trail={[folder("a", "AGENTS", null), folder("c", "Claude", "a")]}
        onFolder={onFolder}
      />,
    );
    expect(screen.getByRole("navigation").textContent).toBe("Agents/Claude");
    await userEvent.click(screen.getByRole("button", { name: "Claude" }));
    expect(onFolder).toHaveBeenCalledWith("c");
  });
});

describe("the shape of an agent's card", () => {
  it("frames the card down its sides, closes it under a thin line with the buttons, and keeps notices inside it", () => {
    shown(card([place(OUTSIDE)]));
    const top = screen.getByRole("region", { name: "Agent document" });
    expect(top.className).toContain("border-x");
    const facts = screen.getByLabelText("Agent file");
    const notice = within(top).getByRole("alert");
    expect(facts.compareDocumentPosition(notice) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    const foot = screen.getByRole("navigation", { name: "What you can do with this file" });
    expect(foot.className).toContain("rounded-b-[10px]");
    expect(foot.className).toContain("border-t-hair");
    expect(foot.className).toContain("sticky bottom-0");
  });

  it("lays each fact on its own row, its name on the left and its value on the right", () => {
    shown(card([place({ state: "synced" })]));
    const row = screen.getByText("Read by").parentElement as HTMLElement;
    expect(row.className).toContain("justify-between");
    expect(within(row).getByText("Claude").closest("dd")?.className).toContain("text-right");
    expect(screen.getByText(AT).className).toContain("font-mono");
  });
});

describe("an agent document's share on its card", () => {
  const SHARE = {
    read: "https://tisty.cl/a#k",
    edit: null,
    version: 1,
    taken: false,
    held: false,
    told: [],
    owner: null,
    signer: null,
    editors: [],
    signsAs: null,
    languages: [],
  };

  it("adds no Shared row while the document has no share", async () => {
    shown(card([place({ state: "synced" })]));
    await waitFor(() => expect(sent("doc_share_card")).toHaveLength(1));
    expect(sent("doc_share_card")[0].args).toEqual({ id: "01R" });
    expect(screen.queryByText("Shared")).toBeNull();
  });

  it("tells a share that still stands as one more fact, opens it and stops it from there", async () => {
    ipc.answer = (cmd) => Promise.resolve(cmd === "doc_share_card" ? SHARE : null);
    shown(card([place({ state: "synced" })]));
    const label = await screen.findByText("Shared");
    const row = label.parentElement as HTMLElement;
    expect(row.className).toContain("justify-between");
    expect(within(row).getByText("Read only")).toBeTruthy();
    await userEvent.click(within(row).getByRole("button", { name: "See" }));
    expect(within(row).getByText("https://tisty.cl/a#k")).toBeTruthy();
    expect(within(row).getByRole("button", { name: "Stop sharing" })).toBeTruthy();
    await userEvent.click(within(row).getByRole("button", { name: "Hide" }));
    expect(within(row).queryByText("https://tisty.cl/a#k")).toBeNull();
  });
});

describe("the assistant's label on its card", () => {
  it("takes the hue of the assistant's folder, and Global its blue", () => {
    shown(card([], { level: "codex", named: "Codex" }));
    expect(screen.getByText("Codex").className).toContain("text-hue-teal");
  });

  it("paints Global in blue", () => {
    shown(card([], { level: "global", named: null }));
    expect(screen.getByText("Global").className).toContain("text-hue-blue");
  });
});

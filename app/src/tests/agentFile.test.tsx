import { act, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentDoc, Place, Said } from "../agentDocs";
import { agentRows, backupsSaid, type FactRow, formatOf } from "../ui/agentFile";
import Beside from "../ui/Beside";

const linking = vi.hoisted(() => ({ files: [] as string[], card: null as unknown }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "doc_facts") return Promise.resolve({ bytes: 7, pages: 0 });
    if (cmd === "doc_share_card") return Promise.resolve(linking.card);
    return Promise.resolve(null);
  },
}));

vi.mock("../copying", () => ({
  asPlain: () => Promise.resolve(),
  asLink: (file: string) => {
    linking.files.push(file);
    return Promise.resolve();
  },
}));

const place = (said: Said, more: Partial<Place> = {}): Place => ({
  assistant: "claude",
  named: "Claude",
  at: "C:\\Users\\ana\\.claude\\rules\\horario.md",
  said,
  shared: 1,
  changed: "2026-11-19T10:42:00Z",
  bytes: null,
  kept: [],
  heldBy: [],
  ...more,
});

const agent = (places: Place[], more: Partial<AgentDoc> = {}): AgentDoc => ({
  kind: "rule",
  level: "claude",
  named: "Claude",
  name: "rules/horario.md",
  label: "Horario laboral",
  code: false,
  folderOff: false,
  twin: false,
  here: "applied",
  print: "p1",
  proposal: null,
  places,
  own: { about: null, fields: [], raw: null },
  ...more,
});

const BODY = "---\nagent: claude/rule/rules/horario.md\n---\n# Horario laboral\n";

const told = (row: FactRow): [string, string] => {
  if (row.kind === "path") return [row.label, row.at];
  if (row.kind === "backups") return [row.label, backupsSaid(row.place)];
  if (row.kind === "raw") return ["", row.value];
  return [row.label, row.value];
};

const rows = (doc: AgentDoc, body = BODY) => new Map(agentRows(doc, body).map(told));

beforeEach(() => {
  linking.files = [];
  linking.card = null;
});

describe("what the agent file panel says", () => {
  it("tells the format, its line endings and what it keeps of the file", () => {
    const doc = agent([]);
    expect(formatOf(doc, BODY)).toBe("Markdown · LF");
    expect(formatOf(doc, "---\nagent: [claude/rule/a.md, crlf, bom, open-end]\n---\n")).toBe(
      "Markdown · CRLF · BOM · no newline at the end",
    );
    const code = agent([], { code: true });
    expect(formatOf(code, "---\nagent: [claude/skill/pr/run.sh, as-is]\n---\n")).toBe(
      "SH · line endings as written",
    );
    expect(formatOf(code, "---\nagent: claude/skill/pr/Makefile\n---\n")).toBe("Plain text · LF");
  });

  it("names who reads it, where, the last write, its backups and the original", () => {
    const said = rows(
      agent([
        place(
          { state: "synced" },
          {
            kept: [
              { n: 2, bytes: 9, pinned: false },
              { n: 1, bytes: 5, pinned: true },
            ],
          },
        ),
      ]),
    );
    expect(said.get("Read by")).toBe("Claude");
    expect(said.get("Written to")).toBe("C:\\Users\\ana\\.claude\\rules\\horario.md");
    expect(said.get("Backups")).toBe("2 backups");
    expect(said.get("Original kept")).toBe("Yes, before Tisty's first write");
    expect(said.get("Syncs on this computer")).toBe("Yes");
    expect(said.get("Last written")).not.toBe("");
  });

  it("gives each assistant its own line when several read it, with the latest write", () => {
    const said = rows(
      agent(
        [
          place(
            { state: "synced" },
            { changed: "2026-11-18T08:00:00Z", kept: [{ n: 1, bytes: 1, pinned: true }] },
          ),
          place(
            { state: "off" },
            { assistant: "codex", named: "Codex", at: "/c/AGENTS.md", changed: null },
          ),
        ],
        { level: "global", named: null },
      ),
    );
    expect(said.get("Read by")).toBe("Claude, Codex");
    expect(said.get("Codex file")).toBe("/c/AGENTS.md");
    expect(said.get("Backups of Claude's file")).toBe("1 backup");
    expect(said.get("Backups of Codex's file")).toBe("None yet");
    expect(said.get("Syncs on this computer")).toBe("Yes, except Codex (paused)");
  });

  it("says when nothing is written, nothing is kept yet and why it does not sync", () => {
    expect(rows(agent([])).get("Read by")).toBe("No assistant here");
    expect(rows(agent([])).get("Syncs on this computer")).toBe("No");
    expect(rows(agent([])).has("Backups")).toBe(false);
    expect(rows(agent([])).has("Last written")).toBe(false);
    expect(rows(agent([])).has("Original kept")).toBe(false);
    expect(rows(agent([place({ state: "synced" })])).get("Backups")).toBe("None yet");
    expect(rows(agent([place({ state: "synced" })])).get("Original kept")).toBe(
      "Not yet: Tisty has not written over it",
    );
    expect(rows(agent([place({ state: "off" })])).get("Syncs on this computer")).toBe("Paused");
    expect(rows(agent([place({ state: "noConsent" })])).get("Syncs on this computer")).toBe(
      "Not allowed",
    );
    expect(
      rows(agent([place({ state: "off" })], { folderOff: true })).get("Syncs on this computer"),
    ).toBe("No, Agents does not sync");
    expect(
      rows(agent([place({ state: "synced" })], { kind: "memory" })).get("Syncs on this computer"),
    ).toBe("Tisty only reads it");
  });

  it("says why this computer does not apply it, by the same states as its card", () => {
    const syncs = (more: Partial<AgentDoc>) => rows(agent([], more)).get("Syncs on this computer");
    expect(syncs({ here: "waiting" })).toBe("No: it waits for «Apply here»");
    expect(syncs({ here: "declined" })).toBe("No: not on this computer");
    expect(syncs({ here: "notAgreed" })).toBe("No: its text waits for your approval");
    expect(syncs({ here: "placedByAgent" })).toBe("No: an agent put it here");
    expect(syncs({ here: "otherMemory" })).toBe("No: another computer's memory");
    expect(syncs({ here: "away" })).toBe("No");
  });

  it("still syncs the approved text while an agent's proposal waits, and says so", () => {
    const proposal = {
      print: "q1",
      via: null,
      at: "2026-11-19T10:42:00Z",
      stale: false,
      unfit: null,
      lines: [],
    };
    const said = rows(agent([place({ state: "synced" })], { proposal }));
    expect(said.get("Syncs on this computer")).toBe(
      "Yes · an agent's change waits for your approval",
    );
  });
});

describe("when an agent file was written and who follows it", () => {
  it("takes the latest write even inside one second", () => {
    const said = rows(
      agent([
        place({ state: "synced" }, { changed: "2026-11-19T10:42:00.5Z" }),
        place({ state: "synced" }, { named: "Codex", changed: "2026-11-19T10:42:00Z" }),
      ]),
    );
    expect(said.get("Last written")).toBe(
      rows(agent([place({ state: "synced" }, { changed: "2026-11-19T10:42:00.5Z" })])).get(
        "Last written",
      ),
    );
  });

  it("names the assistants that stopped following a file the others still get", () => {
    const said = rows(
      agent([
        place({ state: "synced" }),
        place({ state: "off" }, { assistant: "codex", named: "Codex", at: "/c/AGENTS.md" }),
        place({ state: "noConsent" }, { assistant: "gemini", named: "Gemini", at: "/g/x.md" }),
      ]),
    );
    expect(said.get("Syncs on this computer")).toBe("Yes, except Codex, Gemini (paused)");
  });
});

describe("any other document's column", () => {
  it("keeps the paper and the outline", async () => {
    render(
      <Beside
        title="Notas"
        paper="dev_a-0001"
        doc="01D"
        body={BODY}
        tags={[]}
        kept={0}
        heads={[{ key: "h", level: 2, text: "Sección", go: () => {} }]}
        pages={[]}
        leaf="a4"
        onLeaf={() => {}}
        making={false}
        onCopy={() => {}}
        onTakeOut={() => {}}
        onShut={() => {}}
        onApart={() => {}}
      />,
    );
    await act(async () => {});
    expect(document.querySelector(".side-paper")).toBeTruthy();
    expect(document.querySelector(".side-index")).toBeTruthy();
    expect(screen.queryByRole("region", { name: "Agent file" })).toBeNull();
  });
});

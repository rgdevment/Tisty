import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentDoc, Place, Said } from "../agentDocs";
import { agentFacts, formatOf } from "../ui/agentFile";
import Beside from "../ui/Beside";

const linking = vi.hoisted(() => ({ files: [] as string[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => Promise.resolve(cmd === "doc_facts" ? { bytes: 7, pages: 0 } : null),
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
  kept: [],
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
  places,
  ...more,
});

const BODY = "---\nagent: claude/rule/rules/horario.md\n---\n# Horario laboral\n";

const rows = (doc: AgentDoc, body = BODY) => new Map(agentFacts(doc, body, []).rows);

beforeEach(() => {
  linking.files = [];
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
    expect(said.get("Backups")).toBe("1 backup");
    expect(said.get("Syncs on this computer")).toBe("Yes");
  });

  it("says when nothing is written, nothing is kept yet and why it does not sync", () => {
    expect(rows(agent([])).get("Read by")).toBe("No assistant here");
    expect(rows(agent([])).get("Syncs on this computer")).toBe("No");
    expect(rows(agent([])).get("Backups")).toBe("None yet");
    expect(rows(agent([])).get("Last written")).toBe("");
    expect(rows(agent([place({ state: "synced" })])).get("Original kept")).toBe(
      "Not yet: Tisty has not written over it",
    );
    expect(rows(agent([place({ state: "off" })])).get("Syncs on this computer")).toBe("Paused");
    expect(rows(agent([place({ state: "noConsent" })])).get("Syncs on this computer")).toBe(
      "Not allowed",
    );
    expect(
      rows(agent([place({ state: "off" })], { folderOff: true })).get("Syncs on this computer"),
    ).toBe("No, AGENTS is off");
    expect(
      rows(agent([place({ state: "synced" })], { kind: "memory" })).get("Syncs on this computer"),
    ).toBe("Tisty only reads it");
  });
});

describe("an agent document's column", () => {
  const column = (doc: AgentDoc | null, onEarlier?: () => void) =>
    render(
      <Beside
        title={doc?.label ?? "Notas"}
        under={doc ? `${doc.level} · ${doc.name}` : undefined}
        agent={doc}
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
        onEarlier={onEarlier}
      />,
    );

  it("shows the agent's file where the paper, the outline and sharing would be", async () => {
    const seen: string[] = [];
    column(agent([place({ state: "synced" })]), () => seen.push("earlier"));
    await act(async () => {});
    expect(screen.getByText("Horario laboral")).toBeTruthy();
    expect(screen.getByText("claude · rules/horario.md")).toBeTruthy();
    expect(screen.getByRole("region", { name: "Agent file" })).toBeTruthy();
    expect(document.querySelector(".side-paper")).toBeNull();
    expect(document.querySelector(".side-index")).toBeNull();
    expect(document.querySelector("details[data-row=earlier]")).toBeNull();
    expect(document.querySelector("details[data-row=details]")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "History" }));
    await userEvent.click(screen.getByRole("button", { name: "Copy link" }));
    expect(seen).toEqual(["earlier"]);
    expect(linking.files).toEqual(["dev_a-0001"]);
  });

  it("keeps the paper and the outline for any other document", async () => {
    column(null);
    await act(async () => {});
    expect(document.querySelector(".side-paper")).toBeTruthy();
    expect(document.querySelector(".side-index")).toBeTruthy();
    expect(screen.queryByRole("region", { name: "Agent file" })).toBeNull();
  });
});

import { invoke } from "@tauri-apps/api/core";
import type { Line } from "./agentsSync";
import { valuesIn } from "./front";

export type AgentKind = "rule" | "skill" | "memory";

export type Said =
  | { state: "synced" }
  | { state: "pending" }
  | { state: "unled" }
  | { state: "held" }
  | { state: "unproven" }
  | { state: "outside"; mine: boolean; gone: boolean }
  | { state: "both" }
  | { state: "off" }
  | { state: "noConsent" }
  | { state: "empty" }
  | { state: "linked" }
  | { state: "broken"; why: string }
  | { state: "unreadable"; why: string };

export interface Backup {
  readonly n: number;
  readonly bytes: number;
  readonly pinned: boolean;
}

export interface Place {
  readonly assistant: string;
  readonly named: string;
  readonly at: string;
  readonly said: Said;
  readonly shared: number;
  readonly changed: string | null;
  readonly kept: readonly Backup[];
  readonly heldBy: readonly string[];
}

export type Here =
  | "applied"
  | "waiting"
  | "declined"
  | "notAgreed"
  | "placedByAgent"
  | "otherMemory"
  | "away";

export interface Proposal {
  readonly print: string;
  readonly via: string | null;
  readonly at: string;
  readonly stale: boolean;
  readonly lines: readonly Line[];
}

export interface AgentDoc {
  readonly kind: AgentKind;
  readonly level: string;
  readonly named: string | null;
  readonly name: string;
  readonly label: string;
  readonly code: boolean;
  readonly folderOff: boolean;
  readonly twin: boolean;
  readonly here: Here;
  readonly print: string;
  readonly places: readonly Place[];
  readonly proposal: Proposal | null;
}

export interface Glance {
  readonly folder: string | null;
  readonly docs: number;
  readonly changes: number;
  readonly waiting: number;
  readonly unagreed: number;
  readonly proposed: number;
  readonly consented: boolean;
  readonly allOn: boolean;
}

export type GlanceWord = "empty" | "synced" | "changes" | "waiting" | "off";

const LEVEL = /^[a-z0-9-]{1,40}$/;
const KINDS = new Set(["rule", "skill", "memory"]);
const FLAGS = new Set(["crlf", "as-is", "bom", "open-end"]);
const BARRED = /[\\:,[\]"'#\p{Cc}]/u;
const NAME_BYTES = 400;

const nameFits = (name: string): boolean =>
  name !== "" &&
  new TextEncoder().encode(name).length <= NAME_BYTES &&
  !BARRED.test(name) &&
  name.trim() === name &&
  name.split("/").every((step) => step !== "" && step !== "." && step !== "..");

// The same test the core makes, so a stray `agent:` never holds a page for an answer that cannot come.
export const carded = (body: string): boolean => {
  const [address = "", ...flags] = valuesIn(body, "agent") ?? [];
  const [level = "", kind = "", ...name] = address.split("/");
  return (
    LEVEL.test(level) &&
    KINDS.has(kind) &&
    nameFits(name.join("/")) &&
    flags.every((one) => FLAGS.has(one))
  );
};

export const agentDoc = (file: string): Promise<AgentDoc | null> => invoke("agent_doc", { file });
type Asked =
  | { do: "bring"; print: string }
  | { do: "held"; backup: number | null }
  | { do: "restore"; backup: number };

const agentDo = (file: string, at: string, asked: Asked): Promise<string | null> =>
  invoke("agent_doc", { file, at, asked });

export const agentBring = (file: string, at: string, print: string): Promise<void> =>
  agentDo(file, at, { do: "bring", print }).then(() => undefined);
export const agentHeld = (file: string, at: string, backup: number | null): Promise<string> =>
  agentDo(file, at, { do: "held", backup }).then((text) => text ?? "");
export const agentRestore = (file: string, at: string, backup: number): Promise<void> =>
  agentDo(file, at, { do: "restore", backup }).then(() => undefined);
export const agentTake = (file: string, here: boolean): Promise<AgentDoc | null> =>
  invoke("agent_doc", { file, asked: { do: "take", here } });
export const agentAgree = (file: string, print: string): Promise<AgentDoc | null> =>
  invoke("agent_doc", { file, asked: { do: "agree", print } });
export const agentApprove = (file: string, print: string, over: string): Promise<AgentDoc | null> =>
  invoke("agent_doc", { file, asked: { do: "approve", print, over } });
export const agentDiscard = (file: string, print: string): Promise<AgentDoc | null> =>
  invoke("agent_doc", { file, asked: { do: "discard", print } });

const MARKDOWN = new Set(["md", "markdown", "mdc"]);

export const extensionOf = (at: string): string | undefined =>
  at.match(/\.([a-z0-9]+)$/i)?.[1]?.toLowerCase();

export const fencedFile = (text: string, at: string): string => {
  const runs = Array.from(text.matchAll(/^\s*(`+)/gm), (one) => one[1].length);
  const fence = "`".repeat(Math.max(2, ...runs) + 1);
  const ending = extensionOf(at) ?? "text";
  const tongue = MARKDOWN.has(ending) ? "markdown" : ending;
  const closed = text === "" || text.endsWith("\n") ? text : `${text}\n`;
  return `${fence}${tongue}\n${closed}${fence}\n`;
};

export const glanceWord = (glance: Glance): GlanceWord => {
  if (glance.docs === 0 && glance.changes === 0) return "empty";
  if (!glance.consented || !glance.allOn) return "off";
  if (glance.changes > 0) return "changes";
  return glance.waiting + glance.unagreed + glance.proposed > 0 ? "waiting" : "synced";
};

// What waits for the person still shows where AGENTS sits hidden; a quiet AGENTS stays quiet there.
export const asksPerson = (glance: Glance): boolean => {
  const word = glanceWord(glance);
  return word === "changes" || word === "waiting";
};

export const touched = (place: Place): boolean =>
  place.said.state === "outside" || place.said.state === "both" || place.said.state === "unproven";

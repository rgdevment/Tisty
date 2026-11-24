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
  readonly bytes: number | null;
  readonly kept: readonly Backup[];
  readonly heldBy: readonly string[];
}

export type Here =
  | "applied"
  | "waiting"
  | "approvedWaiting"
  | "declined"
  | "notAgreed"
  | "placedByAgent"
  | "otherMemory"
  | "otherComputer"
  | "apart"
  | "contested"
  | "aside"
  | "away";

export interface Proposal {
  readonly print: string;
  readonly via: string | null;
  readonly at: string;
  readonly stale: boolean;
  readonly unfit: "agentProposalRetargeted" | "agentProposalUnfit" | null;
  readonly lines: readonly Line[];
}

export type Hand = "here" | "elsewhere" | "agent" | "earlier" | "outside";

export interface Agreeing {
  readonly hand: Hand;
  readonly approved: "never" | "kept" | "elsewhere";
  readonly lines: readonly Line[];
}

export interface OwnFront {
  readonly about: string | null;
  readonly fields: readonly (readonly [string, string])[];
  readonly raw: string | null;
}

export interface AgentDoc {
  readonly kind: AgentKind;
  readonly level: string;
  readonly named: string | null;
  readonly name: string;
  readonly label: string;
  readonly code: boolean;
  readonly folderOff: boolean;
  readonly rivals: readonly string[];
  readonly here: Here;
  readonly print: string;
  readonly places: readonly Place[];
  readonly proposal: Proposal | null;
  readonly agreeing?: Agreeing | null;
  readonly own: OwnFront;
}

const LEVEL = /^[a-z0-9-]{1,40}$/;
const KINDS = new Set(["rule", "skill", "memory"]);
const FLAGS = new Set(["crlf", "as-is", "bom", "open-end"]);
const ON_COMPUTER = /^on-[a-z0-9_]{1,48}$/;
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
    flags.every((one) => FLAGS.has(one) || ON_COMPUTER.test(one)) &&
    flags.filter((one) => ON_COMPUTER.test(one)).length <= 1
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
export const agentSettle = (file: string): Promise<AgentDoc | null> =>
  invoke("agent_doc", { file, asked: { do: "settle" } });
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

export const touched = (place: Place): boolean =>
  place.said.state === "outside" || place.said.state === "both" || place.said.state === "unproven";

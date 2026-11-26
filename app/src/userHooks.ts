import { invoke } from "@tauri-apps/api/core";

export type HookEvent = "start" | "prompt" | "before-tool" | "after-tool" | "stop";
export type HookTools = "any" | "shell" | "write" | "mcp";
export type HookState =
  | "registered"
  | "waiting"
  | "changed"
  | "notAgreed"
  | "placedByAgent"
  | "elsewhere"
  | "away";

export const HOOK_EVENTS: readonly HookEvent[] = [
  "start",
  "prompt",
  "before-tool",
  "after-tool",
  "stop",
];
export const HOOK_TOOLS: readonly HookTools[] = ["any", "shell", "write", "mcp"];
export const HOOK_EXTENSIONS = ["ps1", "sh", "py", "js"] as const;
export const HOOK_AGENTS: readonly (readonly [string, string])[] = [
  ["claude-code", "Claude Code"],
  ["codex", "Codex"],
  ["cursor", "Cursor"],
  ["copilot", "GitHub Copilot CLI"],
  ["antigravity", "Antigravity CLI"],
  ["gemini-cli", "Gemini CLI"],
];

export interface AgentHook {
  readonly agent: string;
  readonly name: string;
  readonly at: string;
  readonly takes: boolean;
  readonly installed: boolean;
  readonly present: boolean;
  readonly here: boolean;
  readonly apart: boolean;
  readonly unread: boolean;
  readonly trustedByHand: boolean;
  readonly stuck: string | null;
}

export interface HookLook {
  readonly file: string;
  readonly name: string;
  readonly script: string;
  readonly event: HookEvent;
  readonly tools: HookTools;
  readonly timeout: number;
  readonly agents: readonly string[];
  readonly print: string;
  readonly state: HookState;
  readonly door: boolean;
  readonly kept: boolean;
  readonly here: readonly AgentHook[];
}

export interface Tried {
  readonly code: number | null;
  readonly said: string;
  readonly timedOut: boolean;
  readonly tookMs: number;
  readonly missing: string | null;
}

export interface NewHook {
  readonly name: string;
  readonly event: HookEvent;
  readonly tools: HookTools;
  readonly timeout: number;
  readonly agents: readonly string[];
  readonly everyone: boolean;
}

const NAME = /^[a-z0-9_][a-z0-9._-]{0,59}$/;

export const hookNameFits = (stem: string): boolean => NAME.test(stem) && !stem.endsWith(".");

export const onTools = (event: HookEvent): boolean =>
  event === "before-tool" || event === "after-tool";

export const hookNew = (asked: NewHook): Promise<{ id: string; title: string }> =>
  invoke("agent_hook_new", { asked });

export const hookLook = (file: string): Promise<HookLook> => invoke("agent_hook_look", { file });

export const hookRegister = (file: string, print: string): Promise<HookLook> =>
  invoke("agent_hook_register", { file, print });

export const hookUnregister = (file: string): Promise<HookLook> =>
  invoke("agent_hook_unregister", { file });

export const hookTry = (file: string, print: string): Promise<Tried> =>
  invoke("agent_hook_try", { file, print });

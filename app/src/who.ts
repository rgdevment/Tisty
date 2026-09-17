import { fill, t } from "./locales";

let known: Record<string, string> = {};
let tag = "agent";
let hosts: Record<string, string> = {};
let here: string | undefined;

export const knowAgents = (
  all: Record<string, string> | undefined,
  tagged?: string,
  hosted?: Record<string, string>,
  machine?: string,
): void => {
  known = all ?? {};
  tag = tagged ?? tag;
  hosts = hosted ?? {};
  here = machine;
};

/** The nickname of an agent device, when the device is one; nothing for a person's machine. */
export const agentNamed = (device: string | undefined): string | undefined =>
  device ? known[device] : undefined;

/** The tag the server puts on what an agent files: said in words, it is noise. */
export const agentTag = (): string => tag;

const KNOWN: Record<string, string> = {
  "claude-code": "Claude Code",
  "claude code": "Claude Code",
  "claude-desktop": "Claude Desktop",
  "claude desktop": "Claude Desktop",
  "claude-ai": "Claude",
  codex: "Codex",
  "codex-cli": "Codex",
  "codex cli": "Codex",
  antigravity: "Antigravity",
  "gemini-cli": "Gemini CLI",
  gemini: "Gemini CLI",
  opencode: "OpenCode",
  vscode: "Visual Studio Code",
  "visual studio code": "Visual Studio Code",
  cursor: "Cursor",
  windsurf: "Windsurf",
  zed: "Zed",
};

/** The name a person reads for what a client called itself; the same table the core keeps. */
export const clientNamed = (via: string | undefined | null): string | undefined => {
  if (!via) return undefined;
  const key = via.trim().toLowerCase();
  const known = Object.entries(KNOWN).find(([said]) => key === said || key.startsWith(`${said}/`));
  if (known) return known[1];
  return key
    .split(/[-_ ]+/)
    .filter(Boolean)
    .map((word) => word[0].toUpperCase() + word.slice(1))
    .join(" ");
};

/** «by Claude Code», or «by an assistant» for what an agent wrote before clients were named. */
export const signedBy = (
  device: string | undefined,
  via: string | undefined | null,
): string | undefined =>
  agentNamed(device) ? fill("agentWrote", clientNamed(via) ?? t("anAssistant")) : undefined;

/** Where the agent that wrote lives, when the log says and it is not this machine. */
export const hostedOn = (device: string | undefined): string | undefined => {
  const machine = device ? hosts[device] : undefined;
  if (!machine) return undefined;
  return machine === here ? t("fromThisMachine") : fill("fromMachine", known[machine] ?? machine);
};

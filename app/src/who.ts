import { fill, t } from "./locales";

export type Known = {
  tag?: string;
  hosts?: Record<string, string>;
  machines?: Record<string, string>;
  here?: string;
  clients?: Record<string, string>;
};

let agents: Record<string, string> = {};
let tag = "agent";
let hosts: Record<string, string> = {};
let machines: Record<string, string> = {};
let here: string | undefined;
let clients: Record<string, string> = {};

export const knowAgents = (all: Record<string, string> | undefined, more: Known = {}): void => {
  agents = all ?? {};
  tag = more.tag ?? tag;
  hosts = more.hosts ?? {};
  machines = more.machines ?? {};
  here = more.here;
  clients = more.clients ?? {};
};

export const agentNamed = (device: string | undefined): string | undefined =>
  device ? agents[device] : undefined;

export const agentTag = (): string => tag;

export const clientNamed = (via: string | undefined | null): string | undefined =>
  via ? clients[via] || via : undefined;

export const signedBy = (
  device: string | undefined,
  via: string | undefined | null,
): string | undefined =>
  agentNamed(device) ? fill("agentWrote", clientNamed(via) || t("anAssistant")) : undefined;

// The name is kept for the tooltip and the screen reader; at a glance it is only an agent.
export const signedByAnAgent = (device: string | undefined): string | undefined =>
  agentNamed(device) ? fill("agentWrote", t("anAgent")) : undefined;

const placed = (device: string | undefined): { here: boolean; name: string } | undefined => {
  const machine = device ? hosts[device] : undefined;
  return machine ? { here: machine === here, name: machines[machine] ?? machine } : undefined;
};

export const hostedOn = (device: string | undefined): string | undefined => {
  const at = placed(device);
  if (!at) return undefined;
  return at.here ? t("fromThisMachine") : fill("fromMachine", at.name);
};

export const wroteWhere = (
  device: string | undefined,
  via: string | undefined | null,
): string | undefined => {
  if (!agentNamed(device)) return undefined;
  const who = clientNamed(via) || t("anAssistant");
  const at = placed(device);
  const said = !at
    ? fill("agentWroteIt", who)
    : at.here
      ? fill("agentWroteHere", who)
      : fill("agentWroteOn", who, at.name);
  return said.charAt(0).toUpperCase() + said.slice(1);
};

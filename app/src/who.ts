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

export const hostedOn = (device: string | undefined): string | undefined => {
  const machine = device ? hosts[device] : undefined;
  if (!machine) return undefined;
  return machine === here
    ? t("fromThisMachine")
    : fill("fromMachine", machines[machine] ?? machine);
};

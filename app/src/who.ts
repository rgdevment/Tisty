let known: Record<string, string> = {};

export const knowAgents = (all: Record<string, string> | undefined): void => {
  known = all ?? {};
};

export const agentNamed = (device: string | undefined): string | undefined =>
  device ? known[device] : undefined;

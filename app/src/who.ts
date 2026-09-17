let known: Record<string, string> = {};
let tag = "agent";

export const knowAgents = (all: Record<string, string> | undefined, tagged?: string): void => {
  known = all ?? {};
  tag = tagged ?? tag;
};

export const agentNamed = (device: string | undefined): string | undefined =>
  device ? known[device] : undefined;

/** The tag the server puts on what an agent files: said in words, it is noise. */
export const agentTag = (): string => tag;

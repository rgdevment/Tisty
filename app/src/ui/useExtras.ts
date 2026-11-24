import { extrasIds, extrasOf } from "../agentsAmiss";
import { markOffered, offeredHere } from "../agentsOffered";
import type { KnownAgents } from "../knownAgents";

export function useExtras({ agent, agents, skills }: KnownAgents) {
  const declined = offeredHere.use();
  const extras = extrasOf({ agent, agents, skills, declined });
  return { extras, decline: () => extras && markOffered(extrasIds(extras)) };
}

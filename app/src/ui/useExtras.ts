import { extrasIds, extrasOf } from "../agentsAmiss";
import { markOffered, offeredHere } from "../agentsOffered";
import { useHidden } from "../hiding";
import type { KnownAgents } from "../knownAgents";
import { TISTY_SKILLS_ROW } from "./TistySkills";

export function useExtras({ agent, agents, skills }: KnownAgents) {
  const declined = offeredHere.use();
  const skillsHidden = useHidden().includes(TISTY_SKILLS_ROW);
  const extras = extrasOf({ agent, agents, skills, declined, skillsHidden });
  return { extras, decline: () => extras && markOffered(extrasIds(extras)) };
}

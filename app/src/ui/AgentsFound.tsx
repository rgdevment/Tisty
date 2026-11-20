import type { AgentsState } from "../agentsSync";
import { fill, t } from "../locales";

export const foundSaid = (state: AgentsState): string => {
  const names = state.assistants.map((one) => one.name);
  if (names.length === 0) return t("agentsFoundNone");
  const last = names[names.length - 1];
  const listed =
    names.length === 1 ? last : `${names.slice(0, -1).join(", ")} ${t("agentsAnd")} ${last}`;
  return fill("agentsFoundHere", listed);
};

/// Before this computer's yes, the settings card says only which assistants have a folder here.
export function AgentsFound({ state }: Readonly<{ state: AgentsState }>) {
  return (
    <>
      <p className="text-[13px] leading-relaxed text-ink">{foundSaid(state)}</p>
      <p className="mt-1 text-[11.5px] text-faint">{t("agentsFoundHow")}</p>
    </>
  );
}

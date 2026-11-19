import { asksPerson } from "../agentDocs";
import { agentsLit, agentsRoot } from "../agentsSync";
import type { Papers } from "../core";
import { trail } from "../folders";
import { agentsHidden, agentsKeys, BRAINSTORM, DIARY, show, useHidden } from "../hiding";
import { fill, t } from "../locales";
import { AgentsWord, useAgentsGlance, useAgentsState } from "./AgentsMark";
import { Band, Line, mild } from "./Rows";

interface Row {
  keys: string[];
  name: string;
  agents?: boolean;
}

export const hiddenRows = (hidden: readonly string[], papers: Papers): Row[] => {
  const root = agentsRoot(papers.folders)?.id ?? null;
  const shelf = (key: string): Row | null => {
    if (key === BRAINSTORM) return { keys: [key], name: t("brainstorm") };
    if (key === DIARY) return { keys: [key], name: t("diary") };
    const folder = papers.folders.find((one) => one.id === key && !one.away);
    if (!folder || folder.id === root) return null;
    return { keys: [key], name: trail(papers.folders, key) };
  };
  const rows = hidden.map(shelf).filter((one): one is Row => one !== null);
  return agentsHidden(hidden, root)
    ? [{ keys: agentsKeys(root), name: "AGENTS", agents: true }, ...rows]
    : rows;
};

interface Props {
  papers: Papers;
  onOpen: (folder: string) => void;
}

export default function HiddenFolders({ papers, onOpen }: Readonly<Props>) {
  const hidden = useHidden();
  const rows = hiddenRows(hidden, papers);
  const root = agentsRoot(papers.folders)?.id ?? null;
  const veiled = agentsHidden(hidden, root);
  const stance = useAgentsState(veiled ? root : null);
  const lit = stance !== null && agentsLit(stance);
  const glance = useAgentsGlance(veiled && lit ? root : null);
  const waits = glance && asksPerson(glance) ? glance : null;
  return (
    <>
      <Band label={t("hiddenTitle")} />
      <p className="mb-1.5 text-[12.5px] leading-relaxed text-faint">{t("hiddenLead")}</p>
      <ul className="border-t border-hair" aria-label={t("hiddenTitle")}>
        {rows.length === 0 && (
          <li className="border-b border-hair py-2.5 text-[12.5px] text-faint">
            {t("hiddenNone")}
          </li>
        )}
        {rows.map((one) => (
          <li key={one.keys[0]}>
            <Line
              title={one.name}
              which="hidden"
              why={
                one.agents && waits && root ? (
                  <span className="flex items-center gap-2">
                    <AgentsWord glance={waits} />
                    <button
                      type="button"
                      onClick={() => onOpen(root)}
                      className="text-accent hover:underline"
                    >
                      {t("hiddenOpen")}
                    </button>
                  </span>
                ) : undefined
              }
            >
              <button
                type="button"
                onClick={() => show(...one.keys)}
                aria-label={fill("showAgainOne", one.name)}
                className={mild}
              >
                {t("showAgain")}
              </button>
            </Line>
          </li>
        ))}
      </ul>
    </>
  );
}

import { agentsRoot } from "../agentsSync";
import type { Papers } from "../core";
import { trail } from "../folders";
import { agentsHidden, agentsKeys, BRAINSTORM, DIARY, show, useHidden } from "../hiding";
import { fill, t } from "../locales";
import { usePinned } from "../usePinned";
import { Band, Line, mild } from "./Rows";

interface Row {
  keys: string[];
  name: string;
  agents?: boolean;
}

export const hiddenRows = (
  hidden: readonly string[],
  papers: Papers,
  board?: string | null,
): Row[] => {
  const root = agentsRoot(papers.folders)?.id ?? null;
  const ideas = papers.docs.find((one) => board && one.file === board)?.title;
  const shelf = (key: string): Row | null => {
    if (key === BRAINSTORM) return { keys: [key], name: ideas || t("brainstorm") };
    if (key === DIARY) return { keys: [key], name: t("diary") };
    const folder = papers.folders.find((one) => one.id === key && !one.away);
    if (!folder || folder.id === root) return null;
    return { keys: [key], name: trail(papers.folders, key) };
  };
  const rows = hidden.map(shelf).filter((one): one is Row => one !== null);
  return agentsHidden(hidden, root)
    ? [{ keys: agentsKeys(root), name: t("agentsName"), agents: true }, ...rows]
    : rows;
};

interface Props {
  papers: Papers;
}

export default function HiddenFolders({ papers }: Readonly<Props>) {
  // The sheet goes by its own title in the sidebar, so it does here too.
  const rows = hiddenRows(useHidden(), papers, usePinned(papers)?.file);
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
            <Line title={one.name} which="hidden">
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

import { guide, openLink } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import { RowsContext } from "./RowNotes";
import { Band, Line, mild } from "./Rows";
import { useCards } from "./useCards";
import { held } from "./working";

interface Props {
  onGreet: () => void;
  onDoc: (paper: string) => void;
  onChanged: () => void;
}

export default function HelpBand({ onGreet, onDoc, onChanged }: Props) {
  const { busy, said, trouble, fail, clear, run } = useCards(() => {});
  return (
    <RowsContext.Provider value={{ busy, said, trouble }}>
      <Band label={t("help")} />
      <div className="border-t border-hair">
        <Line row="greet" why={t("greetAgainWhy")} which="greet">
          <button type="button" onClick={onGreet} className={mild}>
            {t("greetAgainDo")}
          </button>
          <button
            type="button"
            disabled={held(busy, "greet")}
            onClick={() =>
              run("greet", guide(true), (paper) => {
                onChanged();
                onDoc(paper.id);
              })
            }
            className={mild}
          >
            {t("welcomeGuide")}
          </button>
        </Line>
        <Line row="site" why={t("helpOnSiteWhy")} which="site">
          <button
            type="button"
            onClick={() =>
              openLink(t("helpMoreAt"))
                .then(() => clear("site"))
                .catch((e) => fail({ card: "site", text: saidPlainly(e) }))
            }
            className={mild}
          >
            {t("helpOnSiteOpen")} <span aria-hidden="true">↗</span>
          </button>
        </Line>
      </div>
    </RowsContext.Provider>
  );
}

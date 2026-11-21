import {
  type Repeated as Group,
  tidyMerged,
  type Unvouching,
  type Vouched,
  vouchAttachments,
} from "../core";
import { fill, t } from "../locales";
import Card, { type Run, type Word, working } from "./Card";
import Repeated from "./Repeated";
import { mild } from "./Rows";
import Unvouched from "./Unvouched";
import { useSince } from "./useSince";
import { held as isHeld, type Working } from "./working";

interface Props {
  twice: Group[];
  unproven: Unvouching | null;
  open: (card: "repeated" | "unvouched") => boolean;
  busy: Working;
  run: Run;
  tell: (word: Word) => void;
  relook: () => void;
}

export const vouchedSaid = (told: Vouched): string =>
  [
    fill("unvouchedDone", String(told.kept)),
    told.unlike.length > 0 ? fill("unvouchedUnlike", String(told.unlike.length)) : "",
    told.gone.length > 0 ? fill("unvouchedGone", String(told.gone.length)) : "",
  ]
    .filter(Boolean)
    .join(" ");

export default function Tidying({ twice, unproven, open, busy, run, tell, relook }: Props) {
  const secs = useSince(busy.has("unvouched"), busy.get("unvouched"));
  return (
    <>
      {open("repeated") && (
        <Card title={t("repeatedLists")} which="repeated">
          <Repeated
            seen={twice}
            held={isHeld(busy, "repeated")}
            className={mild}
            join={() =>
              run("repeated", tidyMerged(), () => {
                tell({ card: "repeated", text: t("repeatedDone") });
                relook();
              })
            }
          />
        </Card>
      )}
      {open("unvouched") && (
        <Card title={t("unvouchedTitle")} which="unvouched">
          <Unvouched
            seen={unproven?.found ?? []}
            held={isHeld(busy, "unvouched")}
            className={mild}
            label={busy.has("unvouched") ? working("unvouchedWorking", secs) : t("unvouchedDo")}
            vouch={() =>
              run("unvouched", vouchAttachments(), (told) => {
                tell({ card: "unvouched", text: vouchedSaid(told) });
                relook();
              })
            }
          />
        </Card>
      )}
    </>
  );
}

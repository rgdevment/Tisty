import {
  type Repeated as Group,
  tidyMerged,
  type Unvouching,
  type Vouched,
  vouchAttachments,
} from "../core";
import { fill, t } from "../locales";
import Card, { type Run, type Which, type Word, working } from "./Card";
import Repeated from "./Repeated";
import { mild } from "./Rows";
import Unvouched from "./Unvouched";

interface Props {
  twice: Group[];
  unproven: Unvouching | null;
  open: (card: "repeated" | "unvouched") => boolean;
  busy: Which | null;
  secs: number;
  said?: Word;
  trouble?: Word;
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

export default function Tidying({
  twice,
  unproven,
  open,
  busy,
  secs,
  said,
  trouble,
  run,
  tell,
  relook,
}: Props) {
  const held = busy !== null;
  const shown = { busy, said, trouble };
  return (
    <>
      {open("repeated") && (
        <Card title={t("repeatedLists")} which="repeated" {...shown}>
          <Repeated
            seen={twice}
            held={held}
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
        <Card title={t("unvouchedTitle")} which="unvouched" {...shown}>
          <Unvouched
            seen={unproven?.found ?? []}
            held={held}
            className={mild}
            label={busy === "unvouched" ? working("unvouchedWorking", secs) : t("unvouchedDo")}
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

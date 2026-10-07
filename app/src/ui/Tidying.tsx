import { tidyMerged, type Vouched, vouchAttachments } from "../core";
import { fill, t } from "../locales";
import Card, { type Run, type Which, type Word } from "./Card";
import Repeated from "./Repeated";
import Unvouched from "./Unvouched";

interface Props {
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  run: Run;
  tell: (word: Word) => void;
  mild: string;
}

export const vouchedSaid = (told: Vouched): string =>
  [
    fill("unvouchedDone", String(told.kept)),
    told.unlike.length > 0 ? fill("unvouchedUnlike", String(told.unlike.length)) : "",
    told.gone.length > 0 ? fill("unvouchedGone", String(told.gone.length)) : "",
  ]
    .filter(Boolean)
    .join(" ");

export default function Tidying({ busy, said, trouble, run, tell, mild }: Props) {
  const held = busy !== null;
  const shown = { busy, said, trouble };
  return (
    <>
      <Card title={t("repeatedLists")} which="repeated" {...shown}>
        <Repeated
          held={held}
          className={mild}
          join={(then) =>
            run("repeated", tidyMerged(), () => {
              tell({ card: "repeated", text: t("repeatedDone") });
              then();
            })
          }
        />
      </Card>
      <Card title={t("unvouchedTitle")} which="unvouched" {...shown}>
        <Unvouched
          held={held}
          className={mild}
          vouch={(then) =>
            run("unvouched", vouchAttachments(), (told) => {
              tell({ card: "unvouched", text: vouchedSaid(told) });
              then();
            })
          }
        />
      </Card>
    </>
  );
}

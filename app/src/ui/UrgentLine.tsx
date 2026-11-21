import { useEffect, useState } from "react";
import type { Settings } from "../core";
import { t } from "../locales";
import { openNoticeSettings, urgentOffered } from "../reminding";
import type { Which } from "./Card";
import { Note, NoteButton } from "./RowNotes";
import { Knob, Line, ON_MAC } from "./Rows";

interface Props {
  kept: Settings;
  held: boolean;
  remember: (next: Settings, card: Which) => void;
}

export default function UrgentLine({ kept, held, remember }: Props) {
  const [offered, setOffered] = useState<boolean | null>(null);
  useEffect(() => {
    urgentOffered()
      .then(setOffered)
      .catch(() => {});
  }, []);
  const shown = !kept.quiet.includes("screen");
  const on = offered === true && shown && !!kept.urgent;
  const screenOn = () =>
    remember({ ...kept, quiet: kept.quiet.filter((one) => one !== "screen") }, "screen");

  return (
    <Line
      row="urgent"
      why={t(offered === false ? "noticeUrgentAway" : "noticeUrgentWhy")}
      which="urgent"
      note={
        offered !== true ? null : !shown ? (
          <Note
            kind="dep"
            act={<NoteButton label={t("urgentScreenOn")} disabled={held} onPress={screenOn} />}
          >
            {t("noticeUrgentScreen")}
          </Note>
        ) : (
          on && (
            <Note
              kind="sys"
              act={
                <NoteButton
                  label={t("noticeUrgentOpen")}
                  onPress={() => {
                    openNoticeSettings().catch(() => {});
                  }}
                />
              }
            >
              {t(ON_MAC ? "noticeUrgentMac" : "noticeUrgentWindows")}
            </Note>
          )
        )
      }
    >
      <Knob
        on={on}
        label={t("noticeUrgent")}
        disabled={held || offered !== true || !shown}
        onPress={() => remember({ ...kept, urgent: !kept.urgent }, "urgent")}
      />
    </Line>
  );
}

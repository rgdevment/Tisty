import { useEffect, useState } from "react";
import type { Settings } from "../core";
import { t } from "../locales";
import { openNoticeSettings, urgentOffered } from "../reminding";
import type { Which, Word } from "./Card";
import { Knob, Line, mild, ON_MAC } from "./Rows";

interface Props {
  kept: Settings;
  held: boolean;
  said?: Word;
  trouble?: Word;
  remember: (next: Settings, card: Which) => void;
}

export default function UrgentLine({ kept, held, said, trouble, remember }: Props) {
  const [offered, setOffered] = useState<boolean | null>(null);
  useEffect(() => {
    urgentOffered()
      .then(setOffered)
      .catch(() => {});
  }, []);
  const shown = !kept.quiet.includes("screen");
  const on = offered === true && shown && !!kept.urgent;

  return (
    <Line
      title={t("noticeUrgent")}
      why={t(
        offered === false ? "noticeUrgentAway" : shown ? "noticeUrgentWhy" : "noticeUrgentScreen",
      )}
      which="notices"
      said={said}
      trouble={trouble}
      more={
        on && (
          <div className="mt-2 rounded-[10px] bg-mark-priority px-3 py-2.5">
            <p className="text-[12.5px] leading-relaxed text-ink">
              {t(ON_MAC ? "noticeUrgentMac" : "noticeUrgentWindows")}
            </p>
            <button
              type="button"
              className={`mt-2 ${mild}`}
              onClick={() => {
                openNoticeSettings().catch(() => {});
              }}
            >
              {t("noticeUrgentOpen")}
            </button>
          </div>
        )
      }
    >
      <Knob
        on={on}
        label={t("noticeUrgent")}
        disabled={held || offered !== true || !shown}
        onPress={() => remember({ ...kept, urgent: !kept.urgent }, "notices")}
      />
    </Line>
  );
}

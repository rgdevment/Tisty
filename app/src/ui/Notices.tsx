import { useEffect, useState } from "react";
import { type CalendarChoice, keepCalendar, offered } from "../calendaring";
import { type Settings, shortcut } from "../core";
import { fill, t } from "../locales";
import type { Run, Which, Word } from "./Card";
import { Band, Knob, Line, ON_MAC, off } from "./Rows";
import ShortcutLine from "./ShortcutLine";
import UrgentLine from "./UrgentLine";
import { held, type Working } from "./working";

interface Props {
  kept: Settings | null;
  amend: (edit: (was: Settings) => Settings) => void;
  keys: string | null;
  setKeys: (keys: string | null) => void;
  busy: Working;
  run: Run;
  tell: (word?: Word) => void;
  remember: (next: Settings, card: Which) => void;
}

export default function Notices({ kept, amend, keys, setKeys, busy, run, tell, remember }: Props) {
  const chosenOff = kept?.quickKeys === "off";
  const [ideaKeys, setIdeaKeys] = useState<string | null>(null);
  useEffect(() => {
    shortcut("idea")
      .then(setIdeaKeys)
      .catch(() => {});
  }, []);
  return (
    <>
      <Band label={t("quick")} />
      <div className="border-t border-hair">
        <ShortcutLine
          door="task"
          which="quick"
          row="quick"
          why={keys ? fill("quickOn", keys) : t(chosenOff ? "quickOff" : "quickNone")}
          label={t("quickKeys")}
          ready={Boolean(kept)}
          keys={keys}
          chosen={kept?.quickKeys}
          held={held(busy, "quick")}
          run={run}
          tell={tell}
          onKeys={setKeys}
          onChosen={(wanted) => amend((was) => ({ ...was, quickKeys: wanted }))}
        />
        <ShortcutLine
          door="idea"
          which="jot"
          row="idea"
          why={ideaKeys ? fill("quickIdeaOn", ideaKeys) : t("quickIdeaOff")}
          label={t("quickIdeaKeys")}
          ready={Boolean(kept)}
          keys={ideaKeys}
          chosen={kept?.ideaKeys}
          held={held(busy, "jot")}
          run={run}
          tell={tell}
          onKeys={setIdeaKeys}
          onChosen={(wanted) => amend((was) => ({ ...was, ideaKeys: wanted }))}
        />
      </div>

      <Band label={t("bandNotices")} />
      <div className="border-t border-hair">
        {kept &&
          (["screen", "chime"] as const).map((channel) => (
            <Line
              key={channel}
              row={channel}
              why={channel === "screen" ? t("noticesWhy") : undefined}
              which={channel}
            >
              <Knob
                on={!kept.quiet.includes(channel)}
                label={t(channel === "screen" ? "noticeScreen" : "noticeChime")}
                disabled={held(busy, channel)}
                onPress={() =>
                  remember(
                    {
                      ...kept,
                      quiet: kept.quiet.includes(channel)
                        ? kept.quiet.filter((one) => one !== channel)
                        : [...kept.quiet, channel],
                    },
                    channel,
                  )
                }
              />
            </Line>
          ))}
        {kept && <UrgentLine kept={kept} held={held(busy, "urgent")} remember={remember} />}
        {kept && (
          <Line row="calendar" why={t("calendarSettingWhy")} which="calendar">
            <select
              aria-label={t("calendarSetting")}
              value={kept.calendar ?? ""}
              disabled={held(busy, "calendar")}
              onChange={(e) => {
                const wanted = (e.target.value || undefined) as CalendarChoice | undefined;
                run("calendar", keepCalendar(wanted), (now) => {
                  amend((was) => ({ ...was, calendar: now ?? undefined }));
                  tell({ card: "calendar", text: t("settingsKept") });
                });
              }}
              className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
            >
              <option value="">{t("calendarAsk")}</option>
              {offered(ON_MAC).map((one) => (
                <option key={one.choice} value={one.choice} disabled={one.away}>
                  {t(one.label)}
                </option>
              ))}
            </select>
          </Line>
        )}
      </div>
      <p className="mt-2 text-[11.5px] leading-relaxed text-faint">{t("noticesMore")}</p>
    </>
  );
}

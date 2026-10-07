import {
  guide,
  keepClosing,
  keepLocale,
  keepTheme,
  type Reach,
  type Settings,
  type Theme,
  takeOutOfReach,
  type Waking,
  wakeFor,
} from "../core";
import { adopt, fill, t } from "../locales";
import type { Run, Which, Word } from "./Card";
import { Band, Knob, Line, mild, ON_MAC, off } from "./Rows";
import Signing, { type Signs } from "./Signing";

interface Props {
  kept: Settings | null;
  setKept: (kept: Settings) => void;
  wake: Waking | null;
  setWake: (wake: Waking) => void;
  keys: string | null;
  reach: Reach | null;
  setReach: (reach: Reach) => void;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  run: Run;
  tell: (word?: Word) => void;
  remember: (next: Settings, card: Which) => void;
  signs: Signs;
  onChanged: () => void;
  onGreet: () => void;
  onDoc: (paper: string) => void;
}

export default function General({
  kept,
  setKept,
  wake,
  setWake,
  keys,
  reach,
  setReach,
  busy,
  said,
  trouble,
  run,
  tell,
  remember,
  signs,
  onChanged,
  onGreet,
  onDoc,
}: Props) {
  const held = busy !== null;
  return (
    <>
      <Band label={t("bandWindow")} />
      <div className="border-t border-hair">
        {kept && (
          <Line
            title={t("tongue")}
            why={t("tongueWhy")}
            which="tongue"
            said={said}
            trouble={trouble}
          >
            <select
              aria-label={t("tongue")}
              value={kept.locale ?? ""}
              disabled={held}
              onChange={(e) => {
                const wanted = e.target.value || undefined;
                run("tongue", keepLocale(wanted), (now) => {
                  adopt(now ?? undefined);
                  setKept({ ...kept, locale: now ?? undefined });
                  onChanged();
                });
              }}
              className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
            >
              <option value="">{t("tongueTheirs")}</option>
              <option value="es">Español</option>
              <option value="en">English</option>
            </select>
          </Line>
        )}

        {kept && (
          <Line title={t("look")} why={t("lookWhy")} which="look" said={said} trouble={trouble}>
            <select
              aria-label={t("look")}
              value={kept.theme ?? ""}
              disabled={held}
              onChange={(e) => {
                const wanted = (e.target.value || undefined) as Theme | undefined;
                run("look", keepTheme(wanted), (now) => {
                  setKept({ ...kept, theme: now ?? undefined });
                  onChanged();
                });
              }}
              className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
            >
              <option value="">{t("lookTheirs")}</option>
              <option value="light">{t("lookLight")}</option>
              <option value="dark">{t("lookDark")}</option>
            </select>
          </Line>
        )}

        {kept && (
          <Line
            title={t("closingSetting")}
            why={t("closingSettingWhy")}
            which="closing"
            said={said}
            trouble={trouble}
          >
            <select
              aria-label={t("closingSetting")}
              value={kept.onClose ?? "ask"}
              disabled={held}
              onChange={(e) => {
                const how = e.target.value as "hide" | "quit" | "ask";
                run("closing", keepClosing(how), () =>
                  setKept({ ...kept, onClose: how === "ask" ? undefined : how }),
                );
              }}
              className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
            >
              <option value="ask">{t("closingAsk")}</option>
              <option value="hide">{t(ON_MAC ? "closingHideBar" : "closingHide")}</option>
              <option value="quit">{t("closingQuit")}</option>
            </select>
          </Line>
        )}

        {wake?.offered && (
          <Line
            title={t("wake")}
            why={t(wake.wakes ? "wakeOn" : "wakeOff")}
            which="waking"
            said={said}
            trouble={trouble}
            more={
              wake.theirs &&
              !wake.wakes && (
                <div className="mt-2 rounded-[10px] bg-mark-priority px-3 py-2.5">
                  <p className="text-[12.5px] leading-relaxed text-ink">{t("wakeTheirs")}</p>
                </div>
              )
            }
          >
            <Knob
              on={wake.wakes}
              label={t("wakeAdd")}
              disabled={held}
              onPress={() =>
                run("waking", wakeFor(!wake.wakes), (now) => {
                  setWake(now);
                  if (now.wakes === wake.wakes) {
                    return;
                  }
                  tell({
                    card: "waking",
                    text: t(now.wakes ? "wakeFresh" : "wakeGone"),
                  });
                })
              }
            />
          </Line>
        )}

        <Line
          title={t("quick")}
          why={keys ? fill("quickOn", keys) : t("quickNone")}
          which="quick"
          said={said}
          trouble={trouble}
        />
      </div>

      <Signing busy={busy} said={said} trouble={trouble} run={run} tell={tell} signs={signs} />

      <Band label={t("bandNotices")} />
      <div className="border-t border-hair">
        {kept &&
          (["screen", "chime"] as const).map((channel) => (
            <Line
              key={channel}
              title={t(channel === "screen" ? "noticeScreen" : "noticeChime")}
              why={channel === "screen" ? t("noticesWhy") : undefined}
              which="notices"
              said={said}
              trouble={trouble}
            >
              <Knob
                on={!kept.quiet.includes(channel)}
                label={t(channel === "screen" ? "noticeScreen" : "noticeChime")}
                disabled={held}
                onPress={() =>
                  remember(
                    {
                      ...kept,
                      quiet: kept.quiet.includes(channel)
                        ? kept.quiet.filter((one) => one !== channel)
                        : [...kept.quiet, channel],
                    },
                    "notices",
                  )
                }
              />
            </Line>
          ))}
      </div>
      <p className="mt-2 text-[11.5px] leading-relaxed text-faint">{t("noticesMore")}</p>

      <Band label={t("bandOutside")} />
      <div className="border-t border-hair">
        {reach?.shipped && reach.withinReach && (
          <Line
            title={t("terminal")}
            why={fill("terminalOn", reach.through ?? reach.at ?? "")}
            which="terminal"
            said={said}
            trouble={trouble}
            more={
              <p className="mt-2 text-[11.5px] leading-relaxed text-faint">
                {t("terminalRetiring")}
              </p>
            }
          >
            <button
              type="button"
              disabled={held}
              onClick={() =>
                run("terminal", takeOutOfReach(), (now) => {
                  setReach(now);
                  tell({ card: "terminal", text: t("terminalGone") });
                })
              }
              className={mild}
            >
              {t("terminalRemove")}
            </button>
          </Line>
        )}

        <Line
          title={t("greetAgain")}
          why={t("greetAgainWhy")}
          which="greet"
          said={said}
          trouble={trouble}
        >
          <button type="button" onClick={onGreet} className={mild}>
            {t("greetAgainDo")}
          </button>
          <button
            type="button"
            disabled={held}
            onClick={() =>
              run("greet", guide(), (paper) => {
                onChanged();
                onDoc(paper.id);
              })
            }
            className={mild}
          >
            {t("welcomeGuide")}
          </button>
        </Line>
      </div>
    </>
  );
}

import { openUrl } from "@tauri-apps/plugin-opener";
import {
  keepClosing,
  keepLocale,
  keepTheme,
  type Settings,
  type Theme,
  type Waking,
  wakeFor,
} from "../core";
import { adopt, t } from "../locales";
import { saidPlainly } from "../refusal";
import { QUIET, switched, tell as tellBeaver } from "./Beaver";
import type { Run, Which, Word } from "./Card";
import { onMac } from "./macKeys";
import { Note, NoteButton } from "./RowNotes";
import { Band, Knob, Line, off } from "./Rows";
import { held, type Working } from "./working";

const STARTUP_APPS = "ms-settings:startupapps";

interface Props {
  kept: Settings | null;
  amend: (edit: (was: Settings) => Settings) => void;
  wake: Waking | null;
  setWake: (wake: Waking) => void;
  busy: Working;
  run: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  remember: (next: Settings, card: Which) => void;
  onChanged: () => void;
  hidden?: React.ReactNode;
}

export default function Window({
  kept,
  amend,
  wake,
  setWake,
  busy,
  run,
  tell,
  fail,
  remember,
  onChanged,
  hidden,
}: Props) {
  const holds = wake?.theirs === true && !wake.wakes;
  return (
    <>
      <Band label={t("look")} />
      <div className="border-t border-hair">
        {kept && (
          <Line row="tongue" why={t("tongueWhy")} which="tongue">
            <select
              aria-label={t("tongue")}
              value={kept.locale ?? ""}
              disabled={held(busy, "tongue")}
              onChange={(e) => {
                const wanted = e.target.value || undefined;
                run("tongue", keepLocale(wanted), (now) => {
                  adopt(now ?? undefined);
                  amend((was) => ({ ...was, locale: now ?? undefined }));
                  tell({ card: "tongue", text: t("settingsKept") });
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
          <Line row="look" why={t("lookWhy")} which="look">
            <select
              aria-label={t("look")}
              value={kept.theme ?? ""}
              disabled={held(busy, "look")}
              onChange={(e) => {
                const wanted = (e.target.value || undefined) as Theme | undefined;
                run("look", keepTheme(wanted), (now) => {
                  amend((was) => ({ ...was, theme: now ?? undefined }));
                  tell({ card: "look", text: t("settingsKept") });
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
          <Line row="beaver" why={t("beaverPeeksWhy")} which="beaver">
            <Knob
              on={!kept.quiet.includes(QUIET)}
              label={t("beaverPeeks")}
              disabled={held(busy, "beaver")}
              onPress={() => {
                const next = switched(kept);
                remember(next, "beaver");
                tellBeaver(next);
              }}
            />
          </Line>
        )}
      </div>

      <Band label={t("bandWindow")} />
      <div className="border-t border-hair">
        {kept && (
          <Line row="closing" why={t("closingSettingWhy")} which="closing">
            <select
              aria-label={t("closingSetting")}
              value={kept.onClose ?? "ask"}
              disabled={held(busy, "closing")}
              onChange={(e) => {
                const how = e.target.value as "hide" | "quit" | "ask";
                run("closing", keepClosing(how), () => {
                  amend((was) => ({ ...was, onClose: how === "ask" ? undefined : how }));
                  tell({ card: "closing", text: t("settingsKept") });
                });
              }}
              className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
            >
              <option value="ask">{t("closingAsk")}</option>
              <option value="hide">{t(onMac ? "closingHideBar" : "closingHide")}</option>
              <option value="quit">{t("closingQuit")}</option>
            </select>
          </Line>
        )}

        {wake?.offered && (
          <Line
            row="wake"
            why={t(wake.wakes ? "wakeOn" : "wakeOff")}
            which="waking"
            note={
              holds && (
                <Note
                  kind="sys"
                  act={
                    !onMac && (
                      <NoteButton
                        label={t("wakeOpenWindows")}
                        onPress={() =>
                          void openUrl(STARTUP_APPS).catch((e) =>
                            fail({ card: "waking", text: saidPlainly(e) }),
                          )
                        }
                      />
                    )
                  }
                >
                  {t("wakeTheirs")}
                </Note>
              )
            }
          >
            <Knob
              on={wake.wakes}
              label={t("wakeAdd")}
              disabled={held(busy, "waking") || holds}
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
      </div>
      {hidden}
    </>
  );
}

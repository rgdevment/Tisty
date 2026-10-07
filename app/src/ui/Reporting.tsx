import { save } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { copied, facts, keepReport, logs } from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { written } from "../report";
import Card, { type Run, type Which, type Word } from "./Card";
import { Group, mild, strong } from "./Rows";

const TAIL = 300;
const LOGS = "\n--- tisty.log ---";

interface Props {
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  quietly: Run;
}

export default function Reporting({ busy, said, trouble, tell, fail, quietly }: Props) {
  const held = busy !== null;
  const [told, setTold] = useState({ names: false, paths: false, logs: true });
  const [paper, setPaper] = useState<string | null>(null);

  const compose = () => facts(told.names, told.paths).then(written);

  const showReport = () => {
    if (held || paper !== null) return;
    quietly(
      "report",
      Promise.all([compose(), told.logs ? logs(TAIL) : Promise.resolve(null)]).then(
        ([text, kept]) => (kept ? `${text}\n${LOGS}\n${kept.lines.join("\n")}\n` : text),
      ),
      setPaper,
    );
  };

  const changeTold = (next: typeof told) => {
    setTold(next);
    setPaper(null);
  };

  const saveReport = () => {
    if (held) return;
    tell(undefined);
    fail(undefined);
    Promise.all([
      save({ defaultPath: "tisty-report.zip", filters: [{ name: "Tisty", extensions: ["zip"] }] }),
      paper !== null ? Promise.resolve(paper) : compose(),
    ])
      .then(([at, text]) => {
        setPaper(text);
        if (typeof at !== "string") return;
        quietly("report", keepReport(at, text, told.logs), () =>
          tell({ card: "report", text: fill("reportKept", at) }),
        );
      })
      .catch((e) => fail({ card: "report", text: saidPlainly(e) }));
  };

  const copyReport = () => {
    if (held) return;
    (paper !== null ? Promise.resolve(paper) : compose())
      .then((text) => {
        setPaper(text);
        return copied(text);
      })
      .then(() => tell({ card: "report", text: t("reportCopied") }))
      .catch(() => fail({ card: "report", text: t("reportNoClipboard") }));
  };

  return (
    <>
      <Group label={t("reportTitle")} />

      <Card title={t("reportTitle")} which="report" busy={busy} said={said} trouble={trouble}>
        <p className="text-[12.5px] leading-relaxed text-soft">
          {t("reportWhat")} <span className="text-ink">{t("reportNeverSent")}</span>{" "}
          {t("reportYours")}
        </p>

        <div className="mt-2.5 flex flex-col gap-2">
          <label className="flex items-start gap-2 text-[12.5px]">
            <input
              type="checkbox"
              checked={told.logs}
              disabled={held}
              onChange={(e) => changeTold({ ...told, logs: e.target.checked })}
              className="mt-0.5"
            />
            <span>
              {t("reportLogs")}
              <span className="block text-[11.5px] text-faint">{t("reportLogsWhy")}</span>
            </span>
          </label>
          <label className="flex items-start gap-2 text-[12.5px]">
            <input
              type="checkbox"
              checked={told.names}
              disabled={held}
              onChange={(e) => changeTold({ ...told, names: e.target.checked })}
              className="mt-0.5"
            />
            <span>
              {t("reportNames")}
              <span className="block text-[11.5px] text-faint">{t("reportNamesWhy")}</span>
            </span>
          </label>
          <label className="flex items-start gap-2 text-[12.5px]">
            <input
              type="checkbox"
              checked={told.paths}
              disabled={held}
              onChange={(e) => changeTold({ ...told, paths: e.target.checked })}
              className="mt-0.5"
            />
            <span>
              {t("reportPaths")}
              <span className="block text-[11.5px] text-faint">{t("reportPathsWhy")}</span>
            </span>
          </label>
        </div>

        <p className="mt-2.5 text-[11.5px] leading-relaxed text-faint">{t("reportNever")}</p>

        <details className="mt-2.5" onToggle={showReport}>
          <summary className="cursor-pointer text-[12.5px] text-accent">{t("reportShow")}</summary>
          <pre className="scroller mt-2 max-h-[22rem] overflow-x-auto rounded-[10px] bg-hover px-3 py-2.5 font-mono text-[11.5px] leading-relaxed text-soft">
            {paper ?? "…"}
          </pre>
        </details>

        <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
          <button type="button" disabled={held} onClick={saveReport} className={strong}>
            {t("reportSave")}
          </button>
          <button type="button" disabled={held} onClick={copyReport} className={mild}>
            {t("reportCopy")}
          </button>
        </div>
      </Card>
    </>
  );
}

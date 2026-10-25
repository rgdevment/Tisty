import { save as intoFile } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { identityConfirm, keepFile, pagePdf } from "../core";
import { fill, locale, t } from "../locales";
import { saidPlainly } from "../refusal";
import Modal from "./Modal";
import { escaped } from "./paging";
import { mild, off, strong } from "./Rows";

type Step = "words" | "sheet" | "confirm";

interface Props {
  words: string[];
  fingerprint: string;
  onClose: () => void;
  onConfirmed: () => void;
}

type Told = { text: string; amiss?: boolean };

const THIRD = 8;

// One word from each third, so the check reaches across the whole phrase.
const picked = (): number[] => {
  const drawn = crypto.getRandomValues(new Uint32Array(3));
  return Array.from(drawn, (one, at) => 1 + at * THIRD + (one % THIRD));
};

const today = (): string =>
  new Date().toLocaleDateString(locale(), { day: "numeric", month: "long", year: "numeric" });

const SHEET_STYLE =
  "body{font-family:system-ui,sans-serif;color:#111;margin:0;padding:18mm}h1{font-size:20pt;margin:0 0 4pt}.made{font-size:9.5pt;color:#555;margin:0 0 14pt}ol{display:grid;grid-template-rows:repeat(8,auto);grid-auto-flow:column;gap:7pt 18pt;list-style:none;padding:0;margin:0 0 16pt}li{font-size:13pt;border-bottom:0.5pt solid #ccc;padding:3pt 0}li span{display:inline-block;width:18pt;font-size:9pt;color:#777}.warn{font-size:10pt;line-height:1.5;border:1pt solid #c33;border-radius:6pt;padding:8pt 10pt}";

const sheetPage = (words: string[], fingerprint: string, when: string): string =>
  `<!doctype html><html lang="${document.documentElement.lang}"><head><meta charset="utf-8"><style>${SHEET_STYLE}</style></head><body><h1>${escaped(t("sheetTitle"))}</h1><p class="made">${escaped(fill("sheetMade", when, fingerprint))}</p><ol>${words
    .map((word, at) => `<li><span>${at + 1}</span>${escaped(word)}</li>`)
    .join("")}</ol><p class="warn">${escaped(t("sheetWarn"))}</p></body></html>`;

export default function Phrasing({ words, fingerprint, onClose, onConfirmed }: Props) {
  const [step, setStep] = useState<Step>("words");
  const [making, setMaking] = useState(false);
  const [told, setTold] = useState<Told | null>(null);
  const go = (next: Step) => {
    setTold(null);
    setStep(next);
  };
  const when = today();

  const keep = async () => {
    if (making) return;
    setMaking(true);
    try {
      const where = await intoFile({
        defaultPath: `${t("sheetFile")}.pdf`,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
      if (!where) return;
      await keepFile(where, await pagePdf(sheetPage(words, fingerprint, when), "a4"));
      setStep("words");
      setTold({ text: t("sheetSaved") });
    } catch (e) {
      setTold({ text: saidPlainly(e), amiss: true });
    } finally {
      setMaking(false);
    }
  };

  return (
    <Modal
      title={
        step === "confirm"
          ? t("confirmCheck")
          : step === "sheet"
            ? t("sheetTitle")
            : t("phraseWriteDown")
      }
      wide
      onClose={onClose}
    >
      {step === "words" && (
        <>
          <p className="mb-1 text-[11.5px] font-semibold text-faint uppercase">
            {fill("phraseStep", "1")}
          </p>
          <p className="text-[12.5px] leading-relaxed text-soft">{t("phraseWhat")}</p>
          <Words words={words} />
          <div className="mt-3 rounded-[10px] border border-urgent/40 px-3 py-2.5">
            <p className="text-[12.5px] font-semibold">{t("phraseOutside")}</p>
            <p className="mt-0.5 text-[12.5px] leading-relaxed text-soft">
              {t("phraseOutsideWhy")}
            </p>
          </div>
          <div className="mt-4 flex flex-wrap justify-end gap-2">
            <button type="button" className={`mr-auto ${mild}`} onClick={onClose}>
              {t("close")}
            </button>
            <button type="button" className={mild} onClick={() => go("sheet")}>
              {t("phrasePdf")}
            </button>
            <button type="button" className={strong} onClick={() => go("confirm")}>
              {t("phraseKept")}
            </button>
          </div>
        </>
      )}
      {step === "sheet" && (
        <>
          <p className="text-[11.5px] text-faint">{fill("sheetMade", when, fingerprint)}</p>
          <Words words={words} />
          <p className="mt-3 rounded-[10px] border border-urgent/40 px-3 py-2.5 text-[12.5px] leading-relaxed">
            {t("sheetWarn")}
          </p>
          <div className="mt-4 flex flex-wrap justify-end gap-2">
            <button type="button" className={mild} disabled={making} onClick={() => go("words")}>
              {t("cancel")}
            </button>
            <button type="button" className={strong} disabled={making} onClick={() => void keep()}>
              {t("saveIt")}
            </button>
          </div>
        </>
      )}
      {step === "confirm" && (
        <Confirming
          onBack={() => go("words")}
          onConfirmed={onConfirmed}
          onError={(text) => setTold({ text, amiss: true })}
        />
      )}
      {told && (
        <p
          role={told.amiss ? "alert" : "status"}
          className={`mt-3 text-[11.5px] ${told.amiss ? "text-urgent" : "text-faint"}`}
        >
          {told.text}
        </p>
      )}
    </Modal>
  );
}

function Words({ words }: { words: string[] }) {
  return (
    <ol className="mt-3 grid grid-flow-col grid-cols-3 grid-rows-8 gap-x-4 gap-y-1 text-[13px]">
      {words
        .map((word, at) => ({ word, place: at + 1 }))
        .map(({ word, place }) => (
          <li key={place} className="flex gap-2 border-b border-hair py-0.5">
            <span className="w-5 text-right text-[11px] text-faint tabular-nums">{place}</span>
            <span className="font-medium">{word}</span>
          </li>
        ))}
    </ol>
  );
}

function Confirming({
  onBack,
  onConfirmed,
  onError,
}: {
  onBack: () => void;
  onConfirmed: () => void;
  onError: (text: string) => void;
}) {
  const [places] = useState(picked);
  const [typed, setTyped] = useState<Record<number, string>>({});
  const [amiss, setAmiss] = useState<number[] | null>(null);
  const [asking, setAsking] = useState(false);
  const full = places.every((at) => typed[at]?.trim());

  const confirm = () => {
    if (!full || asking) return;
    setAsking(true);
    identityConfirm(places.map((at) => [at, typed[at] ?? ""]))
      .then((wrong) => {
        if (wrong.length === 0) onConfirmed();
        else setAmiss(wrong);
      })
      .catch((e) => onError(saidPlainly(e)))
      .finally(() => setAsking(false));
  };

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        confirm();
      }}
    >
      <p className="mb-1 text-[11.5px] font-semibold text-faint uppercase">
        {fill("phraseStep", "2")}
      </p>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("confirmThree")}</p>
      <div className="mt-3 flex flex-col gap-2">
        {places.map((at) => {
          const wrong = amiss?.includes(at);
          const right = amiss !== null && !wrong;
          return (
            <label key={at} className="flex items-center gap-3 text-[12.5px]">
              <span className="w-20 text-faint">{fill("confirmWord", String(at))}</span>
              <input
                type="text"
                value={typed[at] ?? ""}
                disabled={asking}
                autoComplete="off"
                autoCapitalize="off"
                spellCheck={false}
                onChange={(e) => {
                  setTyped({ ...typed, [at]: e.target.value });
                  setAmiss(null);
                }}
                className={`w-44 rounded-[10px] border bg-bg px-2 py-1 text-[12.5px] ${
                  wrong ? "border-urgent" : "border-line"
                } ${off}`}
              />
              {right && <span className="text-hue-teal">✓</span>}
              {wrong && <span className="text-urgent">{t("confirmAmiss")}</span>}
            </label>
          );
        })}
      </div>
      <p className="mt-3 text-[11.5px] text-faint">{t("confirmLoose")}</p>
      <div className="mt-4 flex flex-wrap justify-end gap-2">
        <button type="button" className={mild} disabled={asking} onClick={onBack}>
          {t("confirmAgain")}
        </button>
        <button type="submit" className={strong} disabled={!full || asking}>
          {t("confirmDo")}
        </button>
      </div>
    </form>
  );
}

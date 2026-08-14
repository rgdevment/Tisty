import { useEffect, useState } from "react";
import { useEdge } from "./edge";
import { open } from "@tauri-apps/plugin-dialog";
import { attach } from "../core";
import { docLink } from "../markdown";
import { addressed } from "../linking";
import Papers from "./Papers";
import Asking from "./Asking";
import { spawned } from "../making";
import Row from "./Row";
import { t } from "../locales";

interface Props {
  steps?: string[];
  onPut: (snippet: string) => void;
  onClose: () => void;
  onError?: (problem: unknown) => void;
}

export default function Insert({ steps = [], onPut, onClose, onError }: Props) {
  const [step, setStep] = useState<"pick" | "link" | "step" | "doc" | "newdoc">("pick");
  const [busy, setBusy] = useState(false);
  const { box, away } = useEdge<HTMLDivElement>();

  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        onClose();
      }
    };
    window.addEventListener("keydown", key, true);
    return () => window.removeEventListener("keydown", key, true);
  }, [onClose]);

  const pickFile = () => {
    if (busy) return;
    setBusy(true);
    open({ multiple: false })
      .then((at) => (typeof at === "string" ? attach(at) : null))
      .then((markdown) => {
        if (markdown) onPut(markdown);
        else onClose();
      })
      .catch((problem) => {
        onClose();
        onError?.(problem);
      })
      .finally(() => setBusy(false));
  };

  return (
    <>
      <span className="fixed inset-0 z-10" onClick={onClose} />
      <div
        ref={box}
        className={`absolute left-1.5 z-20 w-[258px] rounded-[10px] border border-line bg-bg p-[5px] text-[12.5px] shadow-lift ${
          away.up ? "bottom-full mb-1" : "top-full"
        }`}
      >
        {step === "pick" && (
          <>
            <Row first glyph="🔗" say={t("sayLink")} onPick={() => setStep("link")}>
              {t("insertLink")}
            </Row>
            <Row glyph="▤" say={t("sayDoc")} onPick={() => setStep("doc")}>
              {t("insertDoc")}
            </Row>
            <Row glyph="✚" say={t("sayNewDoc")} onPick={() => setStep("newdoc")}>
              {t("insertNewDoc")}
            </Row>
            <Row glyph="📎" say={busy ? "…" : t("sayAttach")} onPick={pickFile}>
              {t("insertAttach")}
            </Row>
            {steps.length > 0 && (
              <Row glyph="#" say={t("sayStep")} onPick={() => setStep("step")}>
                {t("insertStep")}
              </Row>
            )}
          </>
        )}
        {step === "link" && <Linking onLink={(text, url) => onPut(`[${text}](${url})`)} />}
        {step === "doc" && (
          <Papers onPick={(doc) => onPut(docLink(doc.file, doc.title))} onError={onError} />
        )}
        {step === "newdoc" && (
          <Asking
            onName={(name) => {
              if (busy) return;
              setBusy(true);
              spawned(name)
                .then(onPut)
                .catch((problem) => {
                  onClose();
                  onError?.(problem);
                })
                .finally(() => setBusy(false));
            }}
          />
        )}
        {step === "step" &&
          steps.map((text, at) => (
            <Row key={at} glyph={`${at + 1}`} onPick={() => onPut(`[[#${at + 1}]]`)}>
              {text}
            </Row>
          ))}
      </div>
    </>
  );
}

function Linking({ onLink }: { onLink: (text: string, url: string) => void }) {
  const [label, setLabel] = useState("");
  const [url, setUrl] = useState("");
  const [wrong, setWrong] = useState(false);

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        const full = addressed(url);
        if (!full) return setWrong(Boolean(url.trim()));
        onLink(label.trim() || url.trim(), full);
      }}
    >
      <label className="block px-2.5 pt-1 text-[11px] tracking-[0.04em] text-faint uppercase">
        {t("linkText")}
      </label>
      <input
        autoFocus
        value={label}
        aria-label={t("linkText")}
        onChange={(e) => setLabel(e.target.value)}
        className="mt-0.5 mb-1 w-full rounded-md bg-hover px-2.5 py-1.5 outline-none"
      />
      <label className="block px-2.5 text-[11px] tracking-[0.04em] text-faint uppercase">
        {t("linkUrl")}
      </label>
      <input
        value={url}
        placeholder="https://"
        aria-label={t("linkUrl")}
        aria-invalid={wrong || undefined}
        onChange={(e) => {
          setWrong(false);
          setUrl(e.target.value);
        }}
        className={`mt-0.5 w-full rounded-md px-2.5 py-1.5 outline-none placeholder:text-faint ${
          wrong ? "bg-urgent/15 text-urgent" : "bg-hover"
        }`}
      />
      {wrong && <p className="px-2.5 pt-1 text-[11px] text-urgent">{t("notAnAddress")}</p>}
      <button type="submit" className="sr-only">
        {t("insertLink")}
      </button>
    </form>
  );
}

import { useEffect, useState } from "react";
import { t } from "../locales";

interface Props {
  known: string[];
  onPut: (snippet: string) => void;
  onClose: () => void;
}

/** Two options, not three: a ticket is a link, so it goes in with its code as the text. */
export default function Insert({ known, onPut, onClose }: Props) {
  const [step, setStep] = useState<"pick" | "doc" | "link">("pick");

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

  return (
    <>
      <span className="fixed inset-0 z-10" onClick={onClose} />
      <div className="absolute top-full left-1.5 z-20 w-[258px] rounded-[10px] border border-line bg-bg p-[5px] text-[12.5px] shadow-lift">
        {step === "pick" && (
          <>
            <Row first glyph="📄" say={t("sayDoc")} onPick={() => setStep("doc")}>
              {t("insertDoc")}
            </Row>
            <Row glyph="🔗" say={t("sayLink")} onPick={() => setStep("link")}>
              {t("insertLink")}
            </Row>
          </>
        )}
        {step === "doc" && <Naming known={known} onName={(name) => onPut(`[[${name}]]`)} />}
        {step === "link" && <Linking onLink={(text, url) => onPut(`[${text}](${url})`)} />}
      </div>
    </>
  );
}

function Row({
  glyph,
  say,
  first,
  children,
  onPick,
}: {
  glyph: string;
  say: string;
  first?: boolean;
  children: React.ReactNode;
  onPick: () => void;
}) {
  return (
    <button
      type="button"
      autoFocus={first}
      onClick={onPick}
      className="flex w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-ink hover:bg-hover"
    >
      <span className="w-[15px] text-center">{glyph}</span>
      {children}
      <span className="ml-auto text-[11px] text-faint">{say}</span>
    </button>
  );
}

function Naming({ known, onName }: { known: string[]; onName: (name: string) => void }) {
  const [text, setText] = useState("");
  const typed = text.trim().toLowerCase();
  const offered = known.filter((one) => one.toLowerCase().includes(typed)).slice(0, 6);

  return (
    <>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (text.trim()) onName(text.trim());
        }}
      >
        <input
          autoFocus
          value={text}
          placeholder={t("insertDoc")}
          aria-label={t("insertDoc")}
          onChange={(e) => setText(e.target.value)}
          className="mb-1 w-full rounded-md bg-hover px-2.5 py-1.5 outline-none placeholder:text-faint"
        />
      </form>
      {offered.map((one) => (
        <button
          key={one}
          type="button"
          onClick={() => onName(one)}
          className="block w-full truncate rounded-md px-2.5 py-1.5 text-left text-ink hover:bg-hover"
        >
          {one}
        </button>
      ))}
    </>
  );
}

function Linking({ onLink }: { onLink: (text: string, url: string) => void }) {
  const [label, setLabel] = useState("");
  const [url, setUrl] = useState("");

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        if (url.trim()) onLink(label.trim() || url.trim(), url.trim());
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
        onChange={(e) => setUrl(e.target.value)}
        className="mt-0.5 w-full rounded-md bg-hover px-2.5 py-1.5 outline-none placeholder:text-faint"
      />
      <button type="submit" className="sr-only">
        {t("insertLink")}
      </button>
    </form>
  );
}

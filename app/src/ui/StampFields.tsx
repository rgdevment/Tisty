import { useEffect, useState } from "react";
import { t } from "../locales";
import { Ask, Knob } from "./Rows";

export type Declared = "author" | "version";

interface Props {
  carded: boolean;
  author: string;
  version: string;
  /** Who the card names when the document declares nobody. */
  signer?: string | null;
  onCard: (on: boolean) => void;
  onDeclare: (key: Declared, said: string) => void;
}

function Said({
  label,
  value,
  hint,
  onSay,
}: {
  label: string;
  value: string;
  hint: string;
  onSay: (said: string) => void;
}) {
  const [typed, setTyped] = useState(value);
  useEffect(() => setTyped(value), [value]);
  const say = () => {
    if (typed.trim() !== value) onSay(typed);
  };
  return (
    <label className="flex items-center gap-2 text-[11.5px] text-faint">
      <span className="w-14 shrink-0">{label}</span>
      <input
        type="text"
        value={typed}
        placeholder={hint}
        maxLength={120}
        onChange={(e) => setTyped(e.target.value)}
        onBlur={say}
        onKeyDown={(e) => {
          if (e.key === "Enter") e.currentTarget.blur();
        }}
        className="min-w-0 flex-1 rounded-[8px] border border-line bg-bg px-2 py-0.5 text-[12px] text-ink"
      />
    </label>
  );
}

export default function StampFields({ carded, author, version, signer, onCard, onDeclare }: Props) {
  return (
    <>
      <div className="flex items-center gap-2 text-[11.5px] text-faint">
        <Knob on={carded} label={t("cardShown")} onPress={() => onCard(!carded)} />
        <span>{t("cardShown")}</span>
        <Ask said={t("pdfSignAsk")} end />
      </div>
      {carded && (
        <div className="flex flex-col gap-1.5 pl-1">
          <Said
            label={t("pdfAuthor")}
            value={author}
            hint={signer || t("signNameNone")}
            onSay={(said) => onDeclare("author", said)}
          />
          <Said
            label={t("pdfVersion")}
            value={version}
            hint={t("cardVersionNone")}
            onSay={(said) => onDeclare("version", said)}
          />
        </div>
      )}
    </>
  );
}

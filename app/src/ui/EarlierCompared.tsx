import { useEffect, useRef, useState } from "react";
import type { Carded } from "tisty-editor/read";
import { type Compared, docCompared } from "../earlier";
import { fill, t } from "../locales";
import Page from "./Page";
import { mild } from "./Rows";

interface Props {
  doc: string;
  was: string;
  now: string | null;
  between: string;
  label: string;
  card: (file: string) => Carded;
  ready: () => Promise<unknown>;
  onWhole: () => void;
  onError: (problem: unknown) => void;
}

// The core already drew the page, so the window lays it as it came.
const AS_DRAWN = (html: string) => Promise.resolve(html);

function Counted({ said }: Readonly<{ said: Compared }>) {
  if (said.kind === "tooLong") return null;
  if (said.gone + said.came === 0) return <span>{t("earlierSame")}</span>;
  return (
    <>
      <span className="rounded-[4px] bg-hue-red/15 px-1.5">
        {fill("earlierWent", String(said.gone))}
      </span>
      <span className="rounded-[4px] bg-hue-green/15 px-1.5">
        {fill("earlierCame", String(said.came))}
      </span>
    </>
  );
}

export default function EarlierCompared({
  doc,
  was,
  now,
  between,
  label,
  card,
  ready,
  onWhole,
  onError,
}: Readonly<Props>) {
  const asked = `${was}:${now ?? ""}`;
  const [told, setTold] = useState<{ asked: string; said: Compared } | null>(null);
  const hands = useRef({ ready, onWhole, onError });
  hands.current = { ready, onWhole, onError };

  useEffect(() => {
    let live = true;
    const landed = now ? Promise.resolve() : hands.current.ready();
    landed
      .then(() => docCompared(doc, was, now))
      .then((said) => live && setTold({ asked, said }))
      .catch((e) => {
        if (!live) return;
        hands.current.onError(e);
        hands.current.onWhole();
      });
    return () => {
      live = false;
    };
  }, [doc, was, now, asked]);

  if (told?.asked !== asked) return null;
  const { said } = told;
  return (
    <>
      <p
        role="status"
        className="flex flex-wrap items-center gap-x-3 gap-y-1 border-b border-hair bg-bg px-5 py-1.5 text-[11.5px] text-soft"
      >
        <span className="min-w-0 truncate">{between}</span>
        <Counted said={said} />
      </p>
      {said.kind === "drawn" ? (
        <Page
          key={asked}
          body={said.html}
          draw={AS_DRAWN}
          label={label}
          card={card}
          onDoc={() => undefined}
        />
      ) : (
        <div className="px-8 pt-8">
          <p className="max-w-[460px] text-[12.5px] leading-relaxed text-soft">
            {t("earlierTooLong")}
          </p>
          <button type="button" onClick={onWhole} className={`${mild} mt-3`}>
            {t("earlierWhole")}
          </button>
        </div>
      )}
    </>
  );
}

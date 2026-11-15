import { useEffect, useState } from "react";
import { diaryAgents, diaryOf, diaryStick, diaryWrite, type Today } from "../diary";
import { fill, t } from "../locales";
import { IDEA_AT_MOST } from "../notes";

interface Props {
  onBefore: () => Promise<unknown>;
  onWritten: (today: Today) => void;
  onError: (problem: unknown) => void;
}

type Asked = "writing" | "sticking" | null;

const ASKING =
  "rounded-md px-2 py-1 text-[12.5px] text-accent hover:bg-hover aria-pressed:bg-accent-soft";

export default function DiaryBar({ onBefore, onWritten, onError }: Props) {
  const [open, setOpen] = useState<boolean | null>(null);
  const [asked, setAsked] = useState<Asked>(null);
  const [text, setText] = useState("");

  useEffect(() => {
    let live = true;
    diaryOf()
      .then((kept) => live && setOpen(kept.open))
      .catch(() => live && setOpen(null));
    return () => {
      live = false;
    };
  }, []);

  const turn = () => {
    if (open === null) return;
    const next = !open;
    diaryAgents(next)
      .then(() => setOpen(next))
      .catch(onError);
  };

  const ask = (which: Asked) => {
    setText("");
    setAsked((was) => (was === which ? null : which));
  };

  const keep = () => {
    const said = text.trim();
    if (!said) return;
    onBefore()
      .catch(() => {})
      .then(() => (asked === "sticking" ? diaryStick(said) : diaryWrite(said)))
      .then((today) => {
        setText("");
        setAsked(null);
        onWritten(today);
      })
      .catch(onError);
  };

  const keys = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") setAsked(null);
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey || asked === "sticking")) {
      e.preventDefault();
      keep();
    }
  };

  return (
    <div className="mb-3 flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-1">
        <button
          type="button"
          aria-pressed={asked === "writing"}
          onClick={() => ask("writing")}
          className={ASKING}
        >
          {t("diaryWrite")}
        </button>
        <button
          type="button"
          aria-pressed={asked === "sticking"}
          onClick={() => ask("sticking")}
          className={ASKING}
        >
          {t("diaryStick")}
        </button>
        {open !== null && (
          <button
            type="button"
            aria-pressed={open}
            onClick={turn}
            title={t(open ? "diaryOpenWhy" : "diaryHiddenWhy")}
            className={`ml-auto rounded-full px-2 py-0.5 text-[10.5px] ${
              open ? "bg-accent-soft text-accent" : "bg-hover text-soft"
            }`}
          >
            {t(open ? "diaryOpen" : "diaryHidden")}
          </button>
        )}
      </div>
      {asked && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            keep();
          }}
          className="flex flex-col gap-1.5 rounded-[10px] border border-hair bg-bg p-2"
        >
          {asked === "writing" ? (
            <textarea
              autoFocus
              rows={4}
              value={text}
              placeholder={t("diaryWriting")}
              aria-label={t("diaryWrite")}
              onChange={(e) => setText(e.target.value)}
              onKeyDown={keys}
              className="resize-y bg-transparent text-[12.5px] text-ink outline-none placeholder:text-faint"
            />
          ) : (
            <input
              autoFocus
              value={text}
              maxLength={IDEA_AT_MOST}
              placeholder={fill("diaryNote", String(IDEA_AT_MOST))}
              aria-label={t("diaryStick")}
              onChange={(e) => setText(e.target.value)}
              onKeyDown={keys}
              className="bg-transparent text-[12.5px] text-ink outline-none placeholder:text-faint"
            />
          )}
          <div className="flex items-center gap-1">
            {asked === "writing" && (
              <span className="text-[10.5px] text-faint">{t("diaryWritingKeys")}</span>
            )}
            <button
              type="button"
              onClick={() => setAsked(null)}
              className="ml-auto rounded-md px-2 py-1 text-[12.5px] text-soft hover:bg-hover"
            >
              {t("cancel")}
            </button>
            <button
              type="submit"
              disabled={!text.trim()}
              className="rounded-md bg-accent px-2 py-1 text-[12.5px] text-white disabled:opacity-40"
            >
              {t("diaryKeep")}
            </button>
          </div>
        </form>
      )}
    </div>
  );
}
